use std::thread::Builder;

use super::builder_context::BuilderContext;
use super::statement::{
    FunctionDeclarationStatement, ReturnStatement, Statement, VariableDeclarationStatement,
};

use crate::ast::VariableAssignmentStatement;
use crate::semantic::SemanticError;
use crate::tokenizer::{Token, TokenType};

/// Helper function to create an invalid statement - used when parsing fails
fn create_invalid_statement() -> Statement {
    Statement::VariableDeclaration(VariableDeclarationStatement {
        symbol_name: String::new(),
        data_type: DataType::Invalid,
        line_declared_on: 0,
        assigned_value: Value::invalid(),
    })
}

/// Macro to handle expected token checking with error recovery (find start
/// of next statement).
/// Current logic; return early from the parsing function without creating a
/// new statement. Before returning, add a parsing error indicating that there
/// was a problem parsing the current statement.
macro_rules! expect_token {
    ($context:expr, $expected:pat, $error_msg:expr) => {
        if $context.is_at_end() {
            $context.handle_parse_error(
                $context.get_curr().line_number,
                format!("Unexpected end of file: {}", $error_msg),
            );
            return (create_invalid_statement(), $context);
        }

        if !matches!($context.get_curr().token_type, $expected) {
            $context.handle_parse_error(
                $context.get_curr().line_number,
                format!("{}, found '{}'", $error_msg, $context.get_curr().lexeme),
            );
            return (create_invalid_statement(), $context);
        }
    };
}

/// Build AST from tokens - pure structural parsing, no validation
pub fn build_ast(tokens: Vec<Token>) -> BuilderContext {
    let context = BuilderContext::new(tokens);
    parse_program(context)
}

fn parse_program(mut context: BuilderContext) -> BuilderContext {
    while !context.is_at_end() {
        let (stmt, returned_context) = parse_statement(context);
        context = returned_context;
        if let Some(statement) = stmt {
            context.statements.push(statement);
        }
    }
    context
}

fn parse_statement(mut context: BuilderContext) -> (Option<Statement>, BuilderContext) {
    if context.is_at_end() {
        return (None, context);
    }

    let token_type = context.get_curr().token_type;
    let return_tuple = match token_type {
        TokenType::VarDeclaration => {
            let (stmt, ctx) = parse_variable_declaration(context);
            (Some(stmt), ctx)
        }
        TokenType::FunctionDeclaration => {
            let (stmt, ctx) = parse_function_declaration(context);
            (Some(stmt), ctx)
        }
        TokenType::Return => {
            let (stmt, ctx) = parse_return_statement(context);
            (Some(stmt), ctx)
        }
        TokenType::Identity => {
            let (stmt, ctx) = parse_identity_assignment_statement(context);
            (Some(stmt), ctx)
        }
        TokenType::EOF => {
            println!("Reached EOF in parse_statement; congrats :)");
            context.advance(); // move passed EOF token
            (None, context)
        }
        //TokenType::Identity => (Some(stmt), ctx),
        _ => {
            // Unexpected token - report error and skip to next statement
            context.handle_parse_error(
                context.get_curr().line_number,
                format!("Unexpected token: '{}'", context.get_curr().lexeme),
            );
            (None, context)
        }
    };

    // TODO: need to confirm that there's a semicolon at the end of this statement.
    // Whatever statement was parsed, we should be left on the semicolon now.
    // (Maybe this is naive so open to change later if you need to)
    // Actually this should probably be done in each individual function huh?
    // Maybe here we can just handle creating the

    /*
    if return_tuple.1.get_curr().token_type != TokenType::Semicolon {
        // TODO throw a thing and handle
        return_tuple
            .1
            .errors
            .push(SemanticError::UnexpectedStatement {
                line: return_tuple.1.get_curr().line_number,
                explanation: "no semicolon found TODO: improve this err".to_string(),
            })
    }
    */

    return_tuple
}

fn parse_variable_declaration(mut context: BuilderContext) -> (Statement, BuilderContext) {
    // Parse data type
    let data_type = match context.get_curr().lexeme.as_str() {
        "Number" => DataType::Number,
        "String" => DataType::String,
        _ => {
            context.handle_parse_error(
                context.get_curr().line_number,
                format!("Invalid data type: '{}'", context.get_curr().lexeme),
            );
            return (create_invalid_statement(), context);
        }
    };
    context.advance();

    // Parse identifier
    expect_token!(
        context,
        TokenType::Identity,
        "Expected variable name after data type"
    );
    let symbol_name = context.get_curr().lexeme.clone();
    let line_declared_on = context.get_curr().line_number;
    context.advance();

    // Expect colon
    expect_token!(
        context,
        TokenType::Colon,
        "Expected ':' after variable name"
    );
    context.advance();

    // Parse the assigned value
    let (value, mut context) = parse_value(context);

    // Expect semicolon
    expect_token!(
        context,
        TokenType::Semicolon,
        "Expected semicolon after variable declaration"
    );
    context.advance();

    // Create the variable declaration statement
    let statement = Statement::VariableDeclaration(VariableDeclarationStatement {
        symbol_name,
        data_type,
        line_declared_on,
        assigned_value: value,
    });

    (statement, context)
}

fn parse_identity_assignment_statement(mut context: BuilderContext) -> (Statement, BuilderContext) {
    // Okay; what could this identity be?
    // - var assignment
    // - i think that's it, since there's no reason to call a function without returning something from it.
    let identity_lexeme = context.get_curr().lexeme.clone();
    let line_number = context.get_curr().line_number;
    context.advance();

    // TODO: confirm that this token is <= (assignment token)
    // For now, just advance assuming it's an assignment operator
    if context.is_at_end() {
        context.handle_parse_error(
            line_number,
            "Expected assignment operator after variable name".to_string(),
        );
        return (create_invalid_statement(), context);
    }
    context.advance();

    let (val, mut context) = parse_value(context);

    // Expect semicolon
    expect_token!(
        context,
        TokenType::Semicolon,
        "Expected semicolon after assignment"
    );
    context.advance();

    let assignent_struct = Statement::VariableAssignment(VariableAssignmentStatement {
        var_name: identity_lexeme,
        var_data_type: DataType::Unknown, // unknown until semantic analysis
        assigned_value: val,
        line_var_was_declared_on: 0, // unknown until semantic analysis
        line_number,
    });

    (assignent_struct, context)
}

fn parse_function_declaration(mut context: BuilderContext) -> (Statement, BuilderContext) {
    let start_line = context.get_curr().line_number;
    context.advance(); // Skip "Function" keyword

    // Expect function name
    expect_token!(
        context,
        TokenType::Identity,
        "Expected function name after 'Function'"
    );
    let function_name = context.get_curr().lexeme.clone();
    let line_declared_on = context.get_curr().line_number;
    context.advance();

    // Skip to return type; we already have the function header in the function map.
    // TODO: put the function header in this struct; would make semantics alot easier
    // to just look in the struct than doing a lookup to a map. But this might be naive,
    // not sure what downsides there'd be.
    while !context.is_at_end() && context.get_curr().token_type != TokenType::Returns {
        context.advance();
    }

    if context.is_at_end() {
        context.handle_parse_error(
            start_line,
            "Expected 'Returns' keyword in function declaration".to_string(),
        );
        return (create_invalid_statement(), context);
    }

    // At returns now, go forward and get datatype specified for return type.
    context.advance();

    if context.is_at_end() {
        context.handle_parse_error(
            start_line,
            "Expected return type after 'Returns'".to_string(),
        );
        return (create_invalid_statement(), context);
    }

    let return_type_lexeme: &str = &context.get_curr().lexeme;
    let return_type = match return_type_lexeme {
        "Number" => DataType::Number,
        "String" => DataType::String,
        "Void" => DataType::Void,
        _ => {
            context.handle_parse_error(
                context.get_curr().line_number,
                format!("Invalid return type: '{}'", return_type_lexeme),
            );
            return (create_invalid_statement(), context);
        }
    };

    if !context.is_at_end() {
        context.advance(); // Skip colon
    }

    // Parse function body statements
    let mut body = Vec::new();
    while !context.is_at_end() && context.get_curr().token_type != TokenType::EndFunction {
        let start_idx = context.idx;

        let (stmt, returned_context) = parse_statement(context);
        context = returned_context;

        if let Some(statement) = stmt {
            body.push(statement);
        }

        // Prevent infinite loop if parse_statement doesn't advance for some reason
        if context.idx == start_idx {
            context.advance();
        }
    }

    // Expect EndFunction token
    if context.is_at_end() {
        context.handle_parse_error(
            start_line,
            "Expected 'EndFunction' to close function declaration".to_string(),
        );
        return (create_invalid_statement(), context);
    }

    // Skip EndFunction token
    context.advance();

    // Return the complete function declaration with body
    let statement = Statement::FunctionDeclaration(FunctionDeclarationStatement {
        function_name,
        line_declared_on,
        return_type,
        body,
    });

    (statement, context)
}

fn parse_return_statement(mut context: BuilderContext) -> (Statement, BuilderContext) {
    let line_declared_on = context.get_curr().line_number;
    context.advance(); // Skip "return" keyword

    // What types of things can be returned?
    // A value I suppose.
    let (val, mut context) = parse_value(context);

    // Expect semicolon
    expect_token!(
        context,
        TokenType::Semicolon,
        "Expected semicolon after return statement"
    );
    context.advance();

    // For now, just create a simple return statement
    let statement = Statement::Return(ReturnStatement {
        line_declared_on,
        return_value: Some(val),
    });

    (statement, context)
}

// Parse a value expression - no validation, just structure
fn parse_value(mut context: BuilderContext) -> (Value, BuilderContext) {
    if context.is_at_end() {
        return (Value::invalid(), context);
    }

    let token = context.get_curr().clone();
    let value = match token.token_type {
        TokenType::Number => Value::new(
            DataType::Number,
            ValueType::InlineNumber,
            token.lexeme.clone(),
        ),
        TokenType::Str => Value::new(
            DataType::String,
            ValueType::InlineString,
            token.lexeme.clone(),
        ),
        TokenType::Identity => {
            // For now, treat all identities as variable references
            // Semantic analysis will determine if they're valid
            /*
            Value {
                data_type: DataType::Unknown, // Will be determined in semantic analysis
                value_type: ValueType::Variable,
                raw_text: token.lexeme.clone(),
            }
            */
            // TODO: update this to allow function calls.
            // Indicate its a function call in the AST node somehow, then check
            // for this in the semantic analysis and check appropriate table.

            /*
             * if just variable name (no parameters), treat as variable.
             * if identity name with parameters, treat as function.
             * TODO: we're gonna need some vars to keep track of possible symbols
             * passed into the param of the possible function call this value can be.
             * Actually, we could store *these* as values, since a passed param can
             * be a Value in itself. This starts a possible recursive process.
             *
             * - add new var that's a vector of values
             * - process each argument "value" and add to list
             * Reminder: DO NOT have any validation logic or semantic analysis in here.
             *
             * The only thing I think that could go wrong here is the parsing of each
             * of the values; how does it know when to stop, will it behave normally?
             * I guess we'll see.
             */
            // Check if this is a function call
            if let Some(next_token) = context.peek_next() {
                if next_token.token_type == TokenType::LeftParen {
                    // This is a function call - we need to handle the parameter parsing
                    // but we'll do it after we advance past the current token
                    // For now, just mark it as a function call and let the calling code handle parameters
                    let raw_text = context.get_curr().lexeme.clone();
                    context.advance();
                    context.advance(); // move passed (

                    let (params, ctx) = parse_function_call_parameters(context);
                    context = ctx;
                    // Context moves passed closing paren after this match; at bottom
                    // of function we advance.

                    Value::new_with_params(
                        DataType::Unknown,
                        ValueType::FunctionCall,
                        raw_text,
                        params,
                    )
                } else {
                    // create Value as a variable call
                    Value::new(DataType::Unknown, ValueType::Variable, token.lexeme.clone())
                }
            } else {
                // actually could just be final statement of the file...
                // should test this TODO: treat as variable for now
                Value::new(DataType::Unknown, ValueType::Variable, token.lexeme.clone())
            }
        }
        _ => Value::new(DataType::Invalid, ValueType::Invalid, token.lexeme.clone()),
    };

    context.advance();
    (value, context)
}

// Called when a function call is found, gathers all values specified in a function
// calls parameters.
fn parse_function_call_parameters(mut context: BuilderContext) -> (Vec<Value>, BuilderContext) {
    // so it'll be value comma value etc....

    let mut passed_values: Vec<Value> = Vec::new();

    while !context.is_at_end() && context.get_curr().token_type != TokenType::RightParen {
        let (value, ctx) = parse_value(context);
        context = ctx;
        passed_values.push(value);

        if context.is_at_end() {
            // Missing closing paren - but we'll let the calling function handle this
            break;
        }

        if context.get_curr().token_type == TokenType::Comma {
            context.advance();
        } else if context.get_curr().token_type != TokenType::RightParen {
            // Expected comma or closing paren
            // For now, just break out and let the calling function handle the error
            break;
        }
    }

    // Skip the closing paren if we found it
    if !context.is_at_end() && context.get_curr().token_type == TokenType::RightParen {
        context.advance();
    }

    (passed_values, context)
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Number,
    String,
    Void,
    Unknown, // Used when type needs to be inferred
    Invalid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    FunctionCall,
    Expression,
    InlineNumber,
    InlineString,
    Variable,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub data_type: DataType,
    pub value_type: ValueType,
    pub raw_text: String, // The raw text from the source, for reference
    // Only exists if value_type = FunctionCall; we need to record the values
    // being passed in as params.
    pub param_values: Option<Vec<Value>>,
}

impl Value {
    pub fn new(data_type: DataType, value_type: ValueType, raw_text: String) -> Self {
        Value {
            data_type,
            value_type,
            raw_text,
            param_values: None,
        }
    }

    pub fn new_with_params(
        data_type: DataType,
        value_type: ValueType,
        raw_text: String,
        param_values: Vec<Value>,
    ) -> Self {
        Value {
            data_type,
            value_type,
            raw_text,
            param_values: Some(param_values),
        }
    }

    pub fn invalid() -> Self {
        Value::new(DataType::Invalid, ValueType::Invalid, String::new())
    }
}
