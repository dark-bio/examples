# Running apps

You don't need an Ark to write an app. A WebAssembly runtime and the fixture
data in this repository run both passes on a laptop. When the app works, the
`ark` command line tool runs it on a real Ark.

## Locally

### Toolchains

All of these ship for macOS, Linux and Windows. The `make` targets and
`tools/run.sh` need a POSIX shell, so build from WSL on Windows.

- [`wasmtime`](https://wasmtime.dev/) runs the modules.
- [Binaryen](https://github.com/WebAssembly/binaryen/releases) supplies
  `wasm-opt`, which every build runs over its module.
- Rust needs `rustup` with the `wasm32-wasip1` target.
- Go builds with [TinyGo](https://tinygo.org/getting-started/install/), which
  emits a much smaller module than the standard toolchain.
- C and Python need a [wasi-sdk](https://github.com/WebAssembly/wasi-sdk/releases)
  release. The build looks in `/opt/wasi-sdk`, and `WASI_SDK` points it wherever
  you unpacked it. C alone also accepts any clang with a WASI sysroot, such as
  Homebrew's `llvm`, `lld`, `wasi-libc` and `wasi-runtimes`, which the build
  falls back to.
- Python also needs `make` and a
  [CPython](https://www.python.org/downloads/) of 3.13 or newer. Set `PYTHON`
  when the one to build with is not `python3`.

Package managers cover everything except wasi-sdk, which is a tarball to unpack.
With Homebrew that is:

```sh
brew install wasmtime binaryen tinygo python@3.14
rustup target add wasm32-wasip1
```

C apps build with a WASI toolchain rather than Emscripten, because Emscripten's
standalone modules can't open the files the Ark mounts.

These examples are built and measured with wasmtime 48.0.2, Binaryen 132, Rust
1.98.0, TinyGo 0.42.0, wasi-sdk 33.0 and CPython 3.14.7. Newer versions usually
work.

The Python build takes its version from the interpreter you run it with, because
CPython can only be cross compiled by its own release series. It downloads that
exact source into `build/python`, builds one shared runtime from it and freezes
each app's imports, linking the native extensions they need. The first build
takes a few minutes, and every later Python app reuses the runtime. Dynamic
imports go in `--include` when calling `tools/python_build.py` directly.

### Building and running

```sh
make build                     # build every app into build/<app>.wasm
make run                       # build and run every app
make run APP=03-cilantro-mini-rust
make run APP=03-cilantro-mini-rust FIXTURES=fixtures/no-call
```

`make run` does what an Ark does, through `tools/run.sh`. It runs the module
with no arguments to collect its manifest, mounts only the declared datasets
from the fixture root, read-only, and runs the module again with `/` as its
first argument. An undeclared path is never mounted, which is how
[02-permissions](../apps/02-permissions) can show a blocked read on a laptop.

### Small modules

An Ark uploads and starts a smaller module faster, so every build here is tuned
for size. Rust uses `opt-level = "z"`, fat link-time optimization, a single
codegen unit and stripped symbols, all set in each app's `Cargo.toml`. Go builds
with TinyGo at `-opt=z`. C builds with `-Oz` and drops unused sections. Python
builds a trimmed interpreter carrying only the extensions the app imports. Every
module then goes through `wasm-opt`.

The language decides most of it. The C, Go and Rust modules here are tens to a
few hundred kilobytes, while a Python one runs to megabytes because it carries
the interpreter, and it starts noticeably slower on an Ark. Reach for Python
when the libraries or the clarity are worth that, and for the others when
startup matters.

### Fixtures

The fixtures are small plain-text stand-ins for what an Ark mounts. The default
root, `fixtures`, models one invented genome on GRCh38. Its coordinates,
reference alleles and sequences are public data, while every genotype is made
up. Two more roots cover the answers an app must handle:

- `fixtures/no-call` answers every panel variant with a no-call, `./.`.
- `fixtures/unanswered` has coordinates and reference alleles but no genotypes,
  like reference sites in a variants-only call file.

Both hold only the variant data the rsID apps need, so other apps stop at their
missing grants there.

### What a laptop doesn't reproduce

- **The checks before approval.** Locally nothing refuses a start section, a bad
  name, or a path that can't be granted or isn't on the Ark.
- **The limits.** Memory, output and the manifest pass's time are unbounded.
- **The deterministic sandbox.** A stock `wasmtime` gives real randomness and
  real clocks, so don't depend on either, as
  [09-fortune-cookie](../apps/09-fortune-cookie) explains.
- **Output gating.** You always see standard output and standard error, as if
  `develop` were set.
- **The generated tree.** Plain directories list everything, and no read fails
  with an I/O error or "file too large".
- **Startup cost.** A laptop starts a module far faster than an Ark does, so a
  heavy module feels cheaper here than it is there.

## On an Ark

Install the `ark` tool:

```sh
brew install dark-bio/tap/ark-cli                                                        # macOS
curl -fsSL https://github.com/dark-bio/cli/releases/latest/download/ark-installer.sh | sh # Linux
cargo install darkbio-ark --locked                                                       # anywhere with Rust
```

An Ark is paired with `ark pair` once, and unlocked with `ark unlock` after each
power loss, both approved on the owner's phone. Then check what the Ark holds and
run the app:

```sh
ark status                                    # trust, firmware, pairing and lock state
ark data paths                                # the paths apps can read, and what is available
ark app run build/03-cilantro-mini-rust.wasm > report.md
```

`ark app run` uploads the module and waits while the owner approves it on their
phone. It then writes the report to standard output. A refused app comes back
with the reason. `ark help apps` covers manifests and grants, and
`ark help datasets` covers the data commands.
