use crate::ast::ValueType;
use crate::ast::{DataType, Statement, Value, VariableDeclarationStatement};
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

pub trait GenerateCode {
    fn to_code_str(&self) -> String;
}

impl GenerateCode for VariableDeclarationStatement {
    fn to_code_str(&self) -> String {
        format!(
            "{} {} = {};",
            self.data_type.to_string(),
            self.symbol_name,
            self.assigned_value.to_string()
        )
    }
}
