# 07 - copies-mini (Rust)

The app grants one gene, `v1/genome/genes/UGT2B17`, and lists its `copies`,
one directory per segment of the owner's copy number file that reaches the
gene. It sorts the segments by start, since listing order isn't promised, and
prints each one's `count`. An empty listing prints as no answer, never as zero
copies, since a caller can leave a region out.

A whole gene can go missing, and only a copy number file shows it. A small
variant file has nothing to say about a gene that isn't there.

The same mini exists in [Go](../07-copies-mini-go),
[C](../07-copies-mini-c) and [Python](../07-copies-mini-python), and
[07-doping-test](../07-doping-test) turns it into a report.

## Build and run

```sh
make run APP=07-copies-mini-rust
make run APP=07-copies-mini-rust FIXTURES=fixtures/no-call
make run APP=07-copies-mini-rust FIXTURES=fixtures/unanswered
```
