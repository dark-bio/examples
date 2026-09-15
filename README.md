# Ark Example Apps

Worked examples for writing apps that run on an Ark.

[dark.bio](https://dark.bio) · [Whitepaper](https://dark.bio/whitepaper) · [GitHub](https://github.com/dark-bio) · [Bluesky](https://bsky.app/profile/dark.bio) · [X](https://x.com/dark_dot_bio)

An Ark holds a person's genome on a device only they control. Apps never receive
a copy of that data. They are sent to it instead, and run in a deterministic
sandbox on the Ark with no network and no writable storage. An app reads the
data it was granted as plain files and prints a report. The owner approves every
run on their phone and decides whether to release the result. Because the
sandbox contains the code, anyone can write an app and anyone can run anyone
else's. The [whitepaper](https://dark.bio/whitepaper) lays out the trust model.

These examples teach that model one idea at a time, from printing a line to
scanning a whole call file. Every app is a single source file.

## What an app is

An app is a [WebAssembly](https://webassembly.org/) module built for
[WASI Preview 1](https://github.com/WebAssembly/WASI). The Ark runs it twice.

- **With no arguments**, the app prints a short TOML manifest naming itself and
  the data it wants. The Ark checks it and shows it to the owner for approval.
- **With a data directory as its first argument**, the app reads its granted
  files and prints its report.

```rust
fn main() {
    if std::env::args().nth(1).is_none() {
        print!("[package]\nname = \"hello\"\nversion = \"0.1.0\"\ndatasets = []\n");
        return;
    }
    println!("Hello from an Ark app.");
}
```

## Quickstart

Build and run an example against the fixture data in this repository, with
[`wasmtime`](https://wasmtime.dev/) and the toolchain for its language:

```sh
make run APP=01-hello-rust   # one app
make run                     # every app
```

Run it on a real Ark with the [`ark`](https://github.com/dark-bio/cli) tool,
approving it on your phone:

```sh
ark data paths                                 # what apps can read on this Ark
ark app run build/01-hello-rust.wasm > report.md
```

[docs/05-running.md](docs/05-running.md) covers the toolchains, the fixtures, and
what a laptop can't reproduce.

## The examples

Most data comes in two versions. A *mini* shows the bare read in a few lines,
and a full app turns the same read into a report.

| App | What it shows | Languages |
| :-- | :-- | :-- |
| `01-hello` | the manifest pass and the run pass | Rust, Go, C |
| `02-permissions` | requesting data, least privilege and the `develop` flag | Rust |
| `03-cilantro-mini` | one variant through `rsids/` | Rust, Go, C |
| `03-cilantro-soapiness` | the same variant as a report | Rust |
| `04-drunk-o-type` | a panel of variants | Rust |
| `05-genes-mini` | one gene through `genes/` | Rust |
| `05-bitter-meter` | the gene and its changes as a report | Rust |
| `06-regions-mini` | an interval through `regions/` | Rust |
| `06-powerhouse-of-the-cell` | the mitochondrial genome as a report | Rust |
| `07-vcf-mini` | streaming the raw call file | Rust |
| `07-vcf-roll-call` | parsing the whole call file as a report | Rust |
| `08-motif-finder` | computing over a gene's reference sequence | Rust |
| `09-fortune-cookie` | why the sandbox is deterministic | Rust |

## Documentation

- [01-app-model.md](docs/01-app-model.md) covers the two passes, the checks, the
  sandbox and its limits.
- [02-manifest.md](docs/02-manifest.md) covers the manifest and how to grant
  data.
- [03-data-paths.md](docs/03-data-paths.md) maps the data tree and the rules
  every path follows.
- [04-reading-data.md](docs/04-reading-data.md) covers absence, errors,
  sequences and genotypes, with a grant for every need.
- [05-running.md](docs/05-running.md) covers running apps locally and on an Ark.

## A note on scope

These apps show how to read data and present a result. Their trait readouts are
illustrations, not medical advice, and one variant rarely tells a whole story.

## License

BSD-3-Clause, see [LICENSE](LICENSE). Copy any of these examples as a starting
point for your own apps.
