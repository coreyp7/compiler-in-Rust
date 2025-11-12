use crate::ast::{DataType, FunctionSymbol};
use colored::*;

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
    ReturnMissing {
        funct_name: String,
        func_declared_on_line: u32,
    },
    ReturnTypeIncorrect {
        func_def: FunctionSymbol,
        got_type: DataType,
        line: u32,
    },
    IncorrectParameters {
        parameters_expected: usize,
        parameters_provided: usize,
        line: u32,
    },
    ExpressionInvalid {
        line: u32,
    },
    // To be more helpful to the user, have a catered one for return stmts
    ExpressionInvalidExpectingSpecificType {
        line: u32,
        expected_type: DataType,
    },
    /// First created this for when returns are found outside of functions.
    /// Helpful for more generic errors thatr we can just write a sentence for.
    UnexpectedStatement {
        line: u32,
        explanation: String,
    },
    ComparisonInvalid {
        line: u32,
        first_expr_datatype: DataType,
        got: DataType,
    },
}

// Helper functions for formatting error messages
fn error_header(title: &str, line: u32) {
    eprintln!(
        "{} {} (line {})",
        "✗ Error:".bold().red(),
        title.bold(),
        format_line(line)
    );
}

fn error_line_start() -> ColoredString {
    "┌─".cyan()
}

fn error_line_middle() -> ColoredString {
    "├─".cyan()
}

fn error_line_end() -> ColoredString {
    "└─".cyan()
}

fn format_name(name: &str) -> ColoredString {
    name.yellow().bold()
}

fn format_line(line: u32) -> ColoredString {
    line.to_string().blue().bold()
}

fn format_type(data_type: &DataType) -> ColoredString {
    format!("{:?}", data_type).green().bold()
}

fn format_type_error(data_type: &DataType) -> ColoredString {
    format!("{:?}", data_type).red().bold()
}

pub fn print_success_message() {
    let message = format!("Semantic analysis passed successfully!").green();
    println!("{}", message);
}
pub fn print_failures_message(error_count: usize) {
    let message = format!("{} errors found during compilation:", error_count).red();
    eprintln!("-----------------------------------");
    eprintln!("{}", message);
    eprintln!("-----------------------------------");
}

impl SemanticError {
    pub fn print_error(&self) {
        match self {
            SemanticError::VariableNotDeclared { name, line } => {
                error_header("Variable not declared", *line);
                eprintln!(
                    "  {} Variable '{}' is not declared",
                    error_line_start(),
                    format_name(name)
                );
            }
            SemanticError::VariableAlreadyDeclared {
                name,
                first_line,
                redeclaration_line,
            } => {
                error_header("Variable redeclaration", *redeclaration_line);
                eprintln!(
                    "  {} Variable '{}' is already declared",
                    error_line_start(),
                    format_name(name)
                );
                eprintln!(
                    "  {} First declared at line {}",
                    error_line_middle(),
                    format_line(*first_line)
                );
            }
            SemanticError::TypeMismatch {
                expected,
                found,
                line,
            } => {
                error_header("Type mismatch", *line);
                eprintln!(
                    "  {} Expected: {}",
                    error_line_start(),
                    format_type(expected)
                );
                eprintln!(
                    "  {} Found:    {}",
                    error_line_end(),
                    format_type_error(found)
                );
            }
            SemanticError::FunctionNotDeclared { name, line } => {
                error_header("Function not declared", *line);
                eprintln!(
                    "  {} Function '{}' is not declared",
                    error_line_start(),
                    format_name(name)
                );
            }
            SemanticError::InvalidValueReference { name, line } => {
                error_header("Invalid reference", *line);
                eprintln!(
                    "  {} Invalid reference to '{}'",
                    error_line_start(),
                    format_name(name)
                );
            }
            SemanticError::ReturnMissing {
                funct_name,
                func_declared_on_line,
            } => {
                error_header("Function missing return statement", *func_declared_on_line);
                eprintln!(
                    "{} Function '{}' missing return statement",
                    error_line_start(),
                    funct_name,
                )
            }
            SemanticError::IncorrectParameters {
                parameters_expected,
                parameters_provided,
                line,
            } => {
                error_header("Incorrect number of parameters", *line);
                eprintln!(
                    "  {} Expected: {} parameters",
                    error_line_start(),
                    parameters_expected.to_string().green().bold()
                );
                eprintln!(
                    "  {} Found:    {} parameters",
                    error_line_end(),
                    parameters_provided.to_string().red().bold()
                );
            }
            SemanticError::ReturnTypeIncorrect {
                func_def,
                got_type,
                line,
            } => {
                error_header("Incorrect return type", *line);
                eprintln!(
                    "  {} Return statement in function '{}' is of incorrect type.",
                    error_line_start(),
                    func_def.identifier
                );
                eprintln!(
                    "  {} Expected a {:#?}, got a {:#?}.",
                    error_line_end(),
                    func_def.return_type,
                    got_type
                );
            }
            SemanticError::ExpressionInvalidExpectingSpecificType {
                line,
                expected_type,
            } => {
                error_header("Expression type is invalid", *line);
                eprintln!(
                    "  {} Expression evaluates to invalid type. Are you adding different types together?",
                    error_line_start(),
                );
                eprintln!(
                    "  {} Was expecting type '{:#?}'",
                    error_line_end(),
                    expected_type
                );
            }
            SemanticError::ExpressionInvalid { line } => {
                error_header("Expression type is invalid", *line);
                eprintln!(
                    "  {} Expression evaluates to invalid type. ",
                    error_line_start(),
                );
            }
            SemanticError::UnexpectedStatement { line, explanation } => {
                error_header("Unexpected statement found", *line);
                eprintln!("  {} {}", error_line_start(), explanation);
            }
            SemanticError::ComparisonInvalid {
                line,
                first_expr_datatype,
                got,
            } => {
                error_header("Comparison invalid", *line);
                eprintln!(
                    "  {} First expression in comparison is a '{:#?}'",
                    error_line_start(),
                    first_expr_datatype
                );
                eprintln!(
                    "  {} Instead got expresion of type '{:#?}'",
                    error_line_end(),
                    got
                );
            }
        }
    }
}
