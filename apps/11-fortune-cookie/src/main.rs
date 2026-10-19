//! Why the sandbox is deterministic.
//!
//! The runtime gives an app no randomness. The RNG returns zeros and the clock
//! is a counter, not the time of day. So an app cannot roll dice. To make a
//! choice that feels random it has to derive it from its input. This one folds a
//! few of your genotypes into a number and picks a fortune from it. The same
//! genome gives the same fortune every time it runs. Only the data varies.

use std::path::Path;
use std::{fs, io};

const SITES: &[&str] = &["rs72921001", "rs671", "rs1229984"];

const FORTUNES: &[&str] = &[
    "A quiet allele today keeps the soap away.",
    "Your codons align for a productive afternoon.",
    "Trust your enzymes, and skip the buffet shrimp.",
    "A double helix bends, but never toward regret.",
    "Today, let the recessive traits rest.",
    "Fortune favors the heterozygous.",
    "The reference genome believes in you.",
    "Somewhere, a ribosome is rooting for you.",
];

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Fortune Cookie"
version = "0.2.0"

[reads]
paths = ["v1/genome/rsids/rs72921001", "v1/genome/rsids/rs671", "v1/genome/rsids/rs1229984"]

[listing]
language = "en"
icon = "🥠"
summary = "A fortune baked from three of your genotypes. Same cookie every time."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/11-fortune-cookie"
keywords = ["fortune", "deterministic", "sandbox", "hashing"]
description = '''
The Ark's sandbox gives apps no random bytes and clocks that only count. With no dice to roll, this app hashes the genotypes of three variants into a fortune. Same data, same fortune, forever.

An absent genotype adds nothing to the hash and gets a mention in the report. Any other read error stops the app. It's a deterministic sandbox demo, wrapped in a cookie.
'''

[listing.purposes]
"v1/genome/rsids/rs72921001" = "One of three genotypes hashed into your fortune"
"v1/genome/rsids/rs671" = "One of three genotypes hashed into your fortune"
"v1/genome/rsids/rs1229984" = "One of three genotypes hashed into your fortune"
"#
        );
        return;
    };
    let rsids = Path::new(&dir).join("v1/genome/rsids");

    // Fold the genotypes into a stable number with a small FNV-1a hash. This is
    // the app's only source of variety; the runtime provides none.
    let mut unanswered = 0;
    let mut acc: u64 = 0xcbf29ce484222325;
    for site in SITES {
        let genotype = match fs::read_to_string(rsids.join(site).join("genotype")) {
            Ok(value) => value,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                unanswered += 1;
                continue;
            }
            Err(err) => {
                eprintln!("Could not read {site} genotype: {err}");
                std::process::exit(1);
            }
        };
        for byte in genotype.bytes() {
            acc ^= byte as u64;
            acc = acc.wrapping_mul(0x100000001b3);
        }
    }

    let pick = (acc % FORTUNES.len() as u64) as usize;
    println!("# 🥠 Your Genomic Fortune\n");
    println!("> {}\n", FORTUNES[pick]);
    println!("## Method\n");
    println!(
        "The sandbox has no clock and no dice, so your fortune comes from your genotypes \
         at three variants, folded into one number that picks a line. Same data, same \
         fortune, every single time. A genotype without an answer adds nothing to the \
         number, and this run had {unanswered} of those.\n"
    );
    println!("## Limitations\n");
    println!(
        "A fortune is a fortune. Nothing here reads your genes for meaning, only for \
         bytes."
    );
}
