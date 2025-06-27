use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
//use colored::Colorize;
use std::collections::HashSet;
use std::io;
use std::io::Write;
use colored::Colorize;

pub struct Node {
   value: u8 
}

pub struct AstBuilder<> {
    pub head: Node,
    pub tokens: Vec<Token>,
    curr_idx: usize,
    statements: Vec<Statement>,
    errors: Vec<ErrMsg>
}

impl AstBuilder<> {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        let node: Node = Node {value: 1};

        AstBuilder {
            head: node,
            tokens: token_vec,
            curr_idx: 0,
            statements: Vec::new(),
            errors: Vec::new()
        }
    }


    pub fn generate_ast(&mut self){
        self.program() 
    }

    fn get_curr_token(&self) -> &Token {
        &self.tokens[self.curr_idx]
    }


    fn next_token(&mut self){
        self.curr_idx = self.curr_idx + 1;
        println!("New token: {:?}", self.get_curr_token());
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

    fn program(&mut self){
        println!("program() start");  
        
        while self.get_curr_token().token_type != TokenType::EOF {
            self.statement();
        }
    }

    fn is_curr_token_type(&mut self, t_type: &TokenType) -> bool{
        return self.get_curr_token().token_type == *t_type;
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
                let comparison = self.comparison();
                println!("Here's the comparison created: {:?}", comparison);

                self.add_error_if_curr_not_expected(TokenType::Then);
                self.next_token();

                let mut statements: Vec<Statement> = Vec::new();

                while !self.is_curr_token_type(&TokenType::EndIf){
                    statements.push(self.statement());
                }

                self.next_token();

                statement = Statement::If{
                        comparison: comparison,
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
            TokenType::Newline => {
                statement = Statement::Newline;
            },
            _ => {
            }
        };

        let line_number = self.get_curr_token().line_number;
        let col_number = self.get_curr_token().col_number;
        let token_type = &self.get_curr_token().token_type;
        println!("{:#?} at line number {}", 
            statement,
            line_number
        );

        self.next_token();
        return statement;
    }

    fn comparison(&mut self) -> Comparison {
        let mut comparison = Comparison {
            expressions: Vec::new(),
            operators: Vec::new()
        };
        let expr1: Expression = self.expression(); // have this emulate an ouput
        comparison.expressions.push(expr1);

        // TODO: check that we're getting the op we're expecting.
        // Otherwise, we can include error detail and print error.
        // NEed to figure out a good way to keep track of errors
        // while allowing the coninuation of parsing.
        // Maybe jsut record (invalid) in this file, and then have a whole
        // extra step to analyze the AST and let user know we're expecting
        // something different than what they gave. (this feels sensible)
        let op1: ComparisonOperator = convert_token_type_to_comparison_op(
            self.get_curr_token().token_type.clone()
        );
        comparison.operators.push(op1);
        self.next_token();

        let expr2: Expression = self.expression(); // TODO: get current token as comparison operator
        comparison.expressions.push(expr2);

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

        println!("Created a primary: {:?}", primary);
        primary
    }
}

fn convert_token_type_to_comparison_op(token_type: TokenType) -> ComparisonOperator {
    match token_type {
        TokenType::EqualEqual => ComparisonOperator::equalequal,
        TokenType::NotEqual => ComparisonOperator::notequal,
        TokenType::LessThan => ComparisonOperator::lessthan,
        TokenType::LessThanEqualTo => ComparisonOperator::lessthanequalto,
        TokenType::GreaterThan => ComparisonOperator::greaterthan,
        TokenType::GreaterThanEqualTo => ComparisonOperator::greaterthanequalto,
        _ => ComparisonOperator::invalidop
    }    
}

fn convert_token_type_to_expression_op(token_type: TokenType) -> ExpressionOperator {
    match token_type {
        TokenType::Plus => ExpressionOperator::Plus,
        TokenType::Minus => ExpressionOperator::Minus,
        _ => ExpressionOperator::invalidop
    }
}


fn convert_token_type_to_term_op(token_type: TokenType) -> TermOperator {
    match token_type {
        TokenType::Asterisk => TermOperator::Multiply,
        TokenType::Slash => TermOperator::Divide,
        _ => TermOperator::invalidop
    }
}

#[derive(Debug)]
enum Operation {
    Plus,
    Minus,
    Multiply,
    Divide
}

#[derive(Debug)]
enum Statement {
    Print {
        content: String,
        line_number: u8
    },
    If {
        comparison: Comparison,
        statements: Vec<Statement>
    },
    While {
        comparison: Comparison,
        statements: Vec<Statement>
    },
    Newline,
    TestStub
}




/** 
* The way this works:
* These are two lists of all of the expresisons and operators in between.
* So, it is in the order specified in code.
* that is: expressions[0], operators[0], expressions[1], 
*          operators[1], expressions[2], etc.....
*/
#[derive(Debug)]
struct Comparison {
    expressions: Vec<Expression>,
    operators: Vec<ComparisonOperator>
}

#[derive(Debug)]
enum ComparisonOperator {
    equalequal,
    notequal,
    lessthan,
    lessthanequalto, 
    greaterthan,
    greaterthanequalto,
    invalidop
}

// Either + or -
#[derive(Debug)]
struct Expression {
    terms: Vec<Term>,
    operators: Vec<ExpressionOperator>
}

#[derive(Debug)]
enum ExpressionOperator {
    Plus,
    Minus,
    invalidop
}

#[derive(Debug)]
struct Term {
    unarys: Vec<Unary>,
    operations: Vec<TermOperator>
}

#[derive(Debug)]
enum TermOperator {
    Multiply,
    Divide,
    invalidop // TODO: these should be changed to no op
}

#[derive(Debug)]
struct Unary {
    operation: Option<ExpressionOperator>,
    primary: Primary
}

#[derive(Debug)]
enum Primary {
    Number {
       value: String //TODO: change this to u8 and do conversions :(
    },
    Identity {
        name: String
    },
    Error {
        detail: String
    }
}

#[derive(Debug)]
struct ErrMsg {
    expected: TokenType,
    got: TokenType,
    line_number: u8,
    col_number: usize
}
