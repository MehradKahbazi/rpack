use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[derive(Debug)]
pub struct ImportInfo {
    pub source: String,
    pub named: Vec<String>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub imports: Vec<ImportInfo>,
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
                let mut named = Vec::new();

                for specifiers in import.specifiers.iter() {
                    for specifier in specifiers.iter() {
                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                named.push(specifier.imported.name().to_string());
                            }

                            _ => {}
                        }
                    }
                }

                imports.push(ImportInfo {
                    source: import.source.value.to_string(),
                    named,
                });
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
