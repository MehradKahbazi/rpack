use std::env;

mod bundler;
mod graph;
mod module;
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

    let bundle = bundler::bundle(input)?;

    println!("{bundle}");

    Ok(())
}
