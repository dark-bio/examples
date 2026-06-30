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

use std::error::Error;
use std::fs;
use std::path::Path;

/// The SNP this app reads: rs72921001, near OR6A2 on chromosome 11.
const RSID: &str = "rs72921001";
/// Allele associated with tasting cilantro as soapy; the app counts copies of it.
const RISK_BASE: &str = "C";
/// dbSNP page for the SNP, linked from the report's further-reading section.
const NIH_SNP_URL: &str = "https://www.ncbi.nlm.nih.gov/snp/rs72921001";

fn main() -> Result<(), Box<dyn Error>> {
    // With no data directory, print the manifest the host reads to grant access.
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"cilantro-soapiness\"\n\
             version = \"0.3.0\"\n\
             datasets = [\"v1/genome/rsids/rs72921001\"]\n"
        );
        return Ok(());
    };

    // Everything comes from one lens directory: the user's genotype and the
    // variant's coordinate. The app reads plain files, never a genomic format.
    let base = Path::new(&dir).join("v1/genome/rsids").join(RSID);

    print_header();
    match read_call(&base) {
        Call::Covered(genotype) => {
            let copies = genotype.split(['/', '|']).filter(|a| *a == RISK_BASE).count();
            print_result(&genotype, copies);
            print_footer();
            print_technical_details(&base);
        }
        Call::Uncertain(genotype) => {
            print_uncertain(&genotype);
            print_footer();
        }
        Call::NotCovered => {
            print_not_covered();
            print_footer();
        }
    }

    Ok(())
}

/// What the lens can tell us about the user at this site.
enum Call {
    /// A confident genotype: both alleles are known bases, e.g. `A/C` or `C|C`.
    Covered(String),
    /// The site is covered but an allele is missing (`.`), so copies are unknown.
    Uncertain(String),
    /// No genotype here: the site wasn't sequenced, or the catalog lacks the rsID.
    NotCovered,
}

/// Reads and classifies the user's genotype from the lens. The `genotype` leaf
/// exists only where the user's calls cover the site, so a read error just means
/// the site is not covered.
fn read_call(base: &Path) -> Call {
    let Ok(genotype) = fs::read_to_string(base.join("genotype")) else {
        return Call::NotCovered;
    };
    let genotype = genotype.trim().to_string();

    // The lens returns alleles as bases (`A/C`, or `C|C` when phased), so there
    // is no REF/ALT index decoding to do; a `.` is an uncalled allele.
    if genotype.split(['/', '|']).any(|a| a == "." || a.is_empty()) {
        Call::Uncertain(genotype)
    } else {
        Call::Covered(genotype)
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
    println!("| `{genotype}` | **{copies}**×{RISK_BASE} |");
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
fn print_not_covered() {
    println!("### Variant not covered");
    println!();
    println!(
        "Your variant calls don't cover `{RSID}` (the site wasn't sequenced, or the \
         variant catalog doesn't carry it), so no result can be reported."
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
    let leaf = |name: &str| {
        fs::read_to_string(base.join(name))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    let (chrom, pos, reference) = (leaf("chromosome"), leaf("position"), leaf("reference"));

    println!("### Technical details");
    println!();
    println!("| Field | Value |");
    println!("| :--- | :--- |");
    println!("| **rsID** | `{RSID}` |");
    if !chrom.is_empty() && !pos.is_empty() {
        println!("| **Locus** | `{chrom}:{pos}` |");
    }
    if !reference.is_empty() {
        println!("| **Reference allele** | `{reference}` |");
    }
    println!();
}
