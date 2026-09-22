use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug)]
pub struct ParseResult {
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

pub fn parse(source: &str, filename: &str) -> Result<ParseResult, String> {
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

    let mut imports = Vec::new();
    let mut exports = Vec::new();

    for statement in &result.program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                imports.push(import.source.value.to_string());
            }

            Statement::ExportDeclaration(export) => {
                if let Declaration::FunctionDeclaration(function) = &export.declaration {
                    let function_name = function
                        .id
                        .as_ref()
                        .ok_or_else(|| "Exported function has no name".to_string())?;

                    exports.push(function_name.name.to_string());
                }
            }

            _ => {}
        }
    }

    Ok(ParseResult { imports, exports })
}
