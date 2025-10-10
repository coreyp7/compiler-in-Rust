use super::builder_context::BuilderContext;
use super::statement::{
    FunctionDeclarationStatement, ReturnStatement, Statement, VariableDeclarationStatement,
};

use crate::tokenizer::{Token, TokenType};

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
    match token_type {
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
        _ => {
            // Skip unsupported statements for now
            context.advance();
            (None, context)
        }
    }
}

fn parse_variable_declaration(mut context: BuilderContext) -> (Statement, BuilderContext) {
    // Parse data type
    let data_type = match context.get_curr().lexeme.as_str() {
        "Number" => DataType::Number,
        "String" => DataType::String,
        _ => DataType::Invalid,
    };
    context.advance();

    // Parse identifier
    if context.is_at_end() || context.get_curr().token_type != TokenType::Identity {
        // Return invalid statement for malformed declaration
        let invalid_stmt = Statement::VariableDeclaration(VariableDeclarationStatement {
            symbol_name: String::new(),
            data_type: DataType::Invalid,
            line_declared_on: 0,
            assigned_value: Value::invalid(),
        });
        return (invalid_stmt, context);
    }

    let symbol_name = context.get_curr().lexeme.clone();
    let line_declared_on = context.get_curr().line_number;
    context.advance();

    // Expect colon
    if context.is_at_end() || context.get_curr().token_type != TokenType::Colon {
        // Return invalid statement for malformed declaration
        let invalid_stmt = Statement::VariableDeclaration(VariableDeclarationStatement {
            symbol_name,
            data_type: DataType::Invalid,
            line_declared_on,
            assigned_value: Value::invalid(),
        });
        return (invalid_stmt, context);
    }
    context.advance();

    // Parse the assigned value
    let (value, context) = parse_value(context);

    // Create the variable declaration statement
    let statement = Statement::VariableDeclaration(VariableDeclarationStatement {
        symbol_name,
        data_type,
        line_declared_on,
        assigned_value: value,
    });

    (statement, context)
}

fn parse_function_declaration(mut context: BuilderContext) -> (Statement, BuilderContext) {
    context.advance(); // Skip "Function" keyword

    if context.is_at_end() || context.get_curr().token_type != TokenType::Identity {
        let invalid_stmt = Statement::FunctionDeclaration(FunctionDeclarationStatement {
            function_name: String::new(),
            line_declared_on: 0,
            return_type: DataType::Invalid,
            body: Vec::new(),
        });
        return (invalid_stmt, context);
    }

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
    // At returns now, go forward and get datatype specified for return type.
    context.advance();
    let return_type_lexeme: &str = &context.get_curr().lexeme;
    let mut return_type: DataType = DataType::Invalid;
    match return_type_lexeme {
        "Number" => return_type = DataType::Number,
        "String" => return_type = DataType::String,
        "Void" => return_type = DataType::Void,
        _ => (),
    }

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

    // Skip EndFunction token
    if !context.is_at_end() {
        context.advance();
    }

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
    let (val, returned_context) = parse_value(context);
    context = returned_context;

    // For now, just create a simple return statement
    // TODO: Parse the return value
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

    while context.get_curr().token_type != TokenType::RightParen {
        let (value, ctx) = parse_value(context);
        context = ctx;
        passed_values.push(value);

        if context.get_curr().token_type == TokenType::Comma {
            context.advance();
        }
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

#[derive(Debug, Clone)]
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
