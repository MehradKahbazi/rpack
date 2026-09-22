use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, ImportDeclarationSpecifier, Statement};
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

                            _ => {}
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
                if let Declaration::FunctionDeclaration(function) = &export.declaration {
                    let function_name = function
                        .id
                        .as_ref()
                        .ok_or_else(|| "Exported function has no name".to_string())?;

                    exports.push(Export::Named(function_name.name.to_string()));
                }
            }

            Statement::ExportDefaultDeclaration(export) => {
                // فعلاً فقط default function را پشتیبانی می‌کنیم.
                //
                // ساختار دقیق declaration را در این نسخه
                // از Oxc بررسی می‌کنیم.
                match &export.declaration {
                    oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                        let function_name = function.id.as_ref().ok_or_else(|| {
                            "Anonymous default functions are not supported yet".to_string()
                        })?;

                        exports.push(Export::Default(function_name.name.to_string()));
                    }

                    _ => {
                        return Err("Unsupported default export".to_string());
                    }
                }
            }

            _ => {}
        }
    }

    Ok(ParseResult { imports, exports })
}
