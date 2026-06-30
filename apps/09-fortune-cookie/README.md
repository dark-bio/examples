# 09 - fortune-cookie

Why the sandbox is deterministic. The runtime gives an app no randomness: the
RNG returns zeros and the clock is a counter, not the time of day (see
[../docs/01-app-model.md](../../docs/01-app-model.md)). So an app cannot roll
dice. To make a choice that feels random, it must derive it from its input.

This one folds a few of your genotypes into a small hash and picks a fortune from
it. The same genome gives the same fortune every run. That is the lesson: nothing
varies that is not in the data, which is exactly what makes runs reproducible and
auditable.

## Build

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

## Run

```sh
make run APP=09-fortune-cookie
```
