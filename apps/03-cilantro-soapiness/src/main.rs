//! Cilantro Soapiness: the cilantro taste test as a full Ark report.
//!
//! This is the full version of the cilantro example. The mini version
//! (../03-cilantro-mini-rust) shows the bare lens read in a handful of lines;
//! this one wraps the same single genotype in a complete report in the shape
//! docs/06-reports.md describes: a finding, the evidence behind it, the method,
//! its limitations and sources. The data access is identical.
//! Everything extra here is presentation, which is what separates a demo from
//! an app someone would actually want to run.
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

fn main() {
    // With no data directory, print the manifest the Ark reads to grant access.
    // The reference grant is there for one file, the assembly build the
    // coordinate is reported on.
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"cilantro-soapiness\"\n\
             version = \"0.4.0\"\n\
             datasets = [\"v1/genome/rsids/rs72921001\", \"v1/genome/reference\"]\n"
        );
        return;
    };

    // Everything comes from one lens directory: the user's genotype and the
    // variant's coordinate. The app reads plain files, never a genomic format.
    let genome = Path::new(&dir).join("v1/genome");
    let base = genome.join("rsids").join(RSID);
    let build = leaf(&genome.join("reference"), "build");

    println!("# 🌿 Cilantro Taste Test");
    println!();

    let reference = leaf(&base, "reference");
    let call = read_call(&base);
    match &call {
        Call::Called(genotype) => {
            let alleles: Vec<&str> = genotype
                .strip_prefix(['/', '|'])
                .unwrap_or(genotype)
                .split(['/', '|'])
                .collect();
            let copies = alleles.iter().filter(|a| **a == RISK_BASE).count();
            if alleles.len() == 2 {
                print!("{}", finding(copies));
                if reference.as_deref() == Some(RISK_BASE) && copies == 2 {
                    print!(
                        " C is also the reference base here, so a call file that lists only \
                         variants would have stayed quiet about it. Yours didn't."
                    );
                }
                println!();
            } else {
                println!(
                    "**No verdict.** Genotype `{genotype}` holds {copies} copies of C across \
                     {} alleles, and the taste comparison is built for two-allele calls.",
                    alleles.len()
                );
            }
        }
        Call::Uncertain(genotype) => {
            println!(
                "**Inconclusive** ⚠️. Your genotype at {RSID} is `{genotype}`, with at least \
                 one allele missing, so the copies of C can't be counted."
            );
        }
        Call::Unanswered => {
            println!(
                "**No answer.** No genotype is available at {RSID}, so this app has no \
                 result. An absent genotype is not a reference call. A call file that \
                 records only variants has no record at a reference site, and none at a \
                 site it didn't cover, and the two can't be told apart here."
            );
        }
    }
    println!();

    print_evidence(&base, &call, reference.as_deref(), build.as_deref());
    print_method();
    print_limitations();
    print_sources();
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
            eprintln!("Could not read {}: {err}", base.join(name).display());
            std::process::exit(1);
        }
    }
}

/// The finding for a two-allele call, by the number of C copies it holds. The
/// verdict leads, and the count it rests on follows in the same breath.
fn finding(copies: usize) -> &'static str {
    match copies {
        0 => {
            "**Cilantro lover** 🌿. You carry no copy of C at rs72921001, the genotype with \
             no soapy association. Most people built this way taste cilantro as herby or \
             citrusy."
        }
        1 => {
            "**On the fence** 🫧. You carry one copy of C at rs72921001, so the soapy \
             association is partial. You may notice a mild soapy note, or none at all, \
             depending on the rest of your genome and what you grew up eating."
        }
        _ => {
            "**Soap detector** 🧼. You carry two copies of C at rs72921001, the genotype most \
             strongly tied to tasting cilantro as dish soap. Blame *OR6A2*, the olfactory \
             receptor next door."
        }
    }
}

/// The values the finding rests on: the variant, its coordinate on the assembly
/// this Ark holds, and the genotype exactly as the call file records it. The
/// lens exposes chromosome and position separately; the app composes the
/// `chr:pos` form itself, and labels it only when the build could be read.
fn print_evidence(base: &Path, call: &Call, reference: Option<&str>, build: Option<&str>) {
    println!("## Evidence");
    println!();
    println!("| Variant | Nearest gene | Position |");
    println!("| :-- | :-- | :-- |");
    let position = match (leaf(base, "chromosome"), leaf(base, "position"), build) {
        (Some(chrom), Some(pos), Some(build)) => format!("{chrom}:{pos}, {build}"),
        (Some(chrom), Some(pos), None) => format!("{chrom}:{pos}"),
        _ => "unknown".to_string(),
    };
    println!("| {RSID} | *OR6A2* | {position} |");
    println!();
    let recorded = match call {
        Call::Called(genotype) | Call::Uncertain(genotype) => {
            format!("Your call file says `{genotype}` here")
        }
        Call::Unanswered => "Your call file has no record here".to_string(),
    };
    match reference {
        Some(reference) => println!("{recorded}, and the reference base is {reference}."),
        None => println!("{recorded}."),
    }
    println!();
}

/// How the finding follows from the evidence, and where the association comes from.
fn print_method() {
    println!("## Method");
    println!();
    println!(
        "Some people love cilantro. Others think it tastes like dish soap. Eriksson et al. \
         (2012) went looking for why, in a genome-wide study of self-reported preference \
         among European-ancestry participants, and {RSID} is what they found. The app \
         counts your copies of C. Two is the strongest association, one is somewhere in \
         between, and zero is none. More copies, more soap."
    );
    println!();
}

/// What the app didn't read and what the finding doesn't establish.
fn print_limitations() {
    println!("## Limitations");
    println!();
    println!(
        "One variant, one modest effect. The study measured what people said they \
         preferred, not what they tasted, and taste is polygenic and shaped by diet, \
         culture and exposure besides. Two copies doesn't mean you hate cilantro, and \
         zero doesn't mean you love it. The study was mostly people of European ancestry, \
         so elsewhere the effect is less well known, and this app doesn't know how common \
         your genotype is."
    );
    println!();
}

/// The studies Method names, in that order, with a stable address for each.
fn print_sources() {
    println!("## Sources");
    println!();
    println!(
        "1. Eriksson N, et al. A genetic variant near olfactory receptor genes influences \
         cilantro preference. Flavour. 2012;1:22. https://doi.org/10.1186/2044-7248-1-22"
    );
    println!("2. NCBI dbSNP, {RSID}. https://www.ncbi.nlm.nih.gov/snp/{RSID}");
}
