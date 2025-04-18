use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
use std::process;
use std::io;
use std::io::Write;
use colored::Colorize;
use std::collections::HashSet;

pub struct TokenList {
    vec: Vec<Token>,
    curr_idx: usize,
    line_number: u32,
    symbols: HashSet<String>,
    code_str: String
}

impl TokenList<> {
    pub fn new(token_vec: Vec<Token>) -> TokenList {
        TokenList{ 
            vec: token_vec, 
            curr_idx: 0,
            line_number: 1,
            symbols: HashSet::new(),
            code_str: String::new()
        }
    }

    fn get_curr_token(&self) -> &Token {
        return &self.vec[self.curr_idx];
    }


    fn next_token(&mut self) {
        // If we're in the middle of parsing a statement or something and we
        // don't see a keyword we're expecting, then we'll go past EOF and crash.
        // So, check here if current token is EOF, since we can't go next from EOF.
        // In future, maybe we can keep track of what we're expecting and print
        // it as debug text.
        if self.get_curr_token().token_type == TokenType::EOF {
            println!("-");
            println!("ERROR: Found EOF without close to statement.");
            println!("Check your syntax and see if there's something wrong.");
        }

        if self.get_curr_token().token_type == TokenType::Newline {
            println!("\\n");
            self.line_number = self.line_number + 1;
            print!("{}| ", self.line_number);
            io::stdout().flush().unwrap();
        } else {
            print!("{:#?} ", self.get_curr_token().token_type);
            io::stdout().flush().unwrap();
        }
        self.curr_idx += 1;
    }

    pub fn parse_tokens(&mut self) {
        println!("parse_tokens called!");
        println!("here's the contents of the vec in the struct");
        for token in &self.vec {
            println!("{:#?}", token);
        }
        println!("parser output: -----------------------------");

        self.program();

        println!("Here's the code_str after running program():");
        println!("{}", self.code_str.green().bold());
    }

    fn program(&mut self) {
        self.code_str.push_str("#include <stdio.h>\n");
        self.code_str.push_str("int main() {\n");

        while self.get_curr_token().token_type != TokenType::EOF {
            //println!("{:?} != TokenType::EOF", self.get_curr_token().token_type);
            self.statement();
        }
        // Testing
        //self.code_str.push_str("printf(\"hello, compiler\");\n");

        println!("Reached EOF");
        self.code_str.push_str("}\n");

    }

    fn statement(&mut self) {
        /*
        * Rust note for Corey for learning:
        * Borrowing this because the match statement allows you to take ownership
        * inside of the match statement.
        * Specifically, here the 'x if x != TokenType::NewLine' was taking ownership
        * of token_type (I believe). Gives some insight into how things are 
        * actually passed around.
        */
        //let curr_token: &Token = &self.get_curr_token();
        match &self.get_curr_token().token_type {
        //match curr_token.token_type {
            TokenType::Print => {
                self.code_str.push_str("printf(\"");
                self.next_token();

                // from here, needs to be either expression or a string
                if self.get_curr_token().token_type == TokenType::Str {
                    // This is a shit solution, but I was confused figuring out
                    // where a mutable borrow was ocurring here.
                    // Cloning the token text solves this, although clumsily.
                    let string_content: String = self.get_curr_token().text.clone();
                    self.code_str.push_str(
                        &string_content
                    );
                    self.code_str.push_str("\");\n");
                    self.next_token();
                } else {
                    // Must be an expression
                    //self.expression();
                    // TODO: need to figure out how to turn this to c code
                    self.next_token();
                }

            },
            TokenType::If => {
                self.code_str.push_str("if (");
                self.next_token();

                // parse comparison (part in parentheses)
                self.comparison();
                self.code_str.push_str(") {\n");

                self.assert_curr_type_or_fail(&TokenType::Then);
                self.next_token();


                // parse statement inside of body while curr isn't end if
                while !self.is_curr_token_type(&TokenType::EndIf) {
                    self.statement(); 
                }

                // parse end if token
                self.assert_curr_type_or_fail(&TokenType::EndIf);
                self.code_str.push_str("}\n");
                self.next_token();
                
            },
            TokenType::Let => {
                // this will require some more complex logic to turn this into code.
                // what do I need to know to emit text.
                // - variable name
                // - type
                // in fact, should we rewrite the tokenizer/parser to check for a type?
                // it would make emitting much easier, but perhaps lazy.
                // The reason the teeny tiny parser doesn't run into this is
                // because let is only used for declaring numbers, which is generically
                // a float in c (uses label keyword for strings).
                // Do what you feel.

                /*
                Ideas:
                - Have the declaration being with a type that will be tokenized,
                and we can read it in order easily here; assignment statements
                will just begin with variable name (so, if the parser sees a symbol
                that isn't a keyword or recognized thing in the first position of a line,
                then we'd just check if it is a symbol in the symbol map. More complications
                from this though.....
                - Have distinct keywords for each datatype, at start of statement.
                string, number, etc. Maybe just these two. We'd have to tokenize 
                these different types now, and change parse logic around. But in
                general would make things a bit easier for me when emitting
                (emitting to c, could be harder if going to rust)
                */

                self.next_token();
                self.assert_curr_type_or_fail(&TokenType::Identity);

                let variable_name: String = self.get_curr_token().text.clone();
                //self.symbols.insert(self.get_curr_token().text.clone());
                self.symbols.insert(&variable_name);

                self.next_token();
                self.assert_curr_type_or_fail(&TokenType::Equal);
                self.next_token();

                self.expression();
            },
            TokenType::While => {
                self.next_token();
                self.comparison();
                self.assert_curr_type_or_fail(&TokenType::Do);
                self.next_token();

                // parse statement inside of body while curr isn't end if
                while !self.is_curr_token_type(&TokenType::EndWhile) {
                    self.statement(); 
                }

                // parse end while token
                self.assert_curr_type_or_fail(&TokenType::EndWhile);
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
                //println!("Skipping token below; not implemented yet.");
                self.next_token();
            }//todo!()
        }
        
        self.assert_curr_type_or_fail(&TokenType::Newline);
        self.next_token();
    }

    fn comparison(&mut self) {
        self.expression();
        if !self.is_curr_token_comparison_operator(){
            println!("ERROR: Comparison operator not found between expressions in if statement.");
            println!("Found {:#?} instead of comparison operator.", self.get_curr_token().token_type);
            std::process::exit(0);
        }
        let string_content: String = self.get_curr_token().text.clone();
        self.code_str.push_str(&string_content);
        self.next_token();
        self.expression();
        
        /*
        * This is for processing further expressions (because you can have more than 2)
        * Add back when is_curr_token_comparison_operator actually written.
        */
        while self.is_curr_token_comparison_operator() {
            self.next_token();
            self.expression();
        }
         
    }

    fn expression(&mut self) {
        self.term();
        while self.is_curr_token_type(&TokenType::Plus) || self.is_curr_token_type(&TokenType::Minus) {
            let string_content: String = self.get_curr_token().text.clone();
            self.code_str.push_str(&string_content);

            self.next_token();
            self.term();
        }
    }

    fn term(&mut self) {
        self.unary();
        while self.is_curr_token_type(&TokenType::Asterisk) || self.is_curr_token_type(&TokenType::Slash) {
            let string_content: String = self.get_curr_token().text.clone();
            self.code_str.push_str(&string_content);

            self.next_token();
            self.unary();
        }
    }

    fn unary(&mut self) {
        if self.is_curr_token_type(&TokenType::Plus) || self.is_curr_token_type(&TokenType::Minus) {
            let string_content: String = self.get_curr_token().text.clone();
            self.code_str.push_str(&string_content);

            self.next_token();
        }
        self.primary();
    }

    fn primary(&mut self) {
        if self.is_curr_token_type(&TokenType::Number) {
            let string_content: String = self.get_curr_token().text.clone();
            self.code_str.push_str(&string_content);

            self.next_token();
        } else if self.is_curr_token_type(&TokenType::Identity) {
            if !self.symbols.contains(&self.get_curr_token().text) {
                println!("ERROR");
                println!("|PARSER| referencing symbol before assignment: '{}'", 
                    self.get_curr_token().text 
                );
                std::process::exit(0);
            }
            let string_content: String = self.get_curr_token().text.clone();
            self.code_str.push_str(&string_content);

            self.next_token();
        } else {
            println!("ERROR");
            println!("|PARSER| when parsing token on line {}; {:?}", 
                self.line_number, 
                self.get_curr_token().token_type
            );
            std::process::exit(0);
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

    fn ensure_newline(&mut self){
        if !self.is_curr_token_type(&TokenType::Newline){
            println!("NO NEW LINE PRESENT; something very wrong.");
            process::abort();
        }
    } 

    fn assert_curr_type_or_fail(&mut self, t_type: &TokenType){
       if(self.is_curr_token_type(t_type) == false){
        // TODO: print error information for user
            println!("assert_curr_type_or_fail({:#?}): curr type is actually {:#?}",
                t_type, self.get_curr_token().token_type);
            //println!("exiting via assert_curr_type_or_fail");
            std::process::exit(0);
        } 
    }
}

