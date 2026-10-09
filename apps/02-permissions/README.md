# 02 - permissions

A grant covers only one directory and everything beneath it. This app grants
`v1/genome/rsids/rs72921001` and reads its genotype. It also asks for
`v1/genome/rsids/rs671` as an optional grant, which the owner may switch off,
and reports whether it was mounted. Then it tries to open the call file it never
asked for, and shows that the read is blocked.

A declined optional grant and one whose data the Ark lacks are both left
unmounted, so the app can only say that it was one of the two.

Its manifest sets `develop = true`, so its standard error goes to the owner's
review as well. Leave that flag out of apps you ship.
[02-manifest.md](../../docs/02-manifest.md) covers grants, and
[01-app-model.md](../../docs/01-app-model.md) the flag.

## Build and run

```sh
make run APP=02-permissions
OPTIONAL=off make run APP=02-permissions   # as if the owner declined rs671
make run APP=02-permissions FIXTURES=fixtures/no-call
make run APP=02-permissions FIXTURES=fixtures/unanswered
```
