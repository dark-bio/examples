//! Why the sandbox is deterministic.
//!
//! The runtime gives an app no randomness: the RNG returns zeros and the clock
//! is a counter, not the time of day. So an app cannot roll dice. To make a
//! choice that feels random it has to derive it from its input. This one folds a
//! few of your genotypes into a number and picks a fortune from it. The same
//! genome gives the same fortune on every run, which is the whole point: nothing
//! here varies that is not in the data.

use std::fs;
use std::path::Path;

const SITES: &[&str] = &["rs72921001", "rs671", "rs1229984"];

const FORTUNES: &[&str] = &[
    "A quiet allele today keeps the soap away.",
    "Your codons align for a productive afternoon.",
    "Trust your enzymes; skip the buffet shrimp.",
    "A double helix bends, but never toward regret.",
    "Today, let the recessive traits rest.",
    "Fortune favors the heterozygous.",
    "The reference genome believes in you.",
    "Somewhere, a ribosome is rooting for you.",
];

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"fortune-cookie\"\n\
             version = \"0.1.0\"\n\
             datasets = [\
             \"v1/genome/rsids/rs72921001\", \
             \"v1/genome/rsids/rs671\", \
             \"v1/genome/rsids/rs1229984\"]\n"
        );
        return;
    };
    let rsids = Path::new(&dir).join("v1/genome/rsids");

    // Fold the genotypes into a stable number with a small FNV-1a hash. This is
    // the app's only source of variety; the runtime provides none.
    let mut acc: u64 = 0xcbf29ce484222325;
    for site in SITES {
        let genotype = fs::read_to_string(rsids.join(site).join("genotype")).unwrap_or_default();
        for byte in genotype.trim().bytes() {
            acc ^= byte as u64;
            acc = acc.wrapping_mul(0x100000001b3);
        }
    }

    let pick = (acc % FORTUNES.len() as u64) as usize;
    println!("## Your genomic fortune\n");
    println!("> {}\n", FORTUNES[pick]);
    println!(
        "The sandbox has no randomness, so this is derived entirely from your \
         genotypes. Run it again on the same data and the fortune is the same."
    );
}
