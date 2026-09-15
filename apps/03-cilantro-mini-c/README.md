# 03 - cilantro-mini (C)

[03-cilantro-mini-rust](../03-cilantro-mini-rust) written in C. Only
`errno == ENOENT` means no answer, and any other failure to open or read a file
stops the app. Each file is read into a buffer that grows to fit, since a
genotype can hold long alleles. [01-hello-c](../01-hello-c) covers the
toolchain.

## Build and run

```sh
make run APP=03-cilantro-mini-c
make run APP=03-cilantro-mini-c FIXTURES=fixtures/no-call
make run APP=03-cilantro-mini-c FIXTURES=fixtures/unanswered
```
