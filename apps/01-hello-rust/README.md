# 01 - hello, Rust

The smallest Ark app. It prints its manifest when run with no arguments and a
greeting when given a data directory, and it reads no data. Start here, then
compare the [Go](../01-hello-go) and [C](../01-hello-c) versions.

## Build and run

```sh
make run APP=01-hello-rust
```

To see each pass by hand, build with Cargo and run the module twice:

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
wasmtime target/wasm32-wasip1/release/hello-rust.wasm     # manifest pass
wasmtime target/wasm32-wasip1/release/hello-rust.wasm /   # run pass
```
