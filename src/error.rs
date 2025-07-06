use crate::ast::VarType;
use crate::tokenizer::TokenType;
use colored::Colorize;
use std::str::FromStr;

// TODO: move this into a different crate, why tf is this here
#[derive(Debug)]
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
        //line_number: u8
        identity: String,
    },
    VariableNotDeclared {
        identity: String,
        attempted_assignment_line: u8,
    },
}

impl ErrMsg {
    pub fn new_incorrect_type_assignment(
        expected: VarType,
        got: VarType,
        line_number: u8, //col_number: usize
    ) -> ErrMsg {
        ErrMsg::IncorrectTypeAssignment {
            expected_type: expected,
            got_type: got,
            line_number: line_number, //col_number: col_number
        }
    }

    pub fn new_unexpected_token(
        expected: TokenType,
        got: TokenType,
        line_number: u8, //col_number: usize
    ) -> ErrMsg {
        ErrMsg::UnexpectedToken {
            expected: expected,
            got: got,
            line_number: line_number,
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
            ErrMsg::VariableAlreadyDeclared { identity } => {
                println!(
                    "{} Variable '{}' is already declared",
                    red_error_text, identity
                );
                println!("  Cannot redeclare variable '{}'", identity);
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

pub fn print_all_errors(errors: &Vec<ErrMsg>) {
    for (i, error) in errors.iter().enumerate() {
        error.print_error();
        if i < errors.len() - 1 {
            println!(); // Add spacing between errors
        }
    }
}
