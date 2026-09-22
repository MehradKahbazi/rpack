use std::path::{Path, PathBuf};

use crate::graph::ModuleGraph;

pub fn bundle(input: &str) -> Result<String, String> {
    let entry = PathBuf::from(input);

    let mut graph = ModuleGraph::new();

    graph.build(&entry)?;

    println!("Graph built successfully!");
    println!("Module count: {}", graph.modules.len());

    for (id, module) in &graph.modules {
        println!("Module: {id}");

        for dependency in &module.dependencies {
            println!("  {} -> {}", dependency.request, dependency.resolved_id);
        }
    }

    generate_bundle(&graph, &entry)
}

fn generate_bundle(graph: &ModuleGraph, entry: &PathBuf) -> Result<String, String> {
    let mut output = String::new();

    output.push_str("(function() {\n");

    // -------------------------
    // Modules
    // -------------------------

    output.push_str("  const modules = {\n");

    for (id, module) in &graph.modules {
        output.push_str(&format!(
            "    {:?}: function(module, exports, require) {{\n",
            id
        ));

        for line in module.transformed_source.lines() {
            output.push_str("      ");
            output.push_str(line);
            output.push('\n');
        }

        output.push_str("    },\n");
    }

    output.push_str("  };\n\n");

    // -------------------------
    // Module cache
    // -------------------------

    output.push_str("  const cache = {};\n\n");

    // -------------------------
    // Runtime require
    // -------------------------

    output.push_str("  function require(id) {\n");

    output.push_str("    if (cache[id]) {\n");
    output.push_str("      return cache[id].exports;\n");
    output.push_str("    }\n\n");

    output.push_str("    const module = {\n");
    output.push_str("      exports: {}\n");
    output.push_str("    };\n\n");

    output.push_str("    cache[id] = module;\n\n");

    output.push_str("    modules[id](module, module.exports, require);\n\n");

    output.push_str("    return module.exports;\n");

    output.push_str("  }\n\n");

    // -------------------------
    // Execute entry
    // -------------------------

    let entry_id = module_id(entry);

    output.push_str(&format!("  require({:?});\n", entry_id));

    output.push_str("})();\n");

    Ok(output)
}

fn module_id(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join(path)
    };

    let mut id = absolute.to_string_lossy().replace('\\', "/");

    // Windows verbatim path prefix
    if let Some(stripped) = id.strip_prefix("//?/") {
        id = stripped.to_string();
    }

    id
}
