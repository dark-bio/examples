# 07 - regions-mini (Go)

[07-regions-mini-rust](../07-regions-mini-rust) written in Go. It reads the
sequence in 8 KiB chunks with `os.File.Read`, so the interval never has to fit
in memory.

## Build and run

```sh
make run APP=07-regions-mini-go
```
