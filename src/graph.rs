use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::module::{Export, Module};
use crate::parser;
use crate::resolver;
use crate::transform;
use crate::utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleState {
    Visiting,
    Visited,
}

#[derive(Debug)]
pub struct ModuleGraph {
    pub modules: HashMap<String, Module>,
    states: HashMap<String, ModuleState>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            states: HashMap::new(),
        }
    }

    pub fn build(&mut self, entry: &Path) -> Result<(), String> {
        self.visit(entry)
    }

    fn visit(&mut self, path: &Path) -> Result<(), String> {
        let id = utils::module_id(path);

        if self.states.get(&id) == Some(&ModuleState::Visited) {
            return Ok(());
        }

        if self.states.get(&id) == Some(&ModuleState::Visiting) {
            return Err(format!("Circular dependency detected at module '{}'", id));
        }

        self.states.insert(id.clone(), ModuleState::Visiting);

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

            // Validate named imports.
            for imported_name in &import.named {
                let exists = dependency.exports.iter().any(|export| {
                    matches!(
                        export,
                        Export::Named {
                            exported,
                            ..
                        } if exported == imported_name
                    )
                });

                if !exists {
                    return Err(format!(
                        "Module '{}' does not export '{}'",
                        import.source, imported_name
                    ));
                }
            }

            // Validate default import.
            if import.default.is_some() {
                let has_default = dependency
                    .exports
                    .iter()
                    .any(|export| matches!(export, Export::Default(_)));

                if !has_default {
                    return Err(format!(
                        "Module '{}' does not have a default export",
                        import.source
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

        self.modules.insert(id.clone(), module);
        self.states.insert(id, ModuleState::Visited);

        Ok(())
    }
}
