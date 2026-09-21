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
    println!("PARSER RUNNING: {filename}");

    for statement in &result.program.body {
        println!("STATEMENT: {:?}", statement);

        match statement {
            Statement::ImportDeclaration(import) => {
                println!("IMPORT: {}", import.source.value);
                dependencies.push(import.source.value.to_string());
            }

            Statement::ExportDeclaration(export) => {
                println!("EXPORT FOUND: {:?}", export);
            }

            _ => {}
        }
    }

    Ok(dependencies)
}
