#[derive(Debug)]
pub struct Dependency {
    pub request: String,
    pub resolved_id: String,
}

#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub source: String,
    pub dependencies: Vec<Dependency>,
}

impl Module {
    pub fn new(id: String, source: String, dependencies: Vec<Dependency>) -> Self {
        Self {
            id,
            source,
            dependencies,
        }
    }
}
