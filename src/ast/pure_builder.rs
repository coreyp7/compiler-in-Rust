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
    let mut context = BuilderContext::new(tokens);
    parse_program(&mut context);
    context
}

fn parse_program(context: &mut BuilderContext) {
    while !context.is_at_end() {
        parse_statement(context);
    }
}

fn parse_statement(context: &mut BuilderContext) {
    if context.is_at_end() {
        return;
    }

    let token_type = context.get_curr().token_type;
    match token_type {
        TokenType::VarDeclaration => {
            parse_variable_declaration(context);
        }
        TokenType::FunctionDeclaration => {
            parse_function_declaration(context);
        }
        TokenType::Return => {
            parse_return_statement(context);
        }
        TokenType::EndFunction => {
            parse_end_of_function(context);
        }
        _ => {
            // Skip unsupported statements for now
            context.advance();
        }
    }
}

fn parse_variable_declaration(context: &mut BuilderContext) {
    // Parse data type
    let data_type = match context.get_curr().lexeme.as_str() {
        "Number" => DataType::Number,
        "String" => DataType::String,
        _ => DataType::Invalid,
    };
    context.advance();

    // Parse identifier
    if context.is_at_end() || context.get_curr().token_type != TokenType::Identity {
        // Skip malformed declaration
        return;
    }

    let symbol_name = context.get_curr().lexeme.clone();
    let line_declared_on = context.get_curr().line_number;
    context.advance();

    // Expect colon
    if context.is_at_end() || context.get_curr().token_type != TokenType::Colon {
        // Skip malformed declaration
        return;
    }
    context.advance();

    // Parse the assigned value
    let value = parse_value(context);

    // Create the variable declaration statement
    let statement = Statement::VariableDeclaration(VariableDeclarationStatement {
        symbol_name,
        data_type,
        line_declared_on,
        assigned_value: value,
    });
    context.statements.push(statement);
}

fn parse_function_declaration(context: &mut BuilderContext) {
    context.advance(); // Skip "Function" keyword

    if context.is_at_end() || context.get_curr().token_type != TokenType::Identity {
        return;
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

    // Parse function body - for now we just skip to the end
    while !context.is_at_end() && context.get_curr().token_type != TokenType::EndFunction {
        let start_idx = context.idx;
        parse_statement(context);

        // Prevent infinite loop if parse_statement doesn't advance
        if context.idx == start_idx {
            context.advance();
        }
    }
    let statement = Statement::FunctionDeclaration(FunctionDeclarationStatement {
        function_name,
        line_declared_on,
        return_type: DataType::Void, // Default for now
    });
    context.statements.push(statement);
}

fn parse_return_statement(context: &mut BuilderContext) {
    context.advance(); // Skip "return" keyword

    let line_declared_on = context.get_curr().line_number;

    // For now, just create a simple return statement
    // TODO: Parse the return value
    let statement = Statement::Return(ReturnStatement {
        line_declared_on,
        return_value: None,
    });
    context.statements.push(statement);
}

fn parse_end_of_function(context: &mut BuilderContext) {
    context.advance(); // Skip "endFunction" keyword
}

/// Parse a value expression - no validation, just structure
fn parse_value(context: &mut BuilderContext) -> Value {
    if context.is_at_end() {
        return Value {
            data_type: DataType::Invalid,
            value_type: ValueType::Invalid,
            raw_text: String::new(),
        };
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
    value
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
