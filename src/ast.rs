use std::collections::HashMap;
use std::str::FromStr;

// My stuff
use crate::comparison::*;
use crate::error::ErrMsg;
use crate::expression_parser::ExpressionParser;
use crate::statement::{
    AssignmentStatement, FunctionCallStatement, FunctionInstantiationStatement, IfStatement,
    PrintStatement, ReturnStatement, Statement, VarInstantiationStatement, WhileStatement,
};
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
        self.collect_function_declarations();

        // Reset to beginning for full parsing
        self.curr_idx = 0;

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

    /// Phase 1: Scan through tokens and collect all function declarations and variable declarations
    /// This populates the function_map and var_map so forward declarations work
    fn collect_function_declarations(&mut self) {
        let original_idx = self.curr_idx;
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

        // Restore original position
        self.curr_idx = original_idx;
    }

    /// Parse only the function header (name, parameters, return type) without the body
    /// Used in Phase 1 to collect function signatures
    fn parse_function_header(&mut self) {
        self.next_token(); // Skip 'function'

        let function_name = self.get_curr_token().text.clone();
        self.next_token();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::LeftParen) {
            self.errors.push(error);
        }
        self.next_token();

        // Parse function parameters
        let parameters = self.parse_function_parameters();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::RightParen) {
            self.errors.push(error);
        }
        self.next_token();

        let return_type = self.parse_function_return_type();

        // Create function header and add to function map
        let function_header = FunctionHeader {
            function_name: function_name.clone(),
            parameters,
            return_type,
        };

        self.symbol_table
            .declare_function(function_name, function_header)
            .unwrap_or_else(|err| self.errors.push(err));

        // Skip the function body - just advance until we find endFunction
        let mut brace_depth = 1; // We're inside the function

        // Skip past the colon
        if self.get_curr_token().token_type == TokenType::Colon {
            self.next_token();
        }

        while self.get_curr_token().token_type != TokenType::EOF && brace_depth > 0 {
            match self.get_curr_token().token_type {
                //TokenType::FunctionDeclaration => brace_depth += 1, leaving in case my freak ass wants inner function inits
                TokenType::EndFunction => brace_depth -= 1,
                _ => {}
            }
            self.next_token();
        }
    }

    fn statement(&mut self) -> Statement {
        let statement = match self.get_curr_token().token_type {
            TokenType::Print => self.parse_print_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Identity => {
                let identity = self.get_curr_token().text.clone();
                self.next_token();

                if self.get_curr_token().token_type == TokenType::LeftParen {
                    self.parse_function_call(identity)
                } else {
                    self.parse_variable_assignment(identity)
                }
            }
            TokenType::VarDeclaration => self.parse_var_declaration_statement(),
            TokenType::FunctionDeclaration => self.parse_function_declaration_statement(),
            TokenType::Newline => self.parse_newline_statement(),
            _ => Statement::TestStub,
        };

        self.next_token();
        statement
    }

    fn parse_function_call(&mut self, function_name: String) -> Statement {
        self.next_token(); // Move past '('

        let mut arguments = Vec::new();
        while self.get_curr_token().token_type != TokenType::RightParen {
            if self.get_curr_token().token_type == TokenType::Identity {
                arguments.push(self.get_curr_token().text.clone());
                self.next_token();

                if self.get_curr_token().token_type == TokenType::Comma {
                    self.next_token();
                }
            } else {
                self.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::Identity,
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number,
                });
                self.next_token();
            }
        }

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::RightParen) {
            self.errors.push(error);
        }

        Statement::FunctionCall(FunctionCallStatement {
            function_name,
            arguments,
            line_number: self.get_curr_token().line_number,
        })
    }

    // TODO: need to alter to allow assignment of a function return.
    // will require updating this function and type checking validation in the
    // semantic analyzer.
    fn parse_variable_assignment(&mut self, identity: String) -> Statement {
        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::LessThanEqualTo) {
            self.errors.push(error);
        }
        self.next_token();

        let assignment_token_type = self.get_curr_token().token_type.clone();
        let assignment_var_type = VarType::from(assignment_token_type);
        let assignment_value_text = self.get_curr_token().text.clone();
        self.next_token();

        if let Some(error) =
            self.get_error_if_var_assignment_invalid(&identity, &assignment_var_type)
        {
            self.errors.push(error);
        }

        Statement::Assignment(AssignmentStatement {
            identity,
            value: assignment_value_text,
            assigned_value_type: assignment_var_type,
            line_number: self.get_curr_token().line_number,
        })
    }

    fn parse_print_statement(&mut self) -> Statement {
        self.next_token();
        let string_content = self.get_curr_token().text.clone();
        let mut is_identity = false;
        let mut possible_error = None;

        match &self.get_curr_token().token_type {
            TokenType::Str => (),
            TokenType::Identity => {
                is_identity = true;
            }
            _ => {
                possible_error = Some(ErrMsg::UnexpectedToken {
                    expected: self.get_curr_token().token_type.clone(),
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number.clone(),
                });
            }
        }

        if let Some(error) = possible_error {
            self.errors.push(error);
        }

        // Look up variable type if it's a variable reference
        let variable_type = if is_identity {
            self.symbol_table.get_variable_type(&string_content)
        } else {
            None
        };

        Statement::Print(PrintStatement {
            content: string_content,
            line_number: self.get_curr_token().line_number,
            is_content_identity_name: is_identity,
            variable_type,
        })
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.next_token();
        let conditional = self.logical();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::Then) {
            self.errors.push(error);
        }
        self.next_token();

        let mut statements = Vec::new();

        while !self.is_curr_token_type(&TokenType::EndIf) {
            statements.push(self.statement());
        }

        self.next_token();

        Statement::If(IfStatement {
            logical: conditional,
            statements,
            line_number: self.get_curr_token().line_number,
        })
    }

    fn parse_while_statement(&mut self) -> Statement {
        self.next_token();

        let conditional = self.logical();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::Do) {
            self.errors.push(error);
        }
        self.next_token();

        let mut statements = Vec::new();

        while !self.is_curr_token_type(&TokenType::EndWhile) {
            statements.push(self.statement());
        }

        self.next_token();

        Statement::While(WhileStatement {
            logical: conditional,
            statements,
            line_number: self.get_curr_token().line_number,
        })
    }

    fn parse_var_declaration_statement(&mut self) -> Statement {
        let var_type = VarType::from(self.get_curr_token().text.as_str());
        self.next_token();

        let identity = self.get_curr_token().text.clone();
        self.next_token();

        if let Some(error) = self.get_error_if_curr_not_expected(TokenType::Colon) {
            self.errors.push(error);
        }
        self.next_token();

        // Use primary() to parse the assignment value, which can handle literals, variables, or function calls
        let assignment_primary = self.primary();
        let (assignment_value_text, assignment_var_type) =
            self.extract_value_and_type_from_primary(&assignment_primary);

        if let Some(error) =
            // TODO: move into semantic analyzer
            self.get_error_if_incorrect_type_assignment(&var_type, &assignment_var_type)
        {
            self.errors.push(error);
        }

        // Variable is already in var_map from the first pass (collect_variable_declaration)
        // Just create and return the statement
        Statement::VarInstantiation(VarInstantiationStatement {
            identity,
            value: assignment_value_text,
            var_type,
            assigned_value_type: assignment_var_type,
            line_number: self.get_curr_token().line_number,
        })
    }

    fn parse_function_declaration_statement(&mut self) -> Statement {
        self.next_token(); // Skip 'function'

        let function_name = self.get_curr_token().text.clone();

        // Look up the function header that was already parsed in Phase 1
        let function_header =
            if let Some(header) = self.symbol_table.lookup_function(&function_name).cloned() {
                header
            } else {
                // This shouldn't happen if Phase 1 worked correctly, but handle gracefully
                self.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::FunctionDeclaration,
                    got: self.get_curr_token().token_type.clone(),
                    line_number: self.get_curr_token().line_number,
                });
                return Statement::TestStub;
            };

        // Skip past the function signature since we already have it
        // Skip function name
        self.next_token();

        // Skip '(parameters)'
        if self.get_curr_token().token_type == TokenType::LeftParen {
            let mut paren_depth = 1;
            self.next_token();
            while paren_depth > 0 && self.get_curr_token().token_type != TokenType::EOF {
                match self.get_curr_token().token_type {
                    TokenType::LeftParen => paren_depth += 1,
                    TokenType::RightParen => paren_depth -= 1,
                    _ => {}
                }
                self.next_token();
            }
        }

        // Skip return type if present (-> Type)
        if self.get_curr_token().token_type == TokenType::Arrow {
            self.next_token(); // Skip '->'
            self.next_token(); // Skip type
        }

        // Skip ':'
        if self.get_curr_token().token_type == TokenType::Colon {
            self.next_token();
        }

        // Now parse the function body statements
        let mut function_statements = Vec::new();
        while !self.is_curr_token_type(&TokenType::EndFunction) {
            let statement = self.statement();
            if !matches!(statement, Statement::Newline) {
                function_statements.push(statement);
            }
        }

        self.next_token(); // Skip 'endFunction'

        Statement::FunctionInstantiation(FunctionInstantiationStatement {
            header: function_header,
            statements: function_statements,
            line_number: self.get_curr_token().line_number,
        })
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.next_token(); // Move past 'return'

        let mut return_value = None;
        let mut return_type = VarType::Unrecognized;

        // Check if there's a return value (not just a bare return)
        if self.get_curr_token().token_type != TokenType::Newline {
            match self.get_curr_token().token_type {
                TokenType::Number => {
                    return_value = Some(Var {
                        identity: self.get_curr_token().text.clone(),
                        var_type: VarType::Num,
                        line_declared_on: self.get_curr_token().line_number,
                    });
                    return_type = VarType::Num;
                }
                TokenType::Str => {
                    return_value = Some(Var {
                        identity: self.get_curr_token().text.clone(),
                        var_type: VarType::Str,
                        line_declared_on: self.get_curr_token().line_number,
                    });
                    return_type = VarType::Str;
                }
                TokenType::Identity => {
                    // Look up the variable in the symbol table
                    let var_name = self.get_curr_token().text.clone();
                    if let Some(var_info) = self.symbol_table.lookup_variable(&var_name) {
                        return_value = Some(Var {
                            identity: var_name.clone(),
                            var_type: var_info.var_type.clone(),
                            line_declared_on: self.get_curr_token().line_number,
                        });
                        return_type = var_info.var_type.clone();
                    } else {
                        self.errors.push(ErrMsg::VariableNotDeclared {
                            identity: var_name,
                            attempted_assignment_line: self.get_curr_token().line_number,
                        });
                    }
                }
                _ => {
                    self.errors.push(ErrMsg::UnexpectedToken {
                        expected: TokenType::Identity,
                        got: self.get_curr_token().token_type.clone(),
                        line_number: self.get_curr_token().line_number,
                    });
                }
            }
        }

        Statement::Return(ReturnStatement {
            return_type,
            return_value,
            line_number: self.get_curr_token().line_number,
        })
    }

    fn parse_newline_statement(&mut self) -> Statement {
        Statement::Newline
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

    /// Collect variable declaration during Phase 1 to populate var_map
    /// This allows print statements to resolve variable types during parsing
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
