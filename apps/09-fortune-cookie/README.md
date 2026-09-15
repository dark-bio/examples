# 09 - fortune-cookie

An Ark's sandbox hands apps zero random bytes and clocks that only count, so a
random-looking fortune has to come from the input. This app hashes the genotypes
of three granted variants into a fortune, so the same data always gets the same
one. An absent genotype adds nothing to the hash and is noted in the report, and
any other read error stops the app.

## Build and run

```sh
make run APP=09-fortune-cookie
make run APP=09-fortune-cookie FIXTURES=fixtures/no-call
make run APP=09-fortune-cookie FIXTURES=fixtures/unanswered
```
