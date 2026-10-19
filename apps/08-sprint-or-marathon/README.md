# 08 - sprint-or-marathon

A full report on the ACE insertion, an Alu repeat that sport studies found more
often in endurance athletes. The app looks through the `changes` in a 101-base
window for an insertion of 200 bases or more, and reads its `carried` count as
I/I or I/D. It's a data demonstration, not a talent test.

A D/D genome has no insertion to list, which would look just like a genome
without a structural variant file. So the app also grants
`v1/genome/datasets/structural-variants`, a public folder that reads empty, and
the Ark won't run it without that file. On `fixtures/unanswered` the app never
starts, just as on an Ark without one.

## Build and run

```sh
make run APP=08-sprint-or-marathon
make run APP=08-sprint-or-marathon FIXTURES=fixtures/no-call
make run APP=08-sprint-or-marathon FIXTURES=fixtures/unanswered
```
