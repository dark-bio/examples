# 08 - motif-finder

Reading reference sequence, with no genotype at all. It reads the reference
sequence of `v1/genome/genes/TAS2R38` and scans it for a small panel of
restriction enzyme sites, printing how many times each one cuts.

The point is that the reference is just a string of bases. An app can compute
over it without reading anything personal: this one never opens a genotype. It is
also the cleanest example of an app that produces a useful result from public
data alone.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=08-motif-finder
```
