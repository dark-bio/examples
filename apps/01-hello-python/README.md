# 01 - hello, Python

[01-hello-rust](../01-hello-rust) written in Python. The module carries a
trimmed CPython along with the app, so it is megabytes where the others are
kilobytes, and it takes noticeably longer to start on an Ark.

The first Python build compiles that interpreter once and reuses it for every
Python app. [05-running.md](../../docs/05-running.md) covers what it needs.

## Build and run

```sh
make run APP=01-hello-python
```

To see each pass by hand, build the module and run it twice:

```sh
make build APP=01-hello-python
wasmtime build/01-hello-python.wasm     # manifest pass
wasmtime build/01-hello-python.wasm /   # run pass
```
