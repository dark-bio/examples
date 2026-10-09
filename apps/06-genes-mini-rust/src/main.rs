//! One gene through the genes/ lens (the mini version).
//!
//! Reads a gene's scalar metadata leaves and the length of its reference
//! sequence, in a handful of lines. The full report is ../06-bitter-meter.

use std::fs;
use std::io;
use std::path::Path;

const GENE: &str = "TAS2R38";
const LENS: &str = "v1/genome/genes/TAS2R38";

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "One Gene"
version = "0.1.0"

[reads]
paths = ["v1/genome/genes/TAS2R38"]

[listing]
language = "en"
icon = "🧬"
summary = "A gene's coordinates and sequence length, through the genes lens."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/06-genes-mini-rust"
keywords = ["tutorial", "genes", "sequence", "TAS2R38"]
description = '''
Grant `v1/genome/genes/TAS2R38` and read the gene's `chromosome`, `start`, `end`, `strand` and `biotype` as files. The `sequence` file holds exactly end - start + 1 bases, so its size is the gene's length and the gene never has to fit in memory.

An absent value prints as no answer. A missing sequence is an error.

A gene grant also covers your variants inside the gene, even though this app does not read them. Bitter Meter does.
'''

[listing.purposes]
"v1/genome/genes/TAS2R38" = "A gene to read coordinates and sequence length from"
"#
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
