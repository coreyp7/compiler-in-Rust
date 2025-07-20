use crate::error::ErrMsg;
use crate::statement::Statement;
use crate::symbol_table::SymbolTable;
use crate::tokenizer::{Token, TokenType};

pub mod statement_parsers_impl;

pub trait StatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool;
    fn parse(&mut self, parser_context: &mut ParserContext<'_>) -> Statement;
}

// NOTE: I know this sucks and is a mess.
pub struct ParserContext<'a> {
    pub tokens: &'a [Token],
    pub current: usize,
    pub errors: Vec<ErrMsg>,
    pub symbol_table: &'a mut SymbolTable,
}

impl<'a> ParserContext<'a> {
    pub fn new(tokens: &'a [Token], symbol_table: &'a mut SymbolTable) -> Self {
        ParserContext {
            tokens,
            current: 0,
            errors: Vec::new(),
            symbol_table,
        }
    }

    pub fn get_curr_token(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub fn next_token(&mut self) {
        self.current += 1;
    }

    pub fn is_curr_token_type(&self, token_type: &TokenType) -> bool {
        self.get_curr_token().token_type == *token_type
    }

    pub fn get_error_if_curr_not_expected(&self, expected: TokenType) -> Option<ErrMsg> {
        if self.get_curr_token().token_type != expected {
            Some(ErrMsg::UnexpectedToken {
                expected,
                got: self.get_curr_token().token_type.clone(),
                line_number: self.get_curr_token().line_number,
            })
        } else {
            None
        }
    }

    pub fn expect_and_consume(&mut self, expected: TokenType) -> Result<(), ErrMsg> {
        if let Some(error) = self.get_error_if_curr_not_expected(expected) {
            self.errors.push(error.clone());
            Err(error)
        } else {
            self.next_token();
            Ok(())
        }
    }

    pub fn at_end(&self) -> bool {
        self.get_curr_token().token_type == TokenType::EOF
    }
}

/// Main statement parser coordinator
/// This replaces the giant statement() method in AstBuilder
pub struct StatementParserCoordinator {
    parsers: Vec<Box<dyn StatementParser>>,
}

impl StatementParserCoordinator {
    pub fn new() -> Self {
        use statement_parsers_impl::*;

        StatementParserCoordinator {
            parsers: vec![
                Box::new(PrintStatementParser),
                Box::new(IfStatementParser),
                Box::new(WhileStatementParser),
                Box::new(ReturnStatementParser),
                Box::new(IdentityStatementParser), // Handles function calls and assignments
                Box::new(VarDeclarationStatementParser),
                Box::new(FunctionDeclarationStatementParser),
                Box::new(NewlineStatementParser),
            ],
        }
    }

    pub fn parse_statement(&mut self, context: &mut ParserContext<'_>) -> Statement {
        let token_type = &context.get_curr_token().token_type;

        // Find the appropriate parser for this token type
        for parser in &mut self.parsers {
            if parser.can_parse(token_type) {
                let statement = parser.parse(context);
                context.next_token(); // Advance past the statement
                return statement;
            }
        }

        // No parser found - return a stub and advance
        context.next_token();
        Statement::TestStub
    }
}
