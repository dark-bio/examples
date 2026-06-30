# 03 - cilantro-mini (Go)

The same app as [cilantro-mini (Rust)](../03-cilantro-mini-rust), in Go: read one
variant through the `rsids/` lens and report whether cilantro likely tastes
soapy. For the full report version, see
[cilantro-soapiness](../03-cilantro-soapiness).

## Build

```sh
GOOS=wasip1 GOARCH=wasm go build -o app.wasm .
```

## Run

```sh
make run APP=03-cilantro-mini-go
```
