mod convert_statement;

use crate::ast::{FunctionTable, Statement};
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
        /*
        code_str.push_str(&format!(
            "{} {});\n",
            function_def.return_type.to_string(),
            function_def.identifier
        ));
        */
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
    }

    // Start main function
    code_str.push_str("int main() {\n");

    // Generate code for each statement (with indentation)
    for statement in ast_vec {
        // NOTE: this is temporary, should be generic in the future.
        code_str.push_str(&statement.to_code_str());
        /*
        match statement {
            Statement::VariableDeclaration(struc) => {
                code_str.push_str("    "); // 4-space indentation
                code_str.push_str(&struc.to_code_str());
                code_str.push_str("\n");
            }
            _ => {}
        }
        */
    }

    // Close main function
    code_str.push_str("    return 0;\n");
    code_str.push_str("}\n");

    code_str
}
