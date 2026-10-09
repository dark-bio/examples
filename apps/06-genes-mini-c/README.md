# 06 - genes-mini (C)

[06-genes-mini-rust](../06-genes-mini-rust) written in C. It takes the sequence
length from `stat`, reads each scalar into a buffer that grows to fit, and
treats only `errno == ENOENT` as no answer. [01-hello-c](../01-hello-c) covers
the toolchain.

## Build and run

```sh
make run APP=06-genes-mini-c
```
