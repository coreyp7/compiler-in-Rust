use std::collections::HashMap;
use std::str::FromStr;

// My stuff
use crate::comparison::*;
use crate::error::ErrMsg;
use crate::statement::{
    AssignmentStatement, IfStatement, InstantiationStatement, PrintStatement, Statement,
    WhileStatement,
};
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;

pub struct AstBuilder {
    tokens: Vec<Token>,
    curr_idx: usize,
    errors: Vec<ErrMsg>,
    pub var_map: HashMap<String, Var>,
}

impl AstBuilder {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        AstBuilder {
            tokens: token_vec,
            curr_idx: 0,
            //statements: Vec::new(),
            errors: Vec::new(),
            var_map: HashMap::new(),
        }
    }

    pub fn insert_into_var_map(&mut self, identity: String, var_type: VarType, line: u8) {
        if self.var_map.contains_key(&identity) {
            // TODO: Include line number for error display.
            self.errors
                .push(ErrMsg::VariableAlreadyDeclared { identity: identity });
            return;
        }

        let var = Var {
            var_type: var_type,
            identity: identity,
            line_declared_on: line,
        };

        self.var_map.insert(var.identity.clone(), var);
    }

    pub fn get_error_if_var_assignment_invalid(
        &self,
        identity: &String,
        assignment_var_type: &VarType,
    ) -> Option<ErrMsg> {
        // Check if the variable even has been decalred
        if !self.var_map.contains_key(identity) {
            // add error
        }

        let value: Option<&Var> = self.var_map.get(identity);
        let mut error: Option<ErrMsg> = None;
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
            statements.push(statement);
        }

        statements
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
        let mut statement = Statement::TestStub;
        //let curr_token = self.get_curr_token();
        match self.get_curr_token().token_type {
            TokenType::Print => {
                self.next_token();
                let string_content: String = self.get_curr_token().text.clone();
                let mut is_identity: bool = false;
                let mut possible_error: Option<ErrMsg> = None;

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

                statement = Statement::Print(PrintStatement {
                    content: string_content,
                    line_number: self.get_curr_token().line_number,
                    is_content_identity_name: is_identity,
                });
            }
            TokenType::If => {
                self.next_token();
                let conditional = self.logical();

                if let Some(error) = self.get_error_if_curr_not_expected(TokenType::Then) {
                    self.errors.push(error);
                }
                self.next_token();

                let mut statements: Vec<Statement> = Vec::new();

                while !self.is_curr_token_type(&TokenType::EndIf) {
                    statements.push(self.statement());
                }

                self.next_token();

                statement = Statement::If(IfStatement {
                    logical: conditional,
                    statements: statements,
                    line_number: self.get_curr_token().line_number,
                });
            }
            TokenType::While => {
                self.next_token();

                let conditional = self.logical();

                if let Some(error) = self.get_error_if_curr_not_expected(TokenType::Do) {
                    self.errors.push(error);
                }
                self.next_token();

                let mut statements: Vec<Statement> = Vec::new();

                while !self.is_curr_token_type(&TokenType::EndWhile) {
                    statements.push(self.statement());
                }

                self.next_token();

                statement = Statement::While(WhileStatement {
                    logical: conditional,
                    statements: statements,
                    line_number: self.get_curr_token().line_number,
                });
            }
            TokenType::Identity => {
                let identity = self.get_curr_token().text.clone();
                self.next_token();

                if let Some(error) = self.get_error_if_curr_not_expected(TokenType::LessThanEqualTo)
                {
                    self.errors.push(error);
                }
                self.next_token();

                let assignment_token_type = self.get_curr_token().token_type.clone();
                let assignment_var_type = convert_tokentype_to_vartype(assignment_token_type);
                let assignment_value_text = self.get_curr_token().text.clone();
                self.next_token();

                if let Some(error) =
                    self.get_error_if_var_assignment_invalid(&identity, &assignment_var_type)
                {
                    self.errors.push(error);
                }
                statement = Statement::Assignment(AssignmentStatement {
                    identity: identity,
                    value: assignment_value_text,
                    assigned_value_type: assignment_var_type,
                    line_number: self.get_curr_token().line_number,
                });
            }
            TokenType::VarDeclaration => {
                let var_type = convert_str_to_vartype(&self.get_curr_token().text);
                self.next_token();

                let identity = self.get_curr_token().text.clone();
                self.next_token();

                if let Some(error) = self.get_error_if_curr_not_expected(TokenType::Colon) {
                    self.errors.push(error);
                }
                self.next_token();

                let assignment_value_text = self.get_curr_token().text.clone();
                let assignment_token_type = self.get_curr_token().token_type.clone();
                let assignment_var_type = convert_tokentype_to_vartype(assignment_token_type);
                if let Some(error) =
                    self.get_error_if_incorrect_type_assignment(&var_type, &assignment_var_type)
                {
                    self.errors.push(error);
                }

                self.insert_into_var_map(
                    identity.clone(),
                    var_type.clone(),
                    self.get_curr_token().line_number,
                );

                self.next_token();

                statement = Statement::Instantiation(InstantiationStatement {
                    identity: identity,
                    value: assignment_value_text,
                    var_type: var_type,
                    assigned_value_type: assignment_var_type,
                    line_number: self.get_curr_token().line_number,
                });
            }
            TokenType::Newline => {
                statement = Statement::Newline;
            }
            _ => {}
        };

        self.next_token();
        return statement;
    }

    fn logical(&mut self) -> Logical {
        let mut logical = Logical::new();

        // check for optional bang
        if self.get_curr_token().token_type == TokenType::Bang {
            let op: LogicalOperator = convert_token_type_to_logical_op(TokenType::Bang);
            logical.operators.push(op);
            self.next_token();
        }

        let comp1 = self.comparison();
        logical.comparisons.push(comp1);

        let op1: LogicalOperator =
            convert_token_type_to_logical_op(self.get_curr_token().token_type.clone());

        if op1 == LogicalOperator::invalidop {
            // No logical operators, there's just 1 comparison to process.
            // return the struct as is, with no operations
            println!("skipping this because its not logical op");
            println!("{:?}", self.get_curr_token());
            return logical;
        }

        while self.is_curr_token_logical_operator() {
            let op: LogicalOperator =
                convert_token_type_to_logical_op(self.get_curr_token().token_type.clone());
            logical.operators.push(op);
            self.next_token();

            // check for optional bang
            if self.get_curr_token().token_type == TokenType::Bang {
                let bang: LogicalOperator = convert_token_type_to_logical_op(TokenType::Bang);
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
            let op: ComparisonOperator =
                convert_token_type_to_comparison_op(self.get_curr_token().token_type.clone());
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
            let op = convert_token_type_to_expression_op(self.get_curr_token().token_type.clone());
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
            let op = convert_token_type_to_term_op(self.get_curr_token().token_type.clone());
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
            unary.operation = Some(convert_token_type_to_expression_op(
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
            TokenType::Identity => Primary::Identity {
                name: self.get_curr_token().text.clone(),
            },
            _ => Primary::Error {
                detail: String::new(),
            },
        };
        self.next_token();

        //println!("Created a primary: {:?}", primary);
        primary
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    var_type: VarType,
    identity: String,
    line_declared_on: u8,
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

// Helper functions for VarType conversion
pub fn convert_str_to_vartype(text: &str) -> VarType {
    match text {
        "Number" => VarType::Num,
        "String" => VarType::Str,
        _ => VarType::Unrecognized,
    }
}

pub fn convert_tokentype_to_vartype(token_type: TokenType) -> VarType {
    match token_type {
        TokenType::Number => VarType::Num,
        TokenType::Str => VarType::Str,
        _ => VarType::Unrecognized,
    }
}
