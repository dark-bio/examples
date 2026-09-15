# 05 - genes-mini

The app grants one gene, `v1/genome/genes/TAS2R38`, and prints its chromosome,
start, end, strand, biotype and sequence length. The length comes from the size
of `sequence`, which always holds end - start + 1 bases, so the gene never has
to fit in memory. An absent value prints as no answer, while a missing sequence
is an error.

The gene grant also covers the owner's `changes` in it, even though this mini
doesn't read them. [05-bitter-meter](../05-bitter-meter) does.

## Build and run

```sh
make run APP=05-genes-mini
```
