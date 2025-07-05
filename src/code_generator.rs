use crate::ast::Statement;

pub fn generate_code_str(ast: &Vec<Statement>) -> String {
    let mut code_str: String = String::new();

    for statement in ast {
        let statement_as_str = convert_statement_to_code(&statement);
        code_str.push_str(&statement_as_str);
    }
    code_str
}

fn convert_statement_to_code(statement: &Statement) -> String {
    let statement_code_str: String = match statement {
        Statement::Print(statement_struct) => convert_print_statement_to_code(statement_struct),
        Statement::If(statement_struct) => convert_if_statement_to_code(statement_struct),
        Statement::While(statement_struct) => convert_while_statement_to_code(statement_struct),
        Statement::Assignment(statement_struct) => {
            convert_assignment_statement_to_code(statement_struct)
        }
        Statement::Instantiation(statement_struct) => {
            convert_instantiation_statement_to_code(statement_struct)
        }
        Statement::Newline => convert_newline_to_code(),
        Statement::TestStub => String::from("// TestStub\n"),
    };

    statement_code_str
}

fn convert_print_statement_to_code(statement_struct: &crate::ast::PrintStatement) -> String {
    let mut code = String::new();
    let content = &statement_struct.content;
    let is_content_identity_name = statement_struct.is_content_identity_name;
    code.push_str("print(");
    if !is_content_identity_name {
        code.push_str("\"");
    }
    code.push_str(&content.clone());
    if !is_content_identity_name {
        code.push_str("\"");
    }
    code.push_str(");");
    code
}

fn convert_if_statement_to_code(statement_struct: &crate::ast::IfStatement) -> String {
    let mut code = String::new();
    code.push_str("if (");
    // TODO: convert logical shit in function later
    code.push_str("/* logical condition */");
    code.push_str(") {\n");

    // Convert nested statements
    for stmt in &statement_struct.statements {
        let stmt_code = convert_statement_to_code(stmt);
        code.push_str(&stmt_code);
    }

    code.push_str("}\n");
    code
}

fn convert_while_statement_to_code(statement_struct: &crate::ast::WhileStatement) -> String {
    let mut code = String::new();
    code.push_str("while (");
    // TODO: implement logical
    code.push_str("/* logical */");
    code.push_str(") {\n");

    // Convert nested statements
    for stmt in &statement_struct.statements {
        let stmt_code = convert_statement_to_code(stmt);
        code.push_str(&stmt_code);
    }

    code.push_str("}\n");
    code
}

fn convert_assignment_statement_to_code(
    statement_struct: &crate::ast::AssignmentStatement,
) -> String {
    let mut code = String::new();
    code.push_str(&statement_struct.identity);
    code.push_str(" = ");

    // Handle different value types
    match statement_struct.assigned_value_type {
        crate::ast::VarType::Str => {
            code.push_str("\"");
            code.push_str(&statement_struct.value);
            code.push_str("\"");
        }
        crate::ast::VarType::Num => {
            code.push_str(&statement_struct.value);
        }
        crate::ast::VarType::Unrecognized => {
            code.push_str(&statement_struct.value);
        }
    }

    code.push_str(";\n");
    code
}

fn convert_instantiation_statement_to_code(
    statement_struct: &crate::ast::InstantiationStatement,
) -> String {
    let mut code = String::new();

    // Add C type declaration
    match statement_struct.var_type {
        crate::ast::VarType::Str => code.push_str("char* "),
        crate::ast::VarType::Num => code.push_str("int "),
        crate::ast::VarType::Unrecognized => code.push_str("/* unknown type */ "),
    }

    code.push_str(&statement_struct.identity);
    code.push_str(" = ");

    // Handle different value types
    match statement_struct.assigned_value_type {
        crate::ast::VarType::Str => {
            code.push_str("\"");
            code.push_str(&statement_struct.value);
            code.push_str("\"");
        }
        crate::ast::VarType::Num => {
            code.push_str(&statement_struct.value);
        }
        crate::ast::VarType::Unrecognized => {
            code.push_str(&statement_struct.value);
        }
    }

    code.push_str(";\n");
    code
}

fn convert_newline_to_code() -> String {
    String::from("\n")
}
