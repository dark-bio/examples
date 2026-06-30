//! One interval through the regions/ lens (the mini version).
//!
//! Reads a region's reference sequence and reports its length and GC content.
//! The full report is ../06-powerhouse-of-the-cell.

use std::fs;
use std::path::Path;

const REGION: &str = "chrM:1-16569";
const LENS: &str = "v1/genome/regions/chrM/1-16569";

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"regions-mini\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/regions/chrM/1-16569\"]\n"
        );
        return;
    };

    let reference =
        fs::read_to_string(Path::new(&dir).join(LENS).join("reference")).unwrap_or_default();
    let bases: Vec<char> = reference.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let gc = bases
        .iter()
        .filter(|c| matches!(c.to_ascii_uppercase(), 'G' | 'C'))
        .count();
    let pct = if bases.is_empty() {
        0.0
    } else {
        100.0 * gc as f64 / bases.len() as f64
    };

    println!("## Region {REGION}\n");
    println!("- Reference length: {} bp", bases.len());
    println!("- GC content: {pct:.1}%");
}
