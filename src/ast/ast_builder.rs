use crate::tokenizer::{Token, TokenType};

pub fn build_ast(tokens: &Vec<Token>) -> Vec<Statement> {
    //, Vec<ErrMsg>, SymbolTable) {
    /*
    let mut context = ParseContext {
        tokens,
        curr_idx: 0,
        errors: Vec::new(),
        symbol_table: SymbolTable::new(),
    };
    */

    let statements = parse_program(tokens);
    statements
}

fn parse_program(tokens: &Vec<Token>) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut token_idx = 0;

    while token_idx < tokens.len() {
        let token = &tokens[token_idx];
        match token.token_type {
            // TODO: if first token is datatype, this is a variable declaration
            TokenType::VarDeclaration => {
                if let Some((statement, new_idx)) = parse_variable_declaration(tokens, token_idx) {
                    statements.push(statement);
                    token_idx = new_idx;
                } else {
                    token_idx += 1; // Skip on error
                }
            }
            _ => {
                // Any statements not implemented yet will be skipped.
                token_idx += 1;
            }
        }
    }

    statements
}

fn parse_variable_declaration(tokens: &Vec<Token>, curr: usize) -> Option<(Statement, usize)> {
    let mut idx = curr;

    let data_type = match &tokens[idx].lexeme.as_str() {
        &"Number" => DataType::Number,
        &"String" => DataType::String,
        _ => return None, // Error: invalid data type
    };
    idx += 1;
    println!("Detected datatype is: {:?}", data_type);

    // Get symbol (identifier)
    if idx >= tokens.len() || tokens[idx].token_type != TokenType::Identity {
        return None; // Error: expected identifier
    }

    let _symbol_name = &tokens[idx].lexeme; // We'll use this for symbol table later
    // the symbol key will be given to you by the map impl when you insert it
    let line_declared_on = tokens[idx].line_number;
    idx += 1;

    if idx >= tokens.len() || tokens[idx].token_type != TokenType::Colon {
        return None; // Error: expected colon
    }
    idx += 1;

    // Process value with 'parse_value' function call
    // TODO: here we need to ensure that the datatype of the assigned value
    // matches the datatype of the variable being declared. Well, do we?
    // That should be done in a semantic analysis phase, not here. that's what
    // happened last time and it sucked. So, just analyze what the return type
    // is and put it into our structs.
    if let Some((value, new_idx)) = parse_value(tokens, idx) {
        idx = new_idx;

        // Create the variable declaration statement
        let statement = Statement::VariableDeclaration(VariableDeclarationStatement {
            symbol_key: 3, //TODO: BAD BAD need to figure out how to assign this when adding
            // to the symbol table
            data_type,
            line_declared_on,
            // This is here so in semantic phase, we check that the types match
            asssigned_value_data_type: value.data_type,
        });

        Some((statement, idx))
    } else {
        None // Error parsing value
    }
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
fn parse_value(tokens: &Vec<Token>, curr: usize) -> Option<(Value, usize)> {
    let mut idx = curr;

    if idx >= tokens.len() {
        return None;
    }

    let token = &tokens[idx];

    match token.token_type {
        TokenType::Number => {
            // Parse inline number value
            let value = Value {
                data_type: DataType::Number,
                value_type: ValueType::InlineNumber,
                variable_symbol_key: None,
                function_symbol_key: None,
                //inline_value: token.lexeme.parse().ok(), // Convert string to number
                comparison: None,
            };
            Some((value, idx + 1))
        }
        TokenType::Str => {
            // Parse inline string value
            let value = Value {
                data_type: DataType::String,
                value_type: ValueType::InlineString,
                variable_symbol_key: None,
                function_symbol_key: None,
                // NOTE: do we even need to keep track of values?
                // As long as we have the datatype, who cares?
                //inline_value: Some(token.lexeme),
                comparison: None,
            };
            Some((value, idx + 1))
        }
        TokenType::Identity => {
            // TODO: skipping for now, but this needs to be tested.
            // Could be a variable reference or function call
            // Will have to lookup in the map(s) for these.
            // Now I think having one map with two different enum variants would
            // make sense (function headers and variables).
            let value = Value {
                data_type: DataType::Number, // We'd need symbol table lookup to determine actual type
                value_type: ValueType::Variable,
                variable_symbol_key: Some(0), // Placeholder - should lookup in symbol table
                function_symbol_key: None,
                //inline_value: None,
                comparison: None,
            };
            Some((value, idx + 1))
        }
        _ => {
            // Unsupported value type for now
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataType {
    Number,
    String,
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
