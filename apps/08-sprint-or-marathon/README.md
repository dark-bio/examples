# 08 - sprint-or-marathon

A full report on the ACE insertion, an Alu repeat that sport studies found more
often in endurance athletes. The app looks through the `changes` in a 101-base
window for an insertion of 200 bases or more, and reads its `carried` count as
I/I or I/D. It's a data demonstration, not a talent test.

A D/D genome has no insertion to list, which looks just like a genome without a
structural variant file. The app also grants `v1/genome/datasets`, the public
list of the owner's uploads, and answers D/D only when that file is there.

## Build and run

```sh
make run APP=08-sprint-or-marathon
make run APP=08-sprint-or-marathon FIXTURES=fixtures/no-call
make run APP=08-sprint-or-marathon FIXTURES=fixtures/unanswered
```
