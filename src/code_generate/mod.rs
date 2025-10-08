mod convert_statement;

use crate::ast::Statement;
pub use convert_statement::GenerateCode;

/**
 * Converts an AST into c code equivalent (in the form of a string).
 */
pub fn generate_code_str(ast_vec: &Vec<Statement>) -> String {
    let mut code_str = String::new();

    // Add C headers
    code_str.push_str("#include <stdio.h>\n");
    code_str.push_str("#include <stdlib.h>\n");
    code_str.push_str("#include <string.h>\n");
    code_str.push_str("\n");

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
