//! Permissions and the develop flag.
//!
//! An app reads only what it declares. This one declares a single variant
//! directory, reads it (works), then deliberately reaches for a slot it did NOT
//! declare. The Ark mounts only declared data, so that second read fails. The
//! local runner mounts only the declared datasets too, so you see the same wall.
//!
//! It also sets `develop = true`. On a device that changes what comes back: with
//! develop, standard output is returned even when the app fails, and standard
//! error is always returned; without it, a failed run returns nothing and stderr
//! is never returned. The local runner always shows both, so this is the one
//! place the two differ. Ship without develop.

use std::fs;
use std::path::Path;

/// Declared in the manifest, so the Ark mounts it and this read works.
const GRANTED: &str = "v1/genome/rsids/rs72921001";
/// Not declared, so it is never mounted and reading it fails.
const DENIED: &str = "v1/genome/snp-indel/vcf";

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"permissions\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/rsids/rs72921001\"]\n\
             develop = true\n"
        );
        return;
    };
    let root = Path::new(&dir);

    println!("## Permissions\n");

    // The declared path is mounted, so this succeeds.
    match fs::read_to_string(root.join(GRANTED).join("genotype")) {
        Ok(genotype) => println!("- granted `{GRANTED}`: read genotype `{}`", genotype.trim()),
        Err(err) => println!("- granted `{GRANTED}`: unexpected failure ({err})"),
    }

    // The undeclared path was never mounted, so this fails. On a device the same
    // holds: an app cannot reach data the owner did not approve.
    match fs::read_to_string(root.join(DENIED)) {
        Ok(_) => println!("- undeclared `{DENIED}`: read succeeded (would not happen on a device)"),
        Err(err) => println!("- undeclared `{DENIED}`: blocked, as it should be ({err})"),
    }

    // This goes to standard error. The Ark returns stderr only when the manifest
    // sets develop = true; otherwise you would never see it.
    eprintln!("debug: this line is on stderr, visible only because develop = true");
}
