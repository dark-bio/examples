# 07 - copies-mini (Go)

[07-copies-mini-rust](../07-copies-mini-rust) written in Go. `os.ReadDir`
sorts the segments by name as text, so the app sorts them again by their
numeric start. It tells an absent `count` from a failed read with
`errors.Is(err, fs.ErrNotExist)`.

## Build and run

```sh
make run APP=07-copies-mini-go
make run APP=07-copies-mini-go FIXTURES=fixtures/no-call
make run APP=07-copies-mini-go FIXTURES=fixtures/unanswered
```
