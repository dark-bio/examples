# 07 - vcf-mini (Rust)

The app grants the whole call file, `v1/genome/snp-indel`, and streams `vcf` one
line at a time to count its header lines and records. A whole-genome call file
runs to gigabytes, far past an app's 100 MiB of memory, so it's never loaded
whole. A failed read stops the app instead of reporting zero records.
[07-vcf-roll-call](../07-vcf-roll-call) parses the same file with a library.

It reads through a 64 KiB buffer into one reused line. Both matter over
millions of records, since every refill leaves the sandbox and a fresh line per
record would allocate for each one.

The same mini exists in [Go](../07-vcf-mini-go), [C](../07-vcf-mini-c) and
[Python](../07-vcf-mini-python).

## Build and run

```sh
make run APP=07-vcf-mini-rust
```
