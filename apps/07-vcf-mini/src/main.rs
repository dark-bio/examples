//! Reading the raw variant file (the mini version).
//!
//! Opens the decompressed `vcf` view and counts records, with no genomics
//! library at all: the view is plain text, so a line scan is enough. The full
//! scan with noodles-vcf is ../07-vcf-roll-call.

use std::fs;
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

    let text = fs::read_to_string(Path::new(&dir).join(LENS).join("vcf")).unwrap_or_default();

    let mut headers = 0usize;
    let mut records = 0usize;
    let mut first = None;
    for line in text.lines() {
        if line.starts_with('#') {
            headers += 1;
        } else if !line.is_empty() {
            records += 1;
            first.get_or_insert(line);
        }
    }

    println!("## Variant file\n");
    println!("- Header lines: {headers}");
    println!("- Variant records: {records}");
    if let Some(line) = first {
        // The first five VCF columns are CHROM, POS, ID, REF, ALT.
        let cols: Vec<&str> = line.split('\t').take(5).collect();
        println!("- First record: `{}`", cols.join(" "));
    }
}
