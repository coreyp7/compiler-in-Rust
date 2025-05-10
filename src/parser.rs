use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
//use colored::Colorize;
use std::collections::HashSet;
use std::io;
use std::io::Write;
use colored::Colorize;

pub struct TokenList {
    vec: Vec<Token>,
    curr_idx: usize,
    line_number: u32,
    symbols: HashSet<String>,
    pub code_str: String,
    debug: bool
}

impl TokenList<> {
    pub fn new(token_vec: Vec<Token>) -> TokenList {
        TokenList{ 
            vec: token_vec, 
            curr_idx: 0,
            line_number: 1,
            symbols: HashSet::new(),
            code_str: String::new(),
            debug: false
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

        if self.debug {
            if self.get_curr_token().token_type == TokenType::Newline {
                println!("\\n");
                self.line_number = self.line_number + 1;
                print!("{}| ", self.line_number);
                io::stdout().flush().unwrap();
            } else {
                print!("{:#?} ", self.get_curr_token().token_type);
                io::stdout().flush().unwrap();
            }
        }

        // Increment idx position in vec
        self.curr_idx += 1;
    }

    pub fn parse_tokens(&mut self) {
        if self.debug {
            println!("parse_tokens called!");
            println!("here's the contents of the vec in the struct");
            for token in &self.vec {
                println!("{:#?}", token);
            }
            println!("parser output: -----------------------------");
        }

        self.program();

        /* TODO; add arg that allows you to view debug info.
            This is a cool visual so leaving here.
        */
        if self.debug {
            println!("");
            println!("Here's the compiled c code:");
            println!("{}", self.code_str.green().bold());
        }
    }
    fn program(&mut self) {
        self.code_str.push_str("#include <stdio.h>\n");
        self.code_str.push_str("int main() {\n");

        while self.get_curr_token().token_type != TokenType::EOF {
            self.statement();
        }
        // Testing
        //self.code_str.push_str("printf(\"hello, compiler\");\n");

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
                    self.code_str.push_str("\\n\");\n");
                    //self.code_str.push_str("printf(\"\\n\");");
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
            TokenType::NumberType => {
                self.next_token();
                self.assert_curr_type_or_fail(&TokenType::Identity);

                let variable_name: String = self.get_curr_token().text.clone();
                self.code_str.push_str(&format!("int {} = ", &variable_name));
                self.symbols.insert(variable_name);

                self.next_token();
                self.assert_curr_type_or_fail(&TokenType::Colon);
                self.next_token();

                //self.code_str.push_str("int {} = ", &variable_name);

                self.expression();
                self.code_str.push_str(";\n");
            },
            TokenType::UpdateKeyword => {
                self.next_token();

                self.assert_curr_type_or_fail(&TokenType::Identity);
                let variable_name: String = self.get_curr_token().text.clone();
                self.code_str.push_str(&format!("{} = ", &variable_name));
                self.symbols.insert(variable_name);

                self.next_token();
                self.assert_curr_type_or_fail(&TokenType::LessThanEqualTo);
                self.next_token();

                self.expression();
                self.code_str.push_str(";\n");
            },
            TokenType::While => {
                self.next_token();
                self.code_str.push_str("while (");
                self.comparison();
                self.code_str.push_str(") {\n");
                self.assert_curr_type_or_fail(&TokenType::Do);
                self.next_token();

                // parse statement inside of body while curr isn't end if
                while !self.is_curr_token_type(&TokenType::EndWhile) {
                    self.statement(); 
                }

                // parse end while token
                self.assert_curr_type_or_fail(&TokenType::EndWhile);
                self.code_str.push_str("}\n");
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
                if self.debug {
                    println!("Skipping token below; not implemented yet.");
                }
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

    fn assert_curr_type_or_fail(&mut self, t_type: &TokenType){
       if self.is_curr_token_type(t_type) == false {
            println!("assert_curr_type_or_fail({:#?}): curr type is actually {:#?}",
                t_type, self.get_curr_token().token_type);
            std::process::exit(0);
        } 
    }
}

