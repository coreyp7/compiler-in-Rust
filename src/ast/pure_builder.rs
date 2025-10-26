use std::thread::Builder;

use super::builder_context::BuilderContext;
use super::parse_error::ParseError;
use super::statement::*;
use super::statement::{
    FunctionDeclarationStatement, ReturnStatement, Statement, VariableDeclarationStatement,
};

use crate::ast::comparison::{
    ComparisonOperator, ExpressionOperator, LogicalOperator, Term, TermOperator, Unary,
    convert_token_type_to_expression_op, convert_token_type_to_term_op,
};
use crate::ast::{Expression, VariableAssignmentStatement};
use crate::tokenizer::{Token, TokenType};

/// Helper function to create an invalid statement - used when parsing fails
fn create_invalid_statement() -> Statement {
    Statement::VariableDeclaration(VariableDeclarationStatement {
        symbol_name: String::new(),
        data_type: DataType::Invalid,
        line_declared_on: 0,
        //assigned_value: Value::invalid(),
        assigned_expr: Expression::new(),
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
            $context.handle_parse_error(ParseError::UnexpectedEndOfFile {
                line: $context.get_curr().line_number,
                expected: $error_msg.to_string(),
            });
            return (create_invalid_statement(), $context);
        }

        if !matches!($context.get_curr().token_type, $expected) {
            $context.handle_parse_error(ParseError::UnexpectedToken {
                line: $context.get_curr().line_number,
                expected: $error_msg.to_string(),
                found: $context.get_curr().lexeme.clone(),
            });
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
        _ => {
            // Unexpected token - report error and skip to next statement
            context.handle_parse_error(ParseError::UnexpectedToken {
                line: context.get_curr().line_number,
                expected: "statement".to_string(),
                found: context.get_curr().lexeme.clone(),
            });
            (None, context)
        }
    };

    return_tuple
}

fn parse_variable_declaration(mut context: BuilderContext) -> (Statement, BuilderContext) {
    // Parse data type
    let data_type = match context.get_curr().lexeme.as_str() {
        "Number" => DataType::Number,
        "String" => DataType::String,
        _ => {
            context.handle_parse_error(ParseError::InvalidDataType {
                line: context.get_curr().line_number,
                data_type: context.get_curr().lexeme.clone(),
            });
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
    let (expr, mut context) = parse_expression(context);

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
        //assigned_value: value,
        assigned_expr: expr,
    });

    (statement, context)
}

fn parse_expression(mut context: BuilderContext) -> (Expression, BuilderContext) {
    let mut expr = Expression::new();
    let (term1, returned_context) = parse_term(context);
    context = returned_context;
    expr.terms.push(term1);

    //while self.is_curr_token_expression_operator() {
    while !context.is_at_end()
        && matches!(
            context.get_curr().token_type,
            TokenType::Plus | TokenType::Minus
        )
    {
        let op = convert_token_type_to_expression_op(context.get_curr().token_type.clone());
        expr.operators.push(op);
        context.advance();

        let (term2, returned_context2) = parse_term(context);
        context = returned_context2;
        expr.terms.push(term2);
    }

    (expr, context)
}

fn parse_term(mut context: BuilderContext) -> (Term, BuilderContext) {
    let mut term = Term::new();
    let (unary1, returned_context) = parse_unary(context);
    context = returned_context;
    term.unarys.push(unary1);

    while !context.is_at_end()
        && matches!(
            context.get_curr().token_type,
            TokenType::Asterisk | TokenType::Slash
        )
    {
        let op = convert_token_type_to_term_op(context.get_curr().token_type);
        term.operations.push(op);
        context.advance();

        let (unary2, returned_context2) = parse_unary(context);
        context = returned_context2;
        term.unarys.push(unary2);
    }

    (term, context)
}

fn parse_unary(mut context: BuilderContext) -> (Unary, BuilderContext) {
    let mut operation = None;

    // Check for unary operators (+ or -)
    if !context.is_at_end()
        && matches!(
            context.get_curr().token_type,
            TokenType::Plus | TokenType::Minus
        )
    {
        operation = Some(convert_token_type_to_expression_op(
            context.get_curr().token_type,
        ));
        context.advance();
    }

    // Parse the primary value
    let (primary, returned_context) = parse_value(context);
    context = returned_context;

    let unary = Unary { operation, primary };

    (unary, context)
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
        context.handle_parse_error(ParseError::MissingAssignmentOperator { line: line_number });
        return (create_invalid_statement(), context);
    }
    context.advance();

    //let (val, mut context) = parse_value(context);
    let (expr, mut context) = parse_expression(context);

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
        //assigned_value: val,
        assigned_expr: expr,
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
        context.handle_parse_error(ParseError::MissingKeyword {
            line: start_line,
            keyword: "Returns".to_string(),
            context: "function declaration".to_string(),
        });
        return (create_invalid_statement(), context);
    }

    // At returns now, go forward and get datatype specified for return type.
    context.advance();

    if context.is_at_end() {
        context.handle_parse_error(ParseError::UnexpectedEndOfFile {
            line: start_line,
            expected: "return type after 'Returns'".to_string(),
        });
        return (create_invalid_statement(), context);
    }

    let return_type_lexeme: &str = &context.get_curr().lexeme;
    let return_type = match return_type_lexeme {
        "Number" => DataType::Number,
        "String" => DataType::String,
        "Void" => DataType::Void,
        _ => {
            context.handle_parse_error(ParseError::InvalidReturnType {
                line: context.get_curr().line_number,
                return_type: return_type_lexeme.to_string(),
            });
            return (create_invalid_statement(), context);
        }
    };

    if !context.is_at_end() {
        context.advance(); // Skip return type (we've already confirmed its there)
    }

    expect_token!(
        context,
        TokenType::Colon,
        "Expected delimiter colon after function declaration"
    );
    context.advance();

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
        context.handle_parse_error(ParseError::UnterminatedFunctionDeclaration {
            line: start_line,
            function_name: function_name.clone(),
        });
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

// Called when a function call is found, gathers all expressions specified in a function
// calls parameters.
fn parse_function_call_parameters(mut context: BuilderContext) -> (Vec<Expression>, BuilderContext) {
    // so it'll be expression comma expression etc....

    let mut passed_expressions: Vec<Expression> = Vec::new();

    while !context.is_at_end() && context.get_curr().token_type != TokenType::RightParen {
        let (expr, ctx) = parse_expression(context);
        context = ctx;
        passed_expressions.push(expr);

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
    // This was causing problems when parsing values, leaving for posterity
    // if this comes up again later
    /*
    if !context.is_at_end() && context.get_curr().token_type == TokenType::RightParen {
        context.advance();
    }
    */

    (passed_expressions, context)
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
    // Only exists if value_type = FunctionCall; we need to record the expressions
    // being passed in as params.
    pub param_values: Option<Vec<Expression>>,
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
        param_values: Vec<Expression>,
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
