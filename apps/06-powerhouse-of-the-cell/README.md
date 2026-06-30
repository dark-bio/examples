# 06 - powerhouse-of-the-cell (full)

An interval through the `regions/` lens. It reads the entire mitochondrial genome
as a single region (`v1/genome/regions/chrM/1-16569`) and prints a report over it:
GC content, the non-reference changes you carry, density windows, and a text
"star map" of where the variants fall.

Read [regions-mini](../06-regions-mini) first for the bare lens read; this is the
full report. Mitochondrial DNA is inherited only from your mother, and is the
cell's energy organelle, hence the name. The app is a data-shape demo, not a
haplogroup caller, ancestry test, or medical report.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=06-powerhouse-of-the-cell
```
