//! The smallest possible Ark app, in Rust.
//!
//! With no arguments it prints its manifest. With a data directory as the first
//! argument it does its real work, which here is to print a greeting. See
//! ../../docs/01-app-model.md for the two passes.

fn main() {
    // The manifest pass has no data directory, so describe the app and stop.
    // This app reads nothing, so it has no reads table.
    if std::env::args().nth(1).is_none() {
        print!(
            r#"manifest = 1

[app]
name = "Hello, Ark"
version = "0.1.0"

[listing]
language = "en"
icon = "👋"
summary = "The two passes of an app, in the fewest lines. Start here."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/01-hello-rust"
keywords = ["tutorial", "manifest", "two passes", "WASI"]
description = '''
An Ark app is one WebAssembly file built for WASI Preview 1. The Ark runs it twice.

**The manifest pass.** With no arguments, the app prints a short TOML manifest naming itself, its version and the data it wants. The Ark checks the manifest and shows it on your phone.

**The report pass.** With a data directory as its first argument, the app reads the files it was granted and prints its report. Reports are Markdown, which is how Ark Hub shows them.

This app asks for no data, so approving it grants nothing. It proves the round trip, from upload through approval to a rendered report, in the fewest lines possible. Pick a language, read the source, then run it on your Ark.

Build it yourself from the examples repository:

```sh
make run APP=01-hello-rust
```

The docs cover the two passes, the manifest and the sandbox limits in detail.
'''
"#
        );
        return;
    }

    // The report pass receives a data directory. This app reads nothing.
    println!("Hello from an Ark app written in Rust.");
}
