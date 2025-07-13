use std::collections::HashMap;
use std::str::FromStr;

// My stuff
use crate::comparison::*;
use crate::error::ErrMsg;
use crate::statement::{
    AssignmentStatement, FunctionCallStatement, FunctionInstantiationStatement, IfStatement,
    PrintStatement, ReturnStatement, Statement, VarInstantiationStatement, WhileStatement,
};
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;

pub struct AstBuilder {
    tokens: Vec<Token>,
    curr_idx: usize,
    errors: Vec<ErrMsg>,
    pub var_map: HashMap<String, Var>,
    pub function_map: HashMap<String, FunctionHeader>,
}

impl AstBuilder {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        AstBuilder {
            tokens: token_vec,
            curr_idx: 0,
            errors: Vec::new(),
            var_map: HashMap::new(),
            function_map: HashMap::new(),
        }
    }

    pub fn get_error_if_var_assignment_invalid(
        &self,
        identity: &String,
        assignment_var_type: &VarType,
    ) -> Option<ErrMsg> {
        let mut error: Option<ErrMsg> = None;

        let value: Option<&Var> = self.var_map.get(identity);
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

        self.function_map.insert(function_name, function_header);

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

    // TODO: these really shouldn't be here this sucks
    fn is_curr_token_logical_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::DoubleAmpersand => true,
            TokenType::DoubleBar => true,
            TokenType::Bang => true,
            _ => false,
        }
    }

    fn is_curr_token_comparison_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::EqualEqual => true,
            TokenType::NotEqual => true,
            TokenType::LessThan => true,
            TokenType::LessThanEqualTo => true,
            TokenType::GreaterThan => true,
            TokenType::GreaterThanEqualTo => true,
            _ => false,
        }
    }

    fn is_curr_token_expression_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::Plus => true,
            TokenType::Minus => true,
            _ => false,
        }
    }

    fn is_curr_token_term_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::Asterisk => true,
            TokenType::Slash => true,
            _ => false,
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
            self.var_map
                .get(&string_content)
                .map(|var| var.var_type.clone())
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
        let function_header = if let Some(header) = self.function_map.get(&function_name).cloned() {
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
                    if let Some(var_info) = self.var_map.get(&var_name) {
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

    fn logical(&mut self) -> Logical {
        let mut logical = Logical::new();

        // check for optional bang
        if self.get_curr_token().token_type == TokenType::Bang {
            let op = LogicalOperator::from(TokenType::Bang);
            logical.operators.push(op);
            self.next_token();
        }

        let comp1 = self.comparison();
        logical.comparisons.push(comp1);

        let op1 = LogicalOperator::from(self.get_curr_token().token_type.clone());

        if op1 == LogicalOperator::Invalid {
            // No logical operators, there's just 1 comparison to process.
            // return the struct as is, with no operations
            println!("skipping this because its not logical op");
            println!("{:?}", self.get_curr_token());
            return logical;
        }

        while self.is_curr_token_logical_operator() {
            let op = LogicalOperator::from(self.get_curr_token().token_type.clone());
            logical.operators.push(op);
            self.next_token();

            // check for optional bang
            if self.get_curr_token().token_type == TokenType::Bang {
                let bang = LogicalOperator::from(TokenType::Bang);
                logical.operators.push(bang);
                self.next_token();
            }

            let comp: Comparison = self.comparison();
            logical.comparisons.push(comp);
        }

        logical
    }

    fn comparison(&mut self) -> Comparison {
        let mut comparison = Comparison {
            expressions: Vec::new(),
            operators: Vec::new(),
        };
        let expr1: Expression = self.expression(); // have this emulate an ouput
        comparison.expressions.push(expr1);

        // let op1: ComparisonOperator = convert_token_type_to_comparison_op(
        //     self.get_curr_token().token_type.clone()
        // );
        // (Unused, but may be useful for debugging or future logic)

        while self.is_curr_token_comparison_operator() {
            let op = ComparisonOperator::from(self.get_curr_token().token_type.clone());
            comparison.operators.push(op);
            self.next_token();

            let expr: Expression = self.expression();
            comparison.expressions.push(expr);
        }

        comparison
    }

    fn expression(&mut self) -> Expression {
        let mut expr = Expression {
            terms: Vec::new(),
            operators: Vec::new(),
        };

        let term1 = self.term();
        expr.terms.push(term1);

        while self.is_curr_token_expression_operator() {
            let op = ExpressionOperator::from(self.get_curr_token().token_type.clone());
            expr.operators.push(op);
            self.next_token();

            let term = self.term();
            expr.terms.push(term);
        }

        expr
    }

    fn term(&mut self) -> Term {
        let mut term = Term {
            unarys: Vec::new(),
            operations: Vec::new(),
        };

        let unary1 = self.unary();
        term.unarys.push(unary1);

        while self.is_curr_token_term_operator() {
            let op = TermOperator::from(self.get_curr_token().token_type.clone());
            term.operations.push(op);
            self.next_token();

            let unary = self.unary();
            term.unarys.push(unary);
        }

        term
    }

    fn unary(&mut self) -> Unary {
        let mut unary = Unary {
            operation: None,
            primary: Primary::Error {
                detail: String::new(),
            },
        };

        if self.is_curr_token_expression_operator() {
            unary.operation = Some(ExpressionOperator::from(
                self.get_curr_token().token_type.clone(),
            ));
            self.next_token();
        }

        unary.primary = self.primary();

        unary
    }

    fn primary(&mut self) -> Primary {
        let primary = match self.get_curr_token().token_type {
            TokenType::Number => Primary::Number {
                value: self.get_curr_token().text.clone(),
            },
            TokenType::Str => Primary::String {
                value: self.get_curr_token().text.clone(),
            },
            TokenType::Identity => {
                let identity_name = self.get_curr_token().text.clone();
                let line_number = self.get_curr_token().line_number;

                // Check if next token is '(' which indicates a function call
                let next_idx = self.curr_idx + 1;
                if next_idx < self.tokens.len()
                    && self.tokens[next_idx].token_type == TokenType::LeftParen
                {
                    // This is a function call expression
                    self.next_token(); // Move to '('
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
                    // Don't advance past ')' here since next_token() is called at the end

                    Primary::FunctionCall {
                        name: identity_name,
                        arguments,
                        line_number,
                    }
                } else {
                    // This is just a variable reference
                    Primary::Identity {
                        name: identity_name,
                        line_number,
                    }
                }
            }
            _ => Primary::Error {
                detail: String::new(),
            },
        };
        self.next_token();

        //println!("Created a primary: {:?}", primary);
        primary
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
                if let Some(var_info) = self.var_map.get(name) {
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
                if let Some(function_info) = self.function_map.get(name) {
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

            if self.var_map.contains_key(&identity) {
                self.errors.push(ErrMsg::VariableAlreadyDeclared {
                    identity: identity.clone(),
                    first_declared_line: self.var_map.get(&identity).unwrap().line_declared_on,
                    redeclared_line: var.line_declared_on,
                });
            } else {
                self.var_map.insert(identity, var);
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
