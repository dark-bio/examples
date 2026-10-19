# 07 - copies-mini (C)

[07-copies-mini-rust](../07-copies-mini-rust) written in C. It lists the
segments with `readdir`, skipping `.` and `..`, sorts them by numeric start
with `qsort`, and reads each `count` into a buffer that grows to fit. Only
`errno == ENOENT` means no answer. [01-hello-c](../01-hello-c) covers the
toolchain.

## Build and run

```sh
make run APP=07-copies-mini-c
make run APP=07-copies-mini-c FIXTURES=fixtures/no-call
make run APP=07-copies-mini-c FIXTURES=fixtures/unanswered
```
