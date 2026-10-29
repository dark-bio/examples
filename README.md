# Ark Example Apps

Worked examples for writing apps that run on an Ark.

[dark.bio](https://dark.bio) · [Whitepaper](https://dark.bio/whitepaper.pdf) · [GitHub](https://github.com/dark-bio) · [Bluesky](https://bsky.app/profile/dark.bio) · [X](https://x.com/dark_dot_bio)

An Ark holds a person's genome on a device only they control. Apps never receive
a copy of that data. They are sent to it instead, and run in a deterministic
sandbox on the Ark with no network or storage that outlives the task. An app
reads the data it was granted as plain files and may use other apps in the same
task. The owner approves the task once on their phone, with the combined grants
of all its apps. The main app prints the report, and the owner decides whether
to release it.
Because the sandbox contains the code, anyone can write an app and anyone can
run anyone else's. The [whitepaper](https://dark.bio/whitepaper.pdf) lays out
the trust model.

These examples teach that model one idea at a time, from printing a line to
scanning a whole call file and composing apps. Every app has one handwritten
source file.

## What an app is

An app is a [WebAssembly](https://webassembly.org/) module built for
[WASI Preview 1](https://github.com/WebAssembly/WASI). The Ark runs each app
in two passes.

- **With no arguments**, the app prints a short TOML manifest naming itself and
  the data it wants. The Ark checks every app's manifest before asking the
  owner to approve the task.
- **With a data directory as its first argument**, the app reads its granted
  files and inputs, and writes its declared outputs. Dependencies run before
  the main app, whose standard output is the Markdown report the owner reads.

```rust
fn main() {
    if std::env::args().nth(1).is_none() {
        print!("manifest = 1\n\n[app]\nname = \"hello\"\nversion = \"0.1.0\"\n");
        return;
    }
    println!("Hello from inside an Ark. No network, no disk, no way out.");
}
```

## Quickstart

Build and run an example against the fixture data in this repository, with
[`wasmtime`](https://wasmtime.dev/), a POSIX shell and AWK for the local runner,
and the toolchain for its language. Building Python apps needs
CPython 3.13 or newer to cross compile the interpreter they carry.

```sh
make run APP=01-hello-rust   # one app
make run                     # every app
```

Run it on an Ark with the [`ark`](https://github.com/dark-bio/cli) tool. The
owner approves it on their phone, in Ark Companion for
[iOS](https://apps.apple.com/app/id6751324700) or
[Android](https://play.google.com/store/apps/details?id=bio.dark.companion).
Without hardware, [Ark Emulator](https://github.com/dark-bio/emulator) boots
the real Ark firmware on your computer, and `ark` runs apps on it the same way.

```sh
ark data paths                                 # what apps can read on this Ark
ark run build/01-hello-rust.wasm > report.md
```

AI agents driving an Ark read `ark help agents` first.
[docs/05-running.md](docs/05-running.md) covers the toolchains, the fixtures,
what a laptop can't reproduce, and running on an Ark or an emulator.

## The examples

Most data comes in two versions. A *mini* shows the bare read in a few lines,
and a full app turns the same read into a report. Each row is a folder under
`apps/`, with the language appended for the minis, such as
`04-cilantro-mini-rust`, and that folder name is what `APP=` takes.

| App | What it shows | Languages |
| :-- | :-- | :-- |
| `01-hello` | the manifest pass and the processing pass | Rust, Go, C, Python |
| `02-permissions` | requesting data, optional grants, least privilege and the `develop` flag | Rust |
| `03-sundae-order` | asking the owner questions with inputs | Rust |
| `04-cilantro-mini` | one variant through `rsids/` | Rust, Go, C, Python |
| `04-cilantro-soapiness` | the same variant as a report | Rust |
| `05-genes-mini` | one gene through `genes/` | Rust, Go, C, Python |
| `05-bitter-meter` | the gene and its changes as a report | Rust |
| `06-regions-mini` | an interval through `regions/` | Rust, Go, C, Python |
| `06-powerhouse-of-the-cell` | the mitochondrial genome as a report | Rust |
| `07-copies-mini` | a gene's copy number through `copies/` | Rust, Go, C, Python |
| `07-doping-test` | the copy number as a report, with its dataset required | Rust |
| `08-structural-mini` | a structural variant through `changes/` | Rust, Go, C, Python |
| `08-sprint-or-marathon` | the insertion as a report, with its dataset required | Rust |
| `09-vcf-mini` | streaming the raw call file | Rust, Go, C, Python |
| `09-vcf-roll-call` | parsing the whole call file as a report | Rust |
| `10-motif-finder` | computing over a gene's reference sequence | Rust |
| `11-fortune-cookie` | why the sandbox is deterministic | Rust |
| `12-pigment-variants` | gathering six genotypes into an output, with a typed ordering input | Rust |
| `12-eye-color-model` | turning an untyped variant input into IrisPlex probabilities | Rust |
| `12-eye-color` | composing both apps with digest pins and input bindings | Rust |

## Documentation

- [01-app-model.md](docs/01-app-model.md) covers the two passes, the checks, the
  sandbox and its limits.
- [02-manifest.md](docs/02-manifest.md) covers the manifest, how to grant data
  and how to ask the owner questions.
- [03-data-paths.md](docs/03-data-paths.md) maps the data tree and the rules
  every path follows.
- [04-reading-data.md](docs/04-reading-data.md) covers absence, errors,
  references and genotypes, with a grant for every need.
- [05-running.md](docs/05-running.md) covers running apps locally, on an
  emulator and on an Ark.
- [06-reports.md](docs/06-reports.md) covers the report, its shape and its
  voice.

## A note on scope

These apps show how to read data and present a result. Their trait readouts are
illustrations, not medical advice, and one variant is rarely the whole story.

## License

BSD-3-Clause, see [LICENSE](LICENSE). Copy any of these examples as a starting
point for your own apps.
