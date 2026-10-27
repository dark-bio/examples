//! The original IrisPlex regression, consuming and producing plain text files.

use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Each rsID, counted and other forward alleles, and blue/intermediate weights.
///
/// Brown is the baseline in Walsh et al. (2011), Supplementary Table 3,
/// doi:10.1016/j.fsigen.2010.02.004. The README reproduces the parameters.
const PREDICTORS: [(&str, [u8; 2], [f64; 2]); 6] = [
    ("rs12913832", *b"AG", [-4.81, -1.79]),
    ("rs1800407", *b"TC", [1.40, 0.87]),
    ("rs12896399", *b"GT", [-0.58, -0.03]),
    ("rs16891982", *b"CG", [-1.30, -0.50]),
    ("rs1393350", *b"AG", [0.47, 0.27]),
    ("rs12203592", *b"TC", [0.70, 0.73]),
];

/// Prints the manifest or writes probabilities from the bound variant table.
fn main() -> Result<(), Box<dyn Error>> {
    // An untyped input makes this an app that runs through bindings on an Ark
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Eye Color Model"
version = "1.0.0"

[inputs]
variants = {{}}

[inputs.detail]
type = "choice"
prompt = "How much detail should the report have?"
choices = ["summary", "full"]

[outputs]
colors = {{}}
"#
        );
        return Ok(());
    };

    // Read the bound inputs and compute the model's probabilities
    let root = Path::new(&dir);
    let detail = fs::read_to_string(root.join("inputs/detail"))?;
    let variants = fs::read_to_string(root.join("inputs/variants"))?;
    let probabilities = predict(&variants)?;
    let colors = match probabilities {
        Some(values) => {
            let best = (0..3).fold(0, |best, index| {
                if values[index] > values[best] {
                    index
                } else {
                    best
                }
            });
            ["blue", "intermediate", "brown"]
                .iter()
                .enumerate()
                .filter(|(index, _)| detail == "full" || *index == best)
                .map(|(index, color)| format!("{color}\t{:.9}\n", values[index]))
                .collect::<String>()
        }
        None => "unavailable\tmissing-or-unsupported-genotype\n".into(),
    };

    // The Ark creates each declared output, so open it without create
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(root.join("outputs/colors"))?
        .write_all(colors.as_bytes())?;
    print!("{colors}");
    Ok(())
}

/// Computes blue, intermediate and brown probabilities for a complete panel.
///
/// Returns no estimate for absent, uncalled, non-diploid or unmodeled alleles.
/// Broken rows, duplicate identifiers and incomplete panels are format errors.
fn predict(variants: &str) -> Result<Option<[f64; 3]>, Box<dyn Error>> {
    // Match by rsID so either gatherer order gives the same answer
    let mut counts = [None; 6];
    let mut seen = [false; 6];
    for line in variants.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err("variants must hold four tab-separated fields per row".into());
        }
        let index = PREDICTORS
            .iter()
            .position(|(rsid, _, _)| *rsid == fields[0])
            .ok_or("variants contains an unknown rsID")?;
        if seen[index] {
            return Err("variants contains a duplicate rsID".into());
        }
        seen[index] = true;
        counts[index] = dosage(fields[3], PREDICTORS[index].1);
    }
    if seen.contains(&false) {
        return Err("variants must contain all six rsIDs".into());
    }
    let Some(counts) = counts.into_iter().collect::<Option<Vec<_>>>() else {
        return Ok(None);
    };

    // Brown's logit is zero, and subtracting the largest keeps exp in range
    let mut logits = [3.94, 0.65, 0.0];
    for (count, (_, _, weights)) in counts.iter().zip(&PREDICTORS) {
        logits[0] += f64::from(*count) * weights[0];
        logits[1] += f64::from(*count) * weights[1];
    }
    let largest = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights = logits.map(|value| (value - largest).exp());
    let total: f64 = weights.iter().sum();
    Ok(Some(weights.map(|weight| weight / total)))
}

/// Counts a modeled allele in exactly two single-base calls, ignoring phase.
fn dosage(genotype: &str, [counted, other]: [u8; 2]) -> Option<u8> {
    let value = genotype
        .strip_prefix(['/', '|'])
        .unwrap_or(genotype)
        .as_bytes();
    if value.len() != 3 || !matches!(value[1], b'/' | b'|') {
        return None;
    }
    let mut count = 0;
    for allele in [value[0], value[2]] {
        if allele == counted {
            count += 1;
        } else if allele != other {
            return None;
        }
    }
    Some(count)
}
