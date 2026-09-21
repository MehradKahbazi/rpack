use std::path::PathBuf;

use crate::graph::ModuleGraph;

pub fn bundle(input: &str) -> Result<String, String> {
    let entry = PathBuf::from(input);

    let mut graph = ModuleGraph::new();

    graph.build(&entry)?;

    println!("{graph:#?}");

    Ok(String::from("Bundle generated successfully"))
}
