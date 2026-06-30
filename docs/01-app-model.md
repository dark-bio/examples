# The app model

An Ark app is a WebAssembly module built against WASI Preview 1, with a standard
`_start` entry point. That is the whole interface. Anything that compiles to
that target can be an app; the examples cover Rust, Go, and C.

The Ark never trusts the app. It trusts the box the app runs in. Everything
below follows from that.

## The two passes

The Ark runs an app twice, and tells the two apart by a single command-line
argument.

The **manifest pass** comes first. The Ark runs the app with no arguments. The
app prints a TOML manifest to standard output and exits. The manifest names the
app and lists the data it wants (see [02-manifest.md](02-manifest.md)). Nothing
is mounted yet, so the app cannot read anything during this pass; it only
describes itself.

The **run pass** comes second, and only if the owner approves. The Ark mounts
the requested data, then runs the app again with the data directory as its first
argument, conventionally `/`. The app reads its files and prints a report.

Apps detect the pass by argument count:

| Language | Manifest pass | Run pass |
| :-- | :-- | :-- |
| Rust | `std::env::args().nth(1).is_none()` | otherwise |
| Go | `len(os.Args) < 2` | otherwise |
| C | `argc < 2` | otherwise |

The report an app prints is treated as Markdown. The owner sees it rendered on
their phone, so headings, tables, and emphasis are worth using.

## The sandbox

The run pass executes inside a sandbox with no network, no writable storage, and
no access to anything the app was not explicitly granted. Granted data is mounted
read-only.

The sandbox is also **deterministic**. The same app over the same data produces
the same output every time, because the usual sources of variation are removed:

- Standard input is closed.
- The random number generator returns zeros.
- The wall clock and the monotonic clock are counters that start at one and tick
  up by one on each call, unrelated to real time.

An app that needs a "random" choice must derive it from its input, not from the
runtime. [09-fortune-cookie](../apps) leans on this on purpose.

## Limits

Each pass runs under its own resource limits. The manifest pass is held to a
tight budget because it should only print a few lines and exit.

| | Manifest pass | Run pass |
| :-- | :-- | :-- |
| Memory | about 16 MiB | 100 MiB |
| Standard output | 1 KiB | 1 MiB |
| Standard error | 1 KiB | 1 MiB |
| Time | 250 ms | none, but cancellable |
| Filesystem | none | granted data, read-only |

Output past the cap is dropped, so keep the manifest small and the report within
a megabyte. The run pass has no time limit, but the owner can cancel it from
their phone.

## The develop flag

By default the Ark returns an app's standard output only when the app exits
successfully, and never returns its standard error. A failed run returns
nothing. This keeps incidental output from leaking off the device.

Setting `develop = true` in the manifest changes that: standard output is
returned even on failure, and standard error is always returned. It is the
debugging switch. [02-permissions](../apps) uses it to show an error that would
otherwise be invisible. Ship without it.

## Lifecycle on the device

For context, a run on a real Ark goes:

1. The app is uploaded to the device.
2. The Ark runs the manifest pass and reads the manifest.
3. The owner is shown the app's name and the exact data it asked for, and
   approves or declines on their phone.
4. On approval, the Ark mounts that data read-only and runs the app.
5. The result is returned to the owner, who decides whether to release it.

Every step is recorded in a signed, tamper-evident journal on the device. As an
app author you only write the program; the Ark drives the rest.
