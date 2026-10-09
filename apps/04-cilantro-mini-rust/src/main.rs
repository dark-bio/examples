//! Cilantro taste, from one variant.
//!
//! Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
//! olfactory receptor, tracks the trait: the more copies of the C allele you
//! carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//!
//! This is the simplest real data access there is. The app asks for one
//! directory and the `rsids/` lens hands back the genotype and coordinate as
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
summary = "The smallest real data access, through the rsids lens, in four languages."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/04-cilantro-mini-rust"
keywords = ["tutorial", "rsids", "genotype", "four languages"]
description = '''
The rsids lens hands back one variant as plain files. Grant `v1/genome/rsids/rs72921001` and read `genotype`, `chromosome`, `position` and `reference` from that directory. The app never opens the call file.

Three outcomes every app has to handle. A missing allele, as in `./.`, makes a copy count inconclusive. An absent genotype means no answer, never two reference alleles. Any other read error stops the app.

The same program is written in Rust, Go, C and Python. Compare them, then see the full report version, Cilantro Taste Test.
'''

[listing.purposes]
"v1/genome/rsids/rs72921001" = "The one variant this tutorial reads"
"#
        );
        return;
    };

    // The report pass reads everything from the one lens directory
    let base = Path::new(&dir).join(LENS);

    // ENOENT means no answer, including reference sites in variants-only calls.
    let Some(genotype) = leaf(&base, "genotype") else {
        println!("## Cilantro taste\n");
        println!("No genotype answer for `{RSID}`. Absence does not imply two reference alleles.");
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

    println!("## Cilantro taste\n");
    println!("Your genotype at `{RSID}` is `{genotype}`.\n");
    if missing {
        println!("The copy count is inconclusive because an allele is missing (`.`).");
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
