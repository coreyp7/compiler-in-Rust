use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
use std::process;

pub struct TokenList {
    pub vec: Vec<Token>,
    pub curr_idx: usize
}

impl TokenList<> {
    fn get_curr_token(&self) -> &Token {
        return &self.vec[self.curr_idx];
    }

    fn next_token(&mut self) {
        println!("{:#?}", self.get_curr_token().token_type);
        self.curr_idx += 1;
    }

    pub fn parse_tokens(&mut self) {
        println!("parse_tokens called!");
        println!("here's the contents of the vec in the struct");
        for token in &self.vec {
            println!("{:#?}", token);
        }

        self.program();
    }

    fn program(&mut self) {
        println!("PROGRAM");

        // Parse every statement in the src file
        while self.get_curr_token().token_type != TokenType::EOF {
            println!("start of loop: token type is {:#?}", self.get_curr_token().token_type);
            self.statement();
        }
    }

    fn statement(&mut self) {
        // print stuff
        // print (expression || string)
        if self.get_curr_token().token_type == TokenType::Print {
            self.next_token();

            // from here, needs to be either expression or a string
            if self.get_curr_token().token_type == TokenType::Str {
                // Simple string
                self.next_token();
            } else {
                // Must be an expression
                //self.expression();
                self.next_token();
            }
        } else if self.get_curr_token().token_type != TokenType::Newline {
            // Generic catch-all for all non implemented rules
            self.next_token();
        }    
        
        self.ensure_newline();
        self.next_token();
    }

    fn is_curr_token_type(&mut self, t_type: TokenType) -> bool{
        return self.get_curr_token().token_type == t_type;
    }

    fn ensure_newline(&mut self){
        if !self.is_curr_token_type(TokenType::Newline){
            println!("NO NEW LINE PRESENT; something very wrong.");
            process::abort();
        }
    } 
}

