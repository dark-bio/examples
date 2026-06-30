//! Mitochondrial Star Map: a Markdown report from one BioFS `regions/` lens.
//!
//! The mitochondrial chromosome is small enough to read as one range, which makes
//! it a useful demo of the region lens: one granted interval exposes a streamed
//! reference sequence plus all non-reference changes inside that span.
//!
//! Tutorial note for app authors: the app asks Ark for exactly one dataset path
//! (`v1/genome/regions/chrM/1-16569`). The user-facing report below avoids
//! teaching BioFS mechanics; those details belong here and in the README.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;

const CHROM: &str = "chrM";
const START: u64 = 1;
const END: u64 = 16_569;
const LENS_PATH: &str = "v1/genome/regions/chrM/1-16569";
const MAP_WIDTH: usize = 72;
const MAX_CHANGE_ROWS: usize = 28;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"powerhouse-of-the-cell\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/regions/chrM/1-16569\"]\n"
        );
        return Ok(());
    };

    let base = Path::new(&dir).join(LENS_PATH);
    print_header();

    match RegionReport::load(&base) {
        Ok(report) => report.print(),
        Err(err) => print_unavailable(&base, &err),
    }

    Ok(())
}

#[derive(Debug)]
struct RegionReport {
    reference: String,
    changes: Vec<Change>,
}

#[derive(Debug)]
struct Change {
    pos: u64,
    reference: String,
    genotype: String,
    call: String,
    kind: String,
    substitution: String,
}

impl RegionReport {
    fn load(base: &Path) -> Result<Self, String> {
        let reference = fs::read_to_string(base.join("reference"))
            .map(|s| s.trim_end_matches(['\n', '\r']).to_string())
            .map_err(|err| format!("failed to read `reference`: {err}"))?;
        let changes = read_changes(&base.join("changes"));
        Ok(Self { reference, changes })
    }

    fn print(&self) {
        let span = END - START + 1;
        let gc = gc_percent(&self.reference);
        let density = per_kb(self.changes.len(), span);
        let mixed = self
            .changes
            .iter()
            .filter(|c| c.call == "mixed alleles")
            .count();
        let fixed = self
            .changes
            .iter()
            .filter(|c| c.call == "single change" || c.call == "all copies changed")
            .count();
        let uncertain = self
            .changes
            .iter()
            .filter(|c| c.call == "uncertain")
            .count();
        let indels = self.changes.iter().filter(|c| c.kind == "indel").count();
        let transitions = self
            .changes
            .iter()
            .filter(|c| c.substitution == "transition")
            .count();
        let transversions = self
            .changes
            .iter()
            .filter(|c| c.substitution == "transversion")
            .count();

        println!("## Your Result");
        println!();
        println!("| Signal | Value |");
        println!("| :--- | ---: |");
        println!("| Interval | {}:{}-{} |", CHROM, START, END);
        println!(
            "| Reference bases read | {} bp |",
            format_int(self.reference.len() as u64)
        );
        println!("| Reference GC | {:.1}% |", gc);
        println!("| Observed changes | {} |", self.changes.len());
        println!("| Change density | {:.2} / kb |", density);
        println!("| Mixed-allele calls | {mixed} |");
        println!("| Fixed or single-copy changes | {fixed} |");
        println!("| Indels or complex alleles | {indels} |");
        println!("| Transitions | {transitions} |");
        println!("| Transversions | {transversions} |");
        println!("| Uncertain calls | {uncertain} |");
        println!();

        println!("## Variant Compass");
        println!();
        println!(
            "Each `*` marks at least one non-reference change. `+` means multiple changes share a text column."
        );
        println!();
        print_star_map(&self.changes);
        println!();

        print_windows(&self.changes);
        print_change_table(&self.changes);
        print_data_receipt();
        print_meaning();
        print_fine_print();
        print_references();
    }
}

fn print_header() {
    println!("# Powerhouse of the Cell");
    println!();
    println!(
        "Your mitochondrial chromosome is compact enough to fit on a single \
         report page. This map summarizes observed non-reference changes across \
         `chrM` and shows where they fall along the 16.6 kb sequence."
    );
    println!();
}

fn print_unavailable(_base: &Path, err: &str) {
    println!("## Result unavailable");
    println!();
    println!("The mitochondrial interval data could not be read.");
    println!();
    println!("Reason: `{err}`");
    println!();
    println!(
        "This usually means the loaded genome package does not include the \
         mitochondrial reference sequence or variant calls needed for this report."
    );
    println!();
    print_data_receipt();
    print_fine_print();
}

fn read_changes(path: &Path) -> Vec<Change> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };

    let mut changes = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(pos) = name.parse::<u64>() else {
            continue;
        };
        let change_dir = entry.path();
        let reference = fs::read_to_string(change_dir.join("reference"))
            .map(|s| trim_line(&s))
            .unwrap_or_default();
        let genotype = fs::read_to_string(change_dir.join("genotype"))
            .map(|s| trim_line(&s))
            .unwrap_or_default();
        if reference.is_empty() || genotype.is_empty() {
            continue;
        }
        let call = classify_call(&reference, &genotype).to_string();
        let kind = classify_kind(&reference, &genotype).to_string();
        let substitution = classify_substitution(&reference, &genotype).to_string();
        changes.push(Change {
            pos,
            reference,
            genotype,
            call,
            kind,
            substitution,
        });
    }
    changes.sort_by_key(|c| c.pos);
    changes
}

fn trim_line(s: &str) -> String {
    s.trim_end_matches(['\n', '\r']).to_string()
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
    if reference.len() != 1 || alleles.iter().any(|a| *a != "." && a.len() != 1) {
        "indel"
    } else {
        "snp"
    }
}

fn classify_substitution(reference: &str, genotype: &str) -> &'static str {
    if reference.len() != 1 {
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
        if allele.len() != 1 {
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

fn alleles(genotype: &str) -> Vec<&str> {
    genotype
        .split(['/', '|'])
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .collect()
}

fn is_transition(reference: char, alt: char) -> bool {
    matches!(
        (reference, alt),
        ('A', 'G') | ('G', 'A') | ('C', 'T') | ('T', 'C')
    )
}

fn gc_percent(sequence: &str) -> f64 {
    let mut gc = 0usize;
    let mut acgt = 0usize;
    for base in sequence.bytes() {
        match base {
            b'G' | b'g' | b'C' | b'c' => {
                gc += 1;
                acgt += 1;
            }
            b'A' | b'a' | b'T' | b't' => acgt += 1,
            _ => {}
        }
    }
    if acgt == 0 {
        0.0
    } else {
        (gc as f64) * 100.0 / (acgt as f64)
    }
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
        track[idx] = if track[idx] == '*' { '+' } else { '*' };
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
    println!("## Density Windows");
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
    println!("## Change Receipt");
    println!();
    if changes.is_empty() {
        println!("No non-reference changes were listed under this mitochondrial interval.");
        println!();
        return;
    }

    println!("| Position | Reference | Genotype | Call | Kind | Substitution |");
    println!("| ---: | :---: | :---: | :--- | :--- | :--- |");
    for change in changes.iter().take(MAX_CHANGE_ROWS) {
        println!(
            "| {} | `{}` | `{}` | {} | {} | {} |",
            change.pos,
            change.reference,
            change.genotype,
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

fn print_data_receipt() {
    println!("## What Was Checked");
    println!();
    println!("This report checked:");
    println!();
    println!("- The mitochondrial interval `{CHROM}:{START}-{END}`");
    println!("- The reference sequence for that interval, summarized as length and GC content");
    println!("- Observed non-reference changes inside the interval, with reference and genotype");
    println!();
}

fn print_meaning() {
    println!("## What It Means");
    println!();
    println!(
        "This is a mitochondrial shape report. It shows how many observed \
         non-reference calls appear across `chrM`, where they sit, and whether \
         the calls look like substitutions or indels."
    );
    println!();
    println!(
        "It is not a haplogroup caller. Haplogroups need curated marker trees, \
         careful handling of build and strand conventions, and usually more \
         interpretation than this demo should do."
    );
    println!();
}

fn print_fine_print() {
    println!("## Fine Print");
    println!();
    println!(
        "The change list contains positions where the reported genotype differs \
         from the reference sequence. The absence of a listed change is not a \
         medical or ancestry result. Mitochondrial data can also involve \
         heteroplasmy and platform-specific calling choices that a simple genotype \
         string does not fully describe."
    );
    println!();
    println!("This report is for demo and education only.");
    println!();
}

fn print_references() {
    println!("## Further Reading");
    println!();
    println!("- NCBI Nucleotide `NC_012920.1`: https://www.ncbi.nlm.nih.gov/nuccore/NC_012920.1");
    println!(
        "- MITOMAP human mitochondrial sequence resources: https://www.mitomap.org/MITOMAP/HumanMitoSeq"
    );
    println!(
        "- Andrews RM et al. (1999). Reanalysis and revision of the Cambridge reference sequence. PubMed: https://pubmed.ncbi.nlm.nih.gov/10508508/"
    );
    println!();
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
