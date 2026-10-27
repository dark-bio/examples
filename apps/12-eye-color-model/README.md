# 12 - eye-color-model

Applies the original IrisPlex multinomial logistic regression to six
genotypes from [12-pigment-variants](../12-pigment-variants). It reads no
grants. Its untyped `variants` input requires a binding on an Ark, so this
app has no listing. [12-eye-color](../12-eye-color) binds its inputs and
reports its probabilities.

## Build and run

The local runner can debug a dependency alone, including its untyped input:

```sh
make run APP=12-eye-color-model
```

The sample `inputs/variants` holds the gatherer's default output, and
`inputs/detail` holds `full` without a trailing newline. `INPUTS=<dir>`
selects another directory containing both files. The model prints its output
for local inspection. The Ark drops a dependency's printout.

## Text passed between apps

`inputs/variants` uses the gatherer's
[four-field TSV format](../12-pigment-variants#the-variants-output), in either
order. Every rsID must appear once. A missing row, duplicate or unknown rsID
is a format error. Two called alleles from the modeled pair are required at
each site. Phase and a leading phase marker do not affect the copy count.

`outputs/colors` is UTF-8 text, with no header and a final newline. Each row
holds a color and its probability, separated by one tab. Probabilities range
from 0 to 1 and have nine decimal places. `full` writes `blue`, `intermediate`
and `brown`, in that order. `summary` writes only the largest probability,
with the first color in that order winning a tie. This is a probability
estimate without a classification threshold.

For the sample input, `full` writes:

```text
blue	0.946340538
intermediate	0.035254828
brown	0.018404634
```

An absent genotype, no-call, deletion, non-diploid call or allele outside the
modeled pair gives one row instead:

```text
unavailable	missing-or-unsupported-genotype
```

## Coefficients and source

The coefficients are from Walsh et al. (2011), Supplementary Table 3. Each
weight adds to a log odds score per copy of the counted forward-strand
allele. Brown is the baseline, with score zero.

| Variant | Gene | Counted allele | Other allele | Blue vs brown | Intermediate vs brown |
| :-- | :-- | :-- | :-- | --: | --: |
| intercept | | | | 3.94 | 0.65 |
| rs12913832 | *HERC2* | A | G | -4.81 | -1.79 |
| rs1800407 | *OCA2* | T | C | 1.40 | 0.87 |
| rs12896399 | *SLC24A4* | G | T | -0.58 | -0.03 |
| rs16891982 | *SLC45A2* | C | G | -1.30 | -0.50 |
| rs1393350 | *TYR* | A | G | 0.47 | 0.27 |
| rs12203592 | *IRF4* | T | C | 0.70 | 0.73 |

For copy counts `x`, the scores are `b = 3.94 + sum(blue_weight * x)` and
`i = 0.65 + sum(intermediate_weight * x)`. Divide `exp(b)`, `exp(i)` and `1`
by their sum to obtain blue, intermediate and brown probabilities. The code
subtracts the largest score before exponentiating, which preserves those
ratios.

The model was fitted with Dutch European data. It separates blue from brown
well and calls intermediate colors, such as green and hazel, poorly.

- Walsh S, Liu F, Ballantyne KN, van Oven M, Lao O, Kayser M. IrisPlex: A
  sensitive DNA tool for accurate prediction of blue and brown eye colour in
  the absence of ancestry information. Forensic Science International:
  Genetics. 2011;5:170-180.
  [Paper and supplementary data](https://doi.org/10.1016/j.fsigen.2010.02.004).
- The author reproduces Supplementary Table 3 on printed page 336 of her
  [thesis](https://repub.eur.nl/pub/40312/130605_Walsh%2C%20Susan%20-%20BEWERKT.pdf),
  PDF page 281. This is the table used here.
