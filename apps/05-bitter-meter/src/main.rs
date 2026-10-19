//! Bitter Meter: a report built from one gene under `genes/`.
//!
//! TAS2R38 is a bitter-taste receptor made famous by PTC and PROP tasting demos.
//! This app keeps the claim narrower and does not predict whether someone likes
//! bitter greens or tastes a lab strip. It shows what one gene grant reveals, the
//! gene's metadata, its reference and the changes carried inside it, in the report
//! shape docs/06-reports.md describes.
//!
//! The app asks the Ark for the gene, `v1/genome/genes/TAS2R38`, and for
//! `v1/genome/files/reference-genome`, a public grant it reads only for the
//! assembly build its coordinates are reported on.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const GENE: &str = "TAS2R38";
const LENS_PATH: &str = "v1/genome/genes/TAS2R38";
const MAX_CHANGE_ROWS: usize = 24;
const RULER_WIDTH: usize = 64;

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Bitter Meter"
version = "0.2.0"

[reads]
paths = ["v1/genome/genes/TAS2R38", "v1/genome/files/reference-genome"]

[listing]
language = "en"
icon = "🥦"
summary = "A postcard from TAS2R38, the gene behind broccoli's bitter bite."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/05-bitter-meter"
keywords = ["bitter taste", "TAS2R38", "PTC", "taste receptor"]
description = '''
_TAS2R38_ is the bitter taste receptor from the PTC and PROP classroom experiments, and one reason broccoli tastes harsh to some people. This app reads the gene, streams its reference for GC content and lists every change you carry in it.

It's a postcard, not a taster prediction. The classic taster story needs specific coding variants and their phase, and the gene view infers neither.

One grant covers the whole gene, your variants in it included. A listing that fails stops the app, so a broken read never passes for a quiet gene.
'''

[listing.purposes]
"v1/genome/genes/TAS2R38" = "The bitter taste receptor gene, its reference and your changes in it"
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

    match GeneReport::load(&base) {
        Ok(Some(report)) => report.print(build.as_deref()),
        Ok(None) => {
            println!(
                "No answer. The annotations on this Ark carry no coordinates for *{GENE}*, \
                 so the app has nothing to describe."
            );
            println!();
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
struct GeneReport {
    chromosome: String,
    start: u64,
    end: u64,
    strand: String,
    biotype: String,
    sequence: Sequence,
    changes: Option<Vec<Change>>,
}

#[derive(Debug)]
struct Change {
    pos: u64,
    /// Place among the entries sharing this span, counting from 1.
    ordinal: u64,
    /// Last changed base, which is the anchor base for an insertion.
    end: u64,
    reference: Option<String>,
    genotype: Option<String>,
    call: String,
    kind: String,
}

impl GeneReport {
    fn load(base: &Path) -> Result<Option<Self>, String> {
        let sequence = read_sequence(&base.join("reference"))?;
        let chromosome = read_leaf(base, "chromosome")?;
        let start = read_leaf(base, "start")?;
        let end = read_leaf(base, "end")?;
        let strand = read_leaf(base, "strand")?;
        let biotype = read_leaf(base, "biotype")?.unwrap_or("no answer".into());
        let (Some(chromosome), Some(start), Some(end), Some(strand)) =
            (chromosome, start, end, strand)
        else {
            return Ok(None);
        };
        let start = start
            .parse::<u64>()
            .map_err(|err| format!("Invalid gene start: {err}"))?;
        let end = end
            .parse::<u64>()
            .map_err(|err| format!("Invalid gene end: {err}"))?;
        if start == 0 || end < start || sequence.len != end - start + 1 {
            return Err("Gene sequence length does not match its span".into());
        }
        let changes = read_changes(base)?;
        Ok(Some(Self {
            chromosome,
            start,
            end,
            strand,
            biotype,
            sequence,
            changes,
        }))
    }

    fn print(&self, build: Option<&str>) {
        let span = self.end.saturating_sub(self.start).saturating_add(1);
        let gc = self.sequence.gc_percent();

        // The finding states the counts in words; the tables beneath carry
        // every value they rest on.
        match &self.changes {
            Some(changes) if changes.is_empty() => {
                println!(
                    "**Blank postcard** 🥦. Your call file lists no changes inside *{GENE}*, \
                     the receptor behind broccoli's bitter bite, across its {} bp on {}. A \
                     postcard of the gene, not a verdict on how you taste.",
                    format_int(span),
                    self.chromosome
                );
            }
            Some(changes) => {
                let density = per_kb(changes.len(), span);
                let heterozygous = changes.iter().filter(|c| c.call == "heterozygous").count();
                let homozygous = changes
                    .iter()
                    .filter(|c| c.call == "homozygous change")
                    .count();
                let uncertain = changes.iter().filter(|c| c.call == "uncertain").count();
                let unanswered = changes.iter().filter(|c| c.call == "no answer").count();
                let indels = changes.iter().filter(|c| c.kind == "indel").count();
                print!(
                    "**Bitter postcard** 🥦. *{GENE}*, the receptor behind broccoli's bitter \
                     bite, carries {} {} in your call file across its {} bp on {}, \
                     {heterozygous} on one copy and {homozygous} on both",
                    changes.len(),
                    if changes.len() == 1 {
                        "change"
                    } else {
                        "changes"
                    },
                    format_int(span),
                    self.chromosome
                );
                if indels > 0 {
                    print!(
                        ", {indels} of them {}",
                        plural(indels, "an indel", "indels")
                    );
                }
                if uncertain + unanswered > 0 {
                    print!(", with {uncertain} uncertain and {unanswered} without an answer");
                }
                println!(
                    ". That's {density:.2} per kb, {} for a gene this small. Whether \
                     broccoli hates you back is another story, and Limitations explains why.",
                    density_label(density)
                );
            }
            None => {
                println!(
                    "**No answer.** The Ark has no change listing for *{GENE}*, so there's \
                     nothing to put on the postcard. The reference alone runs {} bp with \
                     {gc:.1}% GC.",
                    format_int(self.sequence.len)
                );
            }
        }
        println!();

        println!("## Evidence");
        println!();
        println!("### Gene postcard");
        println!();
        println!("| Field | Value |");
        println!("| :-- | :-- |");
        println!("| Gene | *{GENE}* |");
        let location = format!("{}:{}-{}", self.chromosome, self.start, self.end);
        match build {
            Some(build) => println!("| Location | {location}, {build} |"),
            None => println!("| Location | {location} |"),
        }
        println!("| Strand | {} |", self.strand);
        println!("| Biotype | {} |", self.biotype);
        println!(
            "| Reference length | {} bp |",
            format_int(self.sequence.len)
        );
        println!("| Reference GC | {gc:.1}% |");
        println!();

        if let Some(changes) = &self.changes {
            println!("### Change ruler");
            println!();
            println!(
                "Each `*` marks one of your changes along the gene, and `+` a pile-up of \
                 several in one column."
            );
            println!();
            print_ruler(self.start, self.end, changes);
            println!();
            print_change_table(changes);
        }

        print_method();
        print_limitations();
        print_sources();
    }
}

fn print_header() {
    println!("# 🥦 The Broccoli Gene");
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
        let name = entry.file_name();
        let (pos, end, ordinal) = change_span(name.to_str().ok_or("Invalid change name")?)?;
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

        // The site's genotype holds only small variant copies, so other entries get no call
        let call = match (dataset.as_deref(), genotype.as_deref(), carried) {
            _ if uncalled.is_some() => "uncertain",
            (Some("small-variants"), Some(genotype), Some(carried)) => {
                classify_call(genotype, carried)
            }
            _ => "no answer",
        };
        let kind = classify_kind(change_type.as_deref(), bases.as_deref());
        changes.push(Change {
            pos,
            end,
            ordinal,
            reference,
            genotype,
            call: call.into(),
            kind: kind.into(),
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
fn classify_call(genotype: &str, carried: usize) -> &'static str {
    let alleles = alleles(genotype);
    if carried == 0 || carried > alleles.len() || alleles.contains(&".") {
        return "uncertain";
    }
    if alleles.len() == 1 {
        return "single change";
    }
    if carried == alleles.len() {
        "homozygous change"
    } else {
        "heterozygous"
    }
}

/// Sorts a change into the report's substitution or indel kind.
fn classify_kind(change_type: Option<&str>, bases: Option<&str>) -> &'static str {
    match (change_type, bases) {
        (Some("SNV"), Some(bases)) if is_base(bases) => "snp",
        (Some("SNV") | None, _) => "no answer",
        _ => "indel",
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

fn per_kb(count: usize, span: u64) -> f64 {
    if span == 0 {
        0.0
    } else {
        (count as f64) * 1000.0 / (span as f64)
    }
}

fn density_label(density: f64) -> &'static str {
    if density == 0.0 {
        "dead quiet"
    } else if density < 1.0 {
        "lightly speckled"
    } else if density < 3.0 {
        "lively"
    } else {
        "a real crowd"
    }
}

fn print_ruler(start: u64, end: u64, changes: &[Change]) {
    let mut track = vec![' '; RULER_WIDTH];
    let span = end.saturating_sub(start).max(1);
    for change in changes {
        if change.pos < start || change.pos > end {
            continue;
        }
        let idx = (((change.pos - start) as usize) * (RULER_WIDTH - 1) / (span as usize))
            .min(RULER_WIDTH - 1);
        track[idx] = if track[idx] == ' ' { '*' } else { '+' };
    }

    println!("```text");
    println!(
        "{}{}",
        start,
        " ".repeat(RULER_WIDTH.saturating_sub(start.to_string().len()))
    );
    println!("|{}| {}", "-".repeat(RULER_WIDTH), end);
    if changes.is_empty() {
        println!(" no non-reference changes listed");
    } else {
        println!(" {}", track.into_iter().collect::<String>());
    }
    println!("```");
}

fn print_change_table(changes: &[Change]) {
    if changes.is_empty() {
        return;
    }
    println!("### Change receipt");
    println!();

    println!("| Position | Reference | Genotype | Call | Kind |");
    println!("| ---: | :---: | :---: | :--- | :--- |");
    for change in changes.iter().take(MAX_CHANGE_ROWS) {
        println!(
            "| {} | `{}` | `{}` | {} | {} |",
            change.pos,
            table_value(&change.reference),
            table_value(&change.genotype),
            change.call,
            change.kind
        );
    }
    if changes.len() > MAX_CHANGE_ROWS {
        println!(
            "| ... | ... | ... | {} more changes omitted | ... |",
            changes.len() - MAX_CHANGE_ROWS
        );
    }
    println!();
}

fn print_method() {
    println!("## Method");
    println!();
    println!(
        "Some bitter flavors are sensed by tiny receptor proteins on the tongue, and \
         *{GENE}* is the famous one from PTC and PROP classroom genetics. The app reads the \
         gene's coordinates, strand and biotype from the annotations, streams its \
         reference for length and GC content, and lists every change you carry that \
         reaches the gene. Each small variant is called heterozygous, homozygous or \
         uncertain from how many copies carry it, and a substitution or an indel from its \
         type. A structural change, or one without a genotype where it starts, counts as \
         no answer. Kim et al. (2003) mapped taste sensitivity to phenylthiocarbamide to \
         this gene, a story resting on specific coding variants and haplotypes the app \
         doesn't call."
    );
    println!();
}

fn print_limitations() {
    println!("## Limitations");
    println!();
    println!(
        "Not a PTC or PROP taster verdict. The classic bitter-taste story rests on specific \
         coding variants and haplotypes, and this app doesn't phase calls or name rsIDs \
         from the gene view. A position without a listed change isn't a result on its \
         own, since a call file can leave reference sites out. Taste is also shaped by \
         other genes, age, exposure, diet and plain preference."
    );
    println!();
}

fn print_sources() {
    println!("## Sources");
    println!();
    println!(
        "1. Kim UK, et al. Positional cloning of the human quantitative trait locus \
         underlying taste sensitivity to phenylthiocarbamide. Science. 2003;299:1221-1225. \
         https://pubmed.ncbi.nlm.nih.gov/12595690/"
    );
    println!("2. NCBI Gene, {GENE}. https://www.ncbi.nlm.nih.gov/gene/?term={GENE}");
    println!("3. Ensembl, {GENE}. https://www.ensembl.org/Homo_sapiens/Gene/Summary?g={GENE}");
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
