# Running locally

You do not need an Ark to develop an app. A WebAssembly runtime and a small tree
of stand-in data are enough to run both passes on your laptop.

## What you need

- [`wasmtime`](https://wasmtime.dev/), the runtime that runs the module.
- A toolchain for your language: `rustup` with the `wasm32-wasip1` target for
  Rust, Go 1.24+ for Go, or a WASI C toolchain for C. Each example's README has
  the exact build command.

C is built with a WASI clang/sysroot toolchain rather than Emscripten on purpose:
standard WASI modules can read the mounted data the Ark provides. Emscripten's
standalone output cannot reach those files. The Makefile auto-detects either an
upstream [wasi-sdk](https://github.com/WebAssembly/wasi-sdk) install at
`/opt/wasi-sdk` or Homebrew's split WASI toolchain (`llvm`, `lld`, `wasi-libc`,
and `wasi-runtimes`).

On macOS with Homebrew, these packages cover the local runner, Go examples, and
C examples:

```sh
brew install wasmtime go llvm lld wasi-libc wasi-runtimes
```

Install Rust through `rustup` if you do not already have it, then add the WASI
target:

```sh
rustup target add wasm32-wasip1
```

## make run

`make run` builds every app and runs both passes for each one:

```sh
make run
```

To run just one app, set `APP`:

```sh
make run APP=03-cilantro-soapiness
```

`make build` follows the same selection rule: without `APP` it builds every app;
with `APP=<name>` it builds only that app.

Every app builds to a single module at `build/<app>.wasm`, whatever its language,
so the compiled output for the whole repo lives in one `build/` directory.

Under the hood `run` does what the device does, via `tools/run.sh`:

1. Run the module with no arguments and capture the manifest it prints.
2. Read the `datasets` list, and mount only those paths from `fixtures/`,
   read-only, each at its own location.
3. Run the module again with `/` as its first argument.

Because only the declared datasets are mounted, an app sees exactly what it
asked for, the same as on a device. That is why
[permissions](../apps/02-permissions) can demonstrate a blocked read locally: the
undeclared path is never mounted.

For C apps, the Makefile first looks for `/opt/wasi-sdk/bin/clang`, then falls
back to `brew --prefix llvm` when Homebrew LLVM is installed. Only set `WASI_SDK`
for a non-standard install:

```sh
make run APP=03-cilantro-mini-c WASI_SDK=/path/to/wasi-sdk
```

## The fixtures

`fixtures/` is a few kilobytes of plain text standing in for the data an Ark
would mount. It works because BioFS serves plain files and the values an app
reads are small (see [03-biofs-paths.md](03-biofs-paths.md)). What is real and
what is not:

- **Reference data is real.** Variant coordinates, reference alleles, gene spans,
  and reference sequence are public facts from public sources (Ensembl, dbSNP).
- **Genotypes are invented.** The calls attributed to "you" are chosen to make
  each example produce an interesting result. They are not anyone's data.

The lens leaves are authored directly, so the fixtures exercise an app's reading
and logic, not the device's synthesis of those answers from raw slots. To add a
case, create the leaf files an app reads; for example, a new rsID is just a
directory under `fixtures/v1/genome/rsids/` with `genotype`, `chromosome`,
`position`, and `reference` files.

## What local runs do not reproduce

The local runner is faithful about which data an app can reach, but a real Ark
enforces more that `wasmtime` does not:

- **Resource limits.** The device caps memory, output size, and the metadata
  pass's runtime. Locally there are no such caps.
- **A deterministic sandbox.** The device replaces the RNG with zeros and the
  clocks with counters. A stock `wasmtime` does not, so an app that reads the
  clock or RNG behaves differently here than on a device. Write apps that do not
  depend on either (see [fortune-cookie](../apps/09-fortune-cookie)).
- **Output gating.** The device returns an app's output only on success, unless
  the manifest sets `develop = true`. Locally you always see stdout and stderr.

None of these affect whether your data-access logic is correct, which is what the
fixtures are for.
