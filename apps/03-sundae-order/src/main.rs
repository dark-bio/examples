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
            "manifest = 1\n\n\
             [app]\n\
             name = \"sundae-order\"\n\
             version = \"0.1.0\"\n\n\
             [inputs.flavor]\n\
             type = \"choice\"\n\
             prompt = \"Which flavor should the scoop be?\"\n\
             choices = [\"Vanilla\", \"Chocolate\", \"Strawberry\", \"Pistachio\"]\n\
             default = \"Vanilla\"\n\n\
             [inputs.toppings]\n\
             type = \"choice\"\n\
             prompt = \"Which toppings go on top?\"\n\
             choices = [\"Sprinkles\", \"Hot fudge\", \"Whipped cream\", \"Cherry\"]\n\
             multiple = true\n\
             default = [\"Hot fudge\", \"Cherry\"]\n\n\
             [inputs.card]\n\
             type = \"text\"\n\
             prompt = \"What should the card say?\"\n\
             max = 40\n"
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
    println!("# Your Sundae\n");
    println!("One scoop of {flavor} with {topping_names}, and a card that reads \"{card}\".\n");
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
