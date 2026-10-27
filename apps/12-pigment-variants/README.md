# 12 - pigment-variants

Reads the six IrisPlex variants and writes their genotypes for
[12-eye-color-model](../12-eye-color-model). Its typed `order` input also lets
it run alone and print a table. `position` sorts by chromosome number, then
position. `rsid` sorts by the numeric part of each rsID.

The app grants only those six rsIDs. Coordinates come from their files and
carry no assembly label, since the app does not read the reference build.
An absent genotype becomes `absent`, and a no-call stays `./.`. Other read
errors stop the app. Phased genotypes keep their pipes and appear below the
report's table, where Markdown leaves them intact.

## Build and run

```sh
make run APP=12-pigment-variants
make run APP=12-pigment-variants FIXTURES=fixtures/no-call
make run APP=12-pigment-variants FIXTURES=fixtures/unanswered
mkdir -p build/position-order
printf position > build/position-order/order
make run APP=12-pigment-variants INPUTS=build/position-order
```

The sample `inputs/order` holds `rsid`, without a trailing newline. On an Ark,
the owner picks the order when approving this app. When
[12-eye-color](../12-eye-color) uses it, a binding supplies `rsid` instead.

## The variants output

`outputs/variants` is UTF-8 text with six rows, no header and a final newline.
Each row has four tab-separated fields, in this order:

| Field | Contents |
| :-- | :-- |
| rsID | the variant identifier |
| chromosome | `chr` followed by the chromosome number |
| position | the 1-based position, in bases |
| genotype | the exact genotype file, or `absent` |

For example, one row is `rs12913832`, `chr15`, `28120472`, `G/G`, separated
by tabs. The [model's sample input](../12-eye-color-model/inputs/variants)
is the complete default output. Rows follow the chosen order, and the model
matches them by rsID.
