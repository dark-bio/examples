//! Bitter Meter: a small Markdown report built from the BioFS `genes/` lens.
//!
//! TAS2R38 is a bitter-taste receptor made famous by PTC/PROP tasting demos. This
//! app deliberately keeps the claim narrower: it does not predict whether someone
//! likes bitter greens or tastes a lab strip. It shows what a gene-scoped lens can
//! reveal: gene metadata, reference sequence summary, and the non-reference changes
//! carried inside that gene.
//!
//! Tutorial note for app authors: the app asks Ark for exactly one dataset path
//! (`v1/genome/genes/TAS2R38`). The user-facing report below avoids teaching
//! BioFS mechanics; those details belong here and in the README.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;

const GENE: &str = "TAS2R38";
const LENS_PATH: &str = "v1/genome/genes/TAS2R38";
const MAX_CHANGE_ROWS: usize = 24;
const RULER_WIDTH: usize = 64;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"bitter-meter\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/genes/TAS2R38\"]\n"
        );
        return Ok(());
    };

    let base = Path::new(&dir).join(LENS_PATH);
    print_header();

    match GeneReport::load(&base) {
        Ok(report) => report.print(),
        Err(err) => print_unavailable(&base, &err),
    }

    Ok(())
}

#[derive(Debug)]
struct GeneReport {
    chromosome: String,
    start: u64,
    end: u64,
    strand: String,
    biotype: String,
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
}

impl GeneReport {
    fn load(base: &Path) -> Result<Self, String> {
        let chromosome = read_leaf(base, "chromosome")?;
        let start = read_leaf(base, "start")?
            .parse::<u64>()
            .map_err(|err| format!("invalid gene start: {err}"))?;
        let end = read_leaf(base, "end")?
            .parse::<u64>()
            .map_err(|err| format!("invalid gene end: {err}"))?;
        let strand = read_leaf(base, "strand")?;
        let biotype = read_leaf(base, "biotype")?;
        let reference = read_sequence(&base.join("reference"))?;
        let changes = read_changes(&base.join("changes"));

        Ok(Self {
            chromosome,
            start,
            end,
            strand,
            biotype,
            reference,
            changes,
        })
    }

    fn print(&self) {
        let span = self.end.saturating_sub(self.start).saturating_add(1);
        let gc = gc_percent(&self.reference);
        let density = per_kb(self.changes.len(), span);
        let heterozygous = self
            .changes
            .iter()
            .filter(|c| c.call == "heterozygous")
            .count();
        let homozygous = self
            .changes
            .iter()
            .filter(|c| c.call == "homozygous change")
            .count();
        let uncertain = self
            .changes
            .iter()
            .filter(|c| c.call == "uncertain")
            .count();
        let indels = self.changes.iter().filter(|c| c.kind == "indel").count();

        println!("## Your Result");
        println!();
        println!("| Signal | Value |");
        println!("| :--- | ---: |");
        println!("| Gene span | {} bp |", format_int(span));
        println!("| Reference GC | {:.1}% |", gc);
        println!("| Observed changes | {} |", self.changes.len());
        println!("| Change density | {:.2} / kb |", density);
        println!("| Heterozygous calls | {heterozygous} |");
        println!("| Homozygous change calls | {homozygous} |");
        println!("| Indels or complex alleles | {indels} |");
        println!("| Uncertain calls | {uncertain} |");
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
            format_int(self.reference.len() as u64)
        );
        println!("| Local label | {} |", density_label(density));
        println!();

        println!("## Change Ruler");
        println!();
        println!(
            "The ruler marks observed non-reference changes across the gene. `+` means more than one change landed in the same text column."
        );
        println!();
        print_ruler(self.start, self.end, &self.changes);
        println!();

        print_change_table(&self.changes);
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

fn print_unavailable(_base: &Path, err: &str) {
    println!("## Result unavailable");
    println!();
    println!("The `{GENE}` gene data could not be read.");
    println!();
    println!("Reason: `{err}`");
    println!();
    println!(
        "This usually means the loaded genome package does not include the gene \
         annotations, reference sequence, or variant calls needed for this report."
    );
    println!();
    print_data_receipt();
    print_fine_print();
}

fn read_leaf(base: &Path, name: &str) -> Result<String, String> {
    fs::read_to_string(base.join(name))
        .map(|s| trim_line(&s))
        .map_err(|err| format!("failed to read `{name}`: {err}"))
}

fn read_sequence(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map(|s| s.trim_end_matches(['\n', '\r']).to_string())
        .map_err(|err| format!("failed to read `reference`: {err}"))
}

fn trim_line(s: &str) -> String {
    s.trim_end_matches(['\n', '\r']).to_string()
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
        changes.push(Change {
            pos,
            reference,
            genotype,
            call,
            kind,
        });
    }
    changes.sort_by_key(|c| c.pos);
    changes
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
    if reference.len() != 1 || alleles.iter().any(|a| *a != "." && a.len() != 1) {
        "indel"
    } else {
        "snp"
    }
}

fn alleles(genotype: &str) -> Vec<&str> {
    genotype
        .split(['/', '|'])
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .collect()
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
        track[idx] = if track[idx] == '*' { '+' } else { '*' };
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
            change.pos, change.reference, change.genotype, change.call, change.kind
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
        "The change list contains positions where the reported genotype differs \
         from the reference sequence. A position not shown here should not be \
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
