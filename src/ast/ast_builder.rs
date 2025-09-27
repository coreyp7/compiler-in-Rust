use std::thread::Builder;

use crate::symbol_table::SymbolTable;
use crate::tokenizer::{Token, TokenType};

pub struct ast_builder {
    /**
     * This symbol table could contain either:
     * - variable
     * - function header
     * not sure yet TODO: still figuring this out
     */
    symbol_table: SymbolTable,
}

pub struct BuilderContext {
    /**
     * This symbol table could contain either:
     * - variable
     * - function header
     * not sure yet TODO: still figuring this out
     */
    pub symbol_table: SymbolTable,
    tokens: Vec<Token>,
    idx: usize,
    pub statements: Vec<Statement>,
}

impl BuilderContext {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            tokens,
            idx: 0,
            statements: Vec::new(),
        }
    }

    pub fn get_curr(&self) -> &Token {
        &self.tokens[self.idx]
    }
}

pub fn build_ast(tokens: Vec<Token>) -> BuilderContext {
    //, Vec<ErrMsg>, SymbolTable) {
    /*
    let mut context = ParseContext {
        tokens,
        curr_idx: 0,
        errors: Vec::new(),
        symbol_table: SymbolTable::new(),
    };
    */
    let context = BuilderContext::new(tokens);
    let updated_context = parse_program(context);
    updated_context
}

// I think context is better because of how the Rust language works.
// Having it in a class makes it difficult to split up functions easily without
// having a very large class, because it'd be dependent on the state being in scope.
// With the context param impl, we can have separate modules for processing different
// types of statements, and can just pass the state around through context variables.
//
// Ownership shouldn't be an issue, because its a linear process.
// When parse_program is done updating the context, it can return permission to
// the caller.
fn parse_program(mut context: BuilderContext) -> BuilderContext {
    //let mut statements = Vec::new();
    //let mut token_idx = 0;
    //let token_vec_len = context.tokens.len();

    while context.idx < context.tokens.len() {
        let token_type = context.get_curr().token_type;
        println!("Tokentype in top of loop: {:?}", token_type);
        match token_type {
            // TODO: if first token is datatype, this is a variable declaration
            TokenType::VarDeclaration => {
                // TODO: return ownership of context back here
                context = parse_variable_declaration(context);
            }
            _ => {
                // Any statements not implemented yet will be skipped.
                context.idx += 1;
            }
        }
    }

    context
}

fn parse_variable_declaration(mut context: BuilderContext) -> BuilderContext {
    //let mut idx = curr;

    let data_type = match &context.get_curr().lexeme.as_str() {
        &"Number" => DataType::Number,
        &"String" => DataType::String,
        _ => DataType::Invalid, // Is this lazy?
    };
    context.idx += 1;

    // Get symbol (identifier)
    if context.idx >= context.tokens.len() || context.get_curr().token_type != TokenType::Identity {
        // Error: expected identifier
        // NOTE: not sure what to do here.
    }

    let symbol_name = &context.get_curr().lexeme.clone(); // We'll use this for symbol table later
    let line_declared_on = &context.get_curr().line_number.clone();
    context.idx += 1;

    // Add to symbol table, get key for variable header
    // TODO: ? is kinda lazy, maybe add better error handling.
    // If something went wrong, there may possibly be a naming collision, which
    // I guess would have to be handled here. Or 'number of declarations' could be
    // added to the map, and analyzed later in the semantic analysis phase.
    let symbol_key = context
        .symbol_table
        .insert(symbol_name, &data_type, line_declared_on);

    if context.idx >= context.tokens.len() || context.get_curr().token_type != TokenType::Colon {
        // NOTE: not sure what to do here.
    }
    context.idx += 1;

    // Process value with 'parse_value' function call
    // TODO: here we need to ensure that the datatype of the assigned value
    // matches the datatype of the variable being declared. Well, do we?
    // That should be done in a semantic analysis phase, not here. that's what
    // happened last time and it sucked. So, just analyze what the return type
    // is and put it into our structs.
    let assigned_value = parse_value(context);
    context = assigned_value.0;
    let value = assigned_value.1;

    // Create the variable declaration statement
    let statement = Statement::VariableDeclaration(VariableDeclarationStatement {
        symbol_key: symbol_key.unwrap(), // FIXME: handle properly upstairs
        // to the symbol table
        data_type,
        line_declared_on: *line_declared_on,
        // This is here so in semantic phase, we check that the types match
        asssigned_value_data_type: value.data_type,
    });
    context.statements.push(statement);

    context
}

/**
 * A value can be a few things:
 * - primitive inline (String/Number)
 * - variable
 * - function call
 * - expression (operations with primitives/vars, excluding conditional ops)
 * Syntax of statement:
 * Type symbol_name: value
 */
fn parse_value(mut context: BuilderContext) -> (BuilderContext, Value) {
    //let mut idx = curr;

    /* TODO: this kinda stuff should be handled in the BuilderContext functions.
    if idx >= tokens.len() {
        return None;
    }
    */

    let token_type = context.get_curr().token_type;

    let value = match token_type {
        TokenType::Number => {
            // Parse inline number value
            Value {
                data_type: DataType::Number,
                value_type: ValueType::InlineNumber,
                variable_symbol_key: None,
                function_symbol_key: None,
                //inline_value: token.lexeme.parse().ok(), // Convert string to number
                comparison: None,
            }
        }
        TokenType::Str => {
            // Parse inline string value
            Value {
                data_type: DataType::String,
                value_type: ValueType::InlineString,
                variable_symbol_key: None,
                function_symbol_key: None,
                // NOTE: do we even need to keep track of values?
                // As long as we have the datatype, who cares?
                //inline_value: Some(token.lexeme),
                comparison: None,
            }
        }
        TokenType::Identity => {
            // TODO: skipping for now, but this needs to be tested.
            // Could be a variable reference or function call
            // Will have to lookup in the map(s) for these.
            // Now I think having one map with two different enum variants would
            // make sense (function headers and variables).
            Value {
                data_type: DataType::Number, // We'd need symbol table lookup to determine actual type
                value_type: ValueType::Variable,
                variable_symbol_key: Some(0), // Placeholder - should lookup in symbol table
                function_symbol_key: None,
                //inline_value: None,
                comparison: None,
            }
        }
        _ => {
            // Unsupported value type for now
            // TODO: make this better?
            Value {
                data_type: DataType::Invalid, // We'd need symbol table lookup to determine actual type
                value_type: ValueType::InlineNumber,
                variable_symbol_key: None, // Placeholder - should lookup in symbol table
                function_symbol_key: None,
                comparison: None,
            }
        }
    };

    context.idx += 1;
    (context, value)
}

#[derive(Debug, Clone)]
pub enum DataType {
    Number,
    String,
    Invalid,
}

#[derive(Debug, Clone)]
pub enum ValueType {
    FunctionCall,
    Expression,
    InlineNumber,
    InlineString,
    Variable,
}

// NOTE: just a stub for now
#[derive(Debug, Clone)]
pub struct Comparison {}

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration(VariableDeclarationStatement),
}

#[derive(Debug)]
pub struct VariableDeclarationStatement {
    // Lookup to symbol table identifier
    pub symbol_key: u8,
    pub data_type: DataType,
    pub line_declared_on: u32,
    pub asssigned_value_data_type: DataType,
}

#[derive(Debug)]
pub struct VariableSymbol {
    pub identifier: String,
    pub data_type: DataType,
    pub line_declared_on: u32,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub data_type: DataType,
    pub value_type: ValueType,
    // These will be in separate maps I believe.
    // Could be made into one, but like having separation for simplicity.
    pub variable_symbol_key: Option<u8>,
    // Use this key to obtain the function header being called here
    // from the function map. That way ownership is given to the map.
    pub function_symbol_key: Option<u8>,
    //pub inline_value: Option<u8>,
    pub comparison: Option<Comparison>,
}
