# 06 - genes-mini (Rust)

The app grants one gene, `v1/genome/genes/TAS2R38`, and prints its chromosome,
start, end, strand, biotype and sequence length. The length comes from the size
of `sequence`, which always holds end - start + 1 bases, so the gene never has
to fit in memory. An absent value prints as no answer, while a missing sequence
is an error.

The gene grant also covers the owner's `changes` in it, even though this mini
doesn't read them. [06-bitter-meter](../06-bitter-meter/README.md) does.

The same mini exists in [Go](../06-genes-mini-go/README.md),
[C](../06-genes-mini-c/README.md) and
[Python](../06-genes-mini-python/README.md).

## Build and run

```sh
make run APP=06-genes-mini-rust
```
