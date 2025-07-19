use std::collections::HashMap;
use std::str::FromStr;

// My stuff
use crate::comparison::*;
use crate::error::ErrMsg;
use crate::expression_parser::ExpressionParser;
use crate::parser::{ParserContext, StatementParserCoordinator};
use crate::statement::Statement;
use crate::symbol_table::SymbolTable;
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;

pub struct AstBuilder {
    tokens: Vec<Token>,
    curr_idx: usize,
    errors: Vec<ErrMsg>,
    pub symbol_table: SymbolTable,
}

impl AstBuilder {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        AstBuilder {
            tokens: token_vec,
            curr_idx: 0,
            errors: Vec::new(),
            symbol_table: SymbolTable::new(),
        }
    }

    // TODO: move to analyzer
    pub fn get_error_if_var_assignment_invalid(
        &self,
        identity: &String,
        assignment_var_type: &VarType,
    ) -> Option<ErrMsg> {
        let mut error: Option<ErrMsg> = None;

        let value: Option<&Var> = self.symbol_table.lookup_variable(identity);
        match value {
            Some(var) => {
                // Check that the type being assigned is correct
                error =
                    self.get_error_if_incorrect_type_assignment(assignment_var_type, &var.var_type);
            }
            None => {
                // add error that var has not been declared
                error = Some(ErrMsg::VariableNotDeclared {
                    identity: identity.clone(),
                    attempted_assignment_line: self.get_curr_token().line_number,
                })
            }
        }

        error
    }

    pub fn get_error_vec(&self) -> &Vec<ErrMsg> {
        &self.errors
    }

    pub fn generate_ast(&mut self) -> Vec<Statement> {
        // Phase 1: Collect all function declarations first
        self.collect_symbol_declarations();

        // Phase 2: Full AST generation with all functions known
        self.program()
    }

    fn get_curr_token(&self) -> &Token {
        &self.tokens[self.curr_idx]
    }

    fn next_token(&mut self) {
        self.curr_idx = self.curr_idx + 1;
    }

    fn is_curr_token_type(&mut self, t_type: &TokenType) -> bool {
        return self.get_curr_token().token_type == *t_type;
    }

    fn get_error_if_curr_not_expected(&mut self, token_type: TokenType) -> Option<ErrMsg> {
        if (self.get_curr_token().token_type != token_type) {
            return Some(ErrMsg::UnexpectedToken {
                expected: token_type,
                got: self.get_curr_token().token_type.clone(),
                line_number: self.get_curr_token().line_number.clone(), //col_number: self.get_curr_token().col_number.clone()
            });
        }
        None
    }

    fn get_error_if_incorrect_type_assignment(
        &self,
        var_type: &VarType,
        assignment_type: &VarType,
    ) -> Option<ErrMsg> {
        if var_type != assignment_type {
            return Some(ErrMsg::new_incorrect_type_assignment(
                var_type.clone(),
                assignment_type.clone(),
                self.get_curr_token().line_number,
            ));
        }
        None
    }

    // Essentially the top of the grammar of the language.
    fn program(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();

        while self.get_curr_token().token_type != TokenType::EOF {
            let statement = self.statement();
            if !matches!(statement, Statement::Newline) {
                statements.push(statement);
            }
        }

        statements
    }

    // Scan through tokens and collect all function declarations and variable declarations.
    // (This will keep self.curr_idx set to 0).
    fn collect_symbol_declarations(&mut self) {
        let og_idx = self.curr_idx;
        self.curr_idx = 0;

        while self.get_curr_token().token_type != TokenType::EOF {
            match self.get_curr_token().token_type {
                TokenType::FunctionDeclaration => {
                    self.parse_function_header();
                }
                TokenType::VarDeclaration => {
                    self.collect_variable_declaration();
                }
                _ => {
                    self.next_token();
                }
            }
        }

        self.curr_idx = og_idx;
    }

    /**
     * Will also add the header to the function map.
     * Just call it when you need it and it adds the declaration to our map.
     */
    fn parse_function_header(&mut self) {
        self.next_token(); // Skip 'function' keyword

        let function_name = self.get_curr_token().text.clone();
        self.next_token();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::LeftParen) {
            self.errors.push(error);
        }
        self.next_token();

        let parameters = self.parse_function_parameters();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::RightParen) {
            self.errors.push(error);
        }
        self.next_token();

        let return_type = self.parse_function_return_type();

        let function_header = FunctionHeader {
            function_name: function_name.clone(),
            parameters,
            return_type,
        };

        self.symbol_table
            .declare_function(function_name, function_header)
            .unwrap_or_else(|err| self.errors.push(err));

        if self.get_curr_token().token_type == TokenType::Colon {
            self.next_token();
        }

        /* // Leaving this in case my freak ass wants to implement inner functions
        while self.get_curr_token().token_type != TokenType::EOF && brace_depth > 0 {
            match self.get_curr_token().token_type {
                //TokenType::FunctionDeclaration => brace_depth += 1, leaving in case my freak ass wants inner function inits
                //TokenType::EndFunction => brace_depth -= 1,
                _ => {}
            }
            self.next_token();
        }
        */
    }

    fn statement(&mut self) -> Statement {
        // Create parser context for the statement parser since we need them to
        // have references to this stuff.
        let mut parser_context = ParserContext {
            tokens: &self.tokens,
            current: self.curr_idx,
            errors: Vec::new(),
            symbol_table: &mut self.symbol_table,
        };

        let mut coordinator = StatementParserCoordinator::new();
        let statement = coordinator.parse_statement(&mut parser_context);

        // Update our state from the parser context
        self.curr_idx = parser_context.current;
        self.errors.extend(parser_context.errors);
        // symbol_table is already updated via mutable reference

        statement
    }

    // Expression parsing methods - delegated to ExpressionParser
    fn logical(&mut self) -> Logical {
        let mut parser = ExpressionParser::new(&self.tokens, &mut self.curr_idx, &mut self.errors);
        parser.logical()
    }

    fn comparison(&mut self) -> Comparison {
        let mut parser = ExpressionParser::new(&self.tokens, &mut self.curr_idx, &mut self.errors);
        parser.comparison()
    }

    fn expression(&mut self) -> Expression {
        let mut parser = ExpressionParser::new(&self.tokens, &mut self.curr_idx, &mut self.errors);
        parser.expression()
    }

    fn term(&mut self) -> Term {
        let mut parser = ExpressionParser::new(&self.tokens, &mut self.curr_idx, &mut self.errors);
        parser.term()
    }

    fn unary(&mut self) -> Unary {
        let mut parser = ExpressionParser::new(&self.tokens, &mut self.curr_idx, &mut self.errors);
        parser.unary()
    }

    fn primary(&mut self) -> Primary {
        let mut parser = ExpressionParser::new(&self.tokens, &mut self.curr_idx, &mut self.errors);
        parser.primary()
    }

    fn parse_function_parameters(&mut self) -> Vec<FunctionParameter> {
        let mut parameters = Vec::new();

        while self.get_curr_token().token_type != TokenType::RightParen {
            // Parse parameter type (e.g., "Number", "String")
            if self.get_curr_token().token_type != TokenType::VarDeclaration {
                self.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::VarDeclaration,
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number,
                });
                self.next_token();
                continue;
            }

            let param_type = VarType::from(self.get_curr_token().text.as_str());
            self.next_token();

            // Parse parameter name
            if self.get_curr_token().token_type != TokenType::Identity {
                self.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::Identity,
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number,
                });
                self.next_token();
                continue;
            }

            let param_name = self.get_curr_token().text.clone();
            self.next_token();

            parameters.push(FunctionParameter {
                name: param_name,
                param_type,
            });

            // Check for comma (more parameters) or right paren (end of parameters)
            if self.get_curr_token().token_type == TokenType::Comma {
                self.next_token(); // Move past comma
            } else if self.get_curr_token().token_type != TokenType::RightParen {
                self.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::RightParen,
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number,
                });
            }
        }

        parameters
    }

    fn parse_function_return_type(&mut self) -> VarType {
        // Parse return type (if present)
        if self.get_curr_token().token_type == TokenType::Arrow {
            self.next_token(); // consume '->'

            // Expect a type after the arrow
            if self.get_curr_token().token_type == TokenType::VarDeclaration {
                let ret_type = VarType::from(self.get_curr_token().text.as_str());
                self.next_token();
                ret_type
            } else if self.get_curr_token().token_type == TokenType::Identity {
                // Handle cases like "Void" which might not be VarDeclaration tokens
                let ret_type = VarType::from(self.get_curr_token().text.as_str());
                self.next_token();
                ret_type
            } else {
                self.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::VarDeclaration,
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number,
                });
                self.next_token();
                VarType::Unrecognized
            }
        } else {
            // No return type specified, default to void/unrecognized
            VarType::Unrecognized
        }
    }

    fn extract_value_and_type_from_primary(&self, primary: &Primary) -> (String, VarType) {
        match primary {
            Primary::Number { value } => (value.clone(), VarType::Num),
            Primary::String { value } => (value.clone(), VarType::Str),
            Primary::Identity {
                name,
                line_number: _,
            } => {
                // Look up the variable type in the symbol table
                if let Some(var_info) = self.symbol_table.lookup_variable(name) {
                    (name.clone(), var_info.var_type.clone())
                } else {
                    // Variable not found, return as unrecognized
                    // Error handling should be done elsewhere
                    (name.clone(), VarType::Unrecognized)
                }
            }
            Primary::FunctionCall {
                name,
                arguments,
                line_number: _,
            } => {
                // Look up the function return type in the function map
                if let Some(function_info) = self.symbol_table.lookup_function(name) {
                    // For code generation, we need to represent this as a function call
                    let mut call_text = name.clone();
                    call_text.push_str("(");
                    for (i, arg) in arguments.iter().enumerate() {
                        if i > 0 {
                            call_text.push_str(", ");
                        }
                        call_text.push_str(arg);
                    }
                    call_text.push_str(")");
                    (call_text, function_info.return_type.clone())
                } else {
                    // Function not found, return as unrecognized
                    (name.clone(), VarType::Unrecognized)
                }
            }
            Primary::Error { detail: _ } => ("/* error */".to_string(), VarType::Unrecognized),
        }
    }

    fn collect_variable_declaration(&mut self) {
        let var_type = VarType::from(self.get_curr_token().text.as_str());
        self.next_token();

        if self.get_curr_token().token_type == TokenType::Identity {
            let identity = self.get_curr_token().text.clone();
            self.next_token();

            // Create the variable and add it to var_map
            let var = Var {
                var_type,
                identity: identity.clone(),
                line_declared_on: self.get_curr_token().line_number,
            };

            // Try to declare the variable in the symbol table
            if let Err(err) = self.symbol_table.declare_variable(identity, var) {
                self.errors.push(err);
            }

            // Skip past the rest of the variable declaration (: value)
            if self.get_curr_token().token_type == TokenType::Colon {
                self.next_token(); // Skip ':'

                // Skip the assignment value (could be literal, variable, or function call)
                // We'll use a simple approach and just skip until we hit a newline or EOF
                while self.get_curr_token().token_type != TokenType::Newline
                    && self.get_curr_token().token_type != TokenType::EOF
                {
                    self.next_token();
                }
            }
        } else {
            // Invalid variable declaration, just skip to next token
            self.next_token();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    pub var_type: VarType,
    pub identity: String,
    pub line_declared_on: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VarType {
    Num,
    Str,
    Unrecognized,
}

impl FromStr for VarType {
    type Err = ();

    fn from_str(input: &str) -> Result<VarType, Self::Err> {
        match input {
            "Number" => Ok(VarType::Num),
            "String" => Ok(VarType::Str),
            _ => Err(()),
        }
    }
}

impl From<TokenType> for VarType {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Number => VarType::Num,
            TokenType::Str => VarType::Str,
            _ => VarType::Unrecognized,
        }
    }
}

impl From<&str> for VarType {
    fn from(text: &str) -> Self {
        match text {
            "Number" => VarType::Num,
            "String" => VarType::Str,
            "Void" => VarType::Unrecognized, // Treat Void as Unrecognized for now
            _ => VarType::Unrecognized,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: String,
    pub param_type: VarType,
}

#[derive(Debug, Clone)]
pub struct FunctionHeader {
    pub function_name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: VarType,
}
