//! Required and optional directory grants and the develop flag.
//!
//! The app reads a required genotype and checks whether an optional grant is
//! mounted before reading its answer. It then checks that an undeclared call
//! file is blocked. A declined or unavailable optional grant stays unmounted.
//! With develop enabled, the owner also reviews stderr, and stdout when the app
//! fails. The local runner always shows both streams. Ship without develop.

use std::path::Path;
use std::{fs, io};

/// Required grant that the Ark mounts before the report pass.
const GRANTED: &str = "v1/genome/rsids/rs72921001";
/// Optional grant that the Ark mounts when available and approved.
const OPTIONAL: &str = "v1/genome/rsids/rs671";
/// Undeclared call file that stays unmounted, so reading it fails.
const DENIED: &str = "v1/genome/snp-indel/vcf";

/// Prints the manifest or reports which grants can be read.
fn main() {
    // Print the manifest when no data root was passed
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "manifest = 1\n\n\
             [app]\n\
             name = \"permissions\"\n\
             version = \"0.2.0\"\n\
             develop = true\n\n\
             [reads]\n\
             paths = [\"{GRANTED}\"]\n\
             optional = [\"{OPTIONAL}\"]\n"
        );
        return;
    };

    // Open the report before checking each grant
    let root = Path::new(&dir);
    println!("## Permissions\n");

    // The required grant allows the read, but the genotype can have no answer
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

    // Check whether the optional directory is mounted before reading its answer
    match fs::read_dir(root.join(OPTIONAL)) {
        Ok(_) => match fs::read_to_string(root.join(OPTIONAL).join("genotype")) {
            Ok(genotype) => println!("- optional `{OPTIONAL}`: read genotype `{genotype}`"),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                println!("- optional `{OPTIONAL}`: no genotype answer");
            }
            Err(err) => {
                eprintln!("Could not read the optional genotype: {err}");
                std::process::exit(1);
            }
        },
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
            ) =>
        {
            println!("- optional `{OPTIONAL}`: not mounted, so either declined or not on this Ark");
        }
        Err(err) => {
            eprintln!("Could not check the optional grant: {err}");
            std::process::exit(1);
        }
    }

    // The undeclared path stays unmounted, so reading it fails
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

    // The owner sees this diagnostic only because the manifest sets develop
    eprintln!("debug: this line is on stderr, visible only because develop = true");
}
