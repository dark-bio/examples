***Note, this is a work-in-progress, early access repository. It hasn't been cleaned up, rather it's an LLM braindump to allow playing with the Dark Bio Arks while the docs are properly prepared.***

# Ark Example Apps

Worked examples for writing apps that run on an Ark.

[dark.bio](https://dark.bio) · [Whitepaper](https://dark.bio/whitepaper) · [GitHub](https://github.com/dark-bio) · [Bluesky](https://bsky.app/profile/dark.bio) · [X](https://x.com/dark_dot_bio)

An Ark holds a person's genome and other health data on a device only they
control. Apps never receive a copy of that data; they are sent to it. An app
runs in a deterministic sandbox on the Ark, with no network and no writable
storage. It reads the data it was granted as plain files, and prints a result.
The owner reviews that result on their phone and decides whether to release it.
Because the sandbox, not the developer, is what contains the code, anyone can
write an app and anyone can run anyone else's. The
[whitepaper](https://dark.bio/whitepaper) lays out the trust model in full.

This repository teaches that model by example. Every app is a single source
file. They start at "print one line" and climb to scanning an entire variant
file, adding one idea at a time.

## What an app is

An app is a [WebAssembly](https://webassembly.org/) module built against
[WASI Preview 1](https://github.com/WebAssembly/WASI), with a standard `_start`
entry point. Rust, Go, and C can all produce one; the first three examples are
the same program in each.

The Ark runs an app in two passes:

- With no arguments, the app prints a short TOML **manifest** that names it and
  lists the data it wants. The Ark reads this to decide what to mount and what
  to show the owner for approval.
- With a data directory as its first argument, the app does its real work: it
  reads the granted files and prints a report to standard output.

The same program handles both. The first thing it does is check whether it was
given a data directory:

```rust
fn main() {
    if std::env::args().nth(1).is_none() {
        // No data directory: print the manifest and stop.
        print!("[package]\nname = \"hello\"\nversion = \"0.1.0\"\ndatasets = []\n");
        return;
    }
    // A data directory was given: do the real work.
    println!("Hello from an Ark app.");
}
```

[docs/01-app-model.md](docs/01-app-model.md) explains the sandbox, the limits,
and why the environment is deterministic.

## Quickstart

You need [`wasmtime`](https://wasmtime.dev/) to run an app locally, plus the
toolchain for the language you want to build (see each example's README). To
build and run the Rust hello world:

```sh
make run APP=01-hello-rust
```

That builds the app to WebAssembly, prints its manifest (the no-argument pass),
then runs it against the `fixtures/` tree, which stands in for the data an Ark
would mount. The fixtures are a few kilobytes of plain text, not a real genome;
[docs/05-running-locally.md](docs/05-running-locally.md) explains where they
come from and how faithfully a laptop can stand in for a device.

## The examples

Each rung adds one idea. Read them in order, or jump to the pattern you need.
Most lenses come in two versions: a *mini* that shows the bare read in a handful
of lines, and a *full* app that turns the same read into a report. Read the mini
to learn the lens, the full to see what a real app looks like.

| App | What it teaches | Languages |
| :-- | :-- | :-- |
| `01-hello` | the manifest pass and the run pass | Rust, Go, C |
| `02-permissions` | requesting data, least privilege, the `develop` flag | Rust |
| `03-cilantro-mini` | one variant through the `rsids/` lens | Rust, Go, C |
| `03-cilantro-soapiness` | the same read, as a full report | Rust |
| `04-drunk-o-type` | a panel of variants | Rust |
| `05-genes-mini` | one gene through the `genes/` lens | Rust |
| `05-bitter-meter` | the gene, as a full report | Rust |
| `06-regions-mini` | an interval through the `regions/` lens | Rust |
| `06-powerhouse-of-the-cell` | the mitochondrial genome, as a full report | Rust |
| `07-vcf-mini` | reading the raw variant file | Rust |
| `07-vcf-roll-call` | scanning the whole file, as a full report | Rust |
| `08-motif-finder` | reading reference sequence, no genotype | Rust |
| `09-fortune-cookie` | why the sandbox is deterministic | Rust |

## Documentation

- [01-app-model.md](docs/01-app-model.md) - the two passes, the sandbox, the limits.
- [02-manifest.md](docs/02-manifest.md) - the manifest format and how data is requested.
- [03-biofs-paths.md](docs/03-biofs-paths.md) - every path the Ark can serve.
- [04-data-access.md](docs/04-data-access.md) - the access patterns, mapped to the examples.
- [05-running-locally.md](docs/05-running-locally.md) - running without a device.

## A note on scope

These apps are demonstrations of how to read data and present a result. The
trait readouts among them are for illustration, not medical advice. One variant
rarely tells a whole story.

## License

BSD-3-Clause; see [LICENSE](LICENSE). Copy any of these examples as a starting
point for your own apps.
