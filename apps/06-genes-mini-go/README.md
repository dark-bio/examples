# 06 - genes-mini (Go)

[06-genes-mini-rust](../06-genes-mini-rust) written in Go. It takes the sequence
length from `os.Stat` and distinguishes an absent scalar from a failed read
with `errors.Is(err, fs.ErrNotExist)`.

## Build and run

```sh
make run APP=06-genes-mini-go
```
