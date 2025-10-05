use crate::ast::DataType;
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
}

// Helper functions for formatting error messages
fn error_header(title: &str) {
    eprintln!("{} {}", "✗ Semantic Error:".bold().red(), title.bold());
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
    let message = format!("Semantic analysis passed successfully! ✓").green();
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
                error_header("Variable not declared");
                eprintln!(
                    "  {} Variable '{}' is not declared",
                    error_line_start(),
                    format_name(name)
                );
                eprintln!("  {} at line {}", error_line_end(), format_line(*line));
            }
            SemanticError::VariableAlreadyDeclared {
                name,
                first_line,
                redeclaration_line,
            } => {
                error_header("Variable redeclaration");
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
                eprintln!(
                    "  {} Redeclared at line {}",
                    error_line_end(),
                    format_line(*redeclaration_line)
                );
            }
            SemanticError::TypeMismatch {
                expected,
                found,
                line,
            } => {
                error_header("Type mismatch");
                eprintln!("  {} at line {}", error_line_start(), format_line(*line));
                eprintln!(
                    "  {} Expected: {}",
                    error_line_middle(),
                    format_type(expected)
                );
                eprintln!(
                    "  {} Found:    {}",
                    error_line_end(),
                    format_type_error(found)
                );
            }
            SemanticError::FunctionNotDeclared { name, line } => {
                error_header("Function not declared");
                eprintln!(
                    "  {} Function '{}' is not declared",
                    error_line_start(),
                    format_name(name)
                );
                eprintln!("  {} at line {}", error_line_end(), format_line(*line));
            }
            SemanticError::InvalidValueReference { name, line } => {
                error_header("Invalid reference");
                eprintln!(
                    "  {} Invalid reference to '{}'",
                    error_line_start(),
                    format_name(name)
                );
                eprintln!("  {} at line {}", error_line_end(), format_line(*line));
            }
        }
    }
}
