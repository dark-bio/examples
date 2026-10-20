# 08 - structural-mini (C)

[08-structural-mini-rust](../08-structural-mini-rust) written in C. It lists
the changes with `readdir`, sorts them by span with `qsort`, and takes the new
bases from `stat` on `bases` without reading the allele. Only `errno == ENOENT`
means no answer. [01-hello-c](../01-hello-c) covers the toolchain.

## Build and run

```sh
make run APP=08-structural-mini-c
make run APP=08-structural-mini-c FIXTURES=fixtures/no-call
make run APP=08-structural-mini-c FIXTURES=fixtures/unanswered
```
