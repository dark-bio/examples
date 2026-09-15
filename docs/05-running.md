# Running apps

You don't need an Ark to write an app. A WebAssembly runtime and the fixture
data in this repository run both passes on a laptop. When the app works, the
`ark` command line tool runs it on a real Ark.

## Locally

### Toolchains

- [`wasmtime`](https://wasmtime.dev/) runs the modules.
- Rust needs `rustup` with the `wasm32-wasip1` target.
- Go needs version 1.24 or newer.
- C needs a WASI clang, either a [wasi-sdk](https://github.com/WebAssembly/wasi-sdk)
  install at `/opt/wasi-sdk` or Homebrew's `llvm`, `lld`, `wasi-libc` and
  `wasi-runtimes`. Set `WASI_SDK` for an install elsewhere.

On macOS with Homebrew:

```sh
brew install wasmtime go llvm lld wasi-libc wasi-runtimes
rustup target add wasm32-wasip1
```

C apps build with a WASI toolchain rather than Emscripten, because Emscripten's
standalone modules can't open the files the Ark mounts.

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
