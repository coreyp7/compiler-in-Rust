mod convert_statement;

use crate::ast::{FunctionDeclarationStatement, FunctionSymbol, FunctionTable, Statement};
pub use convert_statement::GenerateCode;

/**
 * Converts an AST into c code equivalent (in the form of a string).
 */
pub fn generate_code_str(ast_vec: &Vec<Statement>, function_defs: &FunctionTable) -> String {
    let mut code_str = String::new();

    // Add C headers
    code_str.push_str("#include <stdio.h>\n");
    code_str.push_str("#include <stdlib.h>\n");
    code_str.push_str("#include <string.h>\n");
    code_str.push_str("\n");

    // Include function headers that are user declared
    for function_def in function_defs.get_all_defs() {
        code_str.push_str(&convert_function_header_to_code_str(function_def));
    }

    code_str.push_str("int main() {\n");

    let mut func_declaration_statements: Vec<&FunctionDeclarationStatement> = Vec::new();

    // Generate code for each statement (with indentation)
    for statement in ast_vec {
        match statement {
            // Function declarations are put below main in the c file, so just
            // save it for later. (function header already defined above main)
            Statement::FunctionDeclaration(func_decl_statement) => {
                func_declaration_statements.push(func_decl_statement);
            }
            _ => {
                code_str.push_str(&statement.to_code_str());
            }
        }
    }

    code_str.push_str("    return 0;\n");
    code_str.push_str("}\n");

    // TODO: put all function declarations down here.
    for func_decl in func_declaration_statements {
        code_str.push_str(&func_decl.to_code_str());
    }

    code_str
}

fn convert_function_header_to_code_str(function_def: &FunctionSymbol) -> String {
    let mut code_str = String::new();
    code_str.push_str(&format!(
        "{} {}(",
        function_def.return_type.to_string(),
        function_def.identifier
    ));

    // Params
    for (i, param) in function_def.parameters.iter().enumerate() {
        code_str.push_str(&format!("{} {}", param.data_type, param.name));
        if i < function_def.parameters.len() - 1 {
            code_str.push_str(", ");
        }
    }
    code_str.push_str(");\n");

    code_str
}
