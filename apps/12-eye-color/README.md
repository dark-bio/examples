# 12 - eye-color

Composes an eye color report from two apps. It pins
[12-pigment-variants](../12-pigment-variants) as `gather` and
[12-eye-color-model](../12-eye-color-model) as `model`. The owner approves
their six rsID grants together and chooses `summary` or `full` detail.

## Build and run

```sh
make run APP=12-eye-color
make run APP=12-eye-color FIXTURES=fixtures/no-call
make run APP=12-eye-color FIXTURES=fixtures/unanswered
mkdir -p build/full-detail
printf full > build/full-detail/detail
make run APP=12-eye-color INPUTS=build/full-detail
```

The sample `inputs/detail` holds `summary`, the manifest's default, without
a trailing newline. The default fixtures give a blue probability of 94.6%.
The other roots give no estimate, with their absent or uncalled genotypes
shown in the evidence. The report describes an association with eye color.
It states that the model separates blue from brown well and calls
intermediate colors, such as green and hazel, poorly.

## Files and bindings

The gatherer's `order` binds to the literal `rsid`. It writes six rows to
`outputs/variants`, in its documented
[TSV format](../12-pigment-variants#the-variants-output).

The model's `variants` binds to `gather/variants`, and its `detail` binds to
this app's `detail`. The model reads these files under `inputs/` and writes
`outputs/colors`. Its [README](../12-eye-color-model) documents the color
probabilities, the unavailable answer, the regression and its source.

Eye Color reads `apps/model/colors` for the finding and
`apps/gather/variants` for the evidence. It reads no grants itself. Each
dependency runs before the app that consumes its files. The main app prints
the report after both dependencies finish.

## Pins at build time

The `pins` file beside `Cargo.toml` lists each Rust constant name and its
dependency app, one `NAME app` pair per line. `tools/build.sh` builds those
apps first and writes their optimized module digests to `src/pins.rs`, which
the main source includes. `make build APP=12-eye-color` refreshes this
generated file, which Git ignores and `make clean` removes.

The local runner finds dependencies by digest among `build/*.wasm` and runs
the complete chain. The gatherer and model each have sample inputs for
debugging alone with `make run APP=<app>`.
