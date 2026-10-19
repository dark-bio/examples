//! Powerhouse of the Cell: a report from one interval under `regions/`.
//!
//! The mitochondrial chromosome is small enough to read as one interval, which
//! makes it a useful demo of `regions/`. One granted interval exposes a streamed
//! reference plus every change carried inside it, printed in the report shape
//! docs/06-reports.md describes.
//!
//! The app asks the Ark for the interval, `v1/genome/regions/chrM/1-16569`, and
//! for `v1/genome/files/reference-genome`, a public grant it reads only for the
//! assembly build its coordinates are reported on.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const CHROM: &str = "chrM";
const START: u64 = 1;
const END: u64 = 16_569;
const LENS_PATH: &str = "v1/genome/regions/chrM/1-16569";
const MAP_WIDTH: usize = 72;
const MAX_CHANGE_ROWS: usize = 28;

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Powerhouse of the Cell"
version = "0.2.0"

[reads]
paths = ["v1/genome/regions/chrM/1-16569", "v1/genome/files/reference-genome"]

[listing]
language = "en"
icon = "⚡"
summary = "The genome your mitochondria got from your mother, mapped on one page."
category = "tools"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/06-powerhouse-of-the-cell"
keywords = ["mitochondria", "chrM", "GC content", "variants"]
description = '''
Mitochondria carry their own 16,569-base genome, inherited from your mother alone. This app reads all of it as one interval, streams the reference for GC content and maps every change you carry along it.

It counts transitions and transversions and flags indels and mixed calls. It's a map, not a haplogroup caller or an ancestry readout.

An interval grant covers your variants inside it, so approving this app shares every mitochondrial change in your call file.
'''

[listing.purposes]
"v1/genome/regions/chrM/1-16569" = "Your whole mitochondrial genome, its reference and your changes in it"
"v1/genome/files/reference-genome" = "The reference build, to name positions"
"#
        );
        return;
    };

    let base = Path::new(&dir).join(LENS_PATH);
    let build = match read_leaf(
        &Path::new(&dir).join("v1/genome/files/reference-genome"),
        "build",
    ) {
        Ok(build) => build,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    print_header();

    match RegionReport::load(&base) {
        Ok(report) => report.print(build.as_deref()),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
struct RegionReport {
    sequence: Sequence,
    changes: Option<Vec<Change>>,
}

#[derive(Debug)]
struct Change {
    /// Directory name, including any suffix for a shared span.
    name: String,
    pos: u64,
    /// Place among the entries sharing this span, counting from 1.
    ordinal: u64,
    /// Last changed base, which is the anchor base for an insertion.
    end: u64,
    reference: Option<String>,
    genotype: Option<String>,
    call: String,
    kind: String,
    /// The variant file the change came from.
    dataset: Option<String>,
    substitution: String,
}

impl RegionReport {
    fn load(base: &Path) -> Result<Self, String> {
        let sequence = read_sequence(&base.join("reference"))?;
        if sequence.len != END - START + 1 {
            return Err("Region sequence length does not match its span".into());
        }
        let changes = read_changes(base)?;
        Ok(Self { sequence, changes })
    }

    fn print(&self, build: Option<&str>) {
        let span = END - START + 1;
        let gc = self.sequence.gc_percent();

        // The finding states the counts in words; the map and tables beneath
        // carry every value they rest on.
        match &self.changes {
            Some(changes) if changes.is_empty() => {
                println!(
                    "**Factory settings** 🔋. Your call file lists no changes across the {} bp \
                     genome your mitochondria got from your mother alone. A map of the \
                     differences, not a family tree.",
                    format_int(span)
                );
            }
            Some(changes) => {
                let density = per_kb(changes.len(), span);
                let mixed = changes.iter().filter(|c| c.call == "mixed alleles").count();
                let fixed = changes
                    .iter()
                    .filter(|c| c.call == "single change" || c.call == "all copies changed")
                    .count();
                let uncertain = changes.iter().filter(|c| c.call == "uncertain").count();
                let unanswered = changes.iter().filter(|c| c.call == "no answer").count();
                let indels = changes.iter().filter(|c| c.kind == "indel").count();
                let transitions = changes
                    .iter()
                    .filter(|c| c.substitution == "transition")
                    .count();
                let transversions = changes
                    .iter()
                    .filter(|c| c.substitution == "transversion")
                    .count();
                print!(
                    "**Mom's batteries** 🔋. Your mitochondria run on a {2} bp genome you got \
                     from your mother alone, and your call file lists {0} {1} against the \
                     reference, {transitions} {3}, {transversions} {4} and {indels} {5}",
                    changes.len(),
                    plural(changes.len(), "change", "changes"),
                    format_int(span),
                    plural(transitions, "transition", "transitions"),
                    plural(transversions, "transversion", "transversions"),
                    plural(indels, "indel", "indels")
                );
                if mixed > 0 {
                    print!(", {mixed} with mixed alleles and {fixed} in every copy");
                }
                if uncertain + unanswered > 0 {
                    print!(", with {uncertain} uncertain and {unanswered} without an answer");
                }
                println!(
                    ". That's {density:.2} per kb. A map of the differences, not a family \
                     tree."
                );
            }
            None => {
                println!(
                    "**No answer.** The Ark has no change listing for your mitochondria, so \
                     there's nothing to map. The reference alone runs {} bp with {gc:.1}% GC.",
                    format_int(self.sequence.len)
                );
            }
        }
        println!();

        println!("## Evidence");
        println!();
        println!("### Interval");
        println!();
        println!("| Field | Value |");
        println!("| :-- | :-- |");
        match build {
            Some(build) => println!("| Interval | {CHROM}:{START}-{END}, {build} |"),
            None => println!("| Interval | {CHROM}:{START}-{END} |"),
        }
        println!(
            "| Reference bases read | {} bp |",
            format_int(self.sequence.len)
        );
        println!("| Reference GC | {gc:.1}% |");
        println!();

        if let Some(changes) = &self.changes {
            println!("### Variant compass");
            println!();
            println!(
                "Each `*` marks a non-reference change along the chromosome, and `+` more \
                 than one in the same column."
            );
            println!();
            print_star_map(changes);
            println!();
            print_windows(changes);
            print_change_table(changes);
        }

        print_method();
        print_limitations();
        print_sources();
    }
}

fn print_header() {
    println!("# ⚡ The Powerhouse of the Cell");
    println!();
}

/// Stream the forward-strand bases, including lowercase soft-masking.
fn read_sequence(path: &Path) -> Result<Sequence, String> {
    let mut file = File::open(path).map_err(|err| format!("Could not open sequence: {err}"))?;
    let mut summary = Sequence {
        len: 0,
        gc: 0,
        acgt: 0,
    };
    let mut buffer = [0; 8192];
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|err| format!("Could not read sequence: {err}"))?;
        if n == 0 {
            break;
        }
        summary.len += n as u64;
        for base in &buffer[..n] {
            match base.to_ascii_uppercase() {
                b'G' | b'C' => {
                    summary.gc += 1;
                    summary.acgt += 1;
                }
                b'A' | b'T' => summary.acgt += 1,
                _ => {}
            }
        }
    }
    if summary.len == 0 {
        return Err("The sequence is empty".into());
    }
    Ok(summary)
}

#[derive(Debug)]
struct Sequence {
    len: u64,
    gc: u64,
    acgt: u64,
}

impl Sequence {
    fn gc_percent(&self) -> f64 {
        if self.acgt == 0 {
            0.0
        } else {
            self.gc as f64 * 100.0 / self.acgt as f64
        }
    }
}

/// Only ENOENT means no answer. Scalar values have no trailing newline.
fn read_leaf(base: &Path, name: &str) -> Result<Option<String>, String> {
    match fs::read_to_string(base.join(name)) {
        Ok(value) => Ok(Some(value)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("Could not read {name}: {err}")),
    }
}

/// Reads every entry under the place's changes. Past 4000000 bases or 16384
/// entries the listing fails with EFBIG, which this small place never reaches.
fn read_changes(base: &Path) -> Result<Option<Vec<Change>>, String> {
    let entries = match fs::read_dir(base.join("changes")) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("Could not list changes: {err}")),
    };
    let mut changes = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("Could not read changes entry: {err}"))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "Invalid change name")?;
        let (pos, end, ordinal) = change_span(&name)?;
        let change_type = read_leaf(&entry.path(), "type")?;
        let bases = read_leaf(&entry.path(), "bases")?;
        let carried = read_leaf(&entry.path(), "carried")?
            .map(|value| {
                value
                    .parse::<usize>()
                    .map_err(|err| format!("Invalid carried count: {err}"))
            })
            .transpose()?;
        let dataset = read_leaf(&entry.path(), "dataset")?;
        let uncalled = read_leaf(&entry.path(), "uncalled")?;
        let site = base.join("positions").join(pos.to_string());
        let reference = read_leaf(&site, "reference")?;
        let genotype = read_leaf(&site, "genotype")?;
        let alleles = genotype.as_deref().map(alleles);

        // The site's genotype holds only small variant copies, so other entries get no call
        let call = match (dataset.as_deref(), alleles.as_deref(), carried) {
            _ if uncalled.is_some() => "uncertain",
            (_, Some(alleles), _) if alleles.contains(&"*") => "uncertain",
            (Some("small-variants"), Some(alleles), Some(carried)) => {
                classify_call(alleles, carried)
            }
            _ => "no answer",
        };
        let kind = classify_kind(dataset.as_deref(), change_type.as_deref());
        let substitution = classify_substitution(kind, reference.as_deref(), bases.as_deref());
        changes.push(Change {
            name,
            pos,
            end,
            ordinal,
            reference,
            genotype,
            call: call.into(),
            kind: kind.into(),
            dataset,
            substitution: substitution.into(),
        });
    }
    changes.sort_by_key(|c| (c.pos, c.end, c.ordinal));
    Ok(Some(changes))
}

/// Splits an entry name such as 100-100 or 100-100-2 into its start, end and
/// place among the entries sharing that span.
fn change_span(name: &str) -> Result<(u64, u64, u64), String> {
    let parts = name.split('-').collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) {
        return Err(format!("Invalid change name: {name}"));
    }
    let numbers = parts
        .iter()
        .map(|part| {
            if part.starts_with('0') || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(format!("Invalid change name: {name}"));
            }
            part.parse::<u64>()
                .map_err(|_| format!("Invalid change name: {name}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (start, end) = (numbers[0], numbers[1]);
    let ordinal = numbers.get(2).copied().unwrap_or(1);
    if end < start || (numbers.len() == 3 && ordinal < 2) {
        return Err(format!("Invalid change name: {name}"));
    }
    Ok((start, end, ordinal))
}

/// Calls an allele from how many of the site's copies carry it.
fn classify_call(alleles: &[&str], carried: usize) -> &'static str {
    if carried == 0 || carried > alleles.len() || alleles.contains(&".") {
        return "uncertain";
    }
    if alleles.len() == 1 {
        return "single change";
    }
    if carried == alleles.len() {
        "all copies changed"
    } else {
        "mixed alleles"
    }
}

/// Labels a change by its dataset and type.
fn classify_kind(dataset: Option<&str>, change_type: Option<&str>) -> &'static str {
    match (dataset, change_type) {
        (Some("structural-variants"), _) => "structural",
        (_, Some("SNV")) => "snp",
        (_, Some("MNV")) => "mnv",
        (_, Some("INS" | "DEL" | "DELINS")) => "indel",
        _ => "structural",
    }
}

/// Names a substitution a transition or a transversion.
fn classify_substitution(kind: &str, reference: Option<&str>, bases: Option<&str>) -> &'static str {
    if kind != "snp" {
        return "not a snp";
    }
    let (Some(reference), Some(bases)) = (reference, bases) else {
        return "no answer";
    };
    if !is_base(reference) || !is_base(bases) {
        return "no answer";
    }
    let reference = reference.as_bytes()[0].to_ascii_uppercase() as char;
    let alt = bases.as_bytes()[0].to_ascii_uppercase() as char;
    if is_transition(reference, alt) {
        "transition"
    } else {
        "transversion"
    }
}

/// Separators inside symbolic alleles and breakend mate positions are literal.
fn alleles(genotype: &str) -> Vec<&str> {
    let text = genotype.strip_prefix(['/', '|']).unwrap_or(genotype);
    let (mut start, mut angles, mut mate) = (0, 0usize, None);
    let mut result = Vec::new();
    for (i, byte) in text.bytes().enumerate() {
        if let Some(bracket) = mate {
            if byte == bracket {
                mate = None;
            }
        } else if byte == b'<' {
            angles += 1;
        } else if byte == b'>' && angles > 0 {
            angles -= 1;
        } else if angles == 0 {
            match byte {
                b'[' | b']' => mate = Some(byte),
                b'/' | b'|' => {
                    result.push(&text[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
    }
    result.push(&text[start..]);
    result
}

fn is_base(allele: &str) -> bool {
    matches!(
        allele.as_bytes(),
        [b'A' | b'a' | b'C' | b'c' | b'G' | b'g' | b'T' | b't']
    )
}

fn table_value(value: &Option<String>) -> String {
    value.as_deref().unwrap_or("no answer").replace('|', "\\|")
}

fn is_transition(reference: char, alt: char) -> bool {
    matches!(
        (reference, alt),
        ('A', 'G') | ('G', 'A') | ('C', 'T') | ('T', 'C')
    )
}

fn per_kb(count: usize, span: u64) -> f64 {
    if span == 0 {
        0.0
    } else {
        (count as f64) * 1000.0 / (span as f64)
    }
}

fn print_star_map(changes: &[Change]) {
    let mut track = vec![' '; MAP_WIDTH];
    let span = END - START;
    for change in changes {
        if change.pos < START || change.pos > END {
            continue;
        }
        let idx = (((change.pos - START) as usize) * (MAP_WIDTH - 1) / (span as usize))
            .min(MAP_WIDTH - 1);
        track[idx] = if track[idx] == ' ' { '*' } else { '+' };
    }

    println!("```text");
    println!(
        "0kb                 4kb                 8kb                12kb               16.5kb"
    );
    println!("|{}|", "-".repeat(MAP_WIDTH));
    if changes.is_empty() {
        println!(" no non-reference changes listed");
    } else {
        println!(" {}", track.into_iter().collect::<String>());
    }
    println!("```");
}

fn print_windows(changes: &[Change]) {
    println!("### Density windows");
    println!();
    println!("| Window | Changes | Label | Bar |");
    println!("| :--- | ---: | :--- | :--- |");
    for (start, end) in [
        (1, 4_000),
        (4_001, 8_000),
        (8_001, 12_000),
        (12_001, 16_000),
        (16_001, 16_569),
    ] {
        let count = changes
            .iter()
            .filter(|change| change.pos >= start && change.pos <= end)
            .count();
        println!(
            "| {}-{} | {} | {} | `{}` |",
            start,
            end,
            count,
            window_label(count),
            bar(count)
        );
    }
    println!();
}

fn window_label(count: usize) -> &'static str {
    match count {
        0 => "quiet",
        1..=2 => "speckled",
        3..=6 => "busy",
        _ => "crowded",
    }
}

fn bar(count: usize) -> String {
    if count == 0 {
        ".".to_string()
    } else {
        "#".repeat(count.min(20))
    }
}

fn print_change_table(changes: &[Change]) {
    if changes.is_empty() {
        return;
    }
    println!("### Change receipt");
    println!();

    println!("| Change | Reference | Genotype | Call | Kind | File | Substitution |");
    println!("| :--- | :---: | :---: | :--- | :--- | :--- | :--- |");
    for change in changes.iter().take(MAX_CHANGE_ROWS) {
        println!(
            "| {} | `{}` | `{}` | {} | {} | {} | {} |",
            change.name,
            table_value(&change.reference),
            table_value(&change.genotype),
            change.call,
            change.kind,
            change.dataset.as_deref().unwrap_or("no answer"),
            change.substitution
        );
    }
    if changes.len() > MAX_CHANGE_ROWS {
        println!(
            "| ... | ... | ... | {} more changes omitted | ... | ... | ... |",
            changes.len() - MAX_CHANGE_ROWS
        );
    }
    println!();
}

fn print_method() {
    println!("## Method");
    println!();
    println!(
        "Your mitochondrial chromosome is compact enough to fit on a single report page, so \
         the app reads it as one interval. It streams the reference for length and GC \
         content, and lists every change you carry that reaches the interval. Each small \
         variant is classified by how many copies carry it, as a substitution or an indel, \
         and as a transition or a transversion. A structural change, or one without a \
         genotype where it starts, counts as no answer. The reference is the revised \
         Cambridge sequence of Andrews et al. (1999), which GRCh38 carries as chrM."
    );
    println!();
}

fn print_limitations() {
    println!("## Limitations");
    println!();
    println!(
        "A map, not a haplogroup call. Haplogroups need curated marker trees and care with \
         build and strand conventions, none of which this app has. A position without a \
         listed change isn't a result on its own, since a call file can leave reference \
         sites out. Mitochondria can also carry several versions at once, heteroplasmy, \
         which a single genotype string doesn't capture."
    );
    println!();
}

fn print_sources() {
    println!("## Sources");
    println!();
    println!(
        "1. Andrews RM, et al. Reanalysis and revision of the Cambridge reference sequence \
         for human mitochondrial DNA. Nature Genetics. 1999;23:147. \
         https://pubmed.ncbi.nlm.nih.gov/10508508/"
    );
    println!("2. NCBI Nucleotide, NC_012920.1. https://www.ncbi.nlm.nih.gov/nuccore/NC_012920.1");
    println!(
        "3. MITOMAP, the human mitochondrial genome. https://www.mitomap.org/MITOMAP/HumanMitoSeq"
    );
    println!();
}

/// Picks the singular or plural form for a count.
fn plural(n: usize, one: &'static str, many: &'static str) -> &'static str {
    if n == 1 { one } else { many }
}

fn format_int(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}
