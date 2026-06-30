# 04 - drunk-o-type

A panel of variants through the `rsids/` lens. Where cilantro reads one site,
this reads eleven, scoring three axes of how your body handles alcohol (flush,
reward, histamine) from SNPs across ALDH2, ADH1B, OPRM1, DRD2, and more.

The access pattern is just cilantro repeated: one `rsids/<rsid>/` directory per
variant, every one declared in the manifest, each read as plain genotype text.
The lens itself is already covered by [cilantro-mini](../03-cilantro-mini-rust),
so this rung has no separate mini; it is the panel scale-up.

This is a demonstration, not medical advice.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=04-drunk-o-type
```
