# 04 - drunk-o-type

A panel of eleven variants, scored on three axes of how a body handles alcohol:
flushing, reward and histamine. Each variant is its own `rsids/` grant, so the
access pattern is [03-cilantro-mini-rust](../03-cilantro-mini-rust) repeated,
and the owner sees every site by name before approving.

The app counts the allele each study tested, which isn't always the ALT allele.
A missing allele or an absent genotype makes its axis inconclusive rather than
lowering the score. This is a demonstration, not medical advice.

## Build and run

```sh
make run APP=04-drunk-o-type
make run APP=04-drunk-o-type FIXTURES=fixtures/no-call
make run APP=04-drunk-o-type FIXTURES=fixtures/unanswered
```
