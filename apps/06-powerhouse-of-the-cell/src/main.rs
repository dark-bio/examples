//! Powerhouse of the Cell: a report from one genome `regions/` lens.
//!
//! The mitochondrial chromosome is small enough to read as one range, which makes
//! it a useful demo of the region lens: one granted interval exposes a streamed
//! reference sequence plus all non-reference changes inside that span, printed in
//! the report shape docs/06-reports.md describes.
//!
//! Tutorial note for app authors: the app asks Ark for the interval's path
//! (`v1/genome/regions/chrM/1-16569`) and for `v1/genome/reference`, which it
//! reads only for the assembly build its coordinates are reported on.

use std::collections::BTreeSet;
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
            "[package]\n\
             name = \"powerhouse-of-the-cell\"\n\
             version = \"0.2.0\"\n\
             datasets = [\"v1/genome/regions/chrM/1-16569\", \"v1/genome/reference\"]\n"
        );
        return;
    };

    let base = Path::new(&dir).join(LENS_PATH);
    let build = match read_leaf(&Path::new(&dir).join("v1/genome/reference"), "build") {
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
    pos: u64,
    reference: Option<String>,
    genotype: Option<String>,
    call: String,
    kind: String,
    substitution: String,
}

impl RegionReport {
    fn load(base: &Path) -> Result<Self, String> {
        let sequence = read_sequence(&base.join("sequence"))?;
        if sequence.len != END - START + 1 {
            return Err("Region sequence length does not match its span".into());
        }
        let changes = read_changes(&base.join("changes"))?;
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
                    "Your call file lists no changes across the {} bp mitochondrial \
                     chromosome. A map of where your calls differ from the reference, not a \
                     haplogroup.",
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
                    "Your mitochondrial chromosome, all {2} bp of it, carries {0} {1} in your \
                     call file, {transitions} {3}, {transversions} {4} and {indels} {5}",
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
                    ". That's {density:.2} per kb. A map of where your calls differ from the \
                     reference, not a haplogroup."
                );
            }
            None => {
                println!(
                    "No changes answer is available for this interval, so the app can't say \
                     what your call file holds there. The reference sequence is {} bp with \
                     {gc:.1}% GC.",
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
    println!("# Your Mitochondrial Genome");
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

/// changes cannot be granted directly. Past 4000000 bases or 16384 positions,
/// listing or lookup fails with EFBIG. This small-span example does not split.
fn read_changes(path: &Path) -> Result<Option<Vec<Change>>, String> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("Could not list changes: {err}")),
    };
    let mut changes = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("Could not read changes entry: {err}"))?;
        let name = entry.file_name();
        let pos = name
            .to_str()
            .ok_or("Invalid change position")?
            .parse::<u64>()
            .map_err(|err| format!("Invalid change position: {err}"))?;
        let reference = read_leaf(&entry.path(), "reference")?;
        let genotype = read_leaf(&entry.path(), "genotype")?;
        // Several records can start at one listed position, leaving both leaves absent.
        let (call, kind, substitution) = match (&reference, &genotype) {
            (Some(reference), Some(genotype)) => (
                classify_call(reference, genotype),
                classify_kind(reference, genotype),
                classify_substitution(reference, genotype),
            ),
            _ => ("no answer", "no answer", "no answer"),
        };
        changes.push(Change {
            pos,
            reference,
            genotype,
            call: call.into(),
            kind: kind.into(),
            substitution: substitution.into(),
        });
    }
    changes.sort_by_key(|c| c.pos);
    Ok(Some(changes))
}

fn classify_call(reference: &str, genotype: &str) -> &'static str {
    let alleles = alleles(genotype);
    if alleles.is_empty() || alleles.iter().any(|a| *a == ".") {
        return "uncertain";
    }
    if alleles.len() == 1 {
        return if alleles[0] == reference {
            "single reference"
        } else {
            "single change"
        };
    }
    let unique: BTreeSet<&str> = alleles.iter().copied().collect();
    if unique.len() == 1 {
        if unique.contains(reference) {
            "all copies reference"
        } else {
            "all copies changed"
        }
    } else {
        "mixed alleles"
    }
}

fn classify_kind(reference: &str, genotype: &str) -> &'static str {
    let alleles = alleles(genotype);
    if !is_base(reference) || alleles.iter().any(|a| *a != "." && !is_base(a)) {
        "indel"
    } else {
        "snp"
    }
}

fn classify_substitution(reference: &str, genotype: &str) -> &'static str {
    if !is_base(reference) {
        return "not a snp";
    }
    let Some(reference) = reference.chars().next().map(|c| c.to_ascii_uppercase()) else {
        return "not a snp";
    };

    let mut saw_transition = false;
    let mut saw_transversion = false;
    for allele in alleles(genotype) {
        if allele == "." || allele.eq_ignore_ascii_case(&reference.to_string()) {
            continue;
        }
        if !is_base(allele) {
            return "not a snp";
        }
        let Some(alt) = allele.chars().next().map(|c| c.to_ascii_uppercase()) else {
            continue;
        };
        if is_transition(reference, alt) {
            saw_transition = true;
        } else {
            saw_transversion = true;
        }
    }

    match (saw_transition, saw_transversion) {
        (true, false) => "transition",
        (false, true) => "transversion",
        (true, true) => "mixed substitution",
        (false, false) => "reference only",
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

    println!("| Position | Reference | Genotype | Call | Kind | Substitution |");
    println!("| ---: | :---: | :---: | :--- | :--- | :--- |");
    for change in changes.iter().take(MAX_CHANGE_ROWS) {
        println!(
            "| {} | `{}` | `{}` | {} | {} | {} |",
            change.pos,
            table_value(&change.reference),
            table_value(&change.genotype),
            change.call,
            change.kind,
            change.substitution
        );
    }
    if changes.len() > MAX_CHANGE_ROWS {
        println!(
            "| ... | ... | ... | {} more changes omitted | ... | ... |",
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
         the app reads it as one interval. It streams the reference sequence for length and \
         GC content, and lists every position under its changes where a record starting \
         inside the \
         interval holds an ALT allele. Each genotype is classified against the reference \
         base by how many copies carry the change, as a substitution or an indel, and as \
         a transition or a transversion. A position listed without leaves, where several \
         records start, counts as no answer. The reference is the revised Cambridge \
         sequence of Andrews et al. (1999), which GRCh38 carries as chrM."
    );
    println!();
}

fn print_limitations() {
    println!("## Limitations");
    println!();
    println!(
        "A map, not a haplogroup call. Haplogroups need curated marker trees and care with \
         build and strand conventions, none of which this app has. The change list holds \
         records starting inside the interval, and a position not listed isn't a result on \
         its own. Mitochondria can also carry several versions at once, heteroplasmy, which \
         a single genotype string doesn't capture."
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
