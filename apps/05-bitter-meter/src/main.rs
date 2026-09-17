//! Bitter Meter: a report built from the genome `genes/` lens.
//!
//! TAS2R38 is a bitter-taste receptor made famous by PTC/PROP tasting demos. This
//! app deliberately keeps the claim narrower: it does not predict whether someone
//! likes bitter greens or tastes a lab strip. It shows what a gene-scoped lens can
//! reveal: gene metadata, reference sequence summary, and the non-reference changes
//! carried inside that gene, in the report shape docs/06-reports.md describes.
//!
//! Tutorial note for app authors: the app asks Ark for the gene's path
//! (`v1/genome/genes/TAS2R38`) and for `v1/genome/reference`, which it reads only
//! for the assembly build its coordinates are reported on.

use std::collections::BTreeSet;
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
            "[package]\n\
             name = \"bitter-meter\"\n\
             version = \"0.2.0\"\n\
             datasets = [\"v1/genome/genes/TAS2R38\", \"v1/genome/reference\"]\n"
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
    reference: Option<String>,
    genotype: Option<String>,
    call: String,
    kind: String,
}

impl GeneReport {
    fn load(base: &Path) -> Result<Option<Self>, String> {
        let sequence = read_sequence(&base.join("sequence"))?;
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
        let changes = read_changes(&base.join("changes"))?;
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
                    "Your call file lists no changes inside *{GENE}* across {} bp on {}. A \
                     postcard of the gene as recorded, not a verdict on how you taste.",
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
                    "Your bitter receptor gene *{GENE}* carries {} {} in your call file, \
                     {heterozygous} heterozygous and {homozygous} homozygous, across {} bp on {}",
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
                    ". That's {density:.2} per kb, {} for a gene this small. A postcard of \
                     the gene as your call file records it, not a verdict on how you taste.",
                    density_label(density)
                );
            }
            None => {
                println!(
                    "No changes answer is available for *{GENE}*, so the app can't say what \
                     your call file holds there. The reference sequence is {} bp with {gc:.1}% GC.",
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
                "Each `*` marks a non-reference change along the gene, and `+` more than one \
                 in the same column."
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
    println!("# TAS2R38, the Bitter Taste Receptor Gene");
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
        let (call, kind) = match (&reference, &genotype) {
            (Some(reference), Some(genotype)) => (
                classify_call(reference, genotype),
                classify_kind(reference, genotype),
            ),
            _ => ("no answer", "no answer"),
        };
        changes.push(Change {
            pos,
            reference,
            genotype,
            call: call.into(),
            kind: kind.into(),
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
            "homozygous reference"
        } else {
            "homozygous change"
        }
    } else {
        "heterozygous"
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
        "quiet: no non-reference changes listed"
    } else if density < 1.0 {
        "lightly speckled"
    } else if density < 3.0 {
        "busy enough to notice"
    } else {
        "crowded little gene"
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
        println!(" {}", "no non-reference changes listed");
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
         gene's coordinates, strand and biotype from the annotations, \
         streams its reference sequence for length and GC content, and lists every position \
         under the gene's changes where a record starting inside the gene holds an ALT \
         allele. Each genotype is classified against the reference base as heterozygous, \
         homozygous or uncertain, and as a substitution or an indel. A position listed \
         without leaves, where several records start, counts as no answer. Kim et al. \
         (2003) mapped taste sensitivity to phenylthiocarbamide to this gene; that story \
         rests on specific coding variants and haplotypes the app doesn't call."
    );
    println!();
}

fn print_limitations() {
    println!("## Limitations");
    println!();
    println!(
        "Not a PTC or PROP taster test. The classic bitter-taste story rests on specific \
         coding variants and haplotypes, and this app doesn't phase calls or name rsIDs \
         from the gene view. The change list holds records starting inside the span, so \
         one starting just before it is left out, and a position not listed isn't a \
         result on its own. Taste is also shaped by other genes, age, exposure, diet and \
         plain preference."
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
