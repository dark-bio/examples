//! Cilantro taste, from one variant.
//!
//! Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
//! olfactory receptor, tracks the trait: the more copies of the C allele you
//! carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//!
//! This is the simplest real data access there is. The app asks for one
//! directory and the `rsids/` lens hands back the genotype and coordinate as
//! plain files. It never opens a variant file. See ../../docs/04-data-access.md.

use std::fs;
use std::path::Path;

/// The variant this app reads, and the allele it counts.
const RSID: &str = "rs72921001";
const SOAPY_ALLELE: &str = "C";
/// The one directory the app asks for, relative to the data root.
const LENS: &str = "v1/genome/rsids/rs72921001";

fn main() {
    // The manifest pass: name the app and ask for the one variant directory.
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            "[package]\n\
             name = \"cilantro-mini\"\n\
             version = \"0.1.0\"\n\
             datasets = [\"{LENS}\"]\n"
        );
        return;
    };

    // The run pass: everything comes from the one lens directory.
    let base = Path::new(&dir).join(LENS);

    // The genotype leaf exists only where your calls cover the site. A missing
    // file just means the site was not sequenced, so report that and stop.
    let Ok(genotype) = fs::read_to_string(base.join("genotype")) else {
        println!("## Cilantro taste\n");
        println!("Your data does not cover `{RSID}`, so there is nothing to report.");
        return;
    };
    let genotype = genotype.trim();

    // The lens returns alleles as bases (`A/C`, or `C|C` when phased), so just
    // count copies of the soapy allele. There is no REF/ALT index to decode.
    let copies = genotype.split(['/', '|']).filter(|a| *a == SOAPY_ALLELE).count();

    println!("## Cilantro taste\n");
    println!("Your genotype at `{RSID}` is `{genotype}`.\n");
    match copies {
        0 => println!("You carry no copies of the soapy `{SOAPY_ALLELE}` allele. Cilantro probably tastes fresh and herby."),
        1 => println!("You carry one copy of the soapy `{SOAPY_ALLELE}` allele. Cilantro may have a faint soapy note."),
        _ => println!("You carry two copies of the soapy `{SOAPY_ALLELE}` allele. Cilantro likely tastes like dish soap."),
    }

    // The coordinate, read from the lens's other leaves, shown for context.
    let leaf = |name: &str| {
        fs::read_to_string(base.join(name))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    let (chrom, pos, reference) = (leaf("chromosome"), leaf("position"), leaf("reference"));
    if !chrom.is_empty() && !pos.is_empty() {
        println!("\nLocus `{chrom}:{pos}`, reference allele `{reference}`.");
    }
}
