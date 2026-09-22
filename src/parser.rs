use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ExportFromDeclaration, ImportDeclarationSpecifier, Statement,
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
pub enum ReExport {
    Named {
        source: String,
        imported: String,
        exported: String,
    },

    Namespace {
        source: String,
    },
}

#[derive(Debug)]
pub struct ParseResult {
    pub imports: Vec<ImportInfo>,
    pub re_exports: Vec<ReExport>,
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
    let mut re_exports = Vec::new();
    let mut exports = Vec::new();

    for statement in &result.program.body {
        match statement {
            // =========================================================
            // import ...
            // =========================================================
            Statement::ImportDeclaration(import) => {
                let mut named = Vec::new();
                let mut default = None;

                if let Some(specifiers) = &import.specifiers {
                    for specifier in specifiers.iter() {
                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                named.push(specifier.imported.name().to_string());
                            }

                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                default = Some(specifier.local.name.to_string());
                            }

                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                                // Namespace imports do not need
                                // export validation here.
                            }
                        }
                    }
                }

                imports.push(ImportInfo {
                    source: import.source.value.to_string(),
                    named,
                    default,
                });
            }

            // =========================================================
            // export const/function/class ...
            // =========================================================
            Statement::ExportDeclaration(export) => {
                parse_export_declaration(&export.declaration, &mut exports)?;
            }

            // =========================================================
            // export { foo };
            // export { foo as bar };
            // =========================================================
            Statement::ExportNamedDeclaration(export) => {
                for specifier in export.specifiers.iter() {
                    parse_export_specifier(specifier, &mut exports)?;
                }
            }

            // =========================================================
            // export { foo } from "./foo.js";
            // export { foo as bar } from "./foo.js";
            // export { default as foo } from "./foo.js";
            // =========================================================
            Statement::ExportAllDeclaration(export) => {
                re_exports.push(ReExport::Namespace {
                    source: export.source.value.to_string(),
                });
            }
            Statement::ExportFromDeclaration(export) => {
                parse_re_export(export, &mut re_exports)?;
            }

            // =========================================================
            // export default ...
            // =========================================================
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

    Ok(ParseResult {
        imports,
        re_exports,
        exports,
    })
}

// =====================================================================
// export const / let / var
// export function
// export class
// =====================================================================

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

            let name = name.name.to_string();

            exports.push(Export::Named {
                local: name.clone(),
                exported: name,
            });
        }

        Declaration::ClassDeclaration(class) => {
            let name = class
                .id
                .as_ref()
                .ok_or_else(|| "Exported class has no name".to_string())?;

            let name = name.name.to_string();

            exports.push(Export::Named {
                local: name.clone(),
                exported: name,
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

// =====================================================================
// export { foo };
// export { foo as bar };
// =====================================================================

fn parse_export_specifier(
    specifier: &oxc_ast::ast::ExportSpecifier,
    exports: &mut Vec<Export>,
) -> Result<(), String> {
    let local = module_export_name_to_string(&specifier.local);

    let exported = module_export_name_to_string(&specifier.exported);

    exports.push(Export::Named { local, exported });

    Ok(())
}

// =====================================================================
// export { foo } from "./foo.js";
// export { foo as bar } from "./foo.js";
// export { default as foo } from "./foo.js";
// export * from "./foo.js";
// =====================================================================

fn parse_re_export(
    export: &ExportFromDeclaration,
    re_exports: &mut Vec<ReExport>,
) -> Result<(), String> {
    let source = export.source.value.to_string();

    // -------------------------------------------------------------
    // export * from "./math.js";
    // -------------------------------------------------------------

    if export.specifiers.is_empty() {
        re_exports.push(ReExport::Namespace { source });

        return Ok(());
    }

    // -------------------------------------------------------------
    // export { foo } from "./math.js";
    // export { foo as bar } from "./math.js";
    // -------------------------------------------------------------

    for specifier in export.specifiers.iter() {
        let imported = module_export_name_to_string(&specifier.local);

        let exported = module_export_name_to_string(&specifier.exported);

        re_exports.push(ReExport::Named {
            source: source.clone(),
            imported,
            exported,
        });
    }

    Ok(())
}

// =====================================================================
// ModuleExportName -> String
// =====================================================================

fn module_export_name_to_string(name: &oxc_ast::ast::ModuleExportName) -> String {
    name.name().to_string()
}
