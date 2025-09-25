use super::TokenType;

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String, // used for identifiers, strings, numbers
    pub token_type: TokenType,
    pub line_number: u8,
    pub col_number: usize,
}
