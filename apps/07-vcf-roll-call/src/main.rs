//! Coverage Weather: a full-VCF Markdown sanity report.
//!
//! Tutorial note for app authors: this is deliberately a broad-permission demo.
//! It asks Ark for `v1/genome/snp-indel` and streams `vcf` with `noodles-vcf`.
//! That is appropriate for QA-style checks, but it is not a privacy-minimal lens
//! pattern like the `rsids/`, `genes/`, or `regions/` examples.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use noodles_vcf as vcf;
use noodles_vcf::variant::Record as _;
use noodles_vcf::variant::record::AlternateBases as _;
use noodles_vcf::variant::record::Filters as _;
use noodles_vcf::variant::record::samples::Sample as _;
use noodles_vcf::variant::record::samples::keys::key as sample_key;
use noodles_vcf::variant::record::samples::series::Value as SampleValue;

const DATASET_PATH: &str = "v1/genome/snp-indel";
const MAX_CONTIG_ROWS: usize = 12;
const METRIC_MAX: usize = 512;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"vcf-roll-call\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"v1/genome/snp-indel\"]\n"
        );
        return Ok(());
    };

    let path = Path::new(&dir).join(DATASET_PATH).join("vcf");
    print_header();

    match scan_vcf(&path) {
        Ok(report) => report.print(),
        Err(err) => print_unavailable(err.as_ref()),
    }

    Ok(())
}

#[derive(Default)]
struct Report {
    file_format: String,
    samples: usize,
    records: u64,
    parse_errors: u64,
    chromosomes: BTreeMap<String, ChromStats>,
    genotype: GenotypeStats,
    variants: VariantStats,
    depth: MetricStats,
    quality: MetricStats,
    reference_blocks: ReferenceBlockStats,
    pass_records: u64,
    filtered_records: u64,
    unfiltered_records: u64,
}

#[derive(Default)]
struct ChromStats {
    records: u64,
    variant_records: u64,
    hom_ref_records: u64,
    missing_records: u64,
    snps: u64,
    indels: u64,
    reference_block_bases: u64,
    max_pos: u64,
}

#[derive(Default)]
struct GenotypeStats {
    with_gt: u64,
    missing: u64,
    hom_ref: u64,
    heterozygous: u64,
    hom_alt: u64,
    mixed_alt: u64,
    phased: u64,
    no_sample: u64,
}

#[derive(Default)]
struct VariantStats {
    snps: u64,
    indels: u64,
    symbolic: u64,
    multiallelic: u64,
    transitions: u64,
    transversions: u64,
}

#[derive(Default)]
struct ReferenceBlockStats {
    records: u64,
    bases: u64,
}

struct MetricStats {
    bins: [u64; METRIC_MAX + 1],
    overflow: u64,
    count: u64,
}

impl Default for MetricStats {
    fn default() -> Self {
        Self {
            bins: [0; METRIC_MAX + 1],
            overflow: 0,
            count: 0,
        }
    }
}

enum MetricValue {
    Exact(u32),
    Above(u32),
}

impl MetricStats {
    fn push(&mut self, value: u32) {
        self.count += 1;
        if value as usize > METRIC_MAX {
            self.overflow += 1;
        } else {
            self.bins[value as usize] += 1;
        }
    }

    fn len(&self) -> u64 {
        self.count
    }

    fn median(&self) -> Option<MetricValue> {
        if self.count == 0 {
            return None;
        }
        let target = self.count / 2;
        let mut seen = 0u64;
        for (value, count) in self.bins.iter().enumerate() {
            seen += count;
            if seen > target {
                return Some(MetricValue::Exact(value as u32));
            }
        }
        Some(MetricValue::Above(METRIC_MAX as u32))
    }

    fn low_count(&self, threshold: u32) -> u64 {
        let threshold = threshold as usize;
        let in_range: u64 = self.bins.iter().take(threshold.min(self.bins.len())).sum();
        if threshold > METRIC_MAX {
            in_range + self.overflow
        } else {
            in_range
        }
    }
}

enum GenotypeClass {
    Missing,
    HomRef,
    Heterozygous,
    HomAlt,
    MixedAlt,
    NoGt,
}

impl Report {
    fn print(&self) {
        println!("## Your Result");
        println!();
        println!("| Signal | Value |");
        println!("| :--- | ---: |");
        println!("| VCF version | `{}` |", self.file_format);
        println!("| Samples | {} |", self.samples);
        println!("| Records scanned | {} |", format_int(self.records));
        let (primary_count, other_count) = contig_counts(&self.chromosomes);
        println!("| Primary chromosomes seen | {primary_count} |");
        println!("| Other contigs seen | {other_count} |");
        println!(
            "| Variant records | {} |",
            format_int(self.variant_records())
        );
        println!(
            "| Homozygous-reference records | {} |",
            format_int(self.genotype.hom_ref)
        );
        println!(
            "| Reference-block span | {} bp |",
            format_int(self.reference_blocks.bases)
        );
        println!("| Median DP | {} |", optional_metric(self.depth.median()));
        println!("| Median GQ | {} |", optional_metric(self.quality.median()));
        println!(
            "| Missing GT calls | {} |",
            format_int(self.genotype.missing)
        );
        println!(
            "| Parse errors skipped | {} |",
            format_int(self.parse_errors)
        );
        println!();

        println!("## VCF Roll Call");
        println!();
        println!("| Check | Status | Detail |");
        println!("| :--- | :--- | :--- |");
        println!(
            "| File shape | {} | {} |",
            weather_mark(self.shape_score()),
            self.shape_label()
        );
        println!(
            "| Callable evidence | {} | {} |",
            weather_mark(self.callable_score()),
            self.callable_label()
        );
        println!(
            "| Depth fields | {} | {} |",
            weather_mark(if self.depth.len() > 0 { 2 } else { 0 }),
            metric_label("DP", self.depth.len(), self.records)
        );
        println!(
            "| Quality fields | {} | {} |",
            weather_mark(if self.quality.len() > 0 { 2 } else { 0 }),
            metric_label("GQ", self.quality.len(), self.records)
        );
        println!(
            "| Missingness | {} | {} |",
            weather_mark(self.missing_score()),
            self.missing_label()
        );
        println!();

        print_genotype_table(&self.genotype);
        print_variant_table(&self.variants);
        print_quality_table(self);
        print_chromosome_table(&self.chromosomes);
        print_meaning(self);
        print_fine_print();
    }

    fn variant_records(&self) -> u64 {
        self.genotype.heterozygous + self.genotype.hom_alt + self.genotype.mixed_alt
    }

    fn shape_score(&self) -> u8 {
        if self.reference_blocks.records > 0 {
            2
        } else if self.genotype.hom_ref > 0 {
            1
        } else {
            0
        }
    }

    fn shape_label(&self) -> String {
        if self.reference_blocks.records > 0 {
            format!(
                "{} reference-block records give callable-span hints",
                format_int(self.reference_blocks.records)
            )
        } else if self.genotype.hom_ref > 0 {
            format!(
                "{} hom-ref records found, but no reference-block span",
                format_int(self.genotype.hom_ref)
            )
        } else {
            "variant-only shape: sparse regions cannot be called uncovered".to_string()
        }
    }

    fn callable_score(&self) -> u8 {
        if self.reference_blocks.bases > 0 {
            2
        } else if self.genotype.hom_ref > 0 {
            1
        } else {
            0
        }
    }

    fn callable_label(&self) -> String {
        if self.reference_blocks.bases > 0 {
            format!(
                "about {} bp covered by reference blocks",
                format_int(self.reference_blocks.bases)
            )
        } else if self.genotype.hom_ref > 0 {
            "hom-ref sites are present, but callable span is unknown".to_string()
        } else {
            "no direct callable-span evidence in this VCF".to_string()
        }
    }

    fn missing_score(&self) -> u8 {
        if self.genotype.with_gt == 0 {
            return 0;
        }
        let rate = self.genotype.missing as f64 / self.genotype.with_gt as f64;
        if rate < 0.01 {
            2
        } else if rate < 0.05 {
            1
        } else {
            0
        }
    }

    fn missing_label(&self) -> String {
        if self.genotype.with_gt == 0 {
            "no genotype calls were readable".to_string()
        } else {
            format!(
                "{:.2}% of readable GT calls are missing",
                self.genotype.missing as f64 * 100.0 / self.genotype.with_gt as f64
            )
        }
    }
}

fn scan_vcf(path: &Path) -> Result<Report, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = vcf::io::Reader::new(BufReader::new(file));
    let header = reader.read_header()?;
    let file_format = header.file_format();
    let mut report = Report {
        file_format: format!("VCFv{}.{}", file_format.major(), file_format.minor()),
        samples: header.sample_names().len(),
        ..Report::default()
    };

    for result in reader.records() {
        let record = match result {
            Ok(record) => record,
            Err(_) => {
                report.parse_errors += 1;
                continue;
            }
        };
        scan_record(&header, &record, &mut report);
    }

    Ok(report)
}

fn scan_record(header: &vcf::Header, record: &vcf::Record, report: &mut Report) {
    report.records += 1;

    let chrom = record.reference_sequence_name().to_string();
    let pos = record
        .variant_start()
        .and_then(Result::ok)
        .map(|p| p.get() as u64)
        .unwrap_or(0);
    let end = record
        .variant_end(header)
        .ok()
        .map(|p| p.get() as u64)
        .unwrap_or(pos);
    let span = end.saturating_sub(pos).saturating_add(1);

    let chrom_stats = report.chromosomes.entry(chrom).or_default();
    chrom_stats.records += 1;
    chrom_stats.max_pos = chrom_stats.max_pos.max(end);

    let kind = variant_kind(record);
    match kind {
        VariantKind::Snp => {
            report.variants.snps += 1;
            chrom_stats.snps += 1;
            count_substitutions(record, &mut report.variants);
        }
        VariantKind::Indel => {
            report.variants.indels += 1;
            chrom_stats.indels += 1;
        }
        VariantKind::Symbolic => report.variants.symbolic += 1,
        VariantKind::Other => {}
    }
    if alternate_count(record) > 1 {
        report.variants.multiallelic += 1;
    }

    if is_reference_block(record, span) {
        report.reference_blocks.records += 1;
        report.reference_blocks.bases += span;
        chrom_stats.reference_block_bases += span;
    }

    match genotype_class(header, record) {
        GenotypeClass::Missing => {
            report.genotype.with_gt += 1;
            report.genotype.missing += 1;
            chrom_stats.missing_records += 1;
        }
        GenotypeClass::HomRef => {
            report.genotype.with_gt += 1;
            report.genotype.hom_ref += 1;
            chrom_stats.hom_ref_records += 1;
        }
        GenotypeClass::Heterozygous => {
            report.genotype.with_gt += 1;
            report.genotype.heterozygous += 1;
            chrom_stats.variant_records += 1;
        }
        GenotypeClass::HomAlt => {
            report.genotype.with_gt += 1;
            report.genotype.hom_alt += 1;
            chrom_stats.variant_records += 1;
        }
        GenotypeClass::MixedAlt => {
            report.genotype.with_gt += 1;
            report.genotype.mixed_alt += 1;
            chrom_stats.variant_records += 1;
        }
        GenotypeClass::NoGt => report.genotype.no_sample += 1,
    }

    if genotype_is_phased(header, record) {
        report.genotype.phased += 1;
    }

    if let Some(dp) = sample_integer(header, record, sample_key::READ_DEPTH) {
        report.depth.push(dp);
    }
    if let Some(gq) = sample_integer(header, record, sample_key::CONDITIONAL_GENOTYPE_QUALITY) {
        report.quality.push(gq);
    }

    let filters = record.filters();
    if filters.is_pass(header).unwrap_or(false) {
        report.pass_records += 1;
    } else if filters.is_empty() {
        report.unfiltered_records += 1;
    } else {
        report.filtered_records += 1;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VariantKind {
    Snp,
    Indel,
    Symbolic,
    Other,
}

fn variant_kind(record: &vcf::Record) -> VariantKind {
    let reference = record.reference_bases().to_string();
    let alts: Vec<String> = record
        .alternate_bases()
        .iter()
        .filter_map(Result::ok)
        .map(str::to_string)
        .collect();
    if alts
        .iter()
        .any(|alt| alt.starts_with('<') || alt.contains('[') || alt.contains(']'))
    {
        return VariantKind::Symbolic;
    }
    if reference.len() == 1 && alts.iter().all(|alt| alt.len() == 1) {
        VariantKind::Snp
    } else if alts
        .iter()
        .any(|alt| alt != "." && alt.len() != reference.len())
    {
        VariantKind::Indel
    } else if alts.is_empty() || alts.iter().all(|alt| alt == ".") {
        VariantKind::Other
    } else {
        VariantKind::Other
    }
}

fn alternate_count(record: &vcf::Record) -> usize {
    record
        .alternate_bases()
        .iter()
        .filter_map(Result::ok)
        .filter(|alt| *alt != ".")
        .count()
}

fn count_substitutions(record: &vcf::Record, stats: &mut VariantStats) {
    let reference = record.reference_bases().to_string();
    let Some(reference) = reference.chars().next().map(|c| c.to_ascii_uppercase()) else {
        return;
    };
    for alt in record.alternate_bases().iter().filter_map(Result::ok) {
        let Some(alt) = alt.chars().next().map(|c| c.to_ascii_uppercase()) else {
            continue;
        };
        if is_transition(reference, alt) {
            stats.transitions += 1;
        } else {
            stats.transversions += 1;
        }
    }
}

fn is_transition(reference: char, alt: char) -> bool {
    matches!(
        (reference, alt),
        ('A', 'G') | ('G', 'A') | ('C', 'T') | ('T', 'C')
    )
}

fn is_reference_block(record: &vcf::Record, span: u64) -> bool {
    if span > record.reference_bases().len() as u64 {
        return true;
    }
    record
        .alternate_bases()
        .iter()
        .filter_map(Result::ok)
        .any(|alt| alt == "<NON_REF>" || alt == "<*>" || alt == ".")
}

fn genotype_class(header: &vcf::Header, record: &vcf::Record) -> GenotypeClass {
    let samples = record.samples();
    let Some(sample) = samples.get_index(0) else {
        return GenotypeClass::NoGt;
    };
    let genotype = match sample.get(header, sample_key::GENOTYPE) {
        Some(Ok(Some(SampleValue::Genotype(genotype)))) => genotype,
        _ => return GenotypeClass::NoGt,
    };

    let mut alleles = Vec::new();
    for allele in genotype.iter() {
        let Ok((index, _)) = allele else {
            return GenotypeClass::Missing;
        };
        let Some(index) = index else {
            return GenotypeClass::Missing;
        };
        alleles.push(index);
    }
    if alleles.is_empty() {
        return GenotypeClass::Missing;
    }
    if alleles.iter().all(|index| *index == 0) {
        return GenotypeClass::HomRef;
    }
    let unique: BTreeSet<usize> = alleles.iter().copied().collect();
    if unique.len() == 1 {
        GenotypeClass::HomAlt
    } else if unique.contains(&0) {
        GenotypeClass::Heterozygous
    } else {
        GenotypeClass::MixedAlt
    }
}

fn genotype_is_phased(header: &vcf::Header, record: &vcf::Record) -> bool {
    use noodles_vcf::variant::record::samples::series::value::genotype::Phasing;

    let samples = record.samples();
    let Some(sample) = samples.get_index(0) else {
        return false;
    };
    let genotype = match sample.get(header, sample_key::GENOTYPE) {
        Some(Ok(Some(SampleValue::Genotype(genotype)))) => genotype,
        _ => return false,
    };
    genotype
        .iter()
        .any(|allele| matches!(allele, Ok((_, Phasing::Phased))))
}

fn sample_integer(header: &vcf::Header, record: &vcf::Record, key: &str) -> Option<u32> {
    let samples = record.samples();
    let sample = samples.get_index(0)?;
    match sample.get(header, key)? {
        Ok(Some(SampleValue::Integer(n))) if n >= 0 => Some(n as u32),
        Ok(Some(SampleValue::Float(n))) if n >= 0.0 => Some(n.round() as u32),
        _ => None,
    }
}

fn print_header() {
    println!("# VCF Roll Call");
    println!();
    println!(
        "This report scans the loaded variant calls for signs that the file is \
         healthy: chromosomes are present, genotype calls are readable, depth and \
         quality fields appear when available, and reference blocks are counted \
         when the VCF contains them."
    );
    println!();
}

fn print_unavailable(err: &dyn Error) {
    println!("## Result unavailable");
    println!();
    println!("The SNP/indel VCF could not be read.");
    println!();
    println!("Reason: `{err}`");
    println!();
}

fn print_genotype_table(stats: &GenotypeStats) {
    println!("## Genotype Calls");
    println!();
    println!("| Class | Records |");
    println!("| :--- | ---: |");
    println!("| Homozygous reference | {} |", format_int(stats.hom_ref));
    println!("| Heterozygous | {} |", format_int(stats.heterozygous));
    println!("| Homozygous alternate | {} |", format_int(stats.hom_alt));
    println!(
        "| Alternate/alternate mixed | {} |",
        format_int(stats.mixed_alt)
    );
    println!("| Missing GT | {} |", format_int(stats.missing));
    println!(
        "| No readable sample GT | {} |",
        format_int(stats.no_sample)
    );
    println!("| Phased GT records | {} |", format_int(stats.phased));
    println!();
}

fn print_variant_table(stats: &VariantStats) {
    println!("## Variant Shape");
    println!();
    println!("| Signal | Records |");
    println!("| :--- | ---: |");
    println!("| SNP-like records | {} |", format_int(stats.snps));
    println!("| Indel-like records | {} |", format_int(stats.indels));
    println!("| Symbolic records | {} |", format_int(stats.symbolic));
    println!(
        "| Multiallelic records | {} |",
        format_int(stats.multiallelic)
    );
    println!("| Transitions | {} |", format_int(stats.transitions));
    println!("| Transversions | {} |", format_int(stats.transversions));
    println!();
}

fn print_quality_table(report: &Report) {
    println!("## Depth And Quality");
    println!();
    println!("| Signal | Value |");
    println!("| :--- | ---: |");
    println!("| Records with DP | {} |", format_int(report.depth.len()));
    println!("| Median DP | {} |", optional_metric(report.depth.median()));
    println!("| DP < 10 | {} |", format_int(report.depth.low_count(10)));
    println!("| Records with GQ | {} |", format_int(report.quality.len()));
    println!(
        "| Median GQ | {} |",
        optional_metric(report.quality.median())
    );
    println!("| GQ < 20 | {} |", format_int(report.quality.low_count(20)));
    println!("| PASS records | {} |", format_int(report.pass_records));
    println!(
        "| Filtered records | {} |",
        format_int(report.filtered_records)
    );
    println!(
        "| Unfiltered records | {} |",
        format_int(report.unfiltered_records)
    );
    println!();
}

fn print_chromosome_table(chromosomes: &BTreeMap<String, ChromStats>) {
    println!("## Chromosome Scoreboard");
    println!();
    println!(
        "Primary chromosomes are listed separately from alternate, random, unplaced, and patch contigs."
    );
    println!();
    println!("| Chromosome | Records | Variant GTs | Hom-ref GTs | Ref-block bp | Label |");
    println!("| :--- | ---: | ---: | ---: | ---: | :--- |");
    for primary in primary_chromosome_order() {
        let Some((name, stats)) = find_primary(chromosomes, primary) else {
            println!("| `{}` | 0 | 0 | 0 | 0 | missing |", primary.display());
            continue;
        };
        print_contig_row(name, stats);
    }

    let other: Vec<(&String, &ChromStats)> = chromosomes
        .iter()
        .filter(|(name, _)| classify_primary(name).is_none())
        .collect();
    if !other.is_empty() {
        let total_records: u64 = other.iter().map(|(_, stats)| stats.records).sum();
        let total_ref_block_bases: u64 = other
            .iter()
            .map(|(_, stats)| stats.reference_block_bases)
            .sum();
        println!();
        println!("## Other Contigs");
        println!();
        println!("| Signal | Value |");
        println!("| :--- | ---: |");
        println!("| Contigs | {} |", format_int(other.len() as u64));
        println!("| Records | {} |", format_int(total_records));
        println!("| Ref-block bp | {} |", format_int(total_ref_block_bases));
        println!();
        println!("Top non-primary contigs by record count:");
        println!();
        println!("| Contig | Records | Variant GTs | Hom-ref GTs | Ref-block bp | Label |");
        println!("| :--- | ---: | ---: | ---: | ---: | :--- |");
        let mut top = other;
        top.sort_by(|a, b| b.1.records.cmp(&a.1.records).then_with(|| a.0.cmp(b.0)));
        for (name, stats) in top.iter().take(MAX_CONTIG_ROWS) {
            print_contig_row(name, stats);
        }
        if top.len() > MAX_CONTIG_ROWS {
            println!(
                "| ... | ... | ... | ... | ... | {} more non-primary contigs omitted |",
                format_int((top.len() - MAX_CONTIG_ROWS) as u64)
            );
        }
    }
    println!();
}

fn print_contig_row(name: &str, stats: &ChromStats) {
    println!(
        "| `{}` | {} | {} | {} | {} | {} |",
        name,
        format_int(stats.records),
        format_int(stats.variant_records),
        format_int(stats.hom_ref_records),
        format_int(stats.reference_block_bases),
        chromosome_label(stats)
    );
}

fn print_meaning(report: &Report) {
    println!("## What It Means");
    println!();
    if report.reference_blocks.records > 0 {
        println!(
            "This VCF includes reference-block style records, so the report can \
             make a rough callable-span estimate from those blocks. That is a \
             stronger coverage hint than variant density alone."
        );
    } else if report.genotype.hom_ref > 0 {
        println!(
            "This VCF includes homozygous-reference calls, which proves some \
             reference sites were emitted. It does not provide a complete callable \
             span estimate unless those calls cover intervals."
        );
    } else {
        println!(
            "This looks like a variant-only VCF. Sparse windows in this report \
             should be read as low variant density, not as proof of low sequencing \
             coverage."
        );
    }
    println!();
    println!(
        "The most useful red flags are missing genotype calls, absent DP/GQ fields \
         when you expected sequencing data, unexpectedly missing chromosomes, or \
         parse errors while scanning records."
    );
    println!();
}

fn print_fine_print() {
    println!("## Fine Print");
    println!();
    println!(
        "This is a VCF sanity check, not a formal coverage calculator. True base-by-base \
         depth normally comes from alignment data or a gVCF with reference blocks. \
         Genotyping arrays, imputed VCFs, and variant-only exports can all look \
         sparse while still being valid for their intended purpose."
    );
    println!();
    println!("This report is for demo and data-quality triage only.");
    println!();
}

fn weather_mark(score: u8) -> &'static str {
    match score {
        2 => "clear",
        1 => "mixed",
        _ => "limited",
    }
}

fn metric_label(name: &str, count: u64, total: u64) -> String {
    if count == 0 {
        format!("no {name} values found")
    } else {
        format!(
            "{} records carry {name} ({:.1}%)",
            format_int(count),
            count as f64 * 100.0 / total.max(1) as f64
        )
    }
}

fn chromosome_label(stats: &ChromStats) -> &'static str {
    if stats.reference_block_bases > 0 {
        "callable hints"
    } else if stats.records == 0 {
        "empty"
    } else if stats.variant_records == 0 && stats.hom_ref_records > 0 {
        "reference only"
    } else if stats.records < 10 {
        "sparse"
    } else {
        "present"
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PrimaryChromosome {
    Num(u8),
    X,
    Y,
    M,
}

impl PrimaryChromosome {
    fn display(self) -> String {
        match self {
            Self::Num(n) => format!("chr{n}"),
            Self::X => "chrX".to_string(),
            Self::Y => "chrY".to_string(),
            Self::M => "chrM".to_string(),
        }
    }
}

fn primary_chromosome_order() -> Vec<PrimaryChromosome> {
    let mut chromosomes: Vec<PrimaryChromosome> = (1..=22).map(PrimaryChromosome::Num).collect();
    chromosomes.extend([
        PrimaryChromosome::X,
        PrimaryChromosome::Y,
        PrimaryChromosome::M,
    ]);
    chromosomes
}

fn contig_counts(chromosomes: &BTreeMap<String, ChromStats>) -> (usize, usize) {
    let primary: BTreeSet<PrimaryChromosome> = chromosomes
        .keys()
        .filter_map(|name| classify_primary(name))
        .collect();
    let other = chromosomes
        .keys()
        .filter(|name| classify_primary(name).is_none())
        .count();
    (primary.len(), other)
}

fn find_primary(
    chromosomes: &BTreeMap<String, ChromStats>,
    primary: PrimaryChromosome,
) -> Option<(&String, &ChromStats)> {
    chromosomes
        .iter()
        .find(|(name, _)| classify_primary(name) == Some(primary))
}

fn classify_primary(name: &str) -> Option<PrimaryChromosome> {
    let core = name.strip_prefix("chr").unwrap_or(name);
    match core {
        "X" => return Some(PrimaryChromosome::X),
        "Y" => return Some(PrimaryChromosome::Y),
        "M" | "MT" => return Some(PrimaryChromosome::M),
        _ => {}
    }
    if let Ok(n) = core.parse::<u8>()
        && (1..=22).contains(&n)
    {
        return Some(PrimaryChromosome::Num(n));
    }
    classify_refseq_primary(name)
}

fn classify_refseq_primary(name: &str) -> Option<PrimaryChromosome> {
    let serial: u32 = name.strip_prefix("NC_")?.split('.').next()?.parse().ok()?;
    match serial {
        1..=22 => Some(PrimaryChromosome::Num(serial as u8)),
        23 => Some(PrimaryChromosome::X),
        24 => Some(PrimaryChromosome::Y),
        12920 => Some(PrimaryChromosome::M),
        60925..=60946 => Some(PrimaryChromosome::Num((serial - 60924) as u8)),
        60947 => Some(PrimaryChromosome::X),
        60948 => Some(PrimaryChromosome::Y),
        _ => None,
    }
}

fn optional_metric(value: Option<MetricValue>) -> String {
    match value {
        Some(MetricValue::Exact(value)) => value.to_string(),
        Some(MetricValue::Above(value)) => format!(">{value}"),
        None => "not present".to_string(),
    }
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
