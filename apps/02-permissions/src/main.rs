//! A directory grant and the develop flag.
//!
//! The app reads a granted genotype if it has an answer, then checks that an
//! undeclared call file is blocked. The local runner mounts only declared paths.
//! With develop enabled, an Ark returns stderr and stdout even on failure.
//! The local runner always shows both streams. Ship without develop.

use std::path::Path;
use std::{fs, io};

/// Declared in the manifest, so the Ark mounts it.
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

    // The grant allows the read; the genotype can still have no answer.
    match fs::read_to_string(root.join(GRANTED).join("genotype")) {
        Ok(genotype) => println!("- granted `{GRANTED}`: read genotype `{}`", genotype),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!(
                "- granted `{GRANTED}`: no genotype answer (absence is not homozygous reference)"
            );
        }
        Err(err) => {
            eprintln!("Could not read the granted genotype: {err}");
            std::process::exit(1);
        }
    }

    // The undeclared path was never mounted, so this fails. On a device the same
    // holds: an app cannot reach data the owner did not approve.
    match fs::read_to_string(root.join(DENIED)) {
        Ok(_) => {
            eprintln!("The undeclared path was readable; check the sandbox mounts.");
            std::process::exit(1);
        }
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
            ) =>
        {
            println!("- undeclared `{DENIED}`: blocked, as it should be");
        }
        Err(err) => {
            eprintln!("Could not check the undeclared path: {err}");
            std::process::exit(1);
        }
    }

    // This goes to standard error. The Ark returns stderr only when the manifest
    // sets develop = true; otherwise you would never see it.
    eprintln!("debug: this line is on stderr, visible only because develop = true");
}
