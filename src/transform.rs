use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, ImportDeclaration, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn transform(
    source: &str,
    filename: &str,
    dependencies: &[crate::module::Dependency],
) -> Result<String, String> {
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
                output.push_str(&transform_import(import, dependencies)?);

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

            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    let function_name = function.id.as_ref().ok_or_else(|| {
                        "Anonymous default functions are not supported yet".to_string()
                    })?;

                    let start = function.span.start as usize;

                    let end = function.span.end as usize;

                    let function_source = &source[start..end];

                    output.push_str(function_source);
                    output.push_str("\n\n");

                    output.push_str(&format!("exports.default = {};\n", function_name.name));
                }

                _ => {
                    return Err("Unsupported default export".to_string());
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

fn transform_import(
    import: &ImportDeclaration,
    dependencies: &[crate::module::Dependency],
) -> Result<String, String> {
    let source = import.source.value.to_string();

    let dependency = dependencies
        .iter()
        .find(|dependency| dependency.request == source)
        .ok_or_else(|| format!("Dependency '{}' was not resolved", source))?;

    let mut named_bindings = Vec::new();
    let mut default_binding = None;

    for specifiers in import.specifiers.iter() {
        for specifier in specifiers.iter() {
            match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let imported = specifier.imported.name();

                    let local = specifier.local.name;

                    println!("Transforming named import: {} as {}", imported, local);

                    if imported == local {
                        named_bindings.push(imported.to_string());
                    } else {
                        named_bindings.push(format!("{}: {}", imported, local));
                    }
                }

                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let local = specifier.local.name;

                    println!("Transforming default import: {}", local);

                    default_binding = Some(local.to_string());
                }

                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    let local = specifier.local.name;

                    println!("Transforming namespace import: {}", local);

                    return Ok(format!(
                        "const {} = require({:?});",
                        local, dependency.resolved_id
                    ));
                }
            }
        }
    }

    let mut output = String::new();

    if let Some(local) = default_binding {
        output.push_str(&format!(
            "const {} = require({:?}).default;\n",
            local, dependency.resolved_id
        ));
    }

    if !named_bindings.is_empty() {
        output.push_str(&format!(
            "const {{ {} }} = require({:?});",
            named_bindings.join(", "),
            dependency.resolved_id
        ));
    }

    Ok(output.trim_end().to_string())
}
