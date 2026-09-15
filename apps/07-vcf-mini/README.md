# 07 - vcf-mini

The app grants the whole call file, `v1/genome/snp-indel`, and streams `vcf` one
line at a time to count its header lines and records. A whole-genome call file
runs to gigabytes, far past an app's 100 MiB of memory, so it's never loaded
whole. A failed read stops the app instead of reporting zero records.
[07-vcf-roll-call](../07-vcf-roll-call) parses the same file with a library.

## Build and run

```sh
make run APP=07-vcf-mini
```
