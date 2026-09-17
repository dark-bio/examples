//! VCF Roll Call: a full-VCF sanity report, in the shape docs/06-reports.md
//! describes.
//!
//! Tutorial note for app authors: this is deliberately a broad-permission demo.
//! It asks Ark for `v1/genome/snp-indel` and streams `vcf` with `noodles-vcf`.
//! That is appropriate for QA-style checks, but it is not a privacy-minimal lens
//! pattern like the `rsids/`, `genes/`, or `regions/` examples.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use noodles_vcf as vcf;
use noodles_vcf::variant::record::AlternateBases as _;
use noodles_vcf::variant::record::Filters as _;
use noodles_vcf::variant::record::info::field::Value as InfoValue;
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
             version = \"0.2.0\"\n\
             datasets = [\"v1/genome/snp-indel\"]\n"
        );
        return Ok(());
    };

    let path = Path::new(&dir).join(DATASET_PATH).join("vcf");
    print_header();

    match scan_vcf(&path) {
        Ok(report) => report.print(),
        Err(err) => {
            eprintln!("Could not read VCF: {err}");
            std::process::exit(1);
        }
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
        // The finding says what the file is and how it reads, in words, and
        // the check table beneath it grades the same facts.
        let (primary_count, other_count) = contig_counts(&self.chromosomes);
        print!(
            "Your call file holds {} records for {} {} in {} format, across {primary_count} primary \
             chromosomes and {other_count} other contigs. {} carry a non-reference \
             genotype, {} are all-reference and {} have a missing call",
            format_int(self.records),
            self.samples,
            if self.samples == 1 {
                "sample"
            } else {
                "samples"
            },
            self.file_format,
            format_int(self.variant_records()),
            format_int(self.genotype.hom_ref),
            format_int(self.genotype.missing)
        );
        if self.parse_errors > 0 {
            print!(
                ", and {} could not be parsed and were skipped",
                format_int(self.parse_errors)
            );
        }
        println!(". {}", self.shape_sentence());
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

        println!("## Evidence");
        println!();
        print_genotype_table(&self.genotype);
        print_variant_table(&self.variants);
        print_quality_table(self);
        print_chromosome_table(&self.chromosomes);
        print_method();
        print_limitations(self);
        print_sources();
    }

    /// One sentence on what the file's shape lets a reader conclude.
    fn shape_sentence(&self) -> &'static str {
        if self.reference_blocks.records > 0 {
            "The file carries reference blocks, so it says where calls were possible, \
             not only where they differ."
        } else if self.genotype.hom_ref > 0 {
            "The file carries all-reference calls at some sites, but no reference \
             blocks, so it doesn't say how much of the genome was callable."
        } else {
            "The file records variants only, so a sparse stretch is low variant density, \
             not evidence of low coverage."
        }
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
    // Each refill is a call out of the sandbox, so read in large blocks.
    scan_reader(BufReader::with_capacity(64 * 1024, file))
}

fn scan_reader(input: impl BufRead) -> Result<Report, Box<dyn Error>> {
    let mut reader = vcf::io::Reader::new(input);
    let header = reader.read_header()?;
    let file_format = header.file_format();
    let mut report = Report {
        file_format: format!("VCFv{}.{}", file_format.major(), file_format.minor()),
        samples: header.sample_names().len(),
        ..Report::default()
    };

    // Read each line before parsing it: a source read failure is always fatal.
    // A malformed record can then be skipped without retrying the same bytes.
    for line in reader.get_mut().lines() {
        let mut line = line?;
        line.push('\n');
        let mut parser = vcf::io::Reader::new(line.as_bytes());
        let mut record = vcf::Record::default();
        if parser.read_record(&mut record).is_err() {
            report.parse_errors += 1;
            continue;
        }
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
    // A reference block's END still defines its span in VCF 4.5 call files.
    let end = match record.info().get(header, "END") {
        Some(Ok(Some(InfoValue::Integer(end)))) if end >= 0 && end as u64 >= pos => end as u64,
        _ => pos + record.reference_bases().len().saturating_sub(1) as u64,
    };
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

    if is_reference_block(record, span)
        && matches!(genotype_class(header, record), GenotypeClass::HomRef)
    {
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

    if genotype_is_phased(record) {
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
    // A whole-sample dot is a missing call when FORMAT declares GT.
    if sample.as_ref().is_empty() && samples.keys().iter().any(|key| key == sample_key::GENOTYPE) {
        return GenotypeClass::Missing;
    }
    let genotype = match sample.get(header, sample_key::GENOTYPE) {
        Some(Ok(Some(SampleValue::Genotype(genotype)))) => genotype,
        Some(Ok(None)) => return GenotypeClass::Missing,
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

fn genotype_is_phased(record: &vcf::Record) -> bool {
    let samples = record.samples();
    let Some(index) = samples
        .keys()
        .iter()
        .position(|key| key == sample_key::GENOTYPE)
    else {
        return false;
    };
    let Some(sample) = samples.get_index(0) else {
        return false;
    };
    // A haploid call alone does not imply an explicit phase marker.
    sample
        .as_ref()
        .split(':')
        .nth(index)
        .is_some_and(|gt| gt.contains('|'))
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
    println!("# Call File Health Check");
    println!();
}

fn print_genotype_table(stats: &GenotypeStats) {
    println!("### Genotype calls");
    println!();
    println!("| Class | Records |");
    println!("| :--- | ---: |");
    println!("| All copies reference | {} |", format_int(stats.hom_ref));
    println!("| Heterozygous | {} |", format_int(stats.heterozygous));
    println!("| All copies alternate | {} |", format_int(stats.hom_alt));
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
    println!("### Variant shape");
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
    println!("### Depth and quality");
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
    println!("### Chromosome scoreboard");
    println!();
    println!(
        "Primary chromosomes are listed separately from alternate, random, unplaced, and patch contigs."
    );
    println!();
    println!("| Chromosome | Records | Variant GTs | All-ref GTs | Ref-block bp | Label |");
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
        println!("### Other contigs");
        println!();
        println!("| Signal | Value |");
        println!("| :--- | ---: |");
        println!("| Contigs | {} |", format_int(other.len() as u64));
        println!("| Records | {} |", format_int(total_records));
        println!("| Ref-block bp | {} |", format_int(total_ref_block_bases));
        println!();
        println!("Top non-primary contigs by record count:");
        println!();
        println!("| Contig | Records | Variant GTs | All-ref GTs | Ref-block bp | Label |");
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

fn print_method() {
    println!("## Method");
    println!();
    println!(
        "The app streams the whole call file one record at a time and classifies the \
         first sample's genotype, each record's allele shape, its filters, and its DP \
         and GQ fields when present. A record whose END reaches past its reference \
         allele, or whose alternate is a non-reference placeholder, counts as a \
         reference block, and their spans add up to a rough callable span. A record \
         that can't be parsed is counted and skipped, while a failed read stops the \
         app. The signs it looks for are missing genotype calls, absent DP or GQ fields \
         where sequencing data was expected, missing chromosomes and parse errors."
    );
    println!();
}

fn print_limitations(report: &Report) {
    println!("## Limitations");
    println!();
    println!(
        "This is a check of the file, not of how much of a genome was sequenced. \
         Base-by-base depth comes from alignment data or a gVCF with reference blocks, \
         and this file {}. Genotyping arrays, imputed files and variant-only exports \
         all look sparse while being valid for their purpose.",
        if report.reference_blocks.records > 0 {
            "carries such blocks, so its callable span is an estimate from them"
        } else {
            "carries none, so nothing here measures coverage"
        }
    );
    println!();
}

fn print_sources() {
    println!("## Sources");
    println!();
    println!(
        "1. The Variant Call Format specification, VCFv4.5. \
         https://samtools.github.io/hts-specs/VCFv4.5.pdf"
    );
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
