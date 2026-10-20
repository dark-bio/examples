//! Sprint or Marathon: a report on the ACE insertion.
//!
//! ACE comes with or without a 287-base Alu repeat in intron 16, and sport
//! studies found the version with it more often in endurance athletes. The app
//! looks for the insertion in a short window of the gene, in the report shape
//! docs/06-reports.md describes.
//!
//! The app asks the Ark for the window,
//! `v1/genome/regions/chr17/63488479-63488579`, and for two public grants.
//! `v1/genome/datasets/structural-variants` requires the structural variant
//! file, and `v1/genome/files/reference-genome` names the assembly build.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

/// The window the app asks for, relative to the data root.
const LENS: &str = "v1/genome/regions/chr17/63488479-63488579";
/// First base of the window.
const START: u64 = 63_488_479;
/// Last base of the window.
const END: u64 = 63_488_579;

/// One change entry in the window, with the leaves the report reads.
struct Change {
    /// First base, last base and place among the entries sharing the span.
    span: (u64, u64, u64),
    /// Possible first bases, absent without a confidence range.
    start_range: Option<(u64, u64)>,
    /// Possible last bases, absent without a confidence range.
    end_range: Option<(u64, u64)>,
    /// The `type` leaf, such as SNV, INS or DEL.
    kind: Option<String>,
    /// How many bases replace the span.
    new_bases: Option<u64>,
    /// Length supplied for a symbolic insertion.
    length: Option<u64>,
    /// The variant file the change came from.
    dataset: Option<String>,
    /// How many copies carry the change, as written.
    carried: Option<String>,
    /// How many copies have no call, absent when every copy is called.
    uncalled: Option<String>,
    /// Phased copies carrying the allele, one per line.
    copies: Option<String>,
    /// Phase set, absent for the chromosome's default set.
    phase_set: Option<String>,
    /// Filters the change failed, one per line, absent when it passed.
    failed: Option<String>,
}

impl Change {
    /// Recognizes the Alu by its spelled-out size or its caller's label.
    fn is_alu(&self) -> bool {
        match self.kind.as_deref() {
            Some("INS") => self
                .new_bases
                .is_some_and(|length| (200..=400).contains(&length)),
            Some("INS:ME:ALU") => true,
            _ => false,
        }
    }

    /// Leaves the call open for uncertain insertion identity or placement.
    fn leaves_open(&self) -> bool {
        if self.is_alu() {
            let start = self.start_range.unwrap_or((self.span.0, self.span.0));
            let end = self.end_range.unwrap_or((self.span.1, self.span.1));
            return [self.span.0, self.span.1, start.0, start.1, end.0, end.1]
                .into_iter()
                .any(|position| !(START..=END).contains(&position));
        }
        self.kind
            .as_deref()
            .is_some_and(|kind| kind == "INS" || kind.starts_with("INS:"))
            && self
                .new_bases
                .or(self.length)
                .is_none_or(|length| length >= 50)
    }
}

/// Everything the report rests on, read once from the granted paths.
struct Report {
    /// Changes in the window, sorted by span.
    changes: Vec<Change>,
    /// Assembly build, absent when it couldn't be read.
    build: Option<String>,
}

/// What the window says about the owner's ACE genotype.
enum Outcome {
    /// The window's coordinates do not match the assembly.
    WrongBuild,
    /// The insertion on both copies.
    Ii,
    /// The insertion on one copy.
    Id,
    /// No insertion listed in the window.
    Dd,
    /// An insertion that isn't clearly the Alu, or one without a settled copy count.
    Uncertain,
}

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Sprint or Marathon"
version = "0.1.0"

[reads]
paths = ["v1/genome/regions/chr17/63488479-63488579", "v1/genome/datasets/structural-variants", "v1/genome/files/reference-genome"]

[listing]
language = "en"
icon = "🏃"
summary = "Built for the sprint or the marathon? One Alu insertion in ACE has a say."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/08-sprint-or-marathon"
keywords = ["sport", "endurance", "ACE", "structural variant", "Alu"]
description = '''
_ACE_ comes in two common versions, one carrying a 287-base Alu repeat, called I, and one without it, called D. Sport studies found the I version more often in endurance athletes.

This app looks for the insertion in your structural variant file. It's usually too long for a small variant caller, so the app needs your structural variant file, and the Ark won't run it without one.

The grant covers 101 bases around the insertion, including any small variants of yours in them.
'''

[listing.purposes]
"v1/genome/regions/chr17/63488479-63488579" = "101 bases of the ACE gene, around its Alu insertion"
"v1/genome/datasets/structural-variants" = "Your structural variant file has to be on the Ark, though the app never reads it"
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

/// Splits an entry name such as 100-100 or 100-100-2 into start, end and suffix.
fn span(name: &str) -> Result<(u64, u64, u64), String> {
    let (start, rest) = name.split_once('-').ok_or("Invalid change name")?;
    let (end, suffix) = rest.split_once('-').unwrap_or((rest, "1"));
    Ok((
        start
            .parse::<u64>()
            .map_err(|err| format!("Invalid change start: {err}"))?,
        end.parse::<u64>()
            .map_err(|err| format!("Invalid change end: {err}"))?,
        suffix
            .parse::<u64>()
            .map_err(|err| format!("Invalid change suffix: {err}"))?,
    ))
}

/// Measures a spelled-out allele without reading it whole.
fn new_bases(base: &Path) -> Result<Option<u64>, String> {
    match fs::metadata(base.join("bases")) {
        Ok(info) => Ok(Some(info.len())),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("Could not read bases size: {err}")),
    }
}

/// Reads both coordinates of an optional confidence range.
fn range(base: &Path, name: &str) -> Result<Option<(u64, u64)>, String> {
    leaf(base, name)?
        .map(|value| {
            let (start, end) = value
                .split_once('-')
                .ok_or_else(|| format!("Invalid {name}"))?;
            Ok((
                start
                    .parse::<u64>()
                    .map_err(|err| format!("Invalid {name} start: {err}"))?,
                end.parse::<u64>()
                    .map_err(|err| format!("Invalid {name} end: {err}"))?,
            ))
        })
        .transpose()
}

/// Lists the changes in the window, sorted by span.
fn changes(base: &Path) -> Result<Vec<Change>, String> {
    let entries = match fs::read_dir(base.join("changes")) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(format!("Could not list changes: {err}")),
    };
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("Could not read change entry: {err}"))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "Invalid change name")?;
        result.push(Change {
            span: span(&name)?,
            start_range: range(&entry.path(), "start-range")?,
            end_range: range(&entry.path(), "end-range")?,
            kind: leaf(&entry.path(), "type")?,
            new_bases: new_bases(&entry.path())?,
            length: leaf(&entry.path(), "length")?
                .map(|value| {
                    value
                        .parse::<u64>()
                        .map_err(|err| format!("Invalid length: {err}"))
                })
                .transpose()?,
            dataset: leaf(&entry.path(), "dataset")?,
            carried: leaf(&entry.path(), "carried")?,
            uncalled: leaf(&entry.path(), "uncalled")?,
            copies: leaf(&entry.path(), "copy")?,
            phase_set: leaf(&entry.path(), "phase-set")?,
            failed: leaf(&entry.path(), "failed")?,
        });
    }
    result.sort_by_key(|change| change.span);
    Ok(result)
}

impl Report {
    /// Reads the build before opening a window in GRCh38 coordinates.
    fn load(root: &Path) -> Result<Self, String> {
        let build = leaf(&root.join("v1/genome/files/reference-genome"), "build")?;
        let changes = if build
            .as_deref()
            .is_some_and(|build| build.starts_with("GRCh38"))
        {
            changes(&root.join(LENS))?
        } else {
            Vec::new()
        };
        Ok(Self { changes, build })
    }

    /// Counts Alu copies by dataset, leaving other insertions unresolved.
    fn outcome(&self, insertions: &[&Change]) -> Outcome {
        if !self
            .build
            .as_deref()
            .is_some_and(|build| build.starts_with("GRCh38"))
        {
            return Outcome::WrongBuild;
        }
        if self.changes.iter().any(Change::leaves_open) {
            return Outcome::Uncertain;
        }
        if insertions.is_empty() {
            return Outcome::Dd;
        }

        // Prefer the structural total without counting the same call twice
        let mut structural = 0u64;
        let mut small = 0u64;
        for change in insertions {
            if change.uncalled.is_some() {
                return Outcome::Uncertain;
            }
            let Some(carried) = change
                .carried
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
            else {
                return Outcome::Uncertain;
            };
            let total = match change.dataset.as_deref() {
                Some("structural-variants") => &mut structural,
                Some("small-variants") => &mut small,
                _ => return Outcome::Uncertain,
            };
            let Some(sum) = total.checked_add(carried) else {
                return Outcome::Uncertain;
            };
            *total = sum;
        }

        // Several entries add up only when their phased copies are distinct
        let dataset = if structural == 0 {
            "small-variants"
        } else {
            "structural-variants"
        };
        let chosen = insertions
            .iter()
            .filter(|change| change.dataset.as_deref() == Some(dataset))
            .collect::<Vec<_>>();
        if chosen.len() > 1 {
            let phase_set = chosen[0].phase_set.as_deref();
            let mut seen = BTreeSet::new();
            for change in chosen {
                let Some(copies) = change.copies.as_deref() else {
                    return Outcome::Uncertain;
                };
                if copies.is_empty() || change.phase_set.as_deref() != phase_set {
                    return Outcome::Uncertain;
                }
                for copy in copies.lines() {
                    let Ok(copy) = copy.parse::<u64>() else {
                        return Outcome::Uncertain;
                    };
                    if !seen.insert(copy) {
                        return Outcome::Uncertain;
                    }
                }
            }
        }
        match if structural == 0 { small } else { structural } {
            2 => Outcome::Ii,
            1 => Outcome::Id,
            _ => Outcome::Uncertain,
        }
    }

    /// Formats a position or span on chr17, labeled with the build when it was read.
    fn location(&self, start: u64, end: u64) -> String {
        let position = if start == end {
            format!("chr17:{start}")
        } else {
            format!("chr17:{start}-{end}")
        };
        match &self.build {
            Some(build) => format!("{position}, {build}"),
            None => position,
        }
    }

    /// Prints the report, from the finding down to the sources.
    fn print(&self) {
        let insertions = self
            .changes
            .iter()
            .filter(|change| change.is_alu())
            .collect::<Vec<_>>();
        let outcome = self.outcome(&insertions);
        println!("# 🏃 Sprint or Marathon\n");
        print!("{}", finding(&outcome, self.build.as_deref()));

        // Only a genotype can be flagged, since the no-answer outcomes have none
        if matches!(outcome, Outcome::Ii | Outcome::Id) {
            let failed = insertions
                .iter()
                .filter_map(|change| change.failed.as_deref())
                .flat_map(str::lines)
                .collect::<BTreeSet<_>>();
            if !failed.is_empty() {
                print!(
                    " Your caller flagged it as `{}`, so read it with care.",
                    failed.into_iter().collect::<Vec<_>>().join(", ")
                );
            }
        }
        println!("\n");
        if !matches!(outcome, Outcome::WrongBuild) {
            self.print_evidence();
        }
        print_method();
        print_limitations();
        print_sources();
    }

    /// Lists the entries that decide the outcome and the window that was read.
    fn print_evidence(&self) {
        let entries = self
            .changes
            .iter()
            .filter(|change| change.is_alu() || change.leaves_open())
            .collect::<Vec<_>>();
        println!("## Evidence\n");
        if !entries.is_empty() {
            println!("| Change | Type | New bases | File | Copies |");
            println!("| :-- | :-- | :-- | :-- | :-- |");
            for change in &entries {
                let bases = change
                    .new_bases
                    .or(change.length)
                    .map(|length| length.to_string())
                    .unwrap_or("no answer".into());
                let mut copies = change.carried.as_deref().unwrap_or("no answer").to_owned();
                if let Some(uncalled) = &change.uncalled {
                    copies.push_str(&format!(", {uncalled} uncalled"));
                }
                println!(
                    "| {} | {} | {bases} | {} | {copies} |",
                    self.location(change.span.0, change.span.1),
                    change.kind.as_deref().unwrap_or("no answer"),
                    change.dataset.as_deref().unwrap_or("no answer")
                );
            }
            println!();
        }
        print!("The app read {}, inside *ACE*.", self.location(START, END));
        let others = self.changes.len() - entries.len();
        if others > 0 {
            print!(
                " The window holds {others} other {}, which the app skips.",
                if others == 1 { "change" } else { "changes" }
            );
        }
        println!("\n");
    }
}

/// The finding paragraph for an outcome, before any flag sentence.
fn finding(outcome: &Outcome, build: Option<&str>) -> String {
    match outcome {
        Outcome::WrongBuild => format!(
            "**No answer.** This app reads GRCh38 coordinates, and your Ark holds {}, \
            so its window would land in the wrong place.",
            build.unwrap_or("an unnamed build")
        ),
        Outcome::Ii => {
            "**Endurance edition** 🏃. You carry the ACE insertion on both copies, the I/I \
            genotype. Sport studies found I/I more often in endurance athletes, with 1.35 \
            times the odds of D carriers in a pooled analysis."
                .into()
        }
        Outcome::Id => {
            "**One of each** 🔀. You carry the ACE insertion on one copy, the I/D genotype \
            and the most common one. The endurance association in sport studies belongs \
            to I/I, so this genotype sits in the comparison group."
                .into()
        }
        Outcome::Dd => {
            "**No insertion found** ⚡. Your structural variant file lists no ACE insertion. \
            That usually means D/D, but a caller can miss an insertion, so this is a strong \
            hint, not a call."
                .into()
        }
        Outcome::Uncertain => {
            "**No answer.** Your files show an insertion near the ACE site, but not clearly \
            the Alu or not on how many copies, so this app can't name your genotype."
                .into()
        }
    }
}

fn print_method() {
    println!("## Method\n");
    println!(
        "*ACE* comes in two common versions, with or without a 287-base Alu repeat in \
        intron 16, called I and D (Rieder et al., 1999). Montgomery et al. (1998) found I \
        more often among elite British mountaineers, and found that army recruits with \
        two copies of I gained more endurance from training than those with none. A \
        meta-analysis of sport studies tied I/I to endurance events, with 1.35 times the \
        odds of D carriers (Ma et al., 2013). The app lists the changes your files hold in \
        101 bases around the insertion's anchor, chr17:63488529, and counts a spelled-out \
        insertion of 200 to 400 bases, or one its caller labels an Alu, as I. Any other \
        insertion there of 50 bases or more leaves the call open. With no insertion at \
        all, it reports the absence without calling D/D, since a caller can miss one.\n"
    );
}

fn print_limitations() {
    println!("## Limitations\n");
    println!(
        "One variant with a modest effect, and the studies Ma et al. (2013) pooled \
        often disagreed. Training and many other genes shape endurance far more. The app \
        recognizes the Alu by its length or its caller's label, without checking its bases, \
        and reads GRCh38 only. A caller that places the insertion outside the window isn't \
        seen, and one whose confidence range only reaches into it leaves the call open.\n"
    );
}

fn print_sources() {
    println!("## Sources\n");
    println!(
        "1. Rieder MJ, et al. Sequence variation in the human angiotensin converting \
        enzyme. Nat Genet. 1999;22(1):59-62. https://doi.org/10.1038/8760"
    );
    println!(
        "2. Montgomery HE, et al. Human gene for physical performance. Nature. \
        1998;393(6682):221-222. https://doi.org/10.1038/30374"
    );
    println!(
        "3. Ma F, et al. The association of sport performance with ACE and ACTN3 \
        genetic polymorphisms: a systematic review and meta-analysis. PLoS One. \
        2013;8(1):e54685. https://doi.org/10.1371/journal.pone.0054685"
    );
    println!("4. NCBI dbSNP, rs1799752. https://www.ncbi.nlm.nih.gov/snp/rs1799752");
}
