# 08 - motif-finder

The app streams a gene's reference `sequence` and counts where five restriction
enzymes would cut it. It keeps a four-base window across chunks, so a site
spanning two chunks still counts, and uppercases bases because repeats are
soft-masked.

It reads only public sequence, but its grant, `v1/genome/genes/TAS2R38`, also
covers the owner's `changes` in the gene. The owner approves the grant, not what
the code happens to read.

## Build and run

```sh
make run APP=08-motif-finder
```
