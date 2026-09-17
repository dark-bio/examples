# 07 - vcf-mini (C)

[07-vcf-mini-rust](../07-vcf-mini-rust) written in C. It reads the call file
with `getline`, which accepts long records while keeping only one line in
memory, over a 64 KiB `setvbuf` buffer. The default buffer is small enough to
make a whole-genome scan an order of magnitude slower.
[01-hello-c](../01-hello-c) covers the toolchain.

## Build and run

```sh
make run APP=07-vcf-mini-c
```
