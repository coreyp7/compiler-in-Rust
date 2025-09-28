use super::function_table::FunctionTable;
use crate::symbol_table::SymbolTable;
use crate::tokenizer::{Token, TokenType};

pub struct BuilderContext {
    /// Symbol table for variable symbols only
    pub symbol_table: SymbolTable,
    /// Function table for function symbols only
    pub function_table: FunctionTable,
    tokens: Vec<Token>,
    idx: usize,
    pub statements: Vec<Statement>,
}

impl BuilderContext {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            function_table: FunctionTable::new(),
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
            TokenType::VarDeclaration => {
                context = parse_variable_declaration(context);
            }
            TokenType::FunctionDeclaration => {
                context = parse_function_declaration(context);
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

fn parse_function_declaration(mut context: BuilderContext) -> BuilderContext {
    // Skip 'func' or function keyword
    context.idx += 1;

    // Get function name
    if context.idx >= context.tokens.len() || context.get_curr().token_type != TokenType::Identity {
        // Error: expected identifier
        // NOTE: Better error handling needed
    }

    let function_name = context.get_curr().lexeme.clone();
    let line_declared_on = context.get_curr().line_number;
    context.idx += 1;

    // For now, assume no parameters and return type is Number
    // TODO: Parse actual parameters and return type
    let parameters = Vec::new();
    let return_type = DataType::Void;

    // Insert into function table
    let function_key = context.function_table.insert(
        &function_name,
        parameters,
        return_type.clone(),
        &line_declared_on,
    );

    // Create the function declaration statement
    let statement = Statement::FunctionDeclaration(FunctionDeclarationStatement {
        function_key: function_key.unwrap(), // FIXME: handle properly
        return_type,
        line_declared_on,
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

    let token_type = context.get_curr().token_type;
    let token_lexume = &context.get_curr().lexeme;

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
            // First check if it's a variable
            let variable_symbol_key = context.symbol_table.get_id_with_symbol_name(&token_lexume);
            if let Some(var_key) = variable_symbol_key {
                if let Some(variable_symbol) = context.symbol_table.get_using_id(var_key) {
                    let data_type = variable_symbol.data_type.clone();
                    context.idx += 1;
                    return (
                        context,
                        Value {
                            data_type,
                            value_type: ValueType::Variable,
                            variable_symbol_key: Some(var_key),
                            function_symbol_key: None,
                            comparison: None,
                        },
                    );
                }
            }

            // If not a variable, check if it's a function
            let function_symbol_key = context
                .function_table
                .get_id_with_function_name(&token_lexume);
            if let Some(func_key) = function_symbol_key {
                if let Some(function_symbol) = context.function_table.get_using_id(func_key) {
                    let return_type = function_symbol.return_type.clone();
                    context.idx += 1;
                    return (
                        context,
                        Value {
                            data_type: return_type,
                            value_type: ValueType::FunctionCall,
                            variable_symbol_key: None,
                            function_symbol_key: Some(func_key),
                            comparison: None,
                        },
                    );
                }
            }

            // If neither variable nor function, return invalid
            Value {
                data_type: DataType::Invalid,
                value_type: ValueType::Variable,
                variable_symbol_key: None,
                function_symbol_key: None,
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
    Void,
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
    FunctionDeclaration(FunctionDeclarationStatement),
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
pub struct FunctionDeclarationStatement {
    /// Key to lookup function in function table
    pub function_key: u8,
    pub return_type: DataType,
    pub line_declared_on: u32,
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
