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
- **The report pass** runs the module again once the owner has approved it, with
  the data directory as the first argument. The app reads the files it was
  granted and prints its report.

An app picks its pass by counting arguments:

| Language | Manifest pass | Report pass |
| :-- | :-- | :-- |
| Rust | `std::env::args().nth(1).is_none()` | otherwise |
| Go | `len(os.Args) < 2` | otherwise |
| C | `argc < 2` | otherwise |
| Python | `len(sys.argv) < 2` | otherwise |

The report is Markdown, since that is how it is shown, and
[06-reports.md](06-reports.md) covers its shape.

## Before the owner is asked

When an app is scheduled, the Ark checks it before any prompt reaches the owner's
phone, in this order, and refuses it at the first check that fails. The refusal
names the rule that broke.

1. **The module.** A module with a WebAssembly start section is refused, since
   an app begins at `_start`. So is a component, or a module with more than one
   linear memory or table. A module that starts with more memory or table space
   than the limits below is refused, and so is one that imports anything but
   WASI Preview 1 functions. The module has to export `_start` as a function
   without parameters or results.
2. **The manifest pass.** A pass that traps, runs out of time, exits with a
   status other than 0 or prints more than 64 KiB is refused.
3. **The manifest.** A manifest that breaks a rule of
   [02-manifest.md](02-manifest.md) is refused.
4. **The paths.** A path that is misspelled or can't be granted is refused, and
   so is one whose data isn't on the Ark. The refusal names the missing data.

## The sandbox

The report pass has no network, no writable storage, and no access to anything
beyond its grants, which are mounted read-only.

The sandbox is also deterministic, so the same app over the same data prints the
same report every time:

- Standard input is closed.
- The random number generator returns zeros.
- Both clocks read one counter that starts at 1 and ticks once per read,
  unrelated to real time.

An app that needs a random-looking choice derives it from its input.
[10-fortune-cookie](../apps/10-fortune-cookie) does exactly that.

## Limits

| | Manifest pass | Report pass |
| :-- | :-- | :-- |
| Memory | 32 MiB | 128 MiB |
| Table | 65,536 elements | 65,536 elements |
| Standard output | 64 KiB | 1 MiB |
| Standard error | 1 KiB, never returned | 1 MiB |
| Time | 250 ms | none, but cancellable |
| Data | none | granted paths, read-only |

A manifest pass that prints more than 64 KiB is refused, and a report pass that
prints more than 1 MiB to either stream fails. A module can be up to 256 MiB.

## The develop flag

By default the Ark returns an app's standard output only when the app succeeds,
and never returns its standard error, so an app that fails returns nothing.

Setting `develop = true` in the manifest returns standard output even on failure,
and standard error every time. It is a debugging switch, and the owner sees a
developer mode warning when approving such an app. Leave it out of apps you ship.

## Running on an Ark

1. A developer sends the module to the Ark, for example with
   `ark app run app.wasm`.
2. The Ark checks the module, runs the manifest pass, then checks the manifest
   and every path it grants.
3. The owner sees the app's name, version and requested paths on their phone,
   and approves or declines.
4. On approval, the Ark mounts the granted paths read-only and runs the app.
5. The result is returned to the owner, who decides whether to release it.

Every step is recorded in a signed, tamper-evident journal on the device.
