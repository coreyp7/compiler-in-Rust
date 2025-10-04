use crate::{
    ast::DataType,
    symbol_table::SymbolTable,
    tokenizer::{Token, TokenType},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FunctionHeader {
    pub identifier: String,
    pub parameters: Vec<Parameter>,
    pub return_type: DataType,
    pub line_declared_on: u32,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub data_type: DataType,
}

/**
 * Returns a map containing all function definitions defined in a
 * tokenized plank file.
 */
pub fn gather_declarations(tokens: &[Token]) -> HashMap<String, FunctionHeader> {
    let mut function_headers = HashMap::new();
    let mut idx = 0;

    while idx < tokens.len() {
        let token = &tokens[idx];

        if token.token_type == TokenType::FunctionDeclaration {
            let (function_header, new_idx) = parse_function_declaration(&tokens, idx);
            function_headers.insert(function_header.identifier.clone(), function_header);
            idx = new_idx;
        } else {
            idx += 1;
        }
    }

    function_headers
}

// TODO: need to add more security / failure handling in here.
fn parse_function_declaration(tokens: &[Token], mut idx: usize) -> (FunctionHeader, usize) {
    idx += 1; // skip function keyword

    let function_name = tokens[idx].lexeme.clone();
    idx += 1;

    idx += 1; // skip '(', need to enforce this later

    let mut params = Vec::new();
    if idx < tokens.len() && tokens[idx].token_type == TokenType::VarDeclaration {
        let (parameters, new_idx) = parse_function_parameters(tokens, idx);
        params = parameters;
        idx = new_idx;
    } else {
        println!(
            "The current token isn't what we expect for params. TODO: return early with err or something"
        );
        println!("{:#?}", tokens[idx]);
    }

    idx += 1; // skip ')', this should be moved into the above function

    idx += 1; // skip 'returns' keyword, should be validated first

    // Convert string type to DataType enum
    let function_return_type = match tokens[idx].lexeme.as_str() {
        "Number" => DataType::Number,
        "String" => DataType::String,
        "Void" => DataType::Void,
        _ => DataType::Invalid,
    };
    idx += 1;

    let header = FunctionHeader {
        identifier: function_name,
        parameters: params,
        return_type: function_return_type,
        line_declared_on: tokens[idx - 1].line_number,
    };

    (header, idx)
}

// TODO: need to add more security / failure handling in here.
fn parse_function_parameters(tokens: &[Token], mut idx: usize) -> (Vec<Parameter>, usize) {
    let mut parameters = Vec::new();

    while idx < tokens.len() && tokens[idx].token_type != TokenType::RightParen {
        if tokens[idx].token_type != TokenType::VarDeclaration {
            println!("Why isn't the next token a var declaration? Not good.");
            idx += 1;
            continue;
        }

        let param_type = match tokens[idx].lexeme.as_str() {
            "Number" => DataType::Number,
            "String" => DataType::String,
            "Void" => DataType::Void,
            _ => DataType::Invalid,
        };
        idx += 1;

        if idx >= tokens.len() {
            break;
        }

        let param_name = tokens[idx].lexeme.clone();
        idx += 1;
        println!("param name: {}", param_name);

        parameters.push(Parameter {
            name: param_name,
            data_type: param_type,
        });
        println!("after adding param: {:#?}", parameters);

        if idx < tokens.len() && tokens[idx].token_type == TokenType::Comma {
            idx += 1; // skip comma, resume loop on next param var declaration
        }
    }

    (parameters, idx)
}
