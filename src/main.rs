use std::env;
mod bundler;
mod graph;
mod module;
mod parser;
mod resolver;
mod transform;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    println!("RUN STARTED");

    let args: Vec<String> = env::args().collect();

    let input = args
        .get(1)
        .ok_or_else(|| "Missing input file".to_string())?;

    println!("Input: {input}");

    let source = bundler::bundle(input)?;

    println!("BUNDLE RETURNED");

    println!("{source}");

    Ok(())
}
