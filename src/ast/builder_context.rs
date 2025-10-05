use super::statement::Statement;
use crate::tokenizer::Token;

// Used to track state in the ast builder.
pub struct BuilderContext {
    tokens: Vec<Token>,
    pub idx: usize,
    pub statements: Vec<Statement>,
}

impl BuilderContext {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            idx: 0,
            statements: Vec::new(),
        }
    }

    pub fn get_curr(&self) -> &Token {
        &self.tokens[self.idx]
    }

    pub fn is_at_end(&self) -> bool {
        self.idx >= self.tokens.len()
    }

    pub fn advance(&mut self) {
        if !self.is_at_end() {
            self.idx += 1;
        }
    }

    pub fn peek_next(&self) -> Option<&Token> {
        if self.idx + 1 < self.tokens.len() {
            Some(&self.tokens[self.idx + 1])
        } else {
            None
        }
    }
}
