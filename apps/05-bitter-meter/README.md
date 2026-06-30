# 05 - bitter-meter (full)

A whole gene through the `genes/` lens. It reads `v1/genome/genes/TAS2R38`, the
bitter-taste receptor behind whether coffee, broccoli, and brussels sprouts come
across as harsh, and reports the gene's metadata, its reference length and GC
content, the non-reference changes you carry, and a ruler of where they fall
across the gene.

Read [genes-mini](../05-genes-mini) first for the bare lens read; this is the
same data turned into a full report.

This is a curiosity demo, not a taste prediction and not medical advice.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=05-bitter-meter
```
