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
        println!("real output: -----------------------------");

        self.program();
    }

    fn program(&mut self) {
        println!("PROGRAM");

        // Parse every statement in the src file
        while self.get_curr_token().token_type != TokenType::EOF {
            //println!("start of loop: token type is {:#?}", self.get_curr_token().token_type);
            self.statement();
        }
        println!("Reached EOF");
    }

    fn statement(&mut self) {
        // print stuff
        // print (expression || string)

        /*
        * Borrowing this because the match statement allows you to take ownership
        * inside of the match statement.
        * Specifically, here the 'x if x != TokenType::NewLine' was taking ownership
        * of token_type (I believe). Gives some insight into how things are 
        * actually passed around.
        */
        match &self.get_curr_token().token_type {
            TokenType::Print => {
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

            },
            TokenType::If => {
                self.next_token();

                // parse comparison (part in parentheses)
                self.parse_comparison();

                // check for starting curly brace and newline
                self.assert_curr_type_or_fail(TokenType::Then);
                self.next_token();
                self.assert_curr_type_or_fail(TokenType::Newline);
                self.next_token();

                // parse statement inside of body while curr isn't end if
                while !self.is_curr_token_type(TokenType::EndIf) {
                    self.statement(); 
                }

                // parse end if token
                self.assert_curr_type_or_fail(TokenType::EndIf);
                self.next_token();
                
            },
            TokenType::Newline => (),
            /*
            x if *x != TokenType::Newline => {
                // Generic catch-all for all non implemented rules
                self.next_token();
            },
            */
            _ => {
                println!("Skipping token below; not implemented yet.");
                self.next_token();
            }//todo!()
        }
        
        self.assert_curr_type_or_fail(TokenType::Newline);
        self.next_token();
    }

    fn parse_comparison(&mut self) {
        //TODO: COREY write this next.
        
        // temp for testing
        while !self.is_curr_token_type(TokenType::EndIf) {
            self.next_token(); 
        }
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

    fn assert_curr_type_or_fail(&mut self, t_type: TokenType){
       if(self.is_curr_token_type(t_type) == false){
        // TODO: print error information for user
            println!("exiting via assert_curr_type_or_fail");
            std::process::exit(0);
        } 
    }
}

