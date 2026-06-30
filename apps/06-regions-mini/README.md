# 06 - regions-mini

One interval through the `regions/` lens, in the fewest lines. It reads the
reference sequence of `v1/genome/regions/chrM/1-16569` (the whole mitochondrial
genome) and reports its length and GC content.

This is the grok version of the regions lens. For the same data turned into a
full report, see [powerhouse-of-the-cell](../06-powerhouse-of-the-cell).

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=06-regions-mini
```
