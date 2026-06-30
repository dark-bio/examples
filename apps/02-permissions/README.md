# 02 - permissions

How an app's data access is bounded, and what the `develop` flag does.

The app declares one variant directory and reads it (works), then deliberately
reaches for a slot it did not declare (`v1/genome/snp-indel/vcf`). The Ark mounts
only what an app declares, so the second read fails. The local runner mounts only
the declared datasets too, so you see the same boundary here.

The manifest also sets `develop = true`. On a device that is the debugging
switch: with it, standard output is returned even on failure and standard error
is always returned; without it, a failed run returns nothing and stderr is never
returned. The local runner always shows both, so this is the one spot where local
and device behavior differ. The `eprintln!` line is there to make stderr visible.
Ship without `develop`.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=02-permissions
```
