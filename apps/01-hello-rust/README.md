# 01 - hello, Rust

The smallest Ark app. It prints its manifest when run with no arguments and a
greeting when given a data directory, and it reads no data. Start here, then
compare the [Go](../01-hello-go), [C](../01-hello-c) and
[Python](../01-hello-python) versions.

## Build and run

```sh
make run APP=01-hello-rust
```

To see each pass by hand, build the module and run it twice:

```sh
make build APP=01-hello-rust
wasmtime build/01-hello-rust.wasm     # manifest pass
wasmtime build/01-hello-rust.wasm /   # run pass
```
