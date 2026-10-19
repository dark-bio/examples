//! Cilantro taste, from one variant.
//!
//! Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
//! olfactory receptor, tracks the trait: the more copies of the C allele you
//! carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//!
//! This is the simplest real data access there is. The app asks for one
//! directory and the `rsids/` folder hands back the genotype and coordinate as
//! plain files. It never opens a variant file. See ../../docs/04-reading-data.md.

use std::path::Path;
use std::{fs, io};

/// The variant this app reads, and the allele it counts.
const RSID: &str = "rs72921001";
const SOAPY_ALLELE: &str = "C";
/// The one directory the app asks for, relative to the data root.
const LENS: &str = "v1/genome/rsids/rs72921001";

fn main() {
    // The manifest pass names the app and asks for the one variant directory
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "One Variant"
version = "0.1.0"

[reads]
paths = ["{LENS}"]

[listing]
language = "en"
icon = "🌱"
summary = "One variant, four languages, zero call file. The smallest real read."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/04-cilantro-mini-rust"
keywords = ["tutorial", "rsids", "genotype", "four languages"]
description = '''
The `rsids/` folder hands you one variant as plain files. Grant `v1/genome/rsids/rs72921001`, read `genotype`, `chromosome`, `position` and `reference`, and you're done. The call file never comes near it.

Every app handles three outcomes. A missing allele, as in `./.`, makes the count inconclusive. A missing genotype is no answer, never two reference alleles. Any other read error stops the app.

Written in Rust, Go, C and Python. Compare them, then meet the full report, Cilantro Taste Test.
'''

[listing.purposes]
"v1/genome/rsids/rs72921001" = "The one variant this tutorial reads"
"#
        );
        return;
    };

    // The report pass reads everything from the one variant directory
    let base = Path::new(&dir).join(LENS);

    // ENOENT means no answer, including reference sites in variants-only calls.
    let Some(genotype) = leaf(&base, "genotype") else {
        println!("## Cilantro or soap?\n");
        println!("No genotype at `{RSID}`. Silence isn't two reference alleles.");
        return;
    };

    // This known SNP needs only a simple split; keep the original text for display.
    let alleles: Vec<_> = genotype
        .strip_prefix(['/', '|'])
        .unwrap_or(&genotype)
        .split(['/', '|'])
        .collect();
    let copies = alleles.iter().filter(|a| **a == SOAPY_ALLELE).count();
    let missing = alleles.contains(&".");

    println!("## Cilantro or soap?\n");
    println!("Your genotype at `{RSID}` is `{genotype}`.\n");
    if missing {
        println!("An allele is missing (`.`), so the soap count is inconclusive.");
    } else {
        match copies {
            0 => println!(
                "You carry no copies of the soapy `{SOAPY_ALLELE}` allele. Cilantro probably tastes fresh and herby."
            ),
            1 => println!(
                "You carry one copy of the soapy `{SOAPY_ALLELE}` allele. Cilantro may have a faint soapy note."
            ),
            n => println!(
                "You carry {n} copies of the soapy `{SOAPY_ALLELE}` allele. Cilantro likely tastes like dish soap."
            ),
        }
    }

    let (chrom, pos, reference) = (
        leaf(&base, "chromosome"),
        leaf(&base, "position"),
        leaf(&base, "reference"),
    );
    if let (Some(chrom), Some(pos), Some(reference)) = (chrom, pos, reference) {
        println!("\nLocus `{chrom}:{pos}`, reference allele `{reference}`.");
    }
}

/// Generated files hold exactly their value; only ENOENT means no answer.
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
