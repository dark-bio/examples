# 07 - vcf-roll-call

The app streams the whole call file with
[`noodles-vcf`](https://crates.io/crates/noodles-vcf) and reports on its records,
first-sample genotypes, allele shapes, filters, depth and quality. A malformed
record is counted and skipped, while a failed read stops the app, so a damaged
file can't pass as a partial success. The report describes the file, not how
much of a genome was sequenced.

## Build and run

```sh
make run APP=07-vcf-roll-call
```
