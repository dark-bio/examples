# Fixtures

This tree stands in for the data an Ark would mount, so the examples can run on
a laptop. It is a few kilobytes of plain text, not a genome.

It works because the Ark serves plain files, and the files an app reads are
small: a genotype is a string like `C|C`, a position is a number. The synthesized
lenses (`rsids/`, `genes/`, `regions/`) hand back exactly these short leaves, and
even the raw variant view is plain text. So the whole tree is hand written.

What is real and what is not:

- **Reference data is real.** Variant coordinates, reference alleles, gene spans,
  and reference sequence are public facts, copied from public sources.
- **Genotypes are invented.** The calls attributed to "you" are chosen to make
  each example produce an interesting result. They are not anyone's data.

Because the leaves are authored by hand, the fixtures exercise an app's reading
and logic, not the device's synthesis of these answers from raw data. The bytes
an app reads here are the same shape it reads on a device. See
../docs/05-running-locally.md for what that does and does not prove.
