# 03 - cilantro-mini (C)

The same app as [cilantro-mini (Rust)](../03-cilantro-mini-rust), in C: read one
variant through the `rsids/` lens and report whether cilantro likely tastes
soapy. For the full report version, see
[cilantro-soapiness](../03-cilantro-soapiness).

## Build

C apps use [wasi-sdk](https://github.com/WebAssembly/wasi-sdk/releases) (clang
plus the WASI C library), not Emscripten: its standard WASI output is what lets
`fopen` reach the files the Ark mounts. With `WASI_SDK` pointing at the install:

```sh
$WASI_SDK/bin/clang --target=wasm32-wasip1 -O3 main.c -o app.wasm
```

## Run

```sh
make run APP=03-cilantro-mini-c
```
