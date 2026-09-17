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
             version = \"0.2.0\"\n\
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

    // The finding names the totals; the table beneath it carries every count.
    let total: u64 = cuts.iter().sum();
    let (most, most_cuts) = ENZYMES
        .iter()
        .zip(cuts)
        .max_by_key(|(_, count)| *count)
        .map(|((name, _), count)| (*name, count))
        .expect("the enzyme panel is not empty");
    let uncut: Vec<&str> = ENZYMES
        .iter()
        .zip(cuts)
        .filter(|(_, count)| *count == 0)
        .map(|((name, _), _)| *name)
        .collect();

    println!("# Restriction Map of {GENE}\n");
    print!(
        "{} enzymes cut the {} bp reference sequence of *{GENE}* {total} times between \
         them. {most} cuts most often, {most_cuts} times",
        ENZYMES.len(),
        commas(length)
    );
    match uncut.as_slice() {
        [] => println!("."),
        [one] => println!(", and {one} not at all."),
        many => println!(", and {} never do.", many.join(", ")),
    }
    println!();
    println!("## Evidence\n");
    println!("| Enzyme | Site | Cuts |");
    println!("| :-- | :-- | --: |");
    for ((name, site), count) in ENZYMES.iter().zip(cuts) {
        println!("| {name} | `{site}` | {count} |");
    }
    println!();
    println!("## Method\n");
    println!(
        "The app streams the gene's reference sequence through a four-base window and \
         counts every position where an enzyme's recognition site appears, overlaps \
         included. Repeats are soft-masked in lowercase, so bases are uppercased before \
         matching.\n"
    );
    println!("## Limitations\n");
    println!(
        "This is the reference sequence, not yours. The grant covers your changes in the \
         gene, but the app never reads them, so a variant that creates or destroys a site \
         isn't counted."
    );
    Ok(())
}

/// Formats a count with thousands separators, so 1143 reads as 1,143.
fn commas(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}
