# 01 - hello, Rust

The smallest Ark app: it prints its manifest on the manifest pass, and a
greeting on the run pass. It reads no data. Start here, then read the Go and C
versions to see the same shape in another language.

## Build

Install [Rust](https://rustup.rs/) and the WebAssembly target:

```sh
rustup target add wasm32-wasip1
```

Then:

```sh
cargo build --release --target wasm32-wasip1
```

The module lands at `target/wasm32-wasip1/release/hello-rust.wasm`.

## Run

From the repository root:

```sh
make run APP=01-hello-rust
```

Or by hand, to see each pass:

```sh
wasmtime target/wasm32-wasip1/release/hello-rust.wasm          # manifest pass
wasmtime target/wasm32-wasip1/release/hello-rust.wasm /        # run pass
```
