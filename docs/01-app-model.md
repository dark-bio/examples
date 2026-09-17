# The app model

An Ark app is a WebAssembly module built for WASI Preview 1, with a standard
`_start` entry point. Anything that compiles to that target can be an app, and
these examples cover Rust, Go, C and Python.

The Ark doesn't need to trust an app. It trusts the sandbox the app runs in, and
everything below follows from that.

## Two passes

The Ark runs every app twice, and tells the two passes apart by the first
command-line argument.

- **The manifest pass** runs the module with no arguments. The app prints a
  short TOML manifest naming itself and the data it wants, then exits. Nothing
  is mounted yet, so it can't read anything. [02-manifest.md](02-manifest.md)
  covers the format.
- **The run pass** runs the module again once the owner has approved it, with
  the data directory as the first argument. The app reads the files it was
  granted and prints its report.

An app picks its pass by counting arguments:

| Language | Manifest pass | Run pass |
| :-- | :-- | :-- |
| Rust | `std::env::args().nth(1).is_none()` | otherwise |
| Go | `len(os.Args) < 2` | otherwise |
| C | `argc < 2` | otherwise |
| Python | `len(sys.argv) < 2` | otherwise |

Write the report as Markdown, since that is how it is shown. Headings, tables
and emphasis all help.

## Before the owner is asked

When an app is scheduled, the Ark checks it before any prompt reaches the owner's
phone, and refuses it if a check fails.

- A module with a WebAssembly start section is refused, since a run begins at
  `_start`.
- A name or version that is empty, too long, or holds control characters, line
  separators or text direction controls is refused.
- A dataset path that is misspelled or can't be granted is refused, and so is
  one whose data isn't on the Ark.

## The sandbox

The run pass has no network, no writable storage, and no access to anything
beyond its grants, which are mounted read-only.

The sandbox is also deterministic, so the same app over the same data prints the
same report every time:

- Standard input is closed.
- The random number generator returns zeros.
- Both clocks read one counter that starts at 1 and ticks once per read,
  unrelated to real time.

An app that needs a random-looking choice derives it from its input.
[09-fortune-cookie](../apps/09-fortune-cookie) does exactly that.

## Limits

| | Manifest pass | Run pass |
| :-- | :-- | :-- |
| Memory | 16.125 MiB | 100 MiB |
| Standard output | 1 KiB | 1 MiB |
| Standard error | 1 KiB | 1 MiB |
| Time | 250 ms | none, but cancellable |
| Data | none | granted paths, read-only |

Output past a cap is dropped. The module itself can be up to 256 MiB.

## The develop flag

By default the Ark returns an app's standard output only when the app exits
successfully, and never returns its standard error, so a failed run returns
nothing.

Setting `develop = true` in the manifest returns standard output even on failure,
and standard error every time. It is a debugging switch, and the owner sees a
developer mode warning when approving such an app. Leave it out of apps you ship.

## A run on an Ark

1. A developer sends the module to the Ark, for example with
   `ark app run app.wasm`.
2. The Ark runs the manifest pass, then checks the module, its name and version,
   and every dataset path.
3. The owner sees the app's name, version and requested paths on their phone,
   and approves or declines.
4. On approval, the Ark mounts the granted paths read-only and runs the app.
5. The result is returned to the owner, who decides whether to release it.

Every step is recorded in a signed, tamper-evident journal on the device.
