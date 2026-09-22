use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclaration, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::module::Dependency;

pub fn transform(
    source: &str,
    filename: &str,
    dependencies: &[Dependency],
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
                let transformed = transform_import(import, dependencies)?;

                if !transformed.is_empty() {
                    output.push_str(&transformed);
                    output.push('\n');
                }
            }

            Statement::ExportNamedDeclaration(export) => {
                transform_named_export(source, export, &mut output)?;
            }

            Statement::ExportDefaultDeclaration(export) => {
                transform_default_export(source, export, &mut output)?;
            }

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

            Statement::FunctionDeclaration(function) => {
                let start = function.span.start as usize;
                let end = function.span.end as usize;

                output.push_str(&source[start..end]);
                output.push('\n');
            }

            Statement::ClassDeclaration(class) => {
                let start = class.span.start as usize;
                let end = class.span.end as usize;

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
    dependencies: &[Dependency],
) -> Result<String, String> {
    let source = import.source.value.to_string();

    let dependency = dependencies
        .iter()
        .find(|dependency| dependency.request == source)
        .ok_or_else(|| format!("Dependency '{}' was not resolved", source))?;

    // import "./setup.js";
    //
    // becomes:
    //
    // require("./setup.js");
    if import.specifiers.is_none() {
        return Ok(format!("require({:?});", dependency.resolved_id));
    }

    let mut named_bindings = Vec::new();
    let mut default_binding = None;

    if let Some(specifiers) = &import.specifiers {
        for specifier in specifiers.iter() {
            match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let imported = specifier.imported.name();
                    let local = specifier.local.name;

                    if imported == local {
                        named_bindings.push(imported.to_string());
                    } else {
                        named_bindings.push(format!("{}: {}", imported, local));
                    }
                }

                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let local = specifier.local.name;

                    default_binding = Some(local.to_string());
                }

                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    let local = specifier.local.name;

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

fn transform_named_export(
    _source: &str,
    export: &oxc_ast::ast::ExportNamedDeclaration,
    output: &mut String,
) -> Result<(), String> {
    for specifier in export.specifiers.iter() {
        let local = module_export_name_to_string(&specifier.local);

        let exported = module_export_name_to_string(&specifier.exported);

        output.push_str(&format!("exports.{} = {};\n", exported, local));
    }

    Ok(())
}

fn transform_default_export(
    source: &str,
    export: &oxc_ast::ast::ExportDefaultDeclaration,
    output: &mut String,
) -> Result<(), String> {
    match &export.declaration {
        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            let function_name = function
                .id
                .as_ref()
                .ok_or_else(|| "Anonymous default functions are not supported yet".to_string())?;

            let start = function.span.start as usize;
            let end = function.span.end as usize;

            output.push_str(&source[start..end]);
            output.push_str("\n\n");

            output.push_str(&format!("exports.default = {};\n", function_name.name));
        }

        oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let class_name = class
                .id
                .as_ref()
                .ok_or_else(|| "Anonymous default classes are not supported yet".to_string())?;

            let start = class.span.start as usize;
            let end = class.span.end as usize;

            output.push_str(&source[start..end]);
            output.push_str("\n\n");

            output.push_str(&format!("exports.default = {};\n", class_name.name));
        }

        _ => {
            return Err("Unsupported default export declaration".to_string());
        }
    }

    Ok(())
}

fn module_export_name_to_string(name: &oxc_ast::ast::ModuleExportName) -> String {
    name.name().to_string()
}
