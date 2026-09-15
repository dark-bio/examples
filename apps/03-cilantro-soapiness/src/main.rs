//! Cilantro Soapiness: the cilantro taste test as a polished Ark report.
//!
//! This is the full version of the cilantro example. The mini version
//! (../03-cilantro-mini-rust) shows the bare lens read in a handful of lines;
//! this one wraps the same single genotype in a complete Markdown report with a
//! verdict, a result table, caveats, references, and a technical-details section.
//! The data access is identical. Everything extra here is presentation, which is
//! what separates a demo from an app someone would actually want to run.
//!
//! Some people taste cilantro as soap. The trait tracks a SNP near the olfactory
//! receptor gene OR6A2: the more copies of the C allele at rs72921001 you carry,
//! the more soapy cilantro tends to taste (Eriksson et al., 2012).

use std::path::Path;
use std::{fs, io};

/// The SNP this app reads: rs72921001, near OR6A2 on chromosome 11.
const RSID: &str = "rs72921001";
/// Allele associated with tasting cilantro as soapy; the app counts copies of it.
const RISK_BASE: &str = "C";
/// dbSNP page for the SNP, linked from the report's further-reading section.
const NIH_SNP_URL: &str = "https://www.ncbi.nlm.nih.gov/snp/rs72921001";

fn main() {
    // With no data directory, print the manifest the host reads to grant access.
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"cilantro-soapiness\"\n\
             version = \"0.3.0\"\n\
             datasets = [\"v1/genome/rsids/rs72921001\"]\n"
        );
        return;
    };

    // Everything comes from one lens directory: the user's genotype and the
    // variant's coordinate. The app reads plain files, never a genomic format.
    let base = Path::new(&dir).join("v1/genome/rsids").join(RSID);

    print_header();
    match read_call(&base) {
        Call::Called(genotype) => {
            let copies = genotype
                .split(['/', '|'])
                .filter(|a| *a == RISK_BASE)
                .count();
            let ploidy = genotype
                .strip_prefix(['/', '|'])
                .unwrap_or(&genotype)
                .split(['/', '|'])
                .count();
            if ploidy == 2 {
                print_result(&genotype, copies);
            } else {
                println!("### Your result: copy count\n");
                println!(
                    "Genotype `{genotype}` has {copies} copies of `{RISK_BASE}` across {ploidy} alleles.\n"
                );
                println!("The taste comparison uses two-copy SNP calls.\n");
            }
            print_footer();
            print_technical_details(&base);
        }
        Call::Uncertain(genotype) => {
            print_uncertain(&genotype);
            print_footer();
        }
        Call::Unanswered => {
            print_unanswered();
            print_footer();
        }
    }
}

/// What the lens can tell us about the user at this site.
enum Call {
    Called(String),
    Uncertain(String),
    Unanswered,
}

fn read_call(base: &Path) -> Call {
    let Some(genotype) = leaf(base, "genotype") else {
        return Call::Unanswered;
    };
    // This known SNP needs only a simple split; the leading marker is not an allele.
    let alleles = genotype.strip_prefix(['/', '|']).unwrap_or(&genotype);
    if alleles.split(['/', '|']).any(|a| a == ".") {
        Call::Uncertain(genotype)
    } else {
        Call::Called(genotype)
    }
}

/// Only ENOENT means no answer; all other read errors stop the app.
fn leaf(base: &Path, name: &str) -> Option<String> {
    match fs::read_to_string(base.join(name)) {
        Ok(value) => Some(value),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(err) => {
            eprintln!("Could not read {RSID} {name}: {err}");
            std::process::exit(1);
        }
    }
}

/// How a given risk-allele copy count is presented in the report.
struct Verdict {
    emoji: &'static str,
    heading: &'static str,
    detail: &'static str,
}

/// Maps the risk-allele copy count (0, 1, or 2) to its report presentation.
fn interpret(copies: usize) -> Verdict {
    match copies {
        0 => Verdict {
            emoji: "🌿",
            heading: "Cilantro lover",
            detail: "You carry **zero** copies of the risk allele at this locus. \
                     Most people with this genotype perceive cilantro as herby or citrusy.",
        },
        1 => Verdict {
            emoji: "🫧",
            heading: "On the fence",
            detail: "You carry **one** copy of the risk allele (heterozygous). \
                     The effect is partial. You may notice a mild soapy note \
                     or none at all, depending on other genetic and environmental factors.",
        },
        _ => Verdict {
            emoji: "🧼",
            heading: "Soap detector",
            detail: "You carry **two** copies of the risk allele (homozygous). \
                     This is the genotype most strongly associated with perceiving \
                     cilantro as soapy or unpleasant in published GWAS data.",
        },
    }
}

/// Intro: what the test measures and the gene behind it.
fn print_header() {
    println!("## 🌿 Cilantro Taste Test");
    println!();
    println!(
        "Some people love cilantro. Others think it tastes like dish soap. \
         Blame *OR6A2*, an olfactory receptor gene on chromosome 11. \
         A 2012 GWAS (Eriksson *et al.*) pinpointed `{RSID}` as the SNP \
         behind it. More copies of the **{RISK_BASE}** allele, more soap."
    );
    println!();
}

/// The result for a confident genotype: the copy count and its interpretation.
fn print_result(genotype: &str, copies: usize) {
    let verdict = interpret(copies);
    println!("### Your result: {} {}", verdict.heading, verdict.emoji);
    println!();
    println!("| Genotype | Risk-allele copies |");
    println!("| :---: | :---: |");
    println!(
        "| `{}` | **{copies}**×{RISK_BASE} |",
        genotype.replace('|', "\\|")
    );
    println!();
    println!("{}", verdict.detail);
    println!();
}

/// The result when an allele is missing, so no copy count can be reported.
fn print_uncertain(genotype: &str) {
    println!("### Your result: inconclusive ⚠️");
    println!();
    println!(
        "Your genotype at `{RSID}` is `{genotype}` - at least one allele is \
         missing, so the risk-allele count can't be determined."
    );
    println!();
}

/// The result when the lens has no genotype at this site.
fn print_unanswered() {
    println!("### No genotype answer");
    println!();
    println!(
        "No genotype answer is available for `{RSID}`. Reference sites in a \
         variants-only file can be absent, as can missing calls or ambiguous \
         records or placements. Absence does not imply two reference alleles."
    );
    println!();
}

/// Caveats and references, shown after every result.
fn print_footer() {
    println!("### Fine print");
    println!();
    println!(
        "One SNP doesn't tell the whole story. Taste is polygenic and shaped by \
         diet, culture, and exposure. Two copies doesn't mean you hate cilantro, \
         zero copies doesn't mean you love it."
    );
    println!();
    println!("### Further reading");
    println!();
    println!(
        "- Eriksson N *et al.* (2012). \"A genetic variant near olfactory receptor genes influences cilantro preference.\" *Flavour* 1:22."
    );
    println!(
        "- NCBI dbSNP [*{RSID}*]({NIH_SNP_URL}): population frequencies, genomic context, and submission history."
    );
    println!(
        "- NCBI Gene [*OR6A2*](https://www.ncbi.nlm.nih.gov/gene/8590): olfactory receptor family 6 subfamily A member 2."
    );
    println!();
}

/// The resolved locus and reference allele, read from the lens's scalar leaves.
/// The lens exposes chromosome and position separately; the app composes the
/// `chr:pos` form itself.
fn print_technical_details(base: &Path) {
    let (chrom, pos, reference) = (
        leaf(base, "chromosome"),
        leaf(base, "position"),
        leaf(base, "reference"),
    );

    println!("### Technical details");
    println!();
    println!("| Field | Value |");
    println!("| :--- | :--- |");
    println!("| **rsID** | `{RSID}` |");
    if let (Some(chrom), Some(pos)) = (chrom, pos) {
        println!("| **Locus** | `{chrom}:{pos}` |");
    }
    if let Some(reference) = reference {
        println!("| **Reference allele** | `{reference}` |");
    }
    println!();
}
