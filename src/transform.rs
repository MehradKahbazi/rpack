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

    let mut output = String::new();

    for statement in &result.program.body {
        match statement {
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => {
                    let function_name = function
                        .id
                        .as_ref()
                        .ok_or_else(|| "Exported function has no name".to_string())?;

                    let start = function.span.start as usize;
                    let end = function.span.end as usize;

                    let function_source = &source[start..end];

                    output.push_str(function_source);
                    output.push_str("\n\n");
                    output.push_str(&format!(
                        "exports.{} = {};\n",
                        function_name.name, function_name.name
                    ));
                }

                _ => {}
            },

            _ => {}
        }
    }

    Ok(output)
}
