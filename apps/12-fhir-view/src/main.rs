//! Renders FHIR leaves as component tables beside their re-indented JSON.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use serde_json::Value;

/// Largest leaf accepted from the Ark, in bytes.
const MAX_LEAF: usize = 16 * 1024 * 1024;
/// Maximum re-indented JSON bytes printed for one endpoint.
const MAX_RAW: usize = 12 * 1024;
/// Maximum resources rendered per endpoint.
const MAX_RESOURCES: usize = 8;
/// Maximum change or copy endpoints read from one directory.
const MAX_ENTRIES: usize = 3;

/// Runs the manifest pass or writes the report from granted files.
fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "FHIR View"
version = "0.1.0"

[reads]
paths = ["v1/genome/rsids/rs671", "v1/genome/datasets"]
optional = ["v1/genome/genes/TAS2R38", "v1/genome/genes/UGT2B17", "v1/genome/positions/chrM/263"]

[listing]
language = "en"
icon = "🔎"
summary = "Read your genomic FHIR resources as tables beside their original JSON."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/12-fhir-view"
keywords = ["FHIR", "JSON", "genotype", "copy number"]
description = '''
See one site, a gene, individual changes, copy numbers and a haploid site as FHIR R4 JSON. Each leaf appears beside a rendered component table.

Missing answers stay missing. The app counts resources, limits long displays and makes no trait prediction.
'''

[listing.purposes]
"v1/genome/rsids/rs671" = "A site collection and its exact genotype"
"v1/genome/genes/TAS2R38" = "A gene collection and individual change resources"
"v1/genome/genes/UGT2B17" = "Individual copy number resources"
"v1/genome/positions/chrM/263" = "A haploid site and its variant"
"v1/genome/datasets" = "Whether a copy number dataset is uploaded"
"#
        );
        return;
    };
    if let Err(error) = report(Path::new(&dir)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

/// One leaf's path, re-indented JSON and parsed resource.
struct Leaf {
    /// Granted path relative to the data root.
    path: String,
    /// Re-indented JSON with key order preserved, absent when there is no answer.
    json: Option<String>,
    /// Parsed JSON used for the rendered view.
    resource: Option<Value>,
}

impl Leaf {
    /// Reads one bounded leaf, treating only not-found as an absent answer.
    fn read(root: &Path, path: &str) -> io::Result<Self> {
        let mut raw = String::new();
        match File::open(root.join(path)) {
            Ok(file) => {
                file.take(MAX_LEAF as u64 + 1).read_to_string(&mut raw)?;
                if raw.len() > MAX_LEAF {
                    return Err(io::Error::other("FHIR leaf exceeds 16 MiB"));
                }
                let resource: Value = serde_json::from_str(&raw).map_err(io::Error::other)?;
                match resource["resourceType"].as_str() {
                    Some("Observation" | "Bundle") => {}
                    _ => return Err(io::Error::other("unrecognized FHIR resource")),
                }
                let json = serde_json::to_string_pretty(&resource).map_err(io::Error::other)?;
                Ok(Self {
                    path: path.into(),
                    json: Some(json),
                    resource: Some(resource),
                })
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self {
                path: path.into(),
                json: None,
                resource: None,
            }),
            Err(error) => Err(error),
        }
    }

    /// Returns the observations in their stored order.
    fn observations(&self) -> Vec<&Value> {
        match self.resource.as_ref() {
            Some(resource) if resource["resourceType"] == "Bundle" => resource["entry"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|entry| &entry["resource"])
                .collect(),
            Some(resource) => vec![resource],
            None => Vec::new(),
        }
    }

    /// Prints a bounded rendered view followed by the re-indented leaf.
    fn print(&self, number: usize) {
        println!("### Endpoint {number}\n\n`{}`\n", self.path);
        let Some(json) = &self.json else {
            println!("No answer. This leaf is absent.\n");
            return;
        };
        let observations = self.observations();
        if observations.is_empty() {
            println!("An empty `Bundle` collection.\n");
        }
        for observation in observations.iter().take(MAX_RESOURCES) {
            println!(
                "`Observation {}` = {}\n",
                code(&observation["code"]),
                inline(&concept(&observation["valueCodeableConcept"]))
            );
            let components = observation["component"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if !components.is_empty() {
                println!("| Code | Display | Value |\n| :-- | :-- | :-- |");
                for component in components.iter().take(24) {
                    let display = component["code"]["coding"][0]["display"]
                        .as_str()
                        .unwrap_or("Uncoded");
                    let value = if let Some(value) = component.get("valueCodeableConcept") {
                        concept(value)
                    } else if let Some(value) = component.get("valueString") {
                        value.as_str().unwrap_or("").to_string()
                    } else if let Some(value) = component.get("valueQuantity") {
                        format!("{} copies", value["value"])
                    } else if let Some(value) = component.get("valueRange") {
                        format!("{}-{} bp", value["low"]["value"], value["high"]["value"])
                    } else {
                        "No value".into()
                    };
                    println!(
                        "| {} | {} | {} |",
                        inline(&code(&component["code"])),
                        cell(display),
                        cell(&value)
                    );
                }
                println!();
                if components.len() > 24 {
                    println!("{} components omitted.\n", components.len() - 24);
                }
            }
        }
        if observations.len() > MAX_RESOURCES {
            println!(
                "{} resources omitted from the rendered view.\n",
                observations.len() - MAX_RESOURCES
            );
        }
        let mut end = json.len().min(MAX_RAW);
        while !json.is_char_boundary(end) {
            end -= 1;
        }
        println!(
            "Leaf stored compact; shown re-indented with content unchanged:\n\n```json\n{}\n```\n",
            &json[..end]
        );
        if end < json.len() {
            println!(
                "{} bytes omitted from the re-indented JSON.\n",
                json.len() - end
            );
        }
    }
}

/// Returns the first terminology code or an uncoded label.
fn code(value: &Value) -> String {
    value["coding"][0]["code"]
        .as_str()
        .unwrap_or("uncoded")
        .into()
}

/// Renders a coded concept or preserves its exact text.
fn concept(value: &Value) -> String {
    if let Some(text) = value["text"].as_str() {
        return text.into();
    }
    value["coding"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|coding| {
            format!(
                "{} {}",
                coding["code"].as_str().unwrap_or(""),
                coding["display"].as_str().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
        .trim_end()
        .into()
}

/// Limits long values while preserving the count of omitted characters.
fn shortened(value: &str) -> String {
    let count = value.chars().count();
    if count <= 256 {
        return value.into();
    }
    format!(
        "{} ({} characters omitted)",
        value.chars().take(256).collect::<String>(),
        count - 256
    )
}

/// Escapes text for a Markdown table cell.
fn cell(value: &str) -> String {
    shortened(value)
        .replace('|', "\\|")
        .replace(['\n', '\r'], " ")
        .replace('`', "'")
}

/// Keeps exact genotype separators in prose code, outside Markdown tables.
fn inline(value: &str) -> String {
    format!("`{}`", shortened(value).replace('`', "'"))
}

/// Appends a sorted, bounded selection of change or copy leaves.
fn entries(root: &Path, path: &str, paths: &mut Vec<String>) -> io::Result<usize> {
    let directory = match fs::read_dir(root.join(path)) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    let mut names = directory
        .map(|entry| {
            entry?
                .file_name()
                .into_string()
                .map_err(|_| io::Error::other("entry name is not UTF-8"))
        })
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    for name in names.iter().take(MAX_ENTRIES) {
        paths.push(format!("{path}/{name}/fhir"));
    }
    Ok(names.len().saturating_sub(MAX_ENTRIES))
}

/// Reads the five endpoint types and prints the report in evidence order.
fn report(root: &Path) -> io::Result<()> {
    let mut paths = vec![
        "v1/genome/rsids/rs671/fhir".into(),
        "v1/genome/genes/TAS2R38/fhir".into(),
    ];
    let omitted = entries(root, "v1/genome/genes/TAS2R38/changes", &mut paths)?
        + entries(root, "v1/genome/genes/UGT2B17/copies", &mut paths)?;
    paths.push("v1/genome/positions/chrM/263/fhir".into());
    let copies = match fs::metadata(root.join("v1/genome/datasets/copy-number-variants")) {
        Ok(metadata) => metadata.is_dir(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    let leaves = paths
        .iter()
        .map(|path| Leaf::read(root, path))
        .collect::<io::Result<Vec<Leaf>>>()?;
    let observations: usize = leaves.iter().map(|leaf| leaf.observations().len()).sum();
    let answered = leaves.iter().filter(|leaf| leaf.json.is_some()).count();
    println!(
        "# Genomic Data in FHIR\n\nRead {observations} `Observation` {} from {answered} answered {} across {} {}. Repeated resources count each time they are read.\n",
        if observations == 1 {
            "resource"
        } else {
            "resources"
        },
        if answered == 1 { "leaf" } else { "leaves" },
        paths.len(),
        if paths.len() == 1 {
            "endpoint"
        } else {
            "endpoints"
        }
    );
    println!("## Evidence\n");
    for (index, leaf) in leaves.iter().enumerate() {
        leaf.print(index + 1);
    }
    if omitted > 0 {
        println!("{omitted} additional change or copy endpoints omitted.\n");
    }
    println!(
        "{}\n",
        if copies {
            "A copy number dataset is uploaded. Missing counts remain unanswered."
        } else {
            "No copy number dataset is uploaded. An empty copy listing is not a reference answer."
        }
    );
    println!(
        "## Method\n\nRead FHIR R4 leaves shaped by HL7 Genomics Reporting STU3. Render their LOINC codes, component values and re-indented JSON without interpreting traits. Genotypes appear exactly as stored in `valueCodeableConcept.text`.\n"
    );
    println!(
        "## Limitations\n\nThis view samples two genes and two sites. It counts resource instances, not distinct variants. Missing answers never imply reference alleles. Displays are bounded and count omitted content. The leaves contain no dates, `subject` or `specimen`.\n"
    );
    println!(
        "## Sources\n\n1. HL7 International. Genomics Reporting Implementation Guide, STU3, FHIR R4 4.0.1. https://hl7.org/fhir/uv/genomics-reporting/STU3/\n2. Regenstrief Institute. LOINC. https://loinc.org/"
    );
    Ok(())
}
