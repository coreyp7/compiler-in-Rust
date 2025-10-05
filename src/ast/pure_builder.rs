use crate::tokenizer::{Token, TokenType};

/// Simple context for AST building - only tracks parsing state
pub struct BuilderContext {
    tokens: Vec<Token>,
    idx: usize,
    pub statements: Vec<Statement>,
}

impl BuilderContext {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            idx: 0,
            statements: Vec::new(),
        }
    }

    pub fn get_curr(&self) -> &Token {
        &self.tokens[self.idx]
    }

    pub fn is_at_end(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    pub fn advance(&mut self) {
        if !self.is_at_end() {
            self.idx += 1;
        }
    }

    pub fn peek_next(&self) -> Option<&Token> {
        if self.idx + 1 < self.tokens.len() {
            Some(&self.tokens[self.idx + 1])
        } else {
            None
        }
    }
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
        TokenType::EndFunction => {
            let ctx = parse_end_of_function(context);
            (None, ctx)
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
            assigned_value: Value {
                data_type: DataType::Invalid,
                value_type: ValueType::Invalid,
                raw_text: String::new(),
            },
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
            assigned_value: Value {
                data_type: DataType::Invalid,
                value_type: ValueType::Invalid,
                raw_text: String::new(),
            },
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

    // Skip to colon (simplified parsing for now)
    while !context.is_at_end() && context.get_curr().token_type != TokenType::Colon {
        context.advance();
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
        return_type: DataType::Void, // Default for now
        body,
    });

    (statement, context)
}

fn parse_return_statement(mut context: BuilderContext) -> (Statement, BuilderContext) {
    context.advance(); // Skip "return" keyword

    let line_declared_on = if !context.is_at_end() {
        context.get_curr().line_number
    } else {
        0
    };

    // For now, just create a simple return statement
    // TODO: Parse the return value
    let statement = Statement::Return(ReturnStatement {
        line_declared_on,
        return_value: None,
    });

    (statement, context)
}

fn parse_end_of_function(mut context: BuilderContext) -> BuilderContext {
    context.advance(); // Skip "endFunction" keyword
    context
}

/// Parse a value expression - no validation, just structure
fn parse_value(mut context: BuilderContext) -> (Value, BuilderContext) {
    if context.is_at_end() {
        let invalid_value = Value {
            data_type: DataType::Invalid,
            value_type: ValueType::Invalid,
            raw_text: String::new(),
        };
        return (invalid_value, context);
    }

    let token = context.get_curr();
    let value = match token.token_type {
        TokenType::Number => Value {
            data_type: DataType::Number,
            value_type: ValueType::InlineNumber,
            raw_text: token.lexeme.clone(),
        },
        TokenType::Str => Value {
            data_type: DataType::String,
            value_type: ValueType::InlineString,
            raw_text: token.lexeme.clone(),
        },
        TokenType::Identity => {
            // For now, treat all identities as variable references
            // Semantic analysis will determine if they're valid
            Value {
                data_type: DataType::Unknown, // Will be determined in semantic analysis
                value_type: ValueType::Variable,
                raw_text: token.lexeme.clone(),
            }
        }
        _ => Value {
            data_type: DataType::Invalid,
            value_type: ValueType::Invalid,
            raw_text: token.lexeme.clone(),
        },
    };

    context.advance();
    (value, context)
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

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration(VariableDeclarationStatement),
    FunctionDeclaration(FunctionDeclarationStatement),
    Return(ReturnStatement),
}

#[derive(Debug)]
pub struct VariableDeclarationStatement {
    pub symbol_name: String,
    pub data_type: DataType,
    pub line_declared_on: u32,
    pub assigned_value: Value,
}

#[derive(Debug)]
pub struct FunctionDeclarationStatement {
    pub function_name: String,
    pub return_type: DataType,
    pub line_declared_on: u32,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub line_declared_on: u32,
    pub return_value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub data_type: DataType,
    pub value_type: ValueType,
    pub raw_text: String, // The raw text from the source, for reference
}
