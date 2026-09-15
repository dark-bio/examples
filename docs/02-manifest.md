# The manifest

The manifest is the TOML an app prints in its manifest pass. It names the app and
lists the data the app wants. The Ark reads it to decide what to check, what to
show the owner and what to mount.

```toml
[package]
name = "cilantro"
version = "0.1.0"
datasets = ["v1/genome/rsids/rs72921001"]
```

## Fields

All fields live under `[package]`.

- **`name`** is the app's name, shown to the owner, from 1 to 64 characters.
- **`version`** is shown beside the name, from 1 to 32 characters.
- **`datasets`** lists the paths the app wants. It may be empty.
- **`develop`** is optional and `false` by default. See
  [01-app-model.md](01-app-model.md).

Names and versions can't hold control characters, line or paragraph separators,
or text direction controls. The whole manifest has to fit in the manifest pass's
1 KiB of output, which holds a panel of a dozen paths with room to spare.

## Granting data

Each dataset is a directory under the Ark's data root. Granting it gives the app
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
- **Its data must be on the Ark.** A well-formed path is refused when it points
  into an empty slot, names a gene the annotations don't carry or an rsID dbSNP
  doesn't carry, or names a position past the end of its chromosome.

A `changes` directory can't be granted. Grant its gene or interval instead.

## Choosing grants

The owner reads the list of paths before approving, so ask for the narrowest
ones that do the job. One variant reads very differently from the whole call
file. [04-reading-data.md](04-reading-data.md) walks through the options, from
narrowest to broadest.

Some grants reach further than they look. A gene or interval grant includes its
`changes`, which hold the owner's variants, even when the app only reads the
sequence. Only `v1/genome/reference` and `v1/genome/annotations` hold nothing but
public data.
