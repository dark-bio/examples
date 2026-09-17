# 07 - vcf-mini (Go)

[07-vcf-mini-rust](../07-vcf-mini-rust) written in Go. It reads the call file
with `bufio.Reader.ReadString`, which accepts long records while keeping only
one line in memory, over a 64 KiB buffer from `bufio.NewReaderSize`.

## Build and run

```sh
make run APP=07-vcf-mini-go
```
