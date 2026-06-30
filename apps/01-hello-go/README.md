# 01 - hello, Go

The same app as [01-hello-rust](../01-hello-rust), written in Go. It prints its
manifest on the manifest pass and a greeting on the run pass, and reads no data.

## Build

Install [Go](https://go.dev/dl/) (1.24 or newer), then build for WASI:

```sh
GOOS=wasip1 GOARCH=wasm go build -o app.wasm .
```

For a binary roughly five times smaller, [TinyGo](https://tinygo.org/) works
too:

```sh
tinygo build -target=wasip1 -o app.wasm .
```

## Run

From the repository root:

```sh
make run APP=01-hello-go
```

Or by hand:

```sh
wasmtime app.wasm          # manifest pass
wasmtime app.wasm /        # run pass
```
