//! One gene through the genes/ lens (the mini version).
//!
//! Reads a gene's scalar metadata leaves and the length of its reference
//! sequence, in a handful of lines. The full report is ../05-bitter-meter.

use std::fs;
use std::path::Path;

const GENE: &str = "TAS2R38";
const LENS: &str = "v1/genome/genes/TAS2R38";

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"genes-mini\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/genes/TAS2R38\"]\n"
        );
        return;
    };

    let base = Path::new(&dir).join(LENS);
    let leaf = |name: &str| {
        fs::read_to_string(base.join(name))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    println!("## Gene {GENE}\n");
    println!("- Chromosome: {}", leaf("chromosome"));
    println!("- Start: {}", leaf("start"));
    println!("- End: {}", leaf("end"));
    println!("- Strand: {}", leaf("strand"));
    println!("- Biotype: {}", leaf("biotype"));
    println!("- Reference length: {} bp", leaf("reference").len());
}
