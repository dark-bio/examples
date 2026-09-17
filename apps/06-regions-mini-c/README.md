# 06 - regions-mini (C)

[06-regions-mini-rust](../06-regions-mini-rust) written in C. It reads the
sequence in 8 KiB chunks with `fread`, so the interval never has to fit in
memory. [01-hello-c](../01-hello-c) covers the toolchain.

## Build and run

```sh
make run APP=06-regions-mini-c
```
