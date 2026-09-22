use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::module::Module;
use crate::parser;
use crate::resolver;
use crate::transform;
use crate::utils;

#[derive(Debug)]
pub struct ModuleGraph {
    pub modules: HashMap<String, Module>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn build(&mut self, entry: &Path) -> Result<(), String> {
        self.visit(entry)
    }

    fn visit(&mut self, path: &Path) -> Result<(), String> {
        let id = utils::module_id(path);

        if self.modules.contains_key(&id) {
            return Ok(());
        }

        let source = fs::read_to_string(path)
            .map_err(|error| format!("Failed to read '{}': {error}", path.display()))?;

        let parsed = parser::parse(&source, &id)?;

        let mut dependencies = Vec::new();

        for import in &parsed.imports {
            let resolved = resolver::resolve(path, &import.source)?;

            let resolved_id = utils::module_id(&resolved);

            self.visit(&resolved)?;

            let dependency = self.modules.get(&resolved_id).ok_or_else(|| {
                format!("Resolved module '{}' was not added to graph", resolved_id)
            })?;

            for imported_name in &import.named {
                if !dependency.exports.contains(imported_name) {
                    return Err(format!(
                        "Module '{}' does not export '{}'",
                        import.source, imported_name
                    ));
                }
            }

            dependencies.push(crate::module::Dependency {
                request: import.source.clone(),
                resolved_id,
            });
        }

        let transformed_source = transform::transform(&source, &id, &dependencies)?;

        let module = Module::new(
            id.clone(),
            source,
            transformed_source,
            dependencies,
            parsed.exports,
        );

        self.modules.insert(id, module);

        Ok(())
    }
}
