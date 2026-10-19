//! One structural variant through regions/ (the mini version).
//!
//! Lists the changes in a short window of the ACE gene and prints what each one
//! is, in a handful of lines. The full report is ../08-sprint-or-marathon.

use std::fs;
use std::io;
use std::path::Path;

/// The one directory the app asks for, relative to the data root.
const LENS: &str = "v1/genome/regions/chr17/63488479-63488579";

/// One change entry, with the leaves the app prints.
struct Change {
    /// Name as listed, start-end with an optional suffix.
    name: String,
    /// First base, last base and place among the entries sharing the span.
    span: (u64, u64, u64),
    /// The `type` leaf, such as SNV, INS or DEL.
    kind: Option<String>,
    /// How many bases replace the span.
    new_bases: Option<u64>,
    /// The variant file the change came from.
    dataset: Option<String>,
    /// How many copies carry the change, as written.
    carried: Option<String>,
}

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Structural Variant"
version = "0.1.0"

[reads]
paths = ["v1/genome/regions/chr17/63488479-63488579"]

[listing]
language = "en"
icon = "🧩"
summary = "Nearly 300 bases of Alu, read as plain files, in four languages."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/08-structural-mini-rust"
keywords = ["tutorial", "changes", "structural variants", "ACE"]
description = '''
Grant a short window inside the _ACE_ gene and list its `changes`. Each directory is one alternate allele you carry, named after the span it changes. Its `type`, `bases`, `dataset` and `carried` say what changed, into what, in which file and on how many copies.

The window holds the ACE insertion, nearly 300 bases long and too long for a small variant caller, so it comes from your structural variant file. The window spans 101 bases, since callers can place an insertion a few bases apart.

An interval grant also covers your small variants inside it. Sprint or Marathon turns the same read into a report.
'''

[listing.purposes]
"v1/genome/regions/chr17/63488479-63488579" = "101 bases of the ACE gene, around its Alu insertion"
"#
        );
        return;
    };
    if let Err(err) = run(Path::new(&root)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

/// Only ENOENT means no answer. Scalar values have no trailing newline.
fn leaf(base: &Path, name: &str) -> Result<Option<String>, String> {
    match fs::read_to_string(base.join(name)) {
        Ok(value) => Ok(Some(value)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("Could not read {name}: {err}")),
    }
}

/// Splits an entry name such as 100-100 or 100-100-2 into start, end and suffix.
fn span(name: &str) -> Result<(u64, u64, u64), String> {
    let (start, rest) = name.split_once('-').ok_or("Invalid change name")?;
    let (end, suffix) = rest.split_once('-').unwrap_or((rest, "1"));
    Ok((
        start
            .parse::<u64>()
            .map_err(|err| format!("Invalid change start: {err}"))?,
        end.parse::<u64>()
            .map_err(|err| format!("Invalid change end: {err}"))?,
        suffix
            .parse::<u64>()
            .map_err(|err| format!("Invalid change suffix: {err}"))?,
    ))
}

/// Counts the new bases from the size of `bases`, or from `length` for a symbolic
/// insertion. A long allele is measured, never read whole.
fn new_bases(base: &Path) -> Result<Option<u64>, String> {
    match fs::metadata(base.join("bases")) {
        Ok(info) => Ok(Some(info.len())),
        Err(err) if err.kind() == io::ErrorKind::NotFound => leaf(base, "length")?
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|err| format!("Invalid length: {err}"))
            })
            .transpose(),
        Err(err) => Err(format!("Could not read bases size: {err}")),
    }
}

/// Prints every change in the window, reading an empty listing as no answer.
fn run(root: &Path) -> Result<(), String> {
    // A missing listing has no answer, just like an empty one
    let entries = match fs::read_dir(root.join(LENS).join("changes")) {
        Ok(entries) => Some(entries),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => return Err(format!("Could not list changes: {err}")),
    };
    let mut changes = Vec::new();
    if let Some(entries) = entries {
        for entry in entries {
            let entry = entry.map_err(|err| format!("Could not read change entry: {err}"))?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "Invalid change name")?;
            changes.push(Change {
                span: span(&name)?,
                name,
                kind: leaf(&entry.path(), "type")?,
                new_bases: new_bases(&entry.path())?,
                dataset: leaf(&entry.path(), "dataset")?,
                carried: leaf(&entry.path(), "carried")?,
            });
        }
    }
    // Listing order isn't promised, so sort by span before printing
    changes.sort_by_key(|change| change.span);

    println!("## Changes near the ACE insertion\n");
    if changes.is_empty() {
        println!(
            "No change reaches this window. That is no answer, since your files may not cover it."
        );
    }
    for change in changes {
        let bases = match change.new_bases {
            Some(1) => "1 new base".into(),
            Some(n) => format!("{n} new bases"),
            None => "no new bases".into(),
        };
        let copies = match change.carried.as_deref() {
            Some("1") => "1 copy".into(),
            Some(n) => format!("{n} copies"),
            None => "no answer".into(),
        };
        println!(
            "- {}: {}, {bases}, {}, carried on {copies}",
            change.name,
            change.kind.as_deref().unwrap_or("no answer"),
            change.dataset.as_deref().unwrap_or("no answer")
        );
    }
    Ok(())
}
