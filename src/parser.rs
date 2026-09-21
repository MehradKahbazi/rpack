use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn parse(source: &str, filename: &str) -> Result<Vec<String>, String> {
    let allocator = Allocator::default();

    let source_type = SourceType::from_path(filename)
        .map_err(|error| format!("Unsupported file type '{filename}': {error}"))?;

    let result = Parser::new(&allocator, source, source_type).parse();

    if !result.diagnostics.is_empty() {
        return Err(format!(
            "Failed to parse '{filename}': {} error(s)",
            result.diagnostics.len()
        ));
    }

    let mut dependencies = Vec::new();

    for statement in &result.program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                dependencies.push(import.source.value.to_string());
            }

            _ => {}
        }
    }

    Ok(dependencies)
}
