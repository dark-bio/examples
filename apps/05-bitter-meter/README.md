# 05 - bitter-meter

A full report on TAS2R38, the bitter taste receptor gene. It streams the gene's
`sequence` for its GC content, and lists the owner's `changes` inside the gene
with each one's reference allele and genotype. It's a data demonstration, not a
taste prediction.

It shows how to walk `changes` safely. A listed position without leaves still
counts, since several records start there. Genotypes are split around symbolic
and breakend alleles. A listing that fails, including with "file too large",
stops the app instead of reading as no changes.

## Build and run

```sh
make run APP=05-bitter-meter
```
