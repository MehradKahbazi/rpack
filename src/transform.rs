use oxc_ast::ast::Statement;

pub fn transform(source: &str, statements: &[Statement]) -> Result<String, String> {
    let mut output = source.to_string();

    for statement in statements {
        if let Statement::ExportDeclaration(export) = statement {
            println!("Transforming export...");

            match &export.declaration {
                oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                    let function = function
                        .id
                        .as_ref()
                        .ok_or_else(|| "Exported function has no name".to_string())?;

                    println!("Exported function: {}", function.name);
                }

                _ => {
                    println!("Other export");
                }
            }
        }
    }

    Ok(output)
}
