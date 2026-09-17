"""Build a shared CPython runtime and freeze each app's static imports."""
import argparse
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

# The interpreter running this script also builds the one the app embeds, which
# CPython requires of a cross build, so the app is built for whatever is here.
VERSION = platform.python_version()
SERIES = f"{sys.version_info.major}.{sys.version_info.minor}"
ROOT = Path(__file__).resolve().parents[1]
TOOLS = Path(__file__).resolve().parent


def run(command, cwd, env):
    subprocess.run(command, cwd=cwd, env=env, check=True)


def native_names(path):
    text = re.sub(r"@[^@]+@", "", path.read_text())
    return set(re.findall(r"^([a-zA-Z_]\w*)\s+[^\n]*\.c\b", text, re.M))


def runtime(work, sdk, opt, lto, jobs):
    source = work / f"Python-{VERSION}"
    archive = work / f"Python-{VERSION}.tar.xz"
    if not archive.exists():
        url = f"https://www.python.org/ftp/python/{VERSION}/{archive.name}"
        with urllib.request.urlopen(url) as response, archive.open("wb") as output:
            shutil.copyfileobj(response, output)
    if not source.exists():
        with tarfile.open(archive) as bundle:
            bundle.extractall(work, filter="data")
    site = next((source / name for name in ("Tools/wasm/wasi/config.site-wasm32-wasi",
                                            "Tools/wasm/config.site-wasm32-wasi")
                 if (source / name).exists()), None)
    if site is None:
        raise SystemExit(f"CPython {VERSION} carries no WASI build configuration")

    flags = [f"-O{opt}", "-g0", "-ffunction-sections", "-fdata-sections",
             f"-ffile-prefix-map={ROOT}=.", f"-ffile-prefix-map={sdk}=wasi-sdk"]
    key = hashlib.sha256(json.dumps([VERSION, str(sdk), flags, lto]).encode()).hexdigest()[:16]
    dest = work / f"runtime-{key}"
    dest.mkdir(exist_ok=True)
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PATH=f"{sdk / 'bin'}:{os.environ['PATH']}",
               CC=str(sdk / "bin/clang"), CPP=f"{sdk / 'bin/clang'} -E", AR=str(sdk / "bin/llvm-ar"),
               RANLIB=str(sdk / "bin/llvm-ranlib"), CFLAGS=" ".join(flags),
               CONFIG_SITE=str(site),
               PKG_CONFIG_LIBDIR=str(sdk / "share/wasi-sysroot/lib/pkgconfig"))
    if not (dest / "Makefile").exists():
        build = subprocess.check_output([str(source / "config.guess")], text=True).strip()
        config = [f"../{source.name}/configure", "--host=wasm32-wasip1", f"--build={build}",
                  f"--with-build-python={sys.executable}", "--disable-shared", "--without-ensurepip",
                  "--disable-test-modules", "--without-remote-debug", "--without-doc-strings"]
        if lto != "none":
            config.append(f"--with-lto={lto}")
        run(config, dest, env)
    if not (dest / f"libpython{SERIES}.a").exists():
        run(["make", f"-j{jobs}", f"libpython{SERIES}.a"], dest, env)
    return source, dest, flags, env


def dependencies(app, source, runtime, includes):
    optional = native_names(source / "Modules/Setup.stdlib.in")
    bootstrap = native_names(source / "Modules/Setup.bootstrap.in")
    config = (runtime / "Modules/config.c").read_text()
    available = set(re.findall(r'\{"([^"\n]+)",', config))
    # These frozen startup modules and their native dependencies ship with CPython.
    startup = {"abc", "codecs", "io", "os", "stat", "posixpath", "genericpath", "_collections_abc"}

    class Finder(modulefinder.ModuleFinder):
        def find_module(self, name, path, parent=None):
            full = f"{parent.__name__}.{name}" if parent else name
            if full in available | optional | bootstrap | startup:
                return None, None, ("", "", modulefinder._C_BUILTIN)
            return super().find_module(name, path, parent)

    finder = Finder(path=[str(app.parent), str(source / "Lib")])
    finder.run_script(str(app))
    for name in includes:
        finder.import_hook(name)
    missing, uncertain = finder.any_missing_maybe()
    if missing or uncertain:
        raise SystemExit("Unresolved Python imports " + ", ".join(sorted(set(missing + uncertain))))
    selected = set(finder.modules) & optional
    if selected - available:
        raise SystemExit("Unavailable WASI extensions " + ", ".join(sorted(selected - available)))

    lib = source / "Lib"
    modules = {"encodings": lib / "encodings/__init__.py",
               **{f"encodings.{name}": lib / f"encodings/{name}.py"
                  for name in ["aliases", "utf_8", "ascii", "latin_1"]}, "__main__": app}
    for name, module in finder.modules.items():
        if name != "__main__" and module.__file__:
            if not module.__file__.endswith(".py"):
                raise SystemExit(f"Python module needs a WASI build {name}")
            modules[name] = Path(module.__file__)
    config = re.sub(r'^\s*\{"([^"\n]+)",[^\n]+\n',
                    lambda match: "" if match[1] in optional - selected else match[0], config, flags=re.M)
    config = re.sub(r"^/\* Generated automatically.*\n", "", config)
    return modules, selected, config


def freeze(modules, dest, optimize):
    rows = []
    with (dest / "frozen.h").open("w") as output:
        for index, (name, path) in enumerate(sorted(modules.items())):
            code = compile(path.read_bytes(), f"<frozen {name}>", "exec", optimize=optimize)
            data = marshal.dumps(code)
            output.write(f"static const unsigned char frozen_{index}[] = {{")
            output.write(",".join(map(str, data)) + "};\n")
            rows.append(f'{{"{name}", frozen_{index}, {len(data)}, {int(path.name == "__init__.py")}}},')
        output.write("static const struct _frozen app_modules[] = {\n" + "\n".join(rows)
                     + "\n{NULL, NULL, 0, 0}};\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--sdk", type=Path, required=True)
    parser.add_argument("--opt", default="z", choices=["2", "3", "s", "z"])
    parser.add_argument("--lto", default="full", choices=["none", "thin", "full"])
    parser.add_argument("--bytecode", type=int, default=0, choices=[0, 1, 2])
    parser.add_argument("--initial-memory", type=int, default=8388608, help="Initial memory in bytes")
    parser.add_argument("--stack", type=int, default=1048576, help="Stack size in bytes")
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    parser.add_argument("--include", action="append", default=[], help="Include a dynamic import")
    args = parser.parse_args()
    if sys.version_info < (3, 13) or sys.version_info.releaselevel != "final":
        raise SystemExit(f"Python builds need a released CPython 3.13 or newer, not {VERSION}")
    sdk = args.sdk.resolve()
    if not (sdk / "share/wasi-sysroot").is_dir():
        raise SystemExit("Python builds need a wasi-sdk install in WASI_SDK; "
                         "see docs/05-running.md")
    work = args.output.resolve().parent / "python"
    work.mkdir(parents=True, exist_ok=True)
    # A shared runtime lock also covers concurrent make invocations.
    with (work / "runtime.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        source, lib, flags, env = runtime(work, sdk, args.opt, args.lto, max(1, args.jobs))
    modules, selected, config = dependencies(args.app.resolve(), source, lib, args.include)
    dest = work / args.app.parent.name
    dest.mkdir(exist_ok=True)
    freeze(modules, dest, args.bytecode)
    (dest / "config.c").write_text(config)
    (dest / "dependencies.json").write_text(json.dumps({"frozen_modules": sorted(modules),
         "native_extensions": sorted(selected), "explicit_includes": args.include}, indent=2) + "\n")
    archives = [*sorted((lib / "Modules/_hacl").glob("*.a")),
                *(path for path in (lib / "Modules/_decimal/libmpdec/libmpdec.a",
                                    lib / "Modules/expat/libexpat.a") if path.exists())]
    command = [str(sdk / "bin/clang"), *flags, f"-I{lib}", f"-I{source / 'Include'}", f"-I{dest}",
               str(TOOLS / "python_embed.c"), str(dest / "config.c"), str(lib / f"libpython{SERIES}.a"),
               *map(str, archives), "-lm", "-ldl", "-lwasi-emulated-signal", "-lwasi-emulated-getpid",
               "-lwasi-emulated-process-clocks", "-lpthread", "-Wl,--stack-first",
               f"-Wl,-z,stack-size={args.stack}", f"-Wl,--initial-memory={args.initial_memory}",
               "-Wl,--gc-sections", "-Wl,--strip-all", "-o", str(args.output.resolve())]
    if args.lto != "none":
        command.insert(1, f"-flto={args.lto}")
    run(command, ROOT, env)


if __name__ == "__main__":
    main()
