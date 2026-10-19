//! Doping Test: a report on the copies of UGT2B17.
//!
//! UGT2B17's enzyme readies testosterone for urine, where the classic doping
//! test reads it, and a common deletion removes the whole gene. The app counts
//! the owner's copies from the gene's copy number segments, in the report shape
//! docs/06-reports.md describes.
//!
//! The app asks the Ark for the gene, `v1/genome/genes/UGT2B17`, and for two
//! public grants. `v1/genome/datasets/copy-number-variants` requires the copy
//! number file, and `v1/genome/files/reference-genome` names the assembly build.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

/// The one gene the app asks for, relative to the data root.
const LENS: &str = "v1/genome/genes/UGT2B17";

/// One copy number segment reaching the gene.
#[derive(Debug)]
struct Segment {
    /// First base of the segment's span.
    start: u64,
    /// Last base of the segment's span.
    end: u64,
    /// Copy number as the file holds it, absent when it holds none.
    count: Option<String>,
    /// Filters the segment failed, one per line, absent when it passed.
    failed: Option<String>,
}

/// Everything the report rests on, read once from the granted paths.
struct Report {
    /// Chromosome the gene sits on.
    chromosome: String,
    /// First base of the gene.
    start: u64,
    /// Last base of the gene.
    end: u64,
    /// Segments reaching the gene, ordered by their first base.
    segments: Vec<Segment>,
    /// Assembly build, absent when it couldn't be read.
    build: Option<String>,
}

/// What the segments say about the gene's copies.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The copy number file has no segment over the gene.
    Uncalled,
    /// The segments leave part of the gene uncovered.
    Partial,
    /// A segment holds no whole copy count.
    Unreadable,
    /// The segments disagree on the count.
    Split,
    /// One count covers the whole gene.
    Copies(u64),
}

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Doping Test"
version = "0.1.0"

[reads]
paths = ["v1/genome/genes/UGT2B17", "v1/genome/datasets/copy-number-variants", "v1/genome/files/reference-genome"]

[listing]
language = "en"
icon = "🏅"
summary = "Why the classic urine doping test barely sees testosterone in some people."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/07-doping-test"
keywords = ["doping", "testosterone", "UGT2B17", "copy number", "sport"]
description = '''
The classic urine doping test reads testosterone against epitestosterone. The enzyme _UGT2B17_ makes readies testosterone for urine, and a common deletion removes the whole gene. Men with no copies excrete almost none, so their ratio starts near zero.

This app counts your copies of _UGT2B17_ from your copy number file. A small variant file can't see a missing gene, so the app needs your copy number file, and the Ark won't run it without one.

One grant covers the whole gene, including your variants in it, which this app does not read.
'''

[listing.purposes]
"v1/genome/genes/UGT2B17" = "The gene behind the doping test, with your copy number over it"
"v1/genome/datasets/copy-number-variants" = "Your copy number file has to be on the Ark, though the app never reads it"
"v1/genome/files/reference-genome" = "The reference build, to name positions"
"#
        );
        return;
    };
    match Report::load(Path::new(&root)) {
        Ok(report) => report.print(),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
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

/// Reads a gene value the report can't do without.
fn required(base: &Path, name: &str) -> Result<String, String> {
    leaf(base, name)?
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Missing gene {name}"))
}

/// Parses one base coordinate.
fn coordinate(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|err| format!("Invalid coordinate: {err}"))
}

/// Lists the segments reaching the gene, sorted by their first base.
fn segments(base: &Path) -> Result<Vec<Segment>, String> {
    let entries = match fs::read_dir(base.join("copies")) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(format!("Could not list copies: {err}")),
    };
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("Could not read segment: {err}"))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "Invalid segment name")?;
        let (start, end) = name.split_once('-').ok_or("Invalid segment name")?;
        let (start, end) = (coordinate(start)?, coordinate(end)?);
        result.push(Segment {
            start,
            end,
            count: leaf(&entry.path(), "count")?,
            failed: leaf(&entry.path(), "failed")?,
        });
    }
    result.sort_by_key(|segment| (segment.start, segment.end));
    Ok(result)
}

/// Gathers filter names, sorted and without repeats.
fn filters<'a>(values: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    values
        .flat_map(str::lines)
        .filter(|name| !name.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl Report {
    /// Reads the gene, its segments and the build.
    fn load(root: &Path) -> Result<Self, String> {
        let base = root.join(LENS);
        let chromosome = required(&base, "chromosome")?;
        let start = coordinate(&required(&base, "start")?)?;
        let end = coordinate(&required(&base, "end")?)?;
        if end < start {
            return Err("Gene end precedes its start".into());
        }
        let segments = segments(&base)?;
        let build = leaf(&root.join("v1/genome/files/reference-genome"), "build")?;
        Ok(Self {
            chromosome,
            start,
            end,
            segments,
            build,
        })
    }

    /// Decides the outcome, checking for segments, coverage, readable counts and
    /// agreement, in that order.
    fn outcome(&self) -> Outcome {
        if self.segments.is_empty() {
            return Outcome::Uncalled;
        }

        // Walk the segments from the gene's first base, stopping at the first gap
        let mut next = self.start;
        let mut covered = false;
        for segment in &self.segments {
            if segment.end < next {
                continue;
            }
            if segment.start > next {
                break;
            }
            if segment.end >= self.end {
                covered = true;
                break;
            }
            next = segment.end + 1;
        }
        if !covered {
            return Outcome::Partial;
        }

        // Every count must be readable before the counts can agree or disagree
        let counts = self
            .segments
            .iter()
            .map(|segment| {
                segment
                    .count
                    .as_deref()
                    .and_then(|count| count.parse::<u64>().ok())
            })
            .collect::<Option<Vec<_>>>();
        let Some(counts) = counts else {
            return Outcome::Unreadable;
        };
        let first = counts[0];
        if counts.iter().any(|count| *count != first) {
            Outcome::Split
        } else {
            Outcome::Copies(first)
        }
    }

    /// Formats a span, labeled with the build when it was read.
    fn location(&self, start: u64, end: u64) -> String {
        let coordinate = format!("{}:{start}-{end}", self.chromosome);
        match &self.build {
            Some(build) => format!("{coordinate}, {build}"),
            None => coordinate,
        }
    }

    /// Prints the report, from the finding down to the sources.
    fn print(&self) {
        println!("# 🏅 The Doping Test Gene\n");
        let outcome = self.outcome();
        print!("{}", finding(&outcome));

        // Only a count can be flagged, since the other outcomes have none to read
        if let Outcome::Copies(_) = outcome {
            let failed = filters(
                self.segments
                    .iter()
                    .filter_map(|segment| segment.failed.as_deref()),
            );
            if !failed.is_empty() {
                print!(
                    " Your copy number caller flagged this as `{}`, so read the count with care.",
                    failed.join(", ")
                );
            }
        }
        println!("\n\n## Evidence\n");
        if !self.segments.is_empty() {
            println!("| Segment | Copies | Filters |");
            println!("| :-- | :-- | :-- |");
            for segment in &self.segments {
                let failed = filters(segment.failed.as_deref().into_iter());
                let failed = if failed.is_empty() {
                    "passed".into()
                } else {
                    failed.join(", ")
                };
                println!(
                    "| {} | {} | {} |",
                    self.location(segment.start, segment.end),
                    segment.count.as_deref().unwrap_or("absent"),
                    failed
                );
            }
            println!();
        }
        println!("*UGT2B17* spans {}.\n", self.location(self.start, self.end));
        print_method();
        print_limitations();
        print_sources();
    }
}

/// The finding paragraph for an outcome, before any flag sentence.
fn finding(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Copies(0) => "**Ghost mode** 👻. You carry no copies of *UGT2B17*, the gene \
            whose enzyme readies testosterone for urine. In studies of men, those with no \
            copies excreted almost none, so the classic doping test, which reads testosterone \
            against epitestosterone, barely sees it. After a testosterone dose, 40% of them \
            never crossed the test's threshold."
            .into(),
        Outcome::Copies(1) => "**Half volume** 🔉. You carry one copy of *UGT2B17*, the gene \
            whose enzyme readies testosterone for urine. The classic doping test reads that \
            testosterone against epitestosterone, and one copy keeps it visible. The test \
            struggles only with people who carry none."
            .into(),
        Outcome::Copies(2) => "**Full volume** 🔊. You carry two copies of *UGT2B17*, the gene \
            whose enzyme readies testosterone for urine. The classic doping test reads that \
            testosterone against epitestosterone, and with both copies yours shows up the way \
            the test expects."
            .into(),
        Outcome::Copies(count) => format!(
            "**Extra copies** 🔊. You carry {count} copies of \
            *UGT2B17*, the gene whose enzyme readies testosterone for urine. The studies here \
            compared zero, one and two copies, so they say nothing about more."
        ),
        Outcome::Uncalled => "**No answer.** Your copy number file has no segment over \
            *UGT2B17*, so this app has no count. A caller leaves out regions it can't measure, \
            and a missing segment is not zero copies."
            .into(),
        Outcome::Partial => "**No answer.** Your copy number segments cover only part of \
            *UGT2B17*, so this app has no count for the whole gene."
            .into(),
        Outcome::Unreadable => "**No answer.** A segment over *UGT2B17* holds no whole copy \
            count, so this app has no count."
            .into(),
        Outcome::Split => "**No single count.** Your copy number changes inside *UGT2B17*, \
            so part of the gene has a different count from the rest. The studies here \
            compared copies of the whole gene."
            .into(),
    }
}

fn print_method() {
    println!("## Method\n");
    println!(
        "Doping tests have long read testosterone in urine against epitestosterone, \
        its near twin, and a ratio above 4 was the classic flag (Schulze et al., 2008). The \
        test sees testosterone once an enzyme has tagged it for urine, mostly the one \
        *UGT2B17* makes. A common deletion removes the whole gene (Wilson et al., 2004). \
        Jakobsson et al. (2006) found that Swedish and Korean men with no copies excreted \
        almost no testosterone. Schulze et al. (2008) then gave 55 men one 500 mg dose of \
        testosterone, and 40% of those with no copies never reached a ratio of 4 in the 15 \
        days after. The app reads the copy number segments your file holds over the gene \
        and reports their count.\n"
    );
}

fn print_limitations() {
    println!("## Limitations\n");
    println!(
        "A copy count, not a doping verdict. The studies measured men, mostly of Swedish \
        and Korean ancestry, and the share with no copies differed sevenfold between them, \
        9.3% and 66.7% (Jakobsson et al., 2006). A copy number caller can miss or misplace a \
        segment, and this app reads the segments as your file holds them.\n"
    );
}

fn print_sources() {
    println!("## Sources\n");
    println!(
        "1. Schulze JJ, et al. Doping test results dependent on genotype of uridine \
        diphospho-glucuronosyl transferase 2B17, the major enzyme for testosterone \
        glucuronidation. J Clin Endocrinol Metab. 2008;93(7):2500-2506. \
        https://doi.org/10.1210/jc.2008-0218"
    );
    println!(
        "2. Wilson W 3rd, et al. Characterization of a common deletion polymorphism of \
        the UGT2B17 gene linked to UGT2B15. Genomics. 2004;84(4):707-714. \
        https://doi.org/10.1016/j.ygeno.2004.06.011"
    );
    println!(
        "3. Jakobsson J, et al. Large differences in testosterone excretion in Korean \
        and Swedish men are strongly associated with a UDP-glucuronosyl transferase 2B17 \
        polymorphism. J Clin Endocrinol Metab. 2006;91(2):687-693. \
        https://doi.org/10.1210/jc.2005-1643"
    );
    println!("4. NCBI Gene, UGT2B17. https://www.ncbi.nlm.nih.gov/gene/7367");
}
