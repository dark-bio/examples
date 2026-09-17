# 01 - hello, C

[01-hello-rust](../01-hello-rust) written in C. C apps build with a WASI clang
toolchain rather than Emscripten, because Emscripten's standalone modules can't
open the files an Ark mounts.

The Makefile finds a wasi-sdk install at `/opt/wasi-sdk`, or falls back to
Homebrew's WASI toolchain. `WASI_SDK` points it at any other clang that carries
a WASI sysroot:

```sh
make run APP=01-hello-c WASI_SDK=/path/to/wasi-sdk
```

## Build and run

```sh
make run APP=01-hello-c
```

To see each pass by hand, build the module and run it twice:

```sh
make build APP=01-hello-c
wasmtime build/01-hello-c.wasm     # manifest pass
wasmtime build/01-hello-c.wasm /   # run pass
```
