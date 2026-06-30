# 01 - hello, C

The same app as [01-hello-rust](../01-hello-rust), written in C. It prints its
manifest on the manifest pass and a greeting on the run pass, and reads no data.

## Build

C apps use a WASI clang/sysroot toolchain. It produces a standard WASI module,
so file reads resolve against the data the Ark mounts; Emscripten's standalone
output cannot reach those files, so it is not used here.

From the repository root, the Makefile auto-detects either `/opt/wasi-sdk` or
Homebrew's split WASI toolchain:

```sh
make build APP=01-hello-c
```

To compile by hand with upstream
[wasi-sdk](https://github.com/WebAssembly/wasi-sdk/releases):

```sh
$WASI_SDK/bin/clang --target=wasm32-wasip1 -O3 main.c -o app.wasm
```

Or with Homebrew's toolchain:

```sh
$(brew --prefix llvm)/bin/clang --target=wasm32-wasip1 -O3 main.c -o app.wasm
```

For a non-standard upstream install, override with
`make build APP=01-hello-c WASI_SDK=/path/to/wasi-sdk`.

## Run

From the repository root:

```sh
make run APP=01-hello-c
```

Or by hand:

```sh
wasmtime app.wasm          # manifest pass
wasmtime app.wasm /        # run pass
```
