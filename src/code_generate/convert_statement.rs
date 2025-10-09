use crate::ast::{DataType, ReturnStatement, Value, VariableDeclarationStatement};
use crate::ast::{FunctionDeclarationStatement, FunctionSymbol, Statement, ValueType};
use std::fmt;

// Implement Display for DataType so we can call .to_string() on it
// TODO: move this shit it should not be here
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Number => write!(f, "int"),
            DataType::String => write!(f, "char*"),
            DataType::Void => write!(f, "void"),
            DataType::Unknown => write!(f, "auto"),
            DataType::Invalid => write!(f, "/* invalid type */"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value_type {
            ValueType::InlineNumber => write!(f, "{}", &self.raw_text),
            ValueType::InlineString => write!(f, "\"{}\"", &self.raw_text),
            ValueType::Variable => write!(f, "{}", &self.raw_text),
            ValueType::FunctionCall => {
                // NOTE:
                // We need to be given the statement struct of this
                // value for us to properly include arguments.
                // This may jsut workout thoughl
                // For now leaving this naive impl, will come back later.
                write!(f, "{}", &self.raw_text)
            }
            _ => {
                write!(f, "NOT IMPLEMENTED YET")
            }
        }
    }
}

pub fn to_code_str(statement: &Statement) -> String {
    match statement {
        Statement::FunctionDeclaration(func_decl_st) => {
            // This case is handled outside of the function, and this should never
            // be reached. returning empty string.
            String::new()
        }
        Statement::VariableDeclaration(var_decl_st) => to_code_str_var_decl(var_decl_st),
        Statement::Return(return_statement) => to_code_str_return(return_statement),
    }
}

fn to_code_str_var_decl(var_decl: &VariableDeclarationStatement) -> String {
    format!(
        "{} {} = {};\n",
        var_decl.data_type.to_string(),
        var_decl.symbol_name,
        var_decl.assigned_value.to_string()
    )
}

fn to_code_str_return(_return_stmt: &ReturnStatement) -> String {
    "RETURN NOT IMPL YET\n".to_string()
}

pub fn to_code_str_func_decl_stmt(
    func_stmt: &FunctionDeclarationStatement,
    function_def: &FunctionSymbol,
) -> String {
    let mut code = String::new();

    code.push_str(&convert_function_header_to_code_str(function_def));

    code.push_str("{\n");

    // Generate code for function body
    for statement in &func_stmt.body {
        code.push_str("   ");
        // WARNING: this will break I think if a function is declared in a function.
        code.push_str(&to_code_str(statement));
    }

    // Close function
    code.push_str("}\n");
    code
}

pub fn convert_function_header_to_code_str(function_def: &FunctionSymbol) -> String {
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
    code_str.push_str(")");

    code_str
}
