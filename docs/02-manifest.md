# The manifest

The manifest is the TOML an app prints in its manifest pass. It names the app and
lists the data the app reads. The Ark reads it to decide what to check, what to
show the owner and what to mount.

```toml
manifest = 1

[app]
name = "cilantro"
version = "0.1.0"

[reads]
paths = ["v1/genome/rsids/rs72921001"]
```

## Format

A manifest is a TOML 1.1 document. It opens with `manifest = 1`, the version of
this format, followed by the tables below. The Ark refuses a manifest with
another `manifest` value, a top-level table or key it doesn't know, or a key it
doesn't know in the tables below. The whole manifest has to fit in the manifest
pass's 64 KiB of output.

### `[app]`

- **`name`** is the app's name, shown to the owner, from 1 to 64 characters. It
  can't hold control characters, line or paragraph separators, or text
  direction controls.
- **`version`** is shown beside the name. It is a
  [Semantic Versioning 2.0.0](https://semver.org/) version of at most 32
  characters, such as `0.4.0` or `1.0.0-beta.1`, with no leading `v` and no
  build metadata after a `+`.
- **`develop`** is optional and `false` by default. See
  [01-app-model.md](01-app-model.md).

### `[reads]`

- **`paths`** lists the directories the app needs.
- **`optional`** lists directories the owner may decline, which
  [Optional grants](#optional-grants) covers.

An app that reads nothing leaves the table out.

## Granting data

Each path is a directory under the Ark's data root. Granting it gives the app
that directory and everything beneath it, read-only. At run time the grants sit
under the data directory the app receives as its first argument, so with `/` as
that argument, a grant of `v1/genome/rsids/rs72921001` is read at
`/v1/genome/rsids/rs72921001/genotype`.

Spell each path exactly as `ark data paths` shows it, with `v1/` kept and every
placeholder filled in, such as `v1/genome/genes/BRCA1`. The tree marks the
directories a manifest may grant with `+`, and
[03-data-paths.md](03-data-paths.md) explains the rest of it.

The Ark checks every path before the owner is asked, and refuses the app if one
fails.

- **A path must be a grantable directory, spelled the canonical way.** Absolute
  paths, empty, `.` or `..` segments, a trailing `/`, other spellings such as
  `chr01` or `rs0334`, files, `changes` directories, the data root and `v1/`
  itself are all refused.
- **A path is listed once.** A path listed twice, in one list or across both, is
  refused, and so is one inside another listed path's directory, which already
  grants it. A manifest lists at most 1,024 paths across both lists.
- **Its data must be on the Ark.** A well-formed path in `paths` is refused when
  it points into an empty slot, names a gene the annotations don't carry or an
  rsID dbSNP doesn't carry, or names a position past the end of its chromosome.

A `changes` directory can't be granted. Grant its gene or interval instead.

## Optional grants

A path in `optional` is one the owner may decline. The phone lists it with a
switch that starts off, so the owner turns on only what they choose to share.
It suits data that adds to a report without being needed for its answer.

- Public data can't be optional, since there is nothing to decline, and the Ark
  refuses it there.
- An optional path whose data isn't on the Ark doesn't refuse the app. The
  phone shows it as unavailable instead.
- A declined path and one whose data the Ark lacks are both left unmounted, so
  the app can't tell which it was. It reads the missing directory like any
  absent answer, as [04-reading-data.md](04-reading-data.md#least-privilege)
  shows.

[02-permissions](../apps/02-permissions) asks for one optional variant.

## Choosing grants

The owner reads the list of paths before approving, so ask for the narrowest
ones that do the job. One variant reads very differently from the whole call
file. [04-reading-data.md](04-reading-data.md) walks through the options, from
narrowest to broadest. Data that only enriches a report belongs in `optional`,
so an owner who would rather not share it can still run the app.

Some grants reach further than they look. A gene or interval grant includes its
`changes`, which hold the owner's variants, even when the app only reads the
sequence. Only `v1/genome/reference` and `v1/genome/annotations` hold nothing but
public data.
