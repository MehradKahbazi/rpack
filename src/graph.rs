use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::module::Module;
use crate::parser;
use crate::resolver;

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
        let id = path.to_string_lossy().to_string();

        if self.modules.contains_key(&id) {
            return Ok(());
        }

        let source = fs::read_to_string(path)
            .map_err(|error| format!("Failed to read '{}': {error}", path.display()))?;

        let dependencies = parser::parse(&source, &id)?;

        for dependency in &dependencies {
            let resolved = resolver::resolve(path, dependency)?;

            self.visit(&resolved)?;
        }

        let module = Module::new(id.clone(), source, dependencies);

        self.modules.insert(id, module);

        Ok(())
    }
}
