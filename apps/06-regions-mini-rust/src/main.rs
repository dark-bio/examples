//! One interval through the regions/ lens (the mini version).
//!
//! Reads a region's reference sequence and reports its length and GC content.
//! The full report is ../06-powerhouse-of-the-cell.

use std::fs::File;
use std::io::Read;
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
    if let Err(err) = run(Path::new(&dir)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(root: &Path) -> Result<(), String> {
    let mut sequence = File::open(root.join(LENS).join("sequence"))
        .map_err(|err| format!("Could not open region sequence: {err}"))?;
    let (mut length, mut gc) = (0u64, 0u64);
    let mut buffer = [0; 8192];
    // Forward-strand sequence has no newline; lowercase bases are soft-masked.
    loop {
        let n = sequence
            .read(&mut buffer)
            .map_err(|err| format!("Could not read region sequence: {err}"))?;
        if n == 0 {
            break;
        }
        length += n as u64;
        gc += buffer[..n]
            .iter()
            .filter(|b| matches!(b, b'G' | b'g' | b'C' | b'c'))
            .count() as u64;
    }
    if length != 16569 {
        return Err("Region sequence length does not match its span".into());
    }
    let pct = 100.0 * gc as f64 / length as f64;
    println!("## Region {REGION}\n");
    println!("- Reference length: {length} bp");
    println!("- GC content: {pct:.1}%");
    Ok(())
}
