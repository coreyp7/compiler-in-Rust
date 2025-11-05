use super::TokenType;

#[derive(Debug, Clone)]
pub struct Token {
    pub lexeme: String,
    pub token_type: TokenType,
    pub line_number: u32,
    pub col_number: usize,
}
