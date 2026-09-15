# 02 - permissions

A grant covers one directory and everything beneath it, and nothing else. This
app grants `v1/genome/rsids/rs72921001` and reads its genotype, then tries to
open the call file it never asked for, and shows that the read is blocked.

Its manifest sets `develop = true`, so an Ark also returns what it prints to
standard error. Leave that flag out of apps you ship.
[02-manifest.md](../../docs/02-manifest.md) covers grants, and
[01-app-model.md](../../docs/01-app-model.md) the flag.

## Build and run

```sh
make run APP=02-permissions
make run APP=02-permissions FIXTURES=fixtures/no-call
make run APP=02-permissions FIXTURES=fixtures/unanswered
```
