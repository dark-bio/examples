//! Bitter Meter: a small Markdown report built from the genome `genes/` lens.
//!
//! TAS2R38 is a bitter-taste receptor made famous by PTC/PROP tasting demos. This
//! app deliberately keeps the claim narrower: it does not predict whether someone
//! likes bitter greens or tastes a lab strip. It shows what a gene-scoped lens can
//! reveal: gene metadata, reference sequence summary, and the non-reference changes
//! carried inside that gene.
//!
//! Tutorial note for app authors: the app asks Ark for exactly one dataset path
//! (`v1/genome/genes/TAS2R38`). The user-facing report below avoids teaching
//! path details; those details belong here and in the README.

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
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/genes/TAS2R38\"]\n"
        );
        return;
    };

    let base = Path::new(&dir).join(LENS_PATH);
    print_header();

    match GeneReport::load(&base) {
        Ok(Some(report)) => report.print(),
        Ok(None) => println!("No metadata answer for {GENE}."),
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

    fn print(&self) {
        let Some(changes) = &self.changes else {
            println!("No changes answer is available for this span.");
            println!(
                "Reference length: {} bp; GC: {:.1}%.",
                self.sequence.len,
                self.sequence.gc_percent()
            );
            return;
        };
        let span = self.end.saturating_sub(self.start).saturating_add(1);
        let gc = self.sequence.gc_percent();
        let density = per_kb(changes.len(), span);
        let heterozygous = changes.iter().filter(|c| c.call == "heterozygous").count();
        let homozygous = changes
            .iter()
            .filter(|c| c.call == "homozygous change")
            .count();
        let uncertain = changes.iter().filter(|c| c.call == "uncertain").count();
        let indels = changes.iter().filter(|c| c.kind == "indel").count();

        println!("## Your Result");
        println!();
        println!("| Signal | Value |");
        println!("| :--- | ---: |");
        println!("| Gene span | {} bp |", format_int(span));
        println!("| Reference GC | {:.1}% |", gc);
        println!("| Observed changes | {} |", changes.len());
        println!("| Change density | {:.2} / kb |", density);
        println!("| Heterozygous calls | {heterozygous} |");
        println!("| Homozygous change calls | {homozygous} |");
        println!("| Indels or complex alleles | {indels} |");
        println!("| Uncertain calls | {uncertain} |");
        println!(
            "| Positions without an answer | {} |",
            changes.iter().filter(|c| c.call == "no answer").count()
        );
        println!();

        println!("## Gene Postcard");
        println!();
        println!("| Field | Value |");
        println!("| :--- | :--- |");
        println!("| Gene | `{GENE}` |");
        println!(
            "| Location | `{}:{}-{}` |",
            self.chromosome, self.start, self.end
        );
        println!("| Strand | `{}` |", self.strand);
        println!("| Biotype | `{}` |", self.biotype);
        println!(
            "| Reference length | {} bp |",
            format_int(self.sequence.len)
        );
        println!("| Local label | {} |", density_label(density));
        println!();

        println!("## Change Ruler");
        println!();
        println!(
            "The ruler marks observed non-reference changes across the gene. `+` means more than one change landed in the same text column."
        );
        println!();
        print_ruler(self.start, self.end, changes);
        println!();

        print_change_table(changes);
        print_data_receipt();
        print_meaning();
        print_fine_print();
        print_references();
    }
}

fn print_header() {
    println!("# Bitter Meter: {GENE}");
    println!();
    println!(
        "Some bitter flavors are sensed by tiny receptor proteins on the tongue. \
         `{GENE}` is the famous bitter-taste receptor from PTC and PROP classroom \
         genetics. This report keeps the claim modest: it summarizes the visible \
         changes in this gene without trying to predict your taste preferences."
    );
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
    println!("## Change Receipt");
    println!();
    if changes.is_empty() {
        println!("No non-reference changes were listed under `changes/` for this gene.");
        println!();
        return;
    }

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

fn print_data_receipt() {
    println!("## What Was Checked");
    println!();
    println!("This report checked:");
    println!();
    println!("- `{GENE}` chromosome, start, end, strand, and biotype");
    println!("- The reference sequence for this gene, summarized as length and GC content");
    println!("- Observed non-reference changes inside this gene, with reference and genotype");
    println!();
}

fn print_meaning() {
    println!("## What It Means");
    println!();
    println!(
        "This is a gene postcard. It tells you where `{GENE}` sits, how large \
         its reference sequence is, and which observed non-reference calls appear \
         inside that span. The change count is useful for seeing how much personal \
         variation is visible in this small taste-receptor gene."
    );
    println!();
    println!(
        "It is intentionally not a PTC or PROP taster prediction. The classic \
         bitter-taste story involves specific coding variants and haplotypes, and \
         this report does not assume phase or infer named rsIDs from the gene view."
    );
    println!();
}

fn print_fine_print() {
    println!("## Fine Print");
    println!();
    println!(
        "The change list contains record starts within this span whose genotype \
         holds an ALT allele. Records starting before the span are excluded. A position not shown here should not be \
         treated as a standalone trait result. Taste is also shaped by other \
         genes, age, exposure, diet, and plain preference."
    );
    println!();
    println!("This report is for demo and education only.");
    println!();
}

fn print_references() {
    println!("## Further Reading");
    println!();
    println!("- NCBI Gene search for `TAS2R38`: https://www.ncbi.nlm.nih.gov/gene/?term=TAS2R38");
    println!(
        "- Ensembl gene summary for `TAS2R38`: https://www.ensembl.org/Homo_sapiens/Gene/Summary?g=TAS2R38"
    );
    println!(
        "- Kim U et al. (2003). Positional cloning of the human PTC taste-sensitivity locus. PubMed: https://pubmed.ncbi.nlm.nih.gov/12595690/"
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
