use std::env;
use std::fs;
use std::path::Path;

mod bundler;
mod graph;
mod module;
mod parser;
mod resolver;
mod transform;
mod utils;

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

    let output = parse_output_argument(&args)?;

    println!("Input: {input}");
    println!("Output: {output}");

    let source = bundler::bundle(input)?;

    let output_path = Path::new(&output);

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create output directory '{}': {error}",
                    parent.display()
                )
            })?;
        }
    }

    fs::write(output_path, source).map_err(|error| {
        format!(
            "Failed to write bundle '{}': {error}",
            output_path.display()
        )
    })?;

    println!("Bundle written to {}", output_path.display());

    Ok(())
}

fn parse_output_argument(args: &[String]) -> Result<String, String> {
    let output_flag = args
        .iter()
        .position(|arg| arg == "-o" || arg == "--output")
        .ok_or_else(|| "Missing output argument. Use -o <output>".to_string())?;

    args.get(output_flag + 1)
        .cloned()
        .ok_or_else(|| "Missing output path after -o".to_string())
}
