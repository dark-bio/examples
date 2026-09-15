# 01 - hello, Go

[01-hello-rust](../01-hello-rust) written in Go, which builds for WASI from
version 1.24.

## Build and run

```sh
make run APP=01-hello-go
```

By hand:

```sh
GOOS=wasip1 GOARCH=wasm go build -o app.wasm .
wasmtime app.wasm     # manifest pass
wasmtime app.wasm /   # run pass
```

[TinyGo](https://tinygo.org/) builds a module about five times smaller, with
`tinygo build -target=wasip1 -o app.wasm .`.
