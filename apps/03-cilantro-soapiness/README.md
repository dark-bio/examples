# 03 - cilantro-soapiness

The single variant from [03-cilantro-mini-rust](../03-cilantro-mini-rust),
turned into a full report in the shape [06-reports.md](../../docs/06-reports.md)
describes, with a finding, the evidence behind it, the method, its limitations
and sources. A no-call and an absent genotype each get their
own finding instead of a verdict, and a call without exactly two copies shows
its copies without a homozygous or heterozygous label. The `reference` grant is
read for one file, the assembly build the coordinate is reported on.

## Build and run

```sh
make run APP=03-cilantro-soapiness
make run APP=03-cilantro-soapiness FIXTURES=fixtures/no-call
make run APP=03-cilantro-soapiness FIXTURES=fixtures/unanswered
```
