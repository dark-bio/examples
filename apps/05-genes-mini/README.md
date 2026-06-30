# 05 - genes-mini

One gene through the `genes/` lens, in the fewest lines. It reads the scalar
metadata leaves of `v1/genome/genes/TAS2R38` (chromosome, start, end, strand,
biotype) and the length of its reference sequence, then prints them.

This is the grok version of the genes lens. For the same data turned into a full
report, see [bitter-meter](../05-bitter-meter).

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=05-genes-mini
```
