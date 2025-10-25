use super::parse_error::ParseError;
use super::statement::Statement;
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;

// Used to track state in the ast builder.
#[derive(Debug)]
pub struct BuilderContext {
    tokens: Vec<Token>,
    pub idx: usize,
    pub statements: Vec<Statement>,
    pub errors: Vec<ParseError>,
}

impl BuilderContext {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            idx: 0,
            statements: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_curr(&self) -> &Token {
        &self.tokens[self.idx]
    }

    pub fn is_at_end(&self) -> bool {
        //self.idx >= self.tokens.len()

        // Leaving for now, but this makes a lot more sense.
        self.get_curr().token_type == TokenType::EOF
    }

    pub fn advance(&mut self) {
        if !self.is_at_end() {
            self.idx += 1;
        }
    }

    /**
     * When unexpected tokens are found by the builder, then we want to skip
     * the statement. Call this when you want to skip all tokens until the
     * beginning of the next statement (after the next semicolon).
     */
    pub fn advance_to_next_statement(&mut self) {
        println!("entered advance_to_next_statement");
        println!("idx: {}", &self.idx);
        println!("curr: {:#?}", &self.get_curr());
        //wait_for_input();
        while !self.is_at_end() && self.get_curr().token_type != TokenType::Semicolon {
            self.advance();
        }
        if !self.is_at_end() {
            self.advance();
        }
        println!("left advance_to_next_statement: here's state of context now.");
        if !self.is_at_end() {
            println!("idx: {}", &self.idx);
            println!("curr: {:#?}", &self.get_curr());
        } else {
            println!("at EOF");
        }
    }

    pub fn peek_next(&self) -> Option<&Token> {
        if self.idx + 1 < self.tokens.len() {
            Some(&self.tokens[self.idx + 1])
        } else {
            None
        }
    }

    /// Adds error and advances to next statement
    pub fn handle_parse_error(&mut self, error: ParseError) {
        self.errors.push(error);
        self.advance_to_next_statement();
    }
}
