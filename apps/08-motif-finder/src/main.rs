//! Scans a gene's public sequence for restriction enzyme sites.
//!
//! The gene grant also covers personal changes, although this app reads only
//! sequence. A grant always covers everything beneath its directory.

use std::fs::File;
use std::io::{BufReader, Read};
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
    if let Err(err) = run(Path::new(&dir)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(root: &Path) -> Result<(), String> {
    let file = File::open(root.join(LENS).join("sequence"))
        .map_err(|err| format!("Could not open {GENE} sequence: {err}"))?;
    let mut cuts = [0u64; ENZYMES.len()];
    let mut length = 0u64;
    let mut window = [0u8; 4];
    // Keep a sliding window across reads, including overlapping sites.
    for base in BufReader::new(file).bytes() {
        let base = base.map_err(|err| format!("Could not read {GENE} sequence: {err}"))?;
        window.rotate_left(1);
        window[3] = base.to_ascii_uppercase();
        length += 1;
        for (i, (_, site)) in ENZYMES.iter().enumerate() {
            if length >= 4 && window == site.as_bytes() {
                cuts[i] += 1;
            }
        }
    }
    if length == 0 {
        return Err("The gene sequence is empty".into());
    }

    println!("## Restriction map of {GENE}\n");
    println!("Scanning {length} bp of reference sequence.\n");
    println!("| Enzyme | Site | Cuts |");
    println!("| :--- | :--- | ---: |");
    for ((name, site), count) in ENZYMES.iter().zip(cuts) {
        println!("| {name} | `{site}` | {count} |");
    }
    Ok(())
}
