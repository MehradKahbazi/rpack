#[derive(Debug)]
pub struct Module {
    pub id: String,
    pub source: String,
    pub dependencies: Vec<String>,
}

impl Module {
    pub fn new(id: String, source: String) -> Self {
        let dependencies = parse_imports(&source);

        Self {
            id,
            source,
            dependencies,
        }
    }
}

fn parse_imports(source: &str) -> Vec<String> {
    let mut dependencies = Vec::new();

    for line in source.lines() {
        let line = line.trim();

        if !line.starts_with("import ") {
            continue;
        }

        if let Some(from_index) = line.find("from ") {
            let path = line[from_index + 5..]
                .trim()
                .trim_end_matches(';')
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            dependencies.push(path.to_string());
        }
    }

    dependencies
}
