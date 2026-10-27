//! An eye color report built from a genotype gatherer and the IrisPlex model.

use std::error::Error;
use std::fs;
use std::path::Path;

// The build writes these pins from the final optimized dependency modules
include!("pins.rs");

/// Prints the manifest or reports the model's association with eye color.
fn main() -> Result<(), Box<dyn Error>> {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Eye Color"
version = "1.0.0"

[inputs.detail]
type = "choice"
prompt = "How much detail should the report have?"
choices = ["summary", "full"]
default = "summary"

[apps.gather]
sha256 = "{GATHER}"
inputs.order = "rsid"

[apps.model]
sha256 = "{MODEL}"
inputs.variants = {{ output = "gather/variants" }}
inputs.detail = {{ input = "detail" }}

[listing]
language = "en"
icon = "👁️"
summary = "Which eye color is your DNA associated with? Six variants weigh in."
category = "traits"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/12-eye-color"
keywords = ["eye color", "IrisPlex", "pigment", "composition"]
description = '''
Uses six pigment variants and the IrisPlex model to estimate probabilities for blue, intermediate and brown eyes. Choose the leading color or all three probabilities.

Your DNA is associated with a color. The estimate does not establish your eye color. The model separates blue from brown well and calls intermediate colors, such as green and hazel, poorly.

Absent, uncalled or unsupported genotypes produce no estimate. Walsh et al. (2011) supply the model and its coefficients.
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

    // Both inputs to this report come from completed, pinned dependencies
    let root = Path::new(&dir);
    let colors = fs::read_to_string(root.join("apps/model/colors"))?;
    let variants = fs::read_to_string(root.join("apps/gather/variants"))?;
    let mut probabilities = Vec::new();
    for line in colors.lines() {
        let (color, probability) = line
            .split_once('\t')
            .ok_or("model/colors must hold two fields per row")?;
        if color != "unavailable" {
            probabilities.push((color, probability.parse::<f64>()?));
        }
    }

    // Reverse the search so equal probabilities favor the model's first color
    println!("# 👁️ Eye Color\n");
    if let Some((color, probability)) = probabilities
        .iter()
        .rev()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    {
        println!(
            "Your DNA is most strongly associated with {color} eyes in this model, \
            with a probability of {:.1}%.\n",
            probability * 100.0
        );
    } else {
        println!(
            "**No estimate.** At least one of the 6 genotypes is absent, uncalled \
            or outside the model's two-allele panel.\n"
        );
    }
    if probabilities.len() == 3 {
        println!("| Color | Model probability |\n| :-- | --: |");
        for (color, probability) in &probabilities {
            println!("| {color} | {:.1}% |", probability * 100.0);
        }
        println!();
    }

    // Put each exact genotype beside its coordinate, with pipes outside tables
    println!("## Evidence\n");
    for line in variants.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err("gather/variants must hold four fields per row".into());
        }
        println!(
            "- {} at {}:{} reads `{}`.",
            fields[0], fields[1], fields[2], fields[3]
        );
    }
    println!();
    println!("## Method\n");
    println!(
        "Walsh et al. (2011) use six pigment variants in the IrisPlex model, \
        fitted with Dutch European data. Each counted allele adds a published weight \
        to the blue and intermediate scores, with brown as the baseline. \
        The scores become three probabilities that sum to 100%.\n"
    );
    println!(
        "The leading probability is an association, without a classification \
        threshold. The mirror keeps the final word.\n"
    );
    println!("## Limitations\n");
    println!(
        "The model separates blue from brown well and calls intermediate colors, \
        such as green and hazel, poorly. Six variants do not establish your eye color. \
        Performance can vary across populations. An absent answer never counts as \
        two reference alleles.\n"
    );
    println!("## Sources\n");
    println!(
        "1. Walsh S, et al. (2011). IrisPlex: A sensitive DNA tool for accurate \
        prediction of blue and brown eye colour in the absence of ancestry information. \
        Forensic Science International: Genetics 5, 170-180. Supplementary Table 3. \
        https://doi.org/10.1016/j.fsigen.2010.02.004"
    );
    Ok(())
}
