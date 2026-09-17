# 01 - hello, Go

[01-hello-rust](../01-hello-rust) written in Go. The Makefile builds Go apps
with TinyGo, which produces a module a fraction of the size the standard
toolchain emits.

## Build and run

```sh
make run APP=01-hello-go
```

To see each pass by hand, build the module and run it twice:

```sh
make build APP=01-hello-go
wasmtime build/01-hello-go.wasm     # manifest pass
wasmtime build/01-hello-go.wasm /   # run pass
```
