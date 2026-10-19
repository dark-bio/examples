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
const DENIED: &str = "v1/genome/files/small-variants/vcf";

/// Prints the manifest or reports which grants can be read.
fn main() {
    // Print the manifest when no data root was passed
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Permissions"
version = "0.2.0"
develop = true

[reads]
paths = ["{GRANTED}"]
optional = ["{OPTIONAL}"]

[listing]
language = "en"
icon = "🔒"
summary = "One grant, one maybe, one locked door, and the develop flag."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/02-permissions"
keywords = ["tutorial", "grants", "optional grants", "develop flag", "sandbox"]
description = '''
A grant opens one directory and everything under it, nothing else. This app reads its one granted variant, then tries the call file it never asked for, and bounces off.

It also asks for a second variant as an optional grant, switched off until you flip it. A declined grant reads exactly like missing data, so the app can't know you said no.

`develop = true` sends the app's standard error to your review too, failures included, and your phone warns you about it. Leave the flag out of apps you ship.

Approve it and it gets one variant, or two if you flip the switch. The locked door is the Ark's doing, not the app's good manners.
'''

[listing.purposes]
"v1/genome/rsids/rs72921001" = "The variant this demo reads, to show a granted read"
"v1/genome/rsids/rs671" = "An optional second variant, to show what a declined grant looks like"
"#
        );
        return;
    };

    // Open the report before checking each grant
    let root = Path::new(&dir);
    println!("## The guest list\n");

    // The required grant allows the read, but the genotype can have no answer
    match fs::read_to_string(root.join(GRANTED).join("genotype")) {
        Ok(genotype) => println!("- granted `{GRANTED}`: in, genotype `{}`", genotype),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!(
                "- granted `{GRANTED}`: in, but silent, and silence is not homozygous reference"
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
            Ok(genotype) => {
                println!("- optional `{OPTIONAL}`: in, the owner said yes, genotype `{genotype}`")
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                println!("- optional `{OPTIONAL}`: in, the owner said yes, but silent");
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
            println!(
                "- optional `{OPTIONAL}`: out, declined or not on this Ark, and the app can't know which"
            );
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
            println!("- undeclared `{DENIED}`: blocked. Nice try.");
        }
        Err(err) => {
            eprintln!("Could not check the undeclared path: {err}");
            std::process::exit(1);
        }
    }

    // The owner sees this diagnostic only because the manifest sets develop
    eprintln!("debug: you only see this line because develop = true");
}
