# 06 - regions-mini (Rust)

The app grants one interval, the whole mitochondrial genome at
`v1/genome/regions/chrM/1-16569`, and streams its `sequence` in chunks to report
its length and GC content, so memory use doesn't grow with the interval. A
missing sequence or a failed read stops the app.

The interval grant also covers the owner's `changes`, which
[06-powerhouse-of-the-cell](../06-powerhouse-of-the-cell) lists.

The same mini exists in [Go](../06-regions-mini-go),
[C](../06-regions-mini-c) and [Python](../06-regions-mini-python).

## Build and run

```sh
make run APP=06-regions-mini-rust
```
