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
        let mut exports = parsed.exports;

        // ---------------------------------------------------------
        // Regular imports
        // ---------------------------------------------------------

        for import in &parsed.imports {
            let resolved = resolver::resolve(path, &import.source)?;

            let resolved_id = utils::module_id(&resolved);

            self.visit(&resolved)?;

            let dependency = self.modules.get(&resolved_id).ok_or_else(|| {
                format!("Resolved module '{}' was not added to graph", resolved_id)
            })?;

            for imported_name in &import.named {
                let exists = dependency
                    .exports
                    .iter()
                    .any(|export| export_name(export) == Some(imported_name.as_str()));

                if !exists {
                    return Err(format!(
                        "Module '{}' does not export '{}'",
                        import.source, imported_name
                    ));
                }
            }

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

        // ---------------------------------------------------------
        // Re-exports
        // ---------------------------------------------------------

        for re_export in &parsed.re_exports {
            match re_export {
                parser::ReExport::Named {
                    source,
                    imported,
                    exported,
                } => {
                    let resolved = resolver::resolve(path, source)?;

                    let resolved_id = utils::module_id(&resolved);

                    self.visit(&resolved)?;

                    let dependency = self.modules.get(&resolved_id).ok_or_else(|| {
                        format!("Resolved module '{}' was not added to graph", resolved_id)
                    })?;

                    let exists = dependency
                        .exports
                        .iter()
                        .any(|export| export_name(export) == Some(imported.as_str()));

                    if !exists {
                        return Err(format!(
                            "Module '{}' does not export '{}'",
                            source, imported
                        ));
                    }

                    exports.push(Export::ReExport {
                        imported: imported.clone(),
                        exported: exported.clone(),
                        source: source.clone(),
                        resolved_id: resolved_id.clone(),
                    });

                    dependencies.push(crate::module::Dependency {
                        request: source.clone(),
                        resolved_id,
                    });
                }

                parser::ReExport::Namespace { source } => {
                    let resolved = resolver::resolve(path, source)?;

                    let resolved_id = utils::module_id(&resolved);

                    self.visit(&resolved)?;

                    let dependency = self.modules.get(&resolved_id).ok_or_else(|| {
                        format!("Resolved module '{}' was not added to graph", resolved_id)
                    })?;

                    // `export * from "./module.js"` re-exports all
                    // named exports of the dependency.
                    //
                    // The default export is intentionally excluded.
                    for export in &dependency.exports {
                        match export {
                            Export::Named { exported, .. } => {
                                if !exports.iter().any(|existing| {
                                    export_name(existing) == Some(exported.as_str())
                                }) {
                                    exports.push(Export::ReExport {
                                        imported: exported.clone(),
                                        exported: exported.clone(),
                                        source: source.clone(),
                                        resolved_id: resolved_id.clone(),
                                    });
                                }
                            }

                            Export::ReExport {
                                imported, exported, ..
                            } => {
                                if !exports.iter().any(|existing| {
                                    export_name(existing) == Some(exported.as_str())
                                }) {
                                    exports.push(Export::ReExport {
                                        imported: imported.clone(),
                                        exported: exported.clone(),
                                        source: source.clone(),
                                        resolved_id: resolved_id.clone(),
                                    });
                                }
                            }

                            Export::NamespaceReExport { .. } => {
                                // Nested namespace re-exports will be handled
                                // when we improve export resolution.
                            }

                            Export::Default(_) => {
                                // `export *` does not re-export default.
                            }
                        }
                    }

                    dependencies.push(crate::module::Dependency {
                        request: source.clone(),
                        resolved_id,
                    });
                }
            }
        }

        let transformed_source = transform::transform(&source, &id, &dependencies)?;

        let module = Module::new(
            id.clone(),
            source,
            transformed_source,
            dependencies,
            exports,
        );

        self.modules.insert(id.clone(), module);

        self.states.insert(id, ModuleState::Visited);

        Ok(())
    }
}

fn export_name(export: &Export) -> Option<&str> {
    match export {
        Export::Named { exported, .. } => Some(exported.as_str()),

        Export::Default(_) => Some("default"),

        Export::ReExport { exported, .. } => Some(exported.as_str()),

        Export::NamespaceReExport { .. } => None,
    }
}
