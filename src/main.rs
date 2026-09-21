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

    let source = bundler::bundle(input)?;

    Ok(())
}
