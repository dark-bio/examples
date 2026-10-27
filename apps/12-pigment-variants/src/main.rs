//! Six pigment genotypes, passed to another app as a plain text table.

use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// The six variants in the published IrisPlex panel.
const RSIDS: [&str; 6] = [
    "rs12913832",
    "rs1800407",
    "rs12896399",
    "rs16891982",
    "rs1393350",
    "rs12203592",
];

/// Prints the manifest or gathers the panel into its declared output.
fn main() -> Result<(), Box<dyn Error>> {
    // Declare the same typed input whether used alone or as a dependency
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Pigment Variants"
version = "1.0.0"

[reads]
paths = [
  "v1/genome/rsids/rs12913832",
  "v1/genome/rsids/rs1800407",
  "v1/genome/rsids/rs12896399",
  "v1/genome/rsids/rs16891982",
  "v1/genome/rsids/rs1393350",
  "v1/genome/rsids/rs12203592",
]

[inputs.order]
type = "choice"
prompt = "How should the variants be ordered?"
choices = ["position", "rsid"]

[outputs]
variants = {{}}

[listing]
language = "en"
icon = "🎨"
summary = "Six pigment variants, ready for an eye color model or a look at your DNA."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/12-pigment-variants"
keywords = ["eye color", "IrisPlex", "variants", "composition"]
description = '''
Reads six variants used by IrisPlex and shows your DNA letters at each position. Choose position or rs number order.

The same plain text table can feed another app in the task. Absent answers stay absent, and uncalled letters stay uncalled.
'''

[listing.purposes]
"v1/genome/rsids/rs12913832" = "The HERC2 variant used by the eye color model"
"v1/genome/rsids/rs1800407" = "The OCA2 variant used by the eye color model"
"v1/genome/rsids/rs12896399" = "The SLC24A4 variant used by the eye color model"
"v1/genome/rsids/rs16891982" = "The SLC45A2 variant used by the eye color model"
"v1/genome/rsids/rs1393350" = "The TYR variant used by the eye color model"
"v1/genome/rsids/rs12203592" = "The IRF4 variant used by the eye color model"
"#
        );
        return Ok(());
    };

    // Preserve absent genotypes while treating other read failures as errors
    let root = Path::new(&dir);
    let order = fs::read_to_string(root.join("inputs/order"))?;
    let mut variants = Vec::new();
    for rsid in RSIDS {
        let base = root.join("v1/genome/rsids").join(rsid);
        let chromosome = fs::read_to_string(base.join("chromosome"))?;
        let chromosome = chromosome
            .strip_prefix("chr")
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|number| (1..=22).contains(number))
            .ok_or("expected an autosome at the pigment variant")?;
        let position = fs::read_to_string(base.join("position"))?.parse::<u64>()?;
        let genotype = match fs::read_to_string(base.join("genotype")) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => "absent".into(),
            Err(error) => return Err(format!("could not read {rsid}/genotype: {error}").into()),
        };
        variants.push((rsid, chromosome, position, genotype));
    }

    // Sort numerically, by chromosome and position or by the rs number
    match order.as_str() {
        "position" => variants.sort_by_key(|(_, chromosome, position, _)| (*chromosome, *position)),
        "rsid" => variants.sort_by_key(|(rsid, _, _, _)| rsid[2..].parse::<u64>().unwrap()),
        _ => return Err("inputs/order must be position or rsid".into()),
    }

    // Open the output the Ark created, since apps cannot create files there
    let mut output = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(root.join("outputs/variants"))?;
    for (rsid, chromosome, position, genotype) in &variants {
        writeln!(output, "{rsid}\tchr{chromosome}\t{position}\t{genotype}")?;
    }

    // Keep phased genotypes outside Markdown table cells so pipes stay intact
    println!("# 🎨 Pigment Panel\n");
    println!(
        "Your DNA has genotype records at {} of the 6 pigment positions.\n",
        variants
            .iter()
            .filter(|(_, _, _, genotype)| genotype != "absent")
            .count()
    );
    println!("## Evidence\n");
    println!("| Variant | Position | Genotype |\n| :-- | :-- | :-- |");
    for (rsid, chromosome, position, genotype) in &variants {
        let shown = if genotype.contains('|') {
            "see below".to_owned()
        } else {
            format!("`{genotype}`")
        };
        println!("| {rsid} | chr{chromosome}:{position} | {shown} |");
    }
    println!();
    for (rsid, chromosome, position, genotype) in &variants {
        if genotype.contains('|') {
            println!("{rsid} at chr{chromosome}:{position} reads `{genotype}`.\n");
        }
    }
    println!("## Limitations\n");
    println!(
        "An absent answer is not a reference genotype. This table makes no eye color estimate."
    );
    Ok(())
}
