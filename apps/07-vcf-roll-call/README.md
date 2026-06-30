# 07 - vcf-roll-call (full)

Scanning the raw variant file. Unlike the lens apps, this asks for the whole
`v1/genome/snp-indel` slot and streams the decompressed `vcf` view with
[`noodles-vcf`](https://crates.io/crates/noodles-vcf), printing a coverage and QC
report: record and sample counts, variant kinds, reference blocks, substitution
ratios, and missing calls.

This is the broad-access tier. The manifest asks for the entire variant file,
which is what the owner sees at approval, very different from "one variant." The
grok version is [vcf-mini](../07-vcf-mini), which walks the same file with no
genomics library. This is the full scan.

It is not a formal coverage calculator; read depth from BAM/CRAM is out of scope.

## Build

This app has one dependency, `noodles-vcf`, which `cargo` fetches automatically.

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=07-vcf-roll-call
```
