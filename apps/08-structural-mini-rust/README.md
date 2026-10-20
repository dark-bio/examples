# 08 - structural-mini (Rust)

The app grants a 101-base window inside the ACE gene,
`v1/genome/regions/chr17/63488479-63488579`, and lists its `changes`, one
directory per alternate allele the owner carries there. It sorts them by span
and prints each one's `type`, how many new bases it brings, the `dataset` it
came from and how many copies carry it.

The window holds the ACE insertion on GRCh38, nearly 300 bases of Alu repeat.
That's usually too long for a small variant caller, so it comes from the
structural variant file,
and it reads like any other change. The window is wider than the insertion's
anchor, since callers can place an insertion a few bases apart. The app also
grants `v1/genome/datasets/structural-variants`, a public folder that reads
empty, so the Ark won't run it without that file. On `fixtures/unanswered` it
never starts, just as on an Ark without one.

The same mini exists in [Go](../08-structural-mini-go),
[C](../08-structural-mini-c) and [Python](../08-structural-mini-python), and
[08-sprint-or-marathon](../08-sprint-or-marathon) turns it into a report.

## Build and run

```sh
make run APP=08-structural-mini-rust
make run APP=08-structural-mini-rust FIXTURES=fixtures/no-call
make run APP=08-structural-mini-rust FIXTURES=fixtures/unanswered
```
