# The manifest

The manifest is the TOML an app prints in its manifest pass. It names the app,
lists the data the app reads, asks the owner any questions the app needs
answered, and carries the text a listing shows. The Ark reads it to decide what
to check, what to show the owner and what to mount.

```toml
manifest = 1

[app]
name = "Cilantro Taste Test"
version = "0.4.0"

[reads]
paths = ["v1/genome/rsids/rs72921001", "v1/genome/reference"]

[listing]
language = "en"
icon = "🌿"
summary = "Does cilantro taste like soap to you? One variant near OR6A2 has a say."
category = "traits"
license = "BSD-3-Clause"
```

[04-cilantro-soapiness](../apps/04-cilantro-soapiness) prints this manifest,
with a longer listing.

## Format

A manifest is a TOML 1.1.0 document. It opens with `manifest = 1`, the version
of this format, followed by the tables below. The Ark reads `[app]`, `[reads]`,
`[inputs]` and `[output]` strictly, and refuses a manifest with a key it doesn't
know in any of them. It skips `[listing]` without reading it, and refuses any
other table or top-level key, and any other `manifest` value. The whole manifest
has to fit in the manifest pass's 64 KiB of output.

Text that the owner or a listing shows, such as the name, a prompt or a choice,
is display text. It holds at least one character, and no control characters,
line or paragraph separators, or text direction controls. Emoji are fine,
joined ones included. Long text reads best in TOML literal strings, `'...'` and
`'''...'''`, since they need no escaping.

### `[app]`

- **`name`** is the app's name as the owner sees it, on the phone, in the
  journal and on a listing, as display text of 1 to 64 characters.
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

### `[inputs.<name>]`

Each table asks the owner one question, which
[Asking the owner](#asking-the-owner) covers.

### `[output]`

- **`report`** is the report's media type, `text/markdown` by default. The
  format also defines `application/json`, which the current Ark firmware
  refuses, so every example leaves the table out and prints Markdown.

### `[listing]`

The text Ark Hub shows when it lists the app, which
[Listing an app](#listing-an-app) covers.

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

## Asking the owner

An app can ask the owner questions, which they answer on the phone as part of
the approval. Each `[inputs.<name>]` table declares one:

```toml
[inputs.flavor]
type = "choice"
prompt = "Which flavor should the scoop be?"
choices = ["Vanilla", "Chocolate", "Strawberry", "Pistachio"]
default = "Vanilla"
```

- **`type`** is `"choice"` or `"text"`.
- **`prompt`** is the question, display text of at most 80 characters.
- **`choices`**, for a choice, lists 1 to 32 distinct answers, each display
  text of at most 64 characters.
- **`multiple`**, for a choice, lets the owner pick any number of them. It is
  `false` by default.
- **`max`**, for a text, is the longest answer accepted, from 1 to 256
  characters.
- **`default`** is what the phone fills in first. It is one of the choices, a
  list of them in the order of `choices` for a multiple choice, or a text that
  fits `max`. Without one, a choice starts unpicked and a text starts empty.

A name is 1 to 32 lowercase ASCII letters, digits, `-` and `_`, and starts with
a letter. A manifest declares at most 16 inputs. The Ark refuses a declaration
that breaks one of these rules, and checks every answer against its
declaration.

The answers reach the app as files under `inputs/` in the data directory, one
per input and named after it, in UTF-8 with no trailing newline. A choice's file
holds the picked choice. A multiple choice's file holds the picks one per line,
in the order of `choices`, and is empty when nothing was picked. A text's file
holds the text, which is never empty.
[03-sundae-order](../apps/03-sundae-order) reads all three.

Inputs never change what an app reads. An app whose reads depend on an answer
grants everything the answer can select. The Ark's journal records which inputs
the owner filled in, never the answers.

## Listing an app

`[listing]` holds what Ark Hub shows when it lists the app. The Ark never reads
it and it never reaches the phone, so it changes nothing about what an app can
do. Ark Hub needs the first five keys before it lists an app, and every example
carries a full listing.

| Key | To be listed | Rules |
| :-- | :-- | :-- |
| `language` | required | The language of the listing and of the report, a BCP 47 tag such as `en` |
| `icon` | required | Exactly one emoji |
| `summary` | required | Display text of at most 80 characters |
| `category` | required | `traits`, `tools` or `developer` |
| `license` | required | An SPDX expression, such as `BSD-3-Clause` |
| `source` | optional | An `https` URL where the app's source lives |
| `description` | optional | Restricted Markdown of at most 8 KiB |
| `keywords` | optional | At most 5, each display text of at most 20 characters |
| `purposes` | optional | For each grant, why the app reads it |

A description allows paragraphs, emphasis, inline code, fenced code blocks and
lists, and no headings, links, images, HTML, tables or footnotes. A purpose is
display text of at most 80 characters, keyed by a path from `[reads]`:

```toml
[listing.purposes]
"v1/genome/rsids/rs72921001" = "The variant near OR6A2 tied to tasting cilantro as soap"
```

Purposes appear on the listing only. The phone describes every grant in the
Ark's own words, never the app's. A module holds one language, so an app in
another language is a separate app.
