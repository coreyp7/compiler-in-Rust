use crate::ast::DataType;

/// Represents different types of semantic errors
#[derive(Debug, Clone)]
pub enum SemanticError {
    VariableNotDeclared {
        name: String,
        line: u32,
    },
    VariableAlreadyDeclared {
        name: String,
        first_line: u32,
        redeclaration_line: u32,
    },
    TypeMismatch {
        expected: DataType,
        found: DataType,
        line: u32,
    },
    FunctionNotDeclared {
        name: String,
        line: u32,
    },
    InvalidValueReference {
        name: String,
        line: u32,
    },
}

impl SemanticError {
    pub fn print_error(&self) {
        match self {
            SemanticError::VariableNotDeclared { name, line } => {
                eprintln!("Error: Variable '{}' not declared (line {})", name, line);
            }
            SemanticError::VariableAlreadyDeclared {
                name,
                first_line,
                redeclaration_line,
            } => {
                eprintln!("Error: Variable '{}' already declared", name);
                eprintln!("  First declared on line {}", first_line);
                eprintln!("  Redeclared on line {}", redeclaration_line);
            }
            SemanticError::TypeMismatch {
                expected,
                found,
                line,
            } => {
                eprintln!("Error: Type mismatch on line {}", line);
                eprintln!("  Expected: {:?}", expected);
                eprintln!("  Found: {:?}", found);
            }
            SemanticError::FunctionNotDeclared { name, line } => {
                eprintln!("Error: Function '{}' not declared (line {})", name, line);
            }
            SemanticError::InvalidValueReference { name, line } => {
                eprintln!("Error: Invalid reference to '{}' (line {})", name, line);
            }
        }
    }
}
