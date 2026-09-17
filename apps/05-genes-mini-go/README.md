# 05 - genes-mini (Go)

[05-genes-mini-rust](../05-genes-mini-rust) written in Go. It takes the sequence
length from `os.Stat` and tells an absent scalar from a failed read with
`errors.Is(err, fs.ErrNotExist)`.

## Build and run

```sh
make run APP=05-genes-mini-go
```
