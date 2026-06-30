# 03 - cilantro-mini (C)

The same app as [cilantro-mini (Rust)](../03-cilantro-mini-rust), in C: read one
variant through the `rsids/` lens and report whether cilantro likely tastes
soapy. For the full report version, see
[cilantro-soapiness](../03-cilantro-soapiness).

## Build

C apps use a WASI clang/sysroot toolchain, not Emscripten: standard WASI output
is what lets `fopen` reach the files the Ark mounts.

From the repository root, the Makefile auto-detects either `/opt/wasi-sdk` or
Homebrew's split WASI toolchain:

```sh
make build APP=03-cilantro-mini-c
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

## Run

```sh
make run APP=03-cilantro-mini-c
```
