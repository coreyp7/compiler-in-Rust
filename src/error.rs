use crate::ast::VarType;
use crate::tokenizer::TokenType;
use colored::Colorize;
use std::str::FromStr;

// TODO: move this into a different crate, why tf is this here
#[derive(Debug, Clone)]
pub enum ErrMsg {
    UnexpectedToken {
        expected: TokenType,
        got: TokenType,
        line_number: u8, //col_number: usize
    },
    IncorrectTypeAssignment {
        expected_type: VarType,
        got_type: VarType,
        line_number: u8, //col_number: usize
    },
    VariableAlreadyDeclared {
        identity: String,
        first_declared_line: u8,
        redeclared_line: u8,
    },
    VariableNotDeclared {
        identity: String,
        attempted_assignment_line: u8,
    },
}

impl ErrMsg {
    pub fn new_incorrect_type_assignment(
        expected_type: VarType,
        got_type: VarType,
        line_number: u8, //col_number: usize
    ) -> ErrMsg {
        ErrMsg::IncorrectTypeAssignment {
            expected_type,
            got_type,
            line_number, //col_number: col_number
        }
    }

    pub fn new_unexpected_token(
        expected: TokenType,
        got: TokenType,
        line_number: u8, //col_number: usize
    ) -> ErrMsg {
        ErrMsg::UnexpectedToken {
            expected,
            got,
            line_number,
        }
    }

    pub fn print_error(&self) {
        let red_error_text = "Error:".red().bold();
        let blue_arrow = "-->".blue().bold();
        match self {
            ErrMsg::UnexpectedToken {
                expected,
                got,
                line_number,
            } => {
                println!("{} Unexpected token", red_error_text);
                println!(
                    "  {} Line {}",
                    blue_arrow,
                    line_number.to_string().yellow().bold()
                );
                println!("  Expected: '{}'", expected.to_string());
                println!("  Got: '{}'", got.to_string());
            }
            ErrMsg::IncorrectTypeAssignment {
                expected_type,
                got_type,
                line_number,
            } => {
                println!("{} Type mismatch", red_error_text);
                println!(
                    "  {} Line {}",
                    blue_arrow,
                    line_number.to_string().yellow().bold()
                );
                println!("  Expected type: {:?}", expected_type);
                println!("  Got type: {:?}", got_type);
            }
            ErrMsg::VariableAlreadyDeclared { 
                identity, 
                first_declared_line, 
                redeclared_line 
            } => {
                println!(
                    "{} Variable '{}' is already declared",
                    red_error_text, identity
                );
                println!("  First declared on line {}", first_declared_line);
                println!("  Redeclaration attempted on line {}", redeclared_line);
            }
            ErrMsg::VariableNotDeclared {
                identity,
                attempted_assignment_line,
            } => {
                println!("{} Variable '{}' not declared", red_error_text, identity);
                println!(
                    "  {} Line {}",
                    blue_arrow,
                    attempted_assignment_line.to_string().yellow().bold()
                );
                println!("  Variable '{}' must be declared before use", identity);
            }
        }
    }
}

pub fn print_all_errors(errors: &[ErrMsg]) {
    for (i, error) in errors.iter().enumerate() {
        error.print_error();
        if i < errors.len() - 1 {
            println!(); // Add spacing between errors
        }
    }
}
