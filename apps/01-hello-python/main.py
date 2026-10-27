"""The smallest possible Ark app, in Python."""
import sys


def main():
    # With no data directory, print the manifest and stop.
    if len(sys.argv) < 2:
        print("""manifest = 1

[app]
name = "Hello, Ark"
version = "0.1.0"

[listing]
language = "en"
icon = "👋"
summary = "Two passes, a handful of lines, zero data. Start here."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/01-hello-python"
keywords = ["tutorial", "manifest", "two passes", "WASI"]
description = '''
An Ark app is one WebAssembly file built for WASI Preview 1, and the Ark runs it twice.

**The manifest pass.** No arguments. The app prints a short TOML manifest with its name, its version and the data it wants, and your phone shows it to you.

**The processing pass.** A data directory as its first argument. The app reads
what you granted and prints a Markdown report, which Ark Hub renders.

This one asks for nothing, so approving it shares nothing. It's the whole round trip, upload to approval to report, in a handful of lines.

Each language's folder in the examples repository builds and runs it with one `make run`.

Pick a language, read the source, then run it on your Ark.
'''""")
        return

    # The processing pass receives a data directory even without grants
    print("Hello from inside an Ark. No network, no disk, no way out. Python says hi.")


if __name__ == "__main__":
    main()
