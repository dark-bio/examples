# The manifest

The manifest is the TOML an app prints during the manifest pass. It names the
app and declares the data it wants. The Ark reads it to decide what to mount and
what to show the owner for approval.

A complete manifest:

```toml
[package]
name = "cilantro"
version = "0.1.0"
datasets = ["v1/genome/rsids/rs72921001"]
develop = false
```

## Fields

The Ark reads four fields, all under `[package]`:

- **`name`** - the app's name, shown to the owner at approval. Required.
- **`version`** - the app's version, shown alongside the name. Required.
- **`datasets`** - the list of data paths the app wants, each relative to the
  Ark's data root. May be empty. See below.
- **`develop`** - optional, defaults to `false`. The debugging switch described
  in [01-app-model.md](01-app-model.md). Leave it out for a shipped app.

Any other key is ignored. You will see `api = "v1"` in some apps; the Ark does
not read it. The dataset paths already carry the `v1` version, so the field is
redundant. Treat the four fields above as the whole contract.

The manifest pass output is capped at 1 KiB, so keep the manifest small. Even a
panel of a dozen datasets fits comfortably.

## Requesting data

Each entry in `datasets` is a path under the Ark's data tree, written relative
(no leading slash). The Ark mounts each one read-only at the same path under the
data directory it hands the app. So a manifest that requests:

```toml
datasets = ["v1/genome/rsids/rs72921001"]
```

lets the app read `/v1/genome/rsids/rs72921001/...` at run time (the leading `/`
is the data directory passed as the first argument). Every readable path is
listed in [03-biofs-paths.md](03-biofs-paths.md).

An app sees exactly what it asked for, and nothing else. Requesting one variant
directory grants that directory, not the file it came from and not its
neighbors. This is the point: the owner approves a specific, legible request,
and the app cannot read past it. The Ark rejects a few requests outright:

- absolute paths,
- paths that escape the data root (for example with `..`),
- a request for the entire root.

Ask for the narrowest paths that do the job. The owner sees the list verbatim
before approving, so "one variant" reads very differently from "the whole
variant file." [04-data-access.md](04-data-access.md) walks the patterns from
narrowest to broadest.
