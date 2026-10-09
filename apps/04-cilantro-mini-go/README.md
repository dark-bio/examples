# 04 - cilantro-mini (Go)

[04-cilantro-mini-rust](../04-cilantro-mini-rust) written in Go. It distinguishes
an absent genotype from a failed read with `errors.Is(err, fs.ErrNotExist)`.

## Build and run

```sh
make run APP=04-cilantro-mini-go
make run APP=04-cilantro-mini-go FIXTURES=fixtures/no-call
make run APP=04-cilantro-mini-go FIXTURES=fixtures/unanswered
```
