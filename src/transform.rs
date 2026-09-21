use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn transform(source: &str, filename: &str) -> Result<String, String> {
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

    let mut output = source.to_string();

    for statement in &result.program.body {
        if let Statement::ExportDeclaration(export) = statement {
            match &export.declaration {
                Declaration::FunctionDeclaration(function) => {
                    let function = function
                        .id
                        .as_ref()
                        .ok_or_else(|| "Exported function has no name".to_string())?;

                    println!("Transforming exported function: {}", function.name);
                }

                _ => {}
            }
        }
    }

    Ok(output)
}
