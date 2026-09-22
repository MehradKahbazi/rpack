use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, ImportDeclaration, Statement};
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
            Statement::ImportDeclaration(import) => {
                output.push_str(&transform_import(import));
                output.push('\n');
            }

            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => {
                    let function_name = function
                        .id
                        .as_ref()
                        .ok_or_else(|| "Exported function has no name".to_string())?;

                    println!("Transforming exported function: {}", function_name.name);

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

                _ => {
                    return Err("Unsupported export declaration".to_string());
                }
            },

            Statement::VariableDeclaration(declaration) => {
                let start = declaration.span.start as usize;
                let end = declaration.span.end as usize;

                output.push_str(&source[start..end]);
                output.push('\n');
            }

            Statement::ExpressionStatement(statement) => {
                let start = statement.span.start as usize;
                let end = statement.span.end as usize;

                output.push_str(&source[start..end]);
                output.push('\n');
            }

            _ => {
                println!("Skipping unsupported statement");
            }
        }
    }

    Ok(output)
}

fn transform_import(import: &ImportDeclaration) -> String {
    let source = import.source.value;

    let mut bindings = Vec::new();

    for specifiers in import.specifiers.iter() {
        for specifier in specifiers.iter() {
            match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let imported = specifier.imported.name();
                    let local = specifier.local.name;

                    println!("Transforming import: {} as {}", imported, local);

                    if imported == local {
                        bindings.push(imported.to_string());
                    } else {
                        bindings.push(format!("{}: {}", imported, local));
                    }
                }

                _ => {
                    return format!("// Unsupported import from {:?}", source);
                }
            }
        }
    }

    format!(
        "const {{ {} }} = require({:?});",
        bindings.join(", "),
        source
    )
}
