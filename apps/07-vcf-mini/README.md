# 07 - vcf-mini

Reading the raw variant file, in the fewest lines. It asks for the
`v1/genome/snp-indel` slot and opens its decompressed `vcf` view, which is plain
text, so it just counts header lines and records and shows the first one. No
genomics library.

This is the grok version of raw-file access. For a real scan with `noodles-vcf`,
see [vcf-roll-call](../07-vcf-roll-call).

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=07-vcf-mini
```
