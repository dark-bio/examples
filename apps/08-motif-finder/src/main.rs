//! Reading reference sequence, no genotype.
//!
//! This app reads only a gene's reference sequence and scans it for restriction
//! enzyme recognition sites. It never touches your genotype, so it shows the
//! genes/ lens used purely for sequence: the reference is just a string of bases
//! you can compute over.

use std::fs;
use std::path::Path;

const GENE: &str = "TAS2R38";
const LENS: &str = "v1/genome/genes/TAS2R38";

/// A small panel of restriction enzymes and their recognition sites.
const ENZYMES: &[(&str, &str)] = &[
    ("AluI", "AGCT"),
    ("HaeIII", "GGCC"),
    ("RsaI", "GTAC"),
    ("TaqI", "TCGA"),
    ("Sau3AI", "GATC"),
];

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"motif-finder\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/genes/TAS2R38\"]\n"
        );
        return;
    };

    let reference =
        fs::read_to_string(Path::new(&dir).join(LENS).join("reference")).unwrap_or_default();
    let seq: String = reference
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .flat_map(|c| c.to_uppercase())
        .collect();

    println!("## Restriction map of {GENE}\n");
    println!("Scanning {} bp of reference sequence.\n", seq.len());
    println!("| Enzyme | Site | Cuts |");
    println!("| :--- | :--- | ---: |");
    for (name, site) in ENZYMES {
        println!("| {name} | `{site}` | {} |", count_occurrences(&seq, site));
    }
}

/// Counts non-overlapping-from-each-index (i.e. every starting offset) matches
/// of `needle` in `haystack`.
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    let (hay, ndl) = (haystack.as_bytes(), needle.as_bytes());
    if ndl.is_empty() || hay.len() < ndl.len() {
        return 0;
    }
    (0..=hay.len() - ndl.len())
        .filter(|&i| &hay[i..i + ndl.len()] == ndl)
        .count()
}
