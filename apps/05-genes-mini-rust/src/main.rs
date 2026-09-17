//! One gene through the genes/ lens (the mini version).
//!
//! Reads a gene's scalar metadata leaves and the length of its reference
//! sequence, in a handful of lines. The full report is ../05-bitter-meter.

use std::fs;
use std::io;
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
    if let Err(err) = run(Path::new(&dir)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(root: &Path) -> Result<(), String> {
    let base = root.join(LENS);
    // Sequence size is end - start + 1; no need to load a long gene into memory.
    let length = fs::metadata(base.join("sequence"))
        .map_err(|err| format!("Could not read {GENE} sequence: {err}"))?
        .len();
    if length == 0 {
        return Err("The gene sequence is empty".into());
    }
    let leaf = |name: &str| -> Result<Option<String>, String> {
        match fs::read_to_string(base.join(name)) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(format!("Could not read {GENE} {name}: {err}")),
        }
    };
    let chromosome = leaf("chromosome")?;
    let start = leaf("start")?
        .map(|s| s.parse::<u64>())
        .transpose()
        .map_err(|err| format!("Invalid gene start: {err}"))?;
    let end = leaf("end")?
        .map(|s| s.parse::<u64>())
        .transpose()
        .map_err(|err| format!("Invalid gene end: {err}"))?;
    let strand = leaf("strand")?;
    let biotype = leaf("biotype")?;
    if let (Some(start), Some(end)) = (start, end) {
        if start == 0 || end < start || length != end - start + 1 {
            return Err("Gene sequence length does not match its span".into());
        }
    }

    println!("## Gene {GENE}\n");
    println!(
        "- Chromosome: {}",
        chromosome.as_deref().unwrap_or("no answer")
    );
    println!(
        "- Start: {}",
        start
            .map(|n| n.to_string())
            .as_deref()
            .unwrap_or("no answer")
    );
    println!(
        "- End: {}",
        end.map(|n| n.to_string()).as_deref().unwrap_or("no answer")
    );
    println!("- Strand: {}", strand.as_deref().unwrap_or("no answer"));
    println!("- Biotype: {}", biotype.as_deref().unwrap_or("no answer"));
    println!("- Reference length: {length} bp");
    Ok(())
}
