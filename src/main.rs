use std::env;
use std::fs;

mod bundler;
mod graph;
mod module;
mod parser;
mod resolver;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let input = args
        .get(1)
        .ok_or_else(|| "Missing input file".to_string())?;

    let source =
        fs::read_to_string(input).map_err(|error| format!("Failed to read '{input}': {error}"))?;

    parser::parse(&source, input)?;

    Ok(())
}
