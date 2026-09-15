# 03 - cilantro-mini (Go)

[03-cilantro-mini-rust](../03-cilantro-mini-rust) written in Go. It tells an
absent genotype from a failed read with `errors.Is(err, fs.ErrNotExist)`.

## Build and run

```sh
make run APP=03-cilantro-mini-go
make run APP=03-cilantro-mini-go FIXTURES=fixtures/no-call
make run APP=03-cilantro-mini-go FIXTURES=fixtures/unanswered
```
