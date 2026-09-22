#[derive(Debug)]
pub struct Dependency {
    pub request: String,
    pub resolved_id: String,
}

#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub source: String,
    pub transformed_source: String,
    pub dependencies: Vec<Dependency>,
    pub exports: Vec<String>,
}

impl Module {
    pub fn new(
        id: String,
        source: String,
        transformed_source: String,
        dependencies: Vec<Dependency>,
        exports: Vec<String>,
    ) -> Self {
        Self {
            id,
            source,
            transformed_source,
            dependencies,
            exports,
        }
    }
}
