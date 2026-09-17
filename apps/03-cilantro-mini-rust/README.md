# 03 - cilantro-mini (Rust)

The smallest real data access. The app grants one variant,
`v1/genome/rsids/rs72921001`, reads its genotype and counts copies of the `C`
allele, which people who taste cilantro as soap tend to carry (Eriksson et al.,
2012). It never opens the call file.

It shows the outcomes every app has to handle. A missing allele, as in `./.`,
makes the count inconclusive. An absent genotype is reported as no answer, never
as two reference alleles. Any other read error stops the app.
[04-reading-data.md](../../docs/04-reading-data.md) explains why.

The same mini exists in [Go](../03-cilantro-mini-go),
[C](../03-cilantro-mini-c) and [Python](../03-cilantro-mini-python), and
[03-cilantro-soapiness](../03-cilantro-soapiness) turns it into a report.

## Build and run

```sh
make run APP=03-cilantro-mini-rust
make run APP=03-cilantro-mini-rust FIXTURES=fixtures/no-call
make run APP=03-cilantro-mini-rust FIXTURES=fixtures/unanswered
```
