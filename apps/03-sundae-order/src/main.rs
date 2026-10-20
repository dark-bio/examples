//! A sundae order built from the owner's answers to three questions.
//!
//! The manifest declares a choice, a multiple choice and a text input. The Ark
//! mounts each answer as a file under `inputs/` in the data directory, and the
//! report quotes every file exactly as it was read.

use std::fs;
use std::path::Path;

/// Prints the manifest or reports the owner's sundae order.
fn main() {
    // Print the questions when no data root was passed
    let Some(dir) = std::env::args().nth(1) else {
        print!(
            r#"manifest = 1

[app]
name = "Sundae Order"
version = "0.1.0"

[inputs.flavor]
type = "choice"
prompt = "Which flavor should the scoop be?"
choices = ["Vanilla", "Chocolate", "Strawberry", "Pistachio"]
default = "Vanilla"

[inputs.toppings]
type = "choice"
prompt = "Which toppings go on top?"
choices = ["Sprinkles", "Hot fudge", "Whipped cream", "Cherry"]
multiple = true
default = ["Hot fudge", "Cherry"]

[inputs.card]
type = "text"
prompt = "What should the card say?"
max = 40

[listing]
language = "en"
icon = "🍨"
summary = "A flavor, some toppings and a card. The first app that asks you things."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/03-sundae-order"
keywords = ["tutorial", "inputs", "choices", "text input"]
description = '''
Apps can ask you questions when you approve them. This one wants a flavor, any toppings and the words for the card, then serves your sundae.

Each answer lands as a plain file under `inputs`. A choice holds your pick, a multiple choice one pick per line, and a text exactly what you typed. It reads none of your genome, so all it ever learns is your order.

Answers never widen what an app can read. The grants on the approval are the whole deal.
'''
"#
        );
        return;
    };

    // Read each answer exactly as the Ark mounted it, without trimming
    let inputs = Path::new(&dir).join("inputs");
    let flavor = read_input(&inputs, "flavor");
    let toppings = read_input(&inputs, "toppings");
    let card = read_input(&inputs, "card");

    // Multiple choices hold one pick per line, with an empty file for no picks
    let picks: Vec<String> = toppings.lines().map(str::to_owned).collect();
    let topping_names = if picks.is_empty() {
        "no toppings".to_owned()
    } else {
        join_picks(&picks)
    };

    // Put the order first, followed by each input's value beside its path
    println!("# 🍨 Order Up\n");
    println!(
        "One scoop of {flavor} with {topping_names}, and a card that reads \"{card}\". The \
         kitchen saw your answers and nothing else. Your genome stayed home.\n"
    );
    println!("## Evidence\n");
    println!("- `inputs/flavor` holds {}.", inline_code(&flavor));
    match picks.as_slice() {
        [] => println!("- `inputs/toppings` is empty, so no toppings were picked."),
        [pick] => println!("- `inputs/toppings` holds 1 pick: {}.", inline_code(pick)),
        _ => {
            let values: Vec<String> = picks.iter().map(|pick| inline_code(pick)).collect();
            println!(
                "- `inputs/toppings` holds {} picks, one per line: {}.",
                picks.len(),
                join_picks(&values)
            );
        }
    }
    println!("- `inputs/card` holds {}.", inline_code(&card));
}

/// Reads one input, reporting an error and exiting if its file cannot be read.
fn read_input(inputs: &Path, name: &str) -> String {
    match fs::read_to_string(inputs.join(name)) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Could not read inputs/{name}: {err}");
            std::process::exit(1);
        }
    }
}

/// Joins picks in their file order with commas and a final conjunction.
fn join_picks(picks: &[String]) -> String {
    match picks {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// Wraps a value in inline code, preserving backticks and surrounding spaces.
fn inline_code(value: &str) -> String {
    // Use a fence longer than any run of backticks inside the value
    let longest = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest + 1);

    // Padding keeps edge backticks and paired spaces inside the code span
    let pad = value.starts_with('`')
        || value.ends_with('`')
        || (value.starts_with(' ') && value.ends_with(' ') && !value.chars().all(|c| c == ' '));
    if pad {
        format!("{fence} {value} {fence}")
    } else {
        format!("{fence}{value}{fence}")
    }
}
