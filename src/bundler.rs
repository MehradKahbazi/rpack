use std::path::PathBuf;

use crate::graph::ModuleGraph;

pub fn bundle(input: &str) -> Result<String, String> {
    let entry = PathBuf::from(input);

    let mut graph = ModuleGraph::new();

    graph.build(&entry)?;

    generate_bundle(&graph, &entry)
}

fn generate_bundle(graph: &ModuleGraph, entry: &PathBuf) -> Result<String, String> {
    let mut output = String::new();

    output.push_str("(function() {\n");
    output.push_str("  const modules = {\n");

    for (id, module) in &graph.modules {
        output.push_str(&format!(
            "    {:?}: function(module, exports, require) {{\n",
            id
        ));

        for line in module.source.lines() {
            output.push_str("      ");
            output.push_str(line);
            output.push('\n');
        }

        output.push_str("    },\n");
    }

    output.push_str("  };\n");
    output.push_str("})();\n");

    println!("Entry: {}", entry.display());

    Ok(output)
}
