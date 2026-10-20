# 07 - doping-test

A full report on *UGT2B17*, the gene whose enzyme readies testosterone for
urine, where the classic doping test reads it. A common deletion removes the
whole gene. The app counts the owner's copies from the gene's `copies`, and
answers only when one count covers every base of the gene. It's a data
demonstration, not a doping verdict.

It also grants `v1/genome/datasets/copy-number-variants`, a public folder that
reads empty, so the Ark won't run it without a copy number file. A gene the
caller left out still reads as no answer, never as zero copies. On
`fixtures/unanswered` the app never starts, just as on an Ark without that
file.

## Build and run

```sh
make run APP=07-doping-test
make run APP=07-doping-test FIXTURES=fixtures/no-call
make run APP=07-doping-test FIXTURES=fixtures/unanswered
```
