use colored::*;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        line: u32,
        expected: String,
        found: String,
    },
    UnexpectedEndOfFile {
        line: u32,
        expected: String,
    },
    InvalidDataType {
        line: u32,
        data_type: String,
    },
    InvalidReturnType {
        line: u32,
        return_type: String,
    },
    MissingAssignmentOperator {
        line: u32,
    },
    MissingKeyword {
        line: u32,
        keyword: String,
        context: String,
    },
    MissingSemicolon {
        line: u32,
        statement_type: String,
    },
    MissingColon {
        line: u32,
        context: String,
    },
    MissingDelimiter {
        line: u32,
        delimiter: String,
        context: String,
    },
    UnterminatedFunctionDeclaration {
        line: u32,
        function_name: String,
    },
    UnterminatedIfStatement {
        line: u32,
    },
}

impl ParseError {
    pub fn line_number(&self) -> u32 {
        match self {
            ParseError::UnexpectedToken { line, .. } => *line,
            ParseError::UnexpectedEndOfFile { line, .. } => *line,
            ParseError::InvalidDataType { line, .. } => *line,
            ParseError::InvalidReturnType { line, .. } => *line,
            ParseError::MissingAssignmentOperator { line } => *line,
            ParseError::MissingKeyword { line, .. } => *line,
            ParseError::MissingSemicolon { line, .. } => *line,
            ParseError::MissingColon { line, .. } => *line,
            ParseError::MissingDelimiter { line, .. } => *line,
            ParseError::UnterminatedFunctionDeclaration { line, .. } => *line,
            ParseError::UnterminatedIfStatement { line } => *line,
        }
    }

    pub fn print_error(&self) {
        match self {
            ParseError::UnexpectedToken {
                line,
                expected,
                found,
            } => {
                error_header("Unexpected token", *line);
                eprintln!(
                    "  {} Expected: {}",
                    error_line_start(),
                    expected.green().bold()
                );
                eprintln!(
                    "  {} Found:    {}",
                    error_line_end(),
                    format_token_error(found)
                );
            }
            ParseError::UnexpectedEndOfFile { line, expected } => {
                error_header("Unexpected end of file", *line);
                eprintln!(
                    "  {} Expected: {}",
                    error_line_start(),
                    expected.green().bold()
                );
                eprintln!(
                    "  {} Found:    {}",
                    error_line_end(),
                    "end of file".red().bold()
                );
            }
            ParseError::InvalidDataType { line, data_type } => {
                error_header("Invalid data type", *line);
                eprintln!(
                    "  {} Invalid data type: {}",
                    error_line_start(),
                    format_token_error(data_type)
                );
                eprintln!(
                    "  {} Valid types are: {}, {}",
                    error_line_end(),
                    format_token("Number"),
                    format_token("String")
                );
            }
            ParseError::InvalidReturnType { line, return_type } => {
                error_header("Invalid return type", *line);
                eprintln!(
                    "  {} Invalid return type: {}",
                    error_line_start(),
                    format_token_error(return_type)
                );
                eprintln!(
                    "  {} Valid return types are: {}, {}, {}",
                    error_line_end(),
                    format_token("Number"),
                    format_token("String"),
                    format_token("Void")
                );
            }
            ParseError::MissingAssignmentOperator { line } => {
                error_header("Missing assignment operator", *line);
                eprintln!(
                    "  {} Expected assignment operator after variable name",
                    error_line_start()
                );
            }
            ParseError::MissingKeyword {
                line,
                keyword,
                context,
            } => {
                error_header("Missing keyword", *line);
                eprintln!(
                    "  {} Expected keyword {} in {}",
                    error_line_start(),
                    format_token(keyword),
                    context.italic()
                );
            }
            ParseError::MissingSemicolon {
                line,
                statement_type,
            } => {
                error_header("Missing semicolon", *line);
                eprintln!(
                    "  {} Expected semicolon after {}",
                    error_line_start(),
                    statement_type.italic()
                );
            }
            ParseError::MissingColon { line, context } => {
                error_header("Missing colon", *line);
                eprintln!(
                    "  {} Expected colon in {}",
                    error_line_start(),
                    context.italic()
                );
            }
            ParseError::MissingDelimiter {
                line,
                delimiter,
                context,
            } => {
                error_header("Missing delimiter", *line);
                eprintln!(
                    "  {} Expected {} in {}",
                    error_line_start(),
                    format_token(delimiter),
                    context.italic()
                );
            }
            ParseError::UnterminatedFunctionDeclaration {
                line,
                function_name,
            } => {
                error_header("Unterminated function declaration", *line);
                eprintln!(
                    "  {} Function {} is missing closing 'EndFunction'",
                    error_line_start(),
                    format_name(function_name)
                );
            }
            ParseError::UnterminatedIfStatement { line } => {
                error_header("Unterminated if statement", *line);
                eprintln!(
                    "  {} If statement is missing closing 'EndIf'",
                    error_line_start()
                );
            }
        }
    }
}

// Helper functions for formatting error messages
fn error_header(title: &str, line: u32) {
    eprintln!(
        "{} {} (line {})",
        "✗ Parse Error:".bold().red(),
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

fn format_token(token: &str) -> ColoredString {
    format!("'{}'", token).green().bold()
}

fn format_token_error(token: &str) -> ColoredString {
    format!("'{}'", token).red().bold()
}

pub fn print_success_message() {
    let message = format!("Parsing completed successfully! ✓").green();
    println!("{}", message);
}

pub fn print_failures_message(error_count: usize) {
    let message = format!("{} parsing errors found:", error_count).red();
    eprintln!("-----------------------------------");
    eprintln!("{}", message);
    eprintln!("-----------------------------------");
}
