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
        let curr_token = self.get_curr_token();
        match curr_token.token_type {
            TokenType::Print => {
                //println!("{:?}", curr_token);                
                statement = Some(
                    Statement::Print{
                        content: curr_token.text.clone(),
                        line_number: curr_token.line_number
                    }
                );
            },
            _ => {
                
            }
        };

        match statement {
            Some(value) => {
                println!("{:?}", value);                
                self.statements.push(value);
            },
            None => println!("No statement generated. Skipping.")
        };

        self.next_token();
    }
}


#[derive(Debug)]
enum Statement {
    Print {
        content: String,
        line_number: u8
    }
}
