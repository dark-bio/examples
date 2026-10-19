# 05 - bitter-meter

A full report on TAS2R38, the bitter taste receptor gene. It streams the gene's
`reference` for its GC content, and lists the owner's `changes` inside the
gene with each one's reference base and genotype. It's a data demonstration,
not a taste prediction.

It shows how to walk `changes` safely. Each entry is named after the span it
changes, and entries sharing a span add `-2`, `-3` and so on. The call comes
from the entry's `carried` count and the genotype at its first position. A
listing that fails, including with "file too large", stops the app instead of
reading as no changes.

## Build and run

```sh
make run APP=05-bitter-meter
```
