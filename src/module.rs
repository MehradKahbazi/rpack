#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub source: String,
    pub dependencies: Vec<String>,
}

impl Module {
    pub fn new(id: String, source: String, dependencies: Vec<String>) -> Self {
        Self {
            id,
            source,
            dependencies,
        }
    }
}
