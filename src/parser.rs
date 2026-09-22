use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ExportSpecifier, ImportDeclarationSpecifier, ModuleExportName, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::module::Export;

#[derive(Debug)]
pub struct ImportInfo {
    pub source: String,
    pub named: Vec<String>,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<Export>,
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
                let mut named = Vec::new();
                let mut default = None;

                for specifiers in import.specifiers.iter() {
                    for specifier in specifiers.iter() {
                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                named.push(specifier.imported.name().to_string());
                            }

                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                default = Some(specifier.local.name.to_string());
                            }

                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
                        }
                    }
                }

                imports.push(ImportInfo {
                    source: import.source.value.to_string(),
                    named,
                    default,
                });
            }

            Statement::ExportDeclaration(export) => {
                parse_export_declaration(&export.declaration, &mut exports)?;
            }

            Statement::ExportNamedDeclaration(export) => {
                for specifier in export.specifiers.iter() {
                    parse_export_specifier(specifier, &mut exports)?;
                }
            }

            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    let function_name = function.id.as_ref().ok_or_else(|| {
                        "Anonymous default functions are not supported yet".to_string()
                    })?;

                    exports.push(Export::Default(function_name.name.to_string()));
                }

                oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    let class_name = class.id.as_ref().ok_or_else(|| {
                        "Anonymous default classes are not supported yet".to_string()
                    })?;

                    exports.push(Export::Default(class_name.name.to_string()));
                }

                _ => {
                    return Err("Unsupported default export".to_string());
                }
            },

            _ => {}
        }
    }

    Ok(ParseResult { imports, exports })
}

fn parse_export_declaration(
    declaration: &Declaration,
    exports: &mut Vec<Export>,
) -> Result<(), String> {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            let name = function
                .id
                .as_ref()
                .ok_or_else(|| "Exported function has no name".to_string())?;

            exports.push(Export::Named {
                local: name.name.to_string(),
                exported: name.name.to_string(),
            });
        }

        Declaration::ClassDeclaration(class) => {
            let name = class
                .id
                .as_ref()
                .ok_or_else(|| "Exported class has no name".to_string())?;

            exports.push(Export::Named {
                local: name.name.to_string(),
                exported: name.name.to_string(),
            });
        }

        Declaration::VariableDeclaration(declaration) => {
            for declarator in declaration.declarations.iter() {
                let name = match &declarator.id {
                    oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) => {
                        identifier.name.to_string()
                    }

                    _ => {
                        return Err("Destructuring exports are not supported yet".to_string());
                    }
                };

                exports.push(Export::Named {
                    local: name.clone(),
                    exported: name,
                });
            }
        }

        _ => {
            return Err("Unsupported named export declaration".to_string());
        }
    }

    Ok(())
}

fn parse_export_specifier(
    specifier: &oxc_ast::ast::ExportSpecifier,
    exports: &mut Vec<Export>,
) -> Result<(), String> {
    let local = module_export_name_to_string(&specifier.local);
    let exported = module_export_name_to_string(&specifier.exported);

    exports.push(Export::Named { local, exported });

    Ok(())
}

fn module_export_name_to_string(name: &oxc_ast::ast::ModuleExportName) -> String {
    name.name().to_string()
}
