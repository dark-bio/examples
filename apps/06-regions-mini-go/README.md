# 06 - regions-mini (Go)

[06-regions-mini-rust](../06-regions-mini-rust) written in Go. It reads the
sequence in 8 KiB chunks with `os.File.Read`, so the interval never has to fit
in memory.

## Build and run

```sh
make run APP=06-regions-mini-go
```
