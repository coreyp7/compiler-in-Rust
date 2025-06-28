//use colored::Colorize;
use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::str::FromStr;
use colored::Colorize;

// My stuff
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
use crate::comparison::*;

pub struct AstBuilder<> {
    pub tokens: Vec<Token>,
    curr_idx: usize,
    //statements: Vec<Statement>,
    errors: Vec<ErrMsg>
}

impl AstBuilder<> {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        AstBuilder {
            tokens: token_vec,
            curr_idx: 0,
            //statements: Vec::new(),
            errors: Vec::new()
        }
    }


    pub fn generate_ast(&mut self) -> Vec<Statement>{
        self.program()
    }

    fn get_curr_token(&self) -> &Token {
        &self.tokens[self.curr_idx]
    }


    fn next_token(&mut self){
        self.curr_idx = self.curr_idx + 1;
        //println!("New token: {:?}", self.get_curr_token());
    }

    fn is_curr_token_type(&mut self, t_type: &TokenType) -> bool{
        return self.get_curr_token().token_type == *t_type;
    }

    fn add_error_if_curr_not_expected(&mut self, token_type: TokenType) {
        if(self.get_curr_token().token_type != token_type){
            self.errors.push( ErrMsg {
                expected: token_type,
                got: self.get_curr_token().token_type.clone(),
                line_number: self.get_curr_token().line_number.clone(),
                col_number: self.get_curr_token().col_number.clone()
            });
        }
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
            _ => false
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
            _ => false
        }
    }

    fn is_curr_token_expression_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::Plus => true,
            TokenType::Minus => true,
            _ => false
        }
    }

    fn is_curr_token_term_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::Asterisk => true,
            TokenType::Slash => true,
            _ => false
        }
    }


    fn statement(&mut self) -> Statement {
        let mut statement = Statement::TestStub;
        //let curr_token = self.get_curr_token();
        match self.get_curr_token().token_type {
            TokenType::Print => {
                //println!("{:?}", curr_token);                
                self.next_token();
                let string_content: String = self.get_curr_token().text.clone();
                
                if self.get_curr_token().token_type != TokenType::Str {
                    println!("ERROR: expecting string, got {:#?}", 
                        self.get_curr_token().token_type
                    );
                }

                statement = Statement::Print{
                        content: string_content,
                        line_number: self.get_curr_token().line_number
                    };
            },
            TokenType::If => {
                self.next_token();

                // parse comparison
                //let comparison = self.comparison();
                let conditional = self.logical();

                self.add_error_if_curr_not_expected(TokenType::Then);
                self.next_token();

                let mut statements: Vec<Statement> = Vec::new();

                while !self.is_curr_token_type(&TokenType::EndIf){
                    statements.push(self.statement());
                }

                self.next_token();

                statement = Statement::If{
                        logical: conditional,
                        statements: statements 
                };
            },
            TokenType::While => {
                self.next_token();

                let comparison = self.comparison();

                self.add_error_if_curr_not_expected(TokenType::Do);
                self.next_token();

                let mut statements: Vec<Statement> = Vec::new();

                while !self.is_curr_token_type(&TokenType::EndWhile){
                    statements.push(self.statement());
                }

                self.next_token();

                statement = Statement::While{
                        comparison: comparison,
                        statements: statements 
                };
            },
            TokenType::Identity => {
                // variable assignment (existing var)

                // TODO: need to start refactoring for these lazy clones 
                // bc my data isn't setup correctly.
                let identity = self.get_curr_token().text.clone();
                self.next_token(); 

                self.add_error_if_curr_not_expected(TokenType::LessThanEqualTo);
                self.next_token();

                let value = self.get_curr_token().text.clone();
                self.next_token();

                statement = Statement::Assignment {
                    identity: identity,
                    value: value
                };
            },
            TokenType::VarDeclaration => {
                // var init
                let var_type = self.get_curr_token().text.clone();
                self.next_token();

                let identity = self.get_curr_token().text.clone();
                self.next_token(); 

                self.add_error_if_curr_not_expected(TokenType::Colon);
                self.next_token();

                let value = self.get_curr_token().text.clone();
                self.next_token();

                statement = Statement::Instantiation {
                    identity: identity,
                    value: value,
                    var_type: convert_str_to_vartype(&var_type)
                };
                

            },
            TokenType::Newline => {
                statement = Statement::Newline;
            },
            _ => {
            }
        };

        let line_number = self.get_curr_token().line_number;
        let col_number = self.get_curr_token().col_number;
        let token_type = &self.get_curr_token().token_type;
        /*println!("{:#?} at line number {}", 
            statement,
            line_number
        );*/

        self.next_token();
        return statement;
    }

    fn logical(&mut self) -> Logical {
        let mut logical = Logical::new();

        // check for optional bang
        if self.get_curr_token().token_type == TokenType::Bang {
            let op: LogicalOperator = convert_token_type_to_logical_op(
                TokenType::Bang
            );
            logical.operators.push(op);
            self.next_token();
        }

        let comp1 = self.comparison();
        logical.comparisons.push(comp1);

        let op1: LogicalOperator = convert_token_type_to_logical_op(
            self.get_curr_token().token_type.clone()
        );
        
        if op1 == LogicalOperator::invalidop {
            // No logical operators, there's just 1 comparison to process.
            // return the struct as is, with no operations
            println!("skipping this because its not logical op");
            println!("{:?}", self.get_curr_token());
            return logical;
        } 

        while self.is_curr_token_logical_operator() {

            let op: LogicalOperator = convert_token_type_to_logical_op(
                self.get_curr_token().token_type.clone()
            );
            logical.operators.push(op);
            self.next_token();

            // check for optional bang
            if self.get_curr_token().token_type == TokenType::Bang {
                let bang: LogicalOperator = convert_token_type_to_logical_op(
                    TokenType::Bang
                );
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
            operators: Vec::new()
        };
        let expr1: Expression = self.expression(); // have this emulate an ouput
        comparison.expressions.push(expr1);

        let op1: ComparisonOperator = convert_token_type_to_comparison_op(
            self.get_curr_token().token_type.clone()
        );

        while self.is_curr_token_comparison_operator() {
            let op: ComparisonOperator = convert_token_type_to_comparison_op(
                self.get_curr_token().token_type.clone()
            );
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
            operators: Vec::new()
        };

        let term1 = self.term(); 
        expr.terms.push(term1);

        while self.is_curr_token_expression_operator() {
            let op = convert_token_type_to_expression_op(
                self.get_curr_token().token_type.clone()
            );
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
            operations: Vec::new()
        };

        let unary1 = self.unary();
        term.unarys.push(unary1);

        while self.is_curr_token_term_operator() {
            let op = convert_token_type_to_term_op(
                self.get_curr_token().token_type.clone()
            );
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
                detail: String::new()
            }
        };

        if self.is_curr_token_expression_operator() {
            unary.operation = Some(
                convert_token_type_to_expression_op(
                    self.get_curr_token().token_type.clone()
                )
            );
            self.next_token();
        }

        unary.primary = self.primary();

        unary
    }

    fn primary(&mut self) -> Primary {
        
        let primary = match self.get_curr_token().token_type {
            TokenType::Number => {
                Primary::Number {
                    value: self.get_curr_token().text.clone()
                }
            },
            TokenType::Identity => {
                Primary::Identity {
                    name: self.get_curr_token().text.clone()
                }
            },
            _ => {
                Primary::Error{
                    detail: String::new()
                }
            }
        };
        self.next_token();

        //println!("Created a primary: {:?}", primary);
        primary
    }
}


#[derive(Debug)]
pub enum Statement {
    Print {
        content: String,
        line_number: u8
    },
    If {
        logical: Logical,
        statements: Vec<Statement>
    },
    While {
        comparison: Comparison,
        statements: Vec<Statement>
    },
    Assignment {
        identity: String,
        value: String
    },
    Instantiation {
        identity: String,
        value: String,
        var_type: VarType
    },
    Newline,
    TestStub
}

#[derive(Debug)]
pub enum VarType {
    Num,
    Str,
    Unrecognized
}

fn convert_str_to_vartype(text: &str) -> VarType {
    match text {
        "Number" => VarType::Num,
        "String" => VarType::Str,
        _ => VarType::Unrecognized
    }
}

impl FromStr for VarType {
    type Err = ();

    fn from_str(input: &str) -> Result<VarType, Self::Err> {
        match input {
            "Number" => Ok(VarType::Num),
            "String" => Ok(VarType::Str),
            _ => Err(())
        }
    }
}


#[derive(Debug)]
struct ErrMsg {
    expected: TokenType,
    got: TokenType,
    line_number: u8,
    col_number: usize
}
