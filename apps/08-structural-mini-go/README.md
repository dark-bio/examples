# 08 - structural-mini (Go)

[08-structural-mini-rust](../08-structural-mini-rust) written in Go.
`os.ReadDir` sorts the changes by name as text, so the app sorts them again by
span. It takes the new bases from `os.Stat` on `bases`, and tells an absent leaf
from a failed read with `errors.Is(err, fs.ErrNotExist)`.

## Build and run

```sh
make run APP=08-structural-mini-go
make run APP=08-structural-mini-go FIXTURES=fixtures/no-call
make run APP=08-structural-mini-go FIXTURES=fixtures/unanswered
```
