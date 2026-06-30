# 03 - cilantro-soapiness (full)

The full version of the cilantro example. It reads the same single variant as
the [mini](../03-cilantro-mini-rust), through the same `rsids/` lens, but wraps
it in a complete report: a verdict with a copy-count table, caveats, references,
and a technical-details section with the resolved locus.

The point of the pairing is the diff. The mini is the lens read in a dozen lines.
This one is what shipping that read as a report someone wants to run looks like:
the data access is identical, everything else is presentation. Read the mini
first to see the access pattern, then read this to see the polish.

Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2 olfactory
receptor, tracks the trait; the more copies of the `C` allele, the more soapy
cilantro tends to taste (Eriksson et al., 2012).

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=03-cilantro-soapiness
```
