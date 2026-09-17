"""Build an Ark app that embeds CPython, in four steps.

1. Fetch and cross compile CPython for wasm32-wasip1, once per configuration.
2. Work out which Python modules and native extensions the app actually uses.
3. Compile those modules to bytecode and write them out as frozen C arrays.
4. Link the interpreter, the trimmed extension registry and the frozen app.

CPython can only be cross compiled by an interpreter of its own release series,
so the version built here is the version running this script.

Usage: tools/python_build.py apps/<app>/main.py build/<app>.wasm --sdk <wasi-sdk>
"""
import argparse
import dataclasses
import fcntl
import hashlib
import json
import marshal
import modulefinder
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.request

VERSION = platform.python_version()
SERIES = f"{sys.version_info.major}.{sys.version_info.minor}"
ROOT = Path(__file__).resolve().parents[1]
TOOLS = Path(__file__).resolve().parent

# CPython freezes these at startup and they pull in native code of their own, so
# the app never has to carry them.
STARTUP_MODULES = {"abc", "codecs", "io", "os", "stat", "posixpath", "genericpath",
                   "_collections_abc"}

# An interpreter with no filesystem to import from still needs its text codecs.
CODECS = ["aliases", "utf_8", "ascii", "latin_1"]


@dataclasses.dataclass(frozen=True)
class Runtime:
    """A cross compiled CPython that any number of apps can link against."""

    source: Path          # the unpacked CPython release
    build: Path           # where it was configured and built
    flags: list           # compiler flags every object shares
    env: dict             # the cross compiling environment

    @property
    def library(self):
        return self.build / f"libpython{SERIES}.a"

    def extension_archives(self):
        """Static libraries the selected extensions may need at link time."""
        optional = (self.build / "Modules/_decimal/libmpdec/libmpdec.a",
                    self.build / "Modules/expat/libexpat.a")
        return [*sorted((self.build / "Modules/_hacl").glob("*.a")),
                *(archive for archive in optional if archive.exists())]


def run(command, cwd, env):
    subprocess.run(command, cwd=cwd, env=env, check=True)


def fetch_source(work):
    """Download and unpack the CPython release this interpreter comes from."""
    source = work / f"Python-{VERSION}"
    archive = work / f"Python-{VERSION}.tar.xz"
    if not archive.exists():
        url = f"https://www.python.org/ftp/python/{VERSION}/{archive.name}"
        with urllib.request.urlopen(url) as response, archive.open("wb") as output:
            shutil.copyfileobj(response, output)
    if not source.exists():
        with tarfile.open(archive) as bundle:
            bundle.extractall(work, filter="data")
    return source


def build_runtime(work, sdk, opt, lto, jobs):
    """Configure and build CPython for WASI, reusing an earlier identical build."""
    source = fetch_source(work)
    # The file moved between releases, so take whichever one this release has.
    site = next((source / name for name in ("Tools/wasm/wasi/config.site-wasm32-wasi",
                                            "Tools/wasm/config.site-wasm32-wasi")
                 if (source / name).exists()), None)
    if site is None:
        raise SystemExit(f"CPython {VERSION} carries no WASI build configuration")

    flags = [f"-O{opt}", "-g0", "-ffunction-sections", "-fdata-sections",
             f"-ffile-prefix-map={ROOT}=.", f"-ffile-prefix-map={sdk}=wasi-sdk"]
    key = hashlib.sha256(json.dumps([VERSION, str(sdk), flags, lto]).encode()).hexdigest()[:16]
    build = work / f"runtime-{key}"
    build.mkdir(exist_ok=True)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PATH=f"{sdk / 'bin'}:{os.environ['PATH']}",
               CC=str(sdk / "bin/clang"), CPP=f"{sdk / 'bin/clang'} -E", AR=str(sdk / "bin/llvm-ar"),
               RANLIB=str(sdk / "bin/llvm-ranlib"), CFLAGS=" ".join(flags), CONFIG_SITE=str(site),
               PKG_CONFIG_LIBDIR=str(sdk / "share/wasi-sysroot/lib/pkgconfig"))
    runtime = Runtime(source=source, build=build, flags=flags, env=env)

    if not (build / "Makefile").exists():
        host = subprocess.check_output([str(source / "config.guess")], text=True).strip()
        configure = [f"../{source.name}/configure", "--host=wasm32-wasip1", f"--build={host}",
                     f"--with-build-python={sys.executable}", "--disable-shared",
                     "--without-ensurepip", "--disable-test-modules", "--without-remote-debug",
                     "--without-doc-strings"]
        if lto != "none":
            configure.append(f"--with-lto={lto}")
        run(configure, build, env)
    if not runtime.library.exists():
        run(["make", f"-j{jobs}", runtime.library.name], build, env)
    return runtime


def extension_names(setup):
    """Extension names from a Modules/Setup file, whose lines are `name source.c`."""
    text = re.sub(r"@[^@]+@", "", setup.read_text())
    return set(re.findall(r"^([a-zA-Z_]\w*)\s+[^\n]*\.c\b", text, re.M))


def resolve_imports(app, runtime, includes):
    """Follow the app's imports, and refuse anything this interpreter can't serve.

    Returns the Python modules to freeze and the native extensions to keep.
    """
    registry = (runtime.build / "Modules/config.c").read_text()
    available = set(re.findall(r'\{"([^"\n]+)",', registry))
    optional = extension_names(runtime.source / "Modules/Setup.stdlib.in")
    builtin = extension_names(runtime.source / "Modules/Setup.bootstrap.in")

    class Finder(modulefinder.ModuleFinder):
        """Treats anything the interpreter provides in C as already present."""

        def find_module(self, name, path, parent=None):
            full = f"{parent.__name__}.{name}" if parent else name
            if full in available | optional | builtin | STARTUP_MODULES:
                return None, None, ("", "", modulefinder._C_BUILTIN)
            return super().find_module(name, path, parent)

    finder = Finder(path=[str(app.parent), str(runtime.source / "Lib")])
    finder.run_script(str(app))
    for name in includes:
        finder.import_hook(name)

    missing, uncertain = finder.any_missing_maybe()
    if missing or uncertain:
        raise SystemExit("Unresolved Python imports " + ", ".join(sorted(set(missing + uncertain))))
    selected = set(finder.modules) & optional
    if selected - available:
        raise SystemExit("Unavailable WASI extensions " + ", ".join(sorted(selected - available)))

    library = runtime.source / "Lib"
    modules = {"encodings": library / "encodings/__init__.py",
               **{f"encodings.{name}": library / f"encodings/{name}.py" for name in CODECS},
               "__main__": app}
    for name, module in finder.modules.items():
        if name == "__main__" or not module.__file__:
            continue
        if not module.__file__.endswith(".py"):
            raise SystemExit(f"Python module needs a WASI build {name}")
        modules[name] = Path(module.__file__)
    return modules, selected


def write_registry(runtime, selected, destination):
    """Copy the interpreter's extension table, dropping what the app never imports."""
    registry = (runtime.build / "Modules/config.c").read_text()
    optional = extension_names(runtime.source / "Modules/Setup.stdlib.in")
    registry = re.sub(r'^\s*\{"([^"\n]+)",[^\n]+\n',
                      lambda entry: "" if entry[1] in optional - selected else entry[0],
                      registry, flags=re.M)
    (destination / "config.c").write_text(re.sub(r"^/\* Generated automatically.*\n", "", registry))


def write_frozen(modules, destination, optimize):
    """Compile each module and write it as the C array the interpreter imports."""
    rows = []
    with (destination / "frozen.h").open("w") as output:
        for index, (name, path) in enumerate(sorted(modules.items())):
            code = compile(path.read_bytes(), f"<frozen {name}>", "exec", optimize=optimize)
            data = marshal.dumps(code)
            output.write(f"static const unsigned char frozen_{index}[] = {{")
            output.write(",".join(map(str, data)) + "};\n")
            rows.append(f'{{"{name}", frozen_{index}, {len(data)}, '
                        f'{int(path.name == "__init__.py")}}},')
        output.write("static const struct _frozen app_modules[] = {\n" + "\n".join(rows)
                     + "\n{NULL, NULL, 0, 0}};\n")


def link(runtime, sdk, destination, output, args):
    """Link the interpreter, the app's frozen modules and the WASI shims."""
    command = [str(sdk / "bin/clang"), *runtime.flags]
    if args.lto != "none":
        command.append(f"-flto={args.lto}")
    command += [f"-I{runtime.build}", f"-I{runtime.source / 'Include'}", f"-I{destination}",
                str(TOOLS / "python_embed.c"), str(destination / "config.c"),
                str(runtime.library), *map(str, runtime.extension_archives())]
    # WASI has no signals, processes or dynamic loading, so libc emulates them.
    command += ["-lm", "-ldl", "-lwasi-emulated-signal", "-lwasi-emulated-getpid",
                "-lwasi-emulated-process-clocks", "-lpthread"]
    # The stack goes first so an overflow traps instead of writing over the heap.
    command += ["-Wl,--stack-first", f"-Wl,-z,stack-size={args.stack}",
                f"-Wl,--initial-memory={args.initial_memory}", "-Wl,--gc-sections",
                "-Wl,--strip-all", "-o", str(output)]
    run(command, ROOT, runtime.env)


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("app", type=Path, help="the app's main.py")
    parser.add_argument("output", type=Path, help="the module to write")
    parser.add_argument("--sdk", type=Path, required=True, help="a wasi-sdk installation")
    parser.add_argument("--opt", default="z", choices=["2", "3", "s", "z"])
    parser.add_argument("--lto", default="full", choices=["none", "thin", "full"])
    parser.add_argument("--bytecode", type=int, default=0, choices=[0, 1, 2],
                        help="bytecode optimization level")
    parser.add_argument("--initial-memory", type=int, default=8 * 1024 * 1024,
                        help="initial linear memory in bytes")
    parser.add_argument("--stack", type=int, default=1024 * 1024, help="stack size in bytes")
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    parser.add_argument("--include", action="append", default=[], metavar="MODULE",
                        help="a module the app imports dynamically")
    args = parser.parse_args()

    if sys.version_info < (3, 13) or sys.version_info.releaselevel != "final":
        raise SystemExit(f"Python builds need a released CPython 3.13 or newer, not {VERSION}")
    sdk = args.sdk.resolve()
    if not (sdk / "share/wasi-sysroot").is_dir():
        raise SystemExit("Python builds need a wasi-sdk install in WASI_SDK; see docs/05-running.md")

    work = args.output.resolve().parent / "python"
    work.mkdir(parents=True, exist_ok=True)
    # One runtime serves every app, so concurrent builds wait rather than collide.
    with (work / "runtime.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        runtime = build_runtime(work, sdk, args.opt, args.lto, max(1, args.jobs))

    app = args.app.resolve()
    modules, selected = resolve_imports(app, runtime, args.include)

    destination = work / app.parent.name
    destination.mkdir(exist_ok=True)
    write_registry(runtime, selected, destination)
    write_frozen(modules, destination, args.bytecode)
    (destination / "dependencies.json").write_text(json.dumps(
        {"frozen_modules": sorted(modules), "native_extensions": sorted(selected),
         "explicit_includes": args.include}, indent=2) + "\n")

    link(runtime, sdk, destination, args.output.resolve(), args)


if __name__ == "__main__":
    main()
