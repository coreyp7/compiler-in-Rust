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
    statements: Vec<Statement> 
}

impl AstBuilder<> {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        let node: Node = Node {value: 1};

        AstBuilder {
            head: node,
            tokens: token_vec,
            curr_idx: 0,
            statements: Vec::new()
        }
    }


    pub fn generate_ast(&mut self){
        self.program() 
    }

    fn get_curr_token(&mut self) -> &Token {
        &self.tokens[self.curr_idx]
    }


    fn next_token(&mut self){
        self.curr_idx = self.curr_idx + 1;
    }

    fn program(&mut self){
        println!("program() start");  
        
        while self.get_curr_token().token_type != TokenType::EOF {
            self.statement();
        }
    }

    fn statement(&mut self) {
        let mut statement: Option<Statement> = None;
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

                statement = Some(
                    Statement::Print{
                        content: string_content,
                        line_number: self.get_curr_token().line_number
                    }
                );
            },
            TokenType::If => {
                /**

                Notes for future corey.

                So, the way the structs are written are a bit naive.
                A comparison MUST be 2 expressions with a comparison operator
                in between them.

                However, optionally, a comparison can have 0 or more expressions
                prefixed with another operator.

                How will this data look? How should this be organized?

                Idea:
                A vector (or two?) could keep these in order.
                So, a vector<Expression>
                and a vector<ComparisonOperator>.

                As we navigate through each token we add them to these lists.
                Keep looping through tokens until there isn't a comparison operator next.
                Then, condition the data into what makes sense for the
                comparison struct.

                Idea 2:
                Have this be a linkedlist of structs, and have the node pointer
                be an optional. Then the links to the next expression go
                until the optional is None.
                */
            },
            TokenType::Newline => {
                // I don't think I have to do anything here.
            },
            _ => {
                
            }
        };

        match statement {
            Some(value) => {
                println!("{:?}", value);                
                self.statements.push(value);
            },
            None => {
                let line_number = self.get_curr_token().line_number;
                let col_number = self.get_curr_token().col_number;
                let token_type = &self.get_curr_token().token_type;
                if *token_type != TokenType::Newline {
                    println!("Skipping {:?} at {},{}", 
                        token_type,
                        line_number,
                        col_number
                    );
                }
            }
        };

        self.next_token();
    }

    fn comparison(&mut self) {
        //self.expression();j
        //self.next_token();
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
        statement: Box<Statement>
    }
}

#[derive(Debug)]
struct Comparison {
    exp_left: Expression,
    operation: Operation,
    exp_right: Expression
}

// Either + or -
#[derive(Debug)]
struct Expression {
    term_left: Term,
    operation: Operation,
    term_right: Term
}

#[derive(Debug)]
struct Term {
    unary_left: Unary,
    operation: Operation,
    unary_right: Unary
}

#[derive(Debug)]
struct Unary {
    operation: Operation,
    primary: Primary
}

#[derive(Debug)]
enum Primary {
    Number {
       value: u8 
    },
    Identity {
        name: String
    },
    Error {
        detail: String
    }
}
