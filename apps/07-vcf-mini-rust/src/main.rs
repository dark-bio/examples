//! Reading the raw variant file (the mini version).
//!
//! Opens the decompressed `vcf` view and counts records, with no genomics
//! library at all: the view is plain text, so a line scan is enough. The full
//! scan with noodles-vcf is ../07-vcf-roll-call.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const LENS: &str = "v1/genome/snp-indel";

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"vcf-mini\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/snp-indel\"]\n"
        );
        return;
    };
    if let Err(err) = run(Path::new(&dir)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(root: &Path) -> Result<(), String> {
    let file = File::open(root.join(LENS).join("vcf"))
        .map_err(|err| format!("Could not open VCF: {err}"))?;
    let (mut headers, mut records) = (0u64, 0u64);
    let mut first = None;
    // Whole-genome VCFs exceed sandbox memory; keep only one line at a time.
    // A large buffer matters, since each refill is a call out of the sandbox,
    // and reusing one line keeps millions of records from each allocating.
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|err| format!("Could not read VCF: {err}"))?;
        if read == 0 {
            break;
        }
        let record = line.trim_end_matches('\n').trim_end_matches('\r');
        if record.starts_with('#') {
            headers += 1;
        } else if !record.is_empty() {
            records += 1;
            if first.is_none() {
                first = Some(record.split('\t').take(5).collect::<Vec<_>>().join(" "));
            }
        }
    }

    println!("## Variant file\n");
    println!("- Header lines: {headers}");
    println!("- Variant records: {records}");
    if let Some(record) = first {
        println!("- First record: `{record}`");
    }
    Ok(())
}
