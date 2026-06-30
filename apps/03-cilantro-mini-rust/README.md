# 03 - cilantro-mini (Rust)

Read one variant through the `rsids/` lens, in as few lines as possible. This is
the grok version: the smallest real data access there is. The app asks for a
single directory, and the lens hands back the genotype and coordinate as plain
files. It never opens a variant file.

For the same read wrapped in a full report (a verdict, tables, references, a
technical-details section), see [cilantro-soapiness](../03-cilantro-soapiness).
The data access is identical; the difference is all presentation. The same mini
is also written in [Go](../03-cilantro-mini-go) and [C](../03-cilantro-mini-c).

Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2 olfactory
receptor, tracks the trait; the more copies of the `C` allele you carry, the more
soapy cilantro tends to taste (Eriksson et al., 2012).

The manifest asks for exactly one path:

```toml
datasets = ["v1/genome/rsids/rs72921001"]
```

so the owner approves access to one variant, not the whole variant file. The
`rsids/` lens reports alleles as plain bases, so there is no file format to parse
and no REF/ALT index to decode.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=03-cilantro-mini-rust
```
