//! The smallest possible Ark app, in Rust.
//!
//! With no arguments it prints its manifest. With a data directory as the first
//! argument it does its real work, which here is to print a greeting. See
//! ../../docs/01-app-model.md for the two passes.

fn main() {
    // The manifest pass: no data directory was given, so describe the app and
    // stop. This app reads nothing, so it has no reads table.
    if std::env::args().nth(1).is_none() {
        print!(
            "manifest = 1\n\n\
             [app]\n\
             name = \"hello-rust\"\n\
             version = \"0.1.0\"\n"
        );
        return;
    }

    // The report pass receives a data directory. This app reads nothing.
    println!("Hello from an Ark app written in Rust.");
}
