# 01 - hello, C

[01-hello-rust](../01-hello-rust) written in C. C apps build with a WASI clang
toolchain rather than Emscripten, because Emscripten's standalone modules can't
open the files an Ark mounts.

## Build and run

The Makefile finds a wasi-sdk install at `/opt/wasi-sdk` or Homebrew's WASI
toolchain, and `WASI_SDK` points it anywhere else:

```sh
make run APP=01-hello-c
make run APP=01-hello-c WASI_SDK=/path/to/wasi-sdk
```

By hand:

```sh
$WASI_SDK/bin/clang --target=wasm32-wasip1 -O3 main.c -o app.wasm
wasmtime app.wasm     # manifest pass
wasmtime app.wasm /   # run pass
```
