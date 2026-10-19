//! Sprint or Marathon: a report on the ACE insertion.
//!
//! ACE comes with or without a 287-base Alu repeat in intron 16, and sport
//! studies found the version with it more often in endurance athletes. The app
//! looks for the insertion in a short window of the gene, in the report shape
//! docs/06-reports.md describes.
//!
//! The app asks the Ark for the window,
//! `v1/genome/regions/chr17/63488479-63488579`, and for two public grants.
//! `v1/genome/datasets` tells a D/D genome from one without a structural
//! variant file, and `v1/genome/files/reference-genome` names the assembly
//! build.

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
    /// The `type` leaf, such as SNV, INS or DEL.
    kind: Option<String>,
    /// How many bases replace the span.
    new_bases: Option<u64>,
    /// The variant file the change came from.
    dataset: Option<String>,
    /// How many copies carry the change, as written.
    carried: Option<String>,
    /// How many copies have no call, absent when every copy is called.
    uncalled: Option<String>,
    /// Filters the change failed, one per line, absent when it passed.
    failed: Option<String>,
}

/// Everything the report rests on, read once from the granted paths.
struct Report {
    /// Changes in the window, sorted by span.
    changes: Vec<Change>,
    /// Whether the Ark holds a structural variant file.
    uploaded: bool,
    /// Assembly build, absent when it couldn't be read.
    build: Option<String>,
}

/// What the window says about the owner's ACE genotype.
enum Outcome {
    /// The insertion on both copies.
    Ii,
    /// The insertion on one copy.
    Id,
    /// No insertion, with a structural variant file to vouch for it.
    Dd,
    /// An insertion without a settled copy count.
    Uncertain,
    /// No insertion, and no structural variant file to look in.
    NoUpload,
}

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Sprint or Marathon"
version = "0.1.0"

[reads]
paths = ["v1/genome/regions/chr17/63488479-63488579", "v1/genome/datasets", "v1/genome/files/reference-genome"]

[listing]
language = "en"
icon = "🏃"
summary = "The ACE insertion that turns up more often in endurance athletes."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/08-sprint-or-marathon"
keywords = ["sport", "endurance", "ACE", "structural variant", "Alu"]
description = '''
_ACE_ comes in two common versions, one carrying a 287-base Alu repeat, called I, and one without it, called D. Sport studies found the I version more often in endurance athletes.

This app looks for the insertion in your structural variant file. It is too long for a small variant caller, so without a structural variant upload the app has no answer, and says so.

The grant covers 101 bases around the insertion, including any small variants of yours in them.
'''

[listing.purposes]
"v1/genome/regions/chr17/63488479-63488579" = "101 bases of the ACE gene, around its Alu insertion"
"v1/genome/datasets" = "Which files your Ark holds, to tell a missing upload from a missing insertion"
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

/// Counts the new bases from the size of `bases`, or from `length` for a symbolic
/// insertion. A long allele is measured, never read whole.
fn new_bases(base: &Path) -> Result<Option<u64>, String> {
    match fs::metadata(base.join("bases")) {
        Ok(info) => Ok(Some(info.len())),
        Err(err) if err.kind() == io::ErrorKind::NotFound => leaf(base, "length")?
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|err| format!("Invalid length: {err}"))
            })
            .transpose(),
        Err(err) => Err(format!("Could not read bases size: {err}")),
    }
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
            kind: leaf(&entry.path(), "type")?,
            new_bases: new_bases(&entry.path())?,
            dataset: leaf(&entry.path(), "dataset")?,
            carried: leaf(&entry.path(), "carried")?,
            uncalled: leaf(&entry.path(), "uncalled")?,
            failed: leaf(&entry.path(), "failed")?,
        });
    }
    result.sort_by_key(|change| change.span);
    Ok(result)
}

impl Report {
    /// Reads the window's changes, the uploads and the build.
    fn load(root: &Path) -> Result<Self, String> {
        let changes = changes(&root.join(LENS))?;
        let uploaded = root
            .join("v1/genome/datasets/structural-variants")
            .try_exists()
            .map_err(|err| format!("Could not read dataset availability: {err}"))?;
        let build = leaf(&root.join("v1/genome/files/reference-genome"), "build")?;
        Ok(Self {
            changes,
            uploaded,
            build,
        })
    }

    /// Picks the insertion carried on the most copies, the first on a tie. Only an
    /// insertion of 200 bases or more counts.
    fn insertion(&self) -> Option<&Change> {
        let mut chosen = None;
        let mut most = 0;
        for change in &self.changes {
            let insertion = change
                .kind
                .as_deref()
                .is_some_and(|kind| kind == "INS" || kind.starts_with("INS:"));
            if !insertion || !change.new_bases.is_some_and(|length| length >= 200) {
                continue;
            }
            let carried = change
                .carried
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            if chosen.is_none() || carried > most {
                chosen = Some(change);
                most = carried;
            }
        }
        chosen
    }

    /// Decides the outcome from the chosen insertion, or from the uploads without one.
    fn outcome(&self, chosen: Option<&Change>) -> Outcome {
        match chosen {
            Some(change) if change.uncalled.is_some() => Outcome::Uncertain,
            Some(change) => match change
                .carried
                .as_deref()
                .and_then(|value| value.parse::<u64>().ok())
            {
                Some(2) => Outcome::Ii,
                Some(1) => Outcome::Id,
                _ => Outcome::Uncertain,
            },
            None if self.uploaded => Outcome::Dd,
            None => Outcome::NoUpload,
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
        let chosen = self.insertion();
        let outcome = self.outcome(chosen);
        println!("# 🏃 Sprint or Marathon\n");
        print!("{}", finding(&outcome));

        // Only a genotype can be flagged, since the no-answer outcomes have none
        if matches!(outcome, Outcome::Ii | Outcome::Id)
            && let Some(failed) = chosen.and_then(|change| change.failed.as_deref())
        {
            print!(
                " Your structural variant caller flagged it as `{}`, so read it with care.",
                failed.lines().collect::<Vec<_>>().join(", ")
            );
        }
        println!("\n\n## Evidence\n");
        if let Some(change) = chosen {
            println!("| Change | Type | New bases | File | Copies |");
            println!("| :-- | :-- | :-- | :-- | :-- |");
            let bases = change
                .new_bases
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
            println!();
        }
        print!("The app read {}, inside *ACE*.", self.location(START, END));
        let others = self.changes.len() - usize::from(chosen.is_some());
        if others > 0 {
            print!(
                " The window holds {others} other {}, which the app skips.",
                if others == 1 { "change" } else { "changes" }
            );
        }
        println!("\n");
        print_method();
        print_limitations();
        print_sources();
    }
}

/// The finding paragraph for an outcome, before any flag sentence.
fn finding(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Ii => {
            "**Marathon** 🏃. You carry the ACE insertion on both copies, the I/I \
            genotype. Sport studies found I/I more often in endurance athletes, with 1.35 \
            times the odds of D carriers in a pooled analysis."
        }
        Outcome::Id => {
            "**Middle distance** 🏃. You carry the ACE insertion on one copy, the \
            I/D genotype and the most common one. Sport studies tie endurance to I/I, so one \
            copy puts you between the two ends of the story."
        }
        Outcome::Dd => {
            "**Sprint** ⚡. You carry no ACE insertion, the D/D genotype. Sport \
            studies tie endurance to the insertion, so the story points you at the short, \
            fast end instead."
        }
        Outcome::Uncertain => {
            "**No answer.** Your files show the ACE insertion, but not on how \
            many copies, so this app can't name your genotype."
        }
        Outcome::NoUpload => {
            "**No answer.** Your Ark holds no structural variant file, so \
            this app can't tell D/D from a missing call. The insertion runs to nearly 300 \
            bases, too long for a small variant caller, so only a structural variant upload \
            answers this."
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
        101 bases around the insertion's anchor, chr17:63488529, and counts an insertion \
        of 200 bases or more as I. With no such insertion and a structural variant file \
        present, it reads D/D.\n"
    );
}

fn print_limitations() {
    println!("## Limitations\n");
    println!(
        "One variant with a modest effect, and the studies Ma et al. (2013) pooled \
        often disagreed. Training and many other genes shape endurance far more. The app \
        counts any insertion of 200 bases or more in the window as the Alu, without \
        checking its bases. A caller that misses the insertion reads as D/D, and one \
        that places it outside the window isn't seen.\n"
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
