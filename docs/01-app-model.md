# The app model

An Ark app is a WebAssembly module built for WASI Preview 1, with a standard
`_start` entry point. Anything that compiles to that target can be an app, and
these examples cover Rust, Go, C and Python.

The Ark doesn't need to trust an app. It trusts the sandbox the app runs in, and
everything below follows from that.

## Two passes

The Ark runs every app twice, and the first command-line argument distinguishes
the two passes.

- **The manifest pass** runs the module with no arguments. The app prints a
  short TOML manifest naming itself and the data it wants, then exits. Nothing
  is mounted yet, so it can't read anything. [02-manifest.md](02-manifest.md)
  covers the format.
- **The processing pass** runs the module again once the owner has approved
  the task, with the data directory as the first argument. The app reads its
  granted files and inputs, writes any declared outputs and may print to
  standard output and standard error.

An app picks its pass by counting arguments:

| Language | Manifest pass | Processing pass |
| :-- | :-- | :-- |
| Rust | `std::env::args().nth(1).is_none()` | otherwise |
| Go | `len(os.Args) < 2` | otherwise |
| C | `argc < 2` | otherwise |
| Python | `len(sys.argv) < 2` | otherwise |

The main app's standard output is the report. It is Markdown, since that is
how it is shown, and [06-reports.md](06-reports.md) covers its shape. A
dependency's printout is not a report. The Ark drops both of its streams,
while its declared output files remain available to other apps in the task.

## Before the owner is asked

When a task is scheduled, the Ark checks every app before any prompt reaches
the owner's phone. It checks each app in this order and refuses the task at
the first check that fails. The refusal names the rule that broke.

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
   so is a required one whose data isn't on the Ark. The refusal names the
   missing data. An optional path whose data is missing goes to the owner as
   unavailable instead.

## The sandbox

The processing pass has no network or storage that outlives its task. Grants,
inputs and dependency outputs are read-only. An app can write only the output
files its manifest declares, which start empty under `outputs/`.

The sandbox is also deterministic. The same app over the same data and inputs
prints the same streams and writes the same outputs on the same architecture:

- Standard input is closed.
- The random number generator returns zeros.
- Both clocks read one counter that starts at 1 and ticks once per read,
  unrelated to real time.

An app that needs a random-looking choice derives it from its input.
[11-fortune-cookie](../apps/11-fortune-cookie) does exactly that.

## Limits

| | Manifest pass | Processing pass |
| :-- | :-- | :-- |
| Memory | 32 MiB | 128 MiB |
| Table | 65,536 elements | 65,536 elements |
| Standard output | 64 KiB | 1 MiB |
| Standard error | 1 KiB, never returned | 1 MiB |
| Time | 250ms | none, but cancellable |
| Data | none | grants, inputs and dependency outputs, read-only |
| Outputs | none | declared files, writable, 64 MiB shared by the task |

A manifest pass that prints more than 64 KiB is refused. A processing pass
that prints more than 1 MiB to either stream fails. A module can be up to
256 MiB.

## The develop flag

After the processing pass, the owner reviews what the main app printed before
anything leaves the Ark. By default that is the report, the app's standard
output, and only when the task succeeds. A failure reaches the review too,
with nothing but the fact that it failed, and standard error never does.

Setting `develop = true` in the manifest sends standard error to the review as
well, and standard output even when the app fails. It is a debugging switch,
and the owner sees a developer warning when approving such an app. Leave it out
of apps you ship.

## Running on an Ark

1. A developer sends the module to the Ark, for example with
   `ark run app.wasm`.
2. The Ark checks every module in the task, runs its manifest pass, then checks
   its manifest, bindings and granted paths.
3. The owner sees the main app's name and version, with the combined requested
   paths of all its apps. They choose optional grants, answer the main app's
   questions, and approve or decline the task once.
4. On approval, the Ark mounts each app's grants, inputs and dependency outputs
   read-only, with its own outputs writable. It runs each processing pass when
   its dependencies and inputs are ready, with the main app last.
5. The owner reviews the main app's report, and releases it to the session that
   started the task, such as the `ark` command, or keeps it. Nothing leaves the
   Ark before that.

Every step is recorded in a signed, tamper-evident journal on the device.
