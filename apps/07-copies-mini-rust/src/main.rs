//! One gene's copy number through genes/ (the mini version).
//!
//! Lists the copy number segments reaching a gene and prints each one's count,
//! in a handful of lines. The full report is ../07-doping-test.

use std::fs;
use std::io;
use std::path::Path;

/// The one directory the app asks for, relative to the data root.
const LENS: &str = "v1/genome/genes/UGT2B17";

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Copy Number"
version = "0.1.0"

[reads]
paths = ["v1/genome/genes/UGT2B17", "v1/genome/datasets/copy-number-variants"]

[listing]
language = "en"
icon = "🔢"
summary = "Spot a gene that isn't there. Copy numbers as plain files, in four languages."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/07-copies-mini-rust"
keywords = ["tutorial", "copies", "copy number", "UGT2B17"]
description = '''
Grant `v1/genome/genes/UGT2B17` and list its `copies`. Each directory is one segment of your copy number file that reaches the gene, named after its span, and its `count` holds how many copies your genome has there.

Whole genes can go missing, and a small variant file never notices. An empty listing is no answer, never zero copies, since a caller can skip a region. The app also grants `v1/genome/datasets/copy-number-variants`, so the Ark won't run it without a copy number file.

A gene grant also covers your variants inside the gene, even though this app does not read them. Doping Test turns the same read into a report.
'''

[listing.purposes]
"v1/genome/genes/UGT2B17" = "A gene to read your copy number over"
"v1/genome/datasets/copy-number-variants" = "Your copy number file has to be on the Ark, though the app never reads it"
"#
        );
        return;
    };
    if let Err(err) = run(Path::new(&root)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

/// Splits a segment name, start-end, into its first and last base.
fn span(name: &str) -> Result<(u64, u64), String> {
    let (start, end) = name.split_once('-').ok_or("Invalid segment name")?;
    let start = start
        .parse::<u64>()
        .map_err(|err| format!("Invalid segment start: {err}"))?;
    let end = end
        .parse::<u64>()
        .map_err(|err| format!("Invalid segment end: {err}"))?;
    Ok((start, end))
}

/// Prints each segment's count, reading an empty listing as no answer.
fn run(root: &Path) -> Result<(), String> {
    // A missing listing has no answer, just like an empty one
    let base = root.join(LENS).join("copies");
    let entries = match fs::read_dir(&base) {
        Ok(entries) => Some(entries),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => return Err(format!("Could not list copies: {err}")),
    };
    let mut segments = Vec::new();
    if let Some(entries) = entries {
        for entry in entries {
            let entry = entry.map_err(|err| format!("Could not read segment: {err}"))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "Invalid segment name")?;
            let coordinates = span(&name)?;

            // Only a missing count means no answer, any other failure stops the app
            let count = match fs::read_to_string(entry.path().join("count")) {
                Ok(count) => Some(count),
                Err(err) if err.kind() == io::ErrorKind::NotFound => None,
                Err(err) => return Err(format!("Could not read {name} count: {err}")),
            };
            segments.push((coordinates, name, count));
        }
    }
    // Listing order isn't promised, so sort by the first base before printing
    segments.sort_by_key(|segment| segment.0);

    println!("## Copies of UGT2B17\n");
    if segments.is_empty() {
        println!("No copy number segment reaches UGT2B17. That is no answer, not zero copies.");
    }
    for (_, name, count) in segments {
        match count.as_deref() {
            Some("1") => println!("- Segment {name}: 1 copy"),
            Some(count) => println!("- Segment {name}: {count} copies"),
            None => println!("- Segment {name}: no answer"),
        }
    }
    Ok(())
}
