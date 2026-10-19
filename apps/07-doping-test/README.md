# 07 - doping-test

A full report on *UGT2B17*, the gene whose enzyme readies testosterone for
urine, where the classic doping test reads it. A common deletion removes the
whole gene. The app counts the owner's copies from the gene's `copies`, and
answers only when one count covers every base of the gene. It's a data
demonstration, not a doping verdict.

It also grants `v1/genome/datasets`, the public list of the owner's uploads,
to tell a genome with no copy number file from one whose caller left the gene
out. Neither reads as zero copies.

## Build and run

```sh
make run APP=07-doping-test
make run APP=07-doping-test FIXTURES=fixtures/no-call
make run APP=07-doping-test FIXTURES=fixtures/unanswered
```
