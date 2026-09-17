# Writing Ark apps in this repository

This repository teaches how to write apps for the Ark. Read `README.md` first,
then `docs/01-app-model.md` through `docs/06-reports.md` in order. They are
short and they are the rules. When a real Ark is involved, `ark help agents`
comes before anything else.

## Commands

```sh
make build APP=<app>          # one module into build/<app>.wasm
make run APP=<app>            # build, then both passes against fixtures/
make run APP=<app> FIXTURES=fixtures/no-call
make run APP=<app> FIXTURES=fixtures/unanswered
tools/check.sh                # what CI runs: links, builds, runs, ports
```

`make run` mounts only the paths an app's manifest declares, read-only, and
runs it with `/` as the data directory, the way an Ark does. The three fixture
roots are described in `fixtures/README.md`. An app whose grants a root lacks
stops before its run pass there, which is expected.

## Adding an app

- Copy the nearest example. One source file, in `apps/<nn>-<name>-<lang>/` for
  a mini and `apps/<nn>-<name>/` for a full app, with a short `README.md` that
  says what it shows and how to run it.
- The manifest names the app, its version and the narrowest grants that answer
  the question. Spell every path exactly as `ark data paths` shows it, with
  `v1/` kept and no trailing `/`. `docs/02-manifest.md` has the rules.
- Read grants as plain files. Absent means "not found" and is a result. Any
  other error goes to standard error with a non-zero exit. The shapes a
  genotype can take are in `docs/04-reading-data.md`.
- Nothing in the sandbox is random or timed. Sort anything you list.
- Keep the module small. Rust apps carry the release profile from any sibling's
  `Cargo.toml`.
- Run `tools/check.sh` before proposing a change. New fixtures follow
  `fixtures/README.md`, with no trailing newline in any value file.

## The report

Follow `docs/06-reports.md`. In short:

- One `#` title in Title Case naming the subject, then the finding straight
  under it with no heading, within the first ten lines. No app name, version
  or grant list; the Ark reports those beside the report.
- Then sections in this order, `Evidence`, `Method`, `Limitations`,
  `Sources`, each when it has content, headings in sentence case.
- The finding is a sentence and a value. A verdict label may lead it when
  the value follows at once. Say "carries" and "is associated with", never
  "you have" or "you will". An absent answer is stated as a finding, never
  read as reference.
- Every value beside its coordinate. Genotypes exactly as the file holds
  them, in prose or code when they contain a pipe. Coordinates labelled with
  the assembly only when the app granted `v1/genome/reference` and read
  `build`.
- Studies named inline as author and year, listed in Sources with a stable
  URL. Links nowhere else.
- Pick one voice and keep it. Light or serious, never both in one report.
  Genomics words, never clinic words.
- Plain Markdown only. No images, HTML, footnotes, encoded content, dates or
  run ids. Tables with as few columns as carry the evidence, rows about 80
  characters.

## Never

- Never read absence as homozygous reference.
- Never assume the ALT allele is the risk allele. Derive it from the study.
- Never grant more than the question needs.
- Never depend on randomness, time, the network or writable storage.
- Never put anything in a report the owner can't read.
