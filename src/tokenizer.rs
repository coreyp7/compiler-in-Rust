use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str::FromStr;

pub struct Tokenizer {
    line_number: u8,
    /*
    Okay, so having this here is pretty wack, but allows easy tracking of the
    current column in the current line being parsed.

    So, this is exclusively used in tokenize_line. And purely for informational
    purposes to show as error messages to the user.
    */
    curr_byte_index_in_line: usize,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            line_number: 1,
            curr_byte_index_in_line: 0,
        }
    }

    pub fn tokenize_file(&mut self, src_file: &mut File) -> Vec<Token> {
        let reader = BufReader::new(src_file);

        let mut token_vec: Vec<Token> = Vec::new();

        for line_result in reader.lines() {
            if let Ok(line_str) = line_result {
                let mut tokens = self.tokenize_line(line_str);
                token_vec.append(&mut tokens);
                self.line_number += 1;
            }
        }
        token_vec.push(self.create_token(TokenType::EOF, String::new()));

        token_vec
    }

    fn create_token(&mut self, token_type_param: TokenType, text_param: String) -> Token {
        Token {
            token_type: token_type_param,
            text: text_param,
            line_number: self.line_number,
            col_number: self.curr_byte_index_in_line + 1,
        }
    }

    fn tokenize_line(&mut self, line: String) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let line_bytes: &[u8] = line.as_bytes();
        //let mut curr_byte_index = 0;
        self.curr_byte_index_in_line = 0;

        while self.curr_byte_index_in_line < line_bytes.len() {
            let curr = line_bytes[self.curr_byte_index_in_line];
            let curr = curr as char;
            let mut next: Option<char> = None;
            if self.curr_byte_index_in_line < line_bytes.len() - 1 {
                next = Some(line_bytes[self.curr_byte_index_in_line + 1] as char);
            }

            let (token, skip_rest_of_line) = self.match_character(&line_bytes, curr, next);

            self.curr_byte_index_in_line += 1;

            if token.token_type != TokenType::UnsupportedSymbolError {
                tokens.push(token);
            }

            if skip_rest_of_line {
                // This is a comment; we're skipping the rest of the line.
                self.curr_byte_index_in_line = line_bytes.len();
            }
        }

        // End tokens list with new line, since this is the end of the line
        tokens.push(self.create_token(TokenType::Newline, String::new()));

        tokens
    }

    /**
    Can either be:
    - a String
    - a Number
    - an Identifier

    Specify which kind of token you're looking for the end of.

    Given
    - string buffer
    - start of token

    Will return -> (Token, new position in string buffer)
    */
    fn create_token_from_text(
        &mut self,
        line_bytes: &[u8],
        token_start: usize,
        token_type: TokenType,
    ) -> (Token, usize) {
        let mut str_byte_buffer = token_start;
        let mut end_of_string_idx: Option<usize> = None;
        let mut string_content: String = String::new();

        while str_byte_buffer < line_bytes.len() {
            let curr_char = line_bytes[str_byte_buffer] as char;
            // 3 cases: String, Number, or Identifier
            // this could be organized better I think but leaving as is for now,
            // since its clear to me how this is organized
            match token_type {
                TokenType::Number => {
                    if curr_char.is_numeric() == false {
                        end_of_string_idx = Some(str_byte_buffer - 1);
                        str_byte_buffer = line_bytes.len();
                    } else if str_byte_buffer == line_bytes.len() - 1 {
                        string_content.push(curr_char);
                        end_of_string_idx = Some(str_byte_buffer);
                        str_byte_buffer = line_bytes.len();
                    } else {
                        string_content.push(curr_char);
                    }
                }
                TokenType::Str => {
                    if curr_char == '"' {
                        end_of_string_idx = Some(str_byte_buffer);
                        str_byte_buffer = line_bytes.len();
                    } else {
                        string_content.push(curr_char);
                    }
                }
                TokenType::Identity => {
                    /*
                    Identities follow the same parsing rules as keywords.
                    So, they are both processed in here, and then we figure out
                    later if this token is a keyword instead of an Identity.
                    */
                    if curr_char.is_alphabetic() == false {
                        end_of_string_idx = Some(str_byte_buffer - 1);
                        str_byte_buffer = line_bytes.len();
                    } else if str_byte_buffer == line_bytes.len() - 1 {
                        string_content.push(curr_char);
                        end_of_string_idx = Some(str_byte_buffer);
                        str_byte_buffer = line_bytes.len();
                    } else {
                        string_content.push(curr_char);
                    }
                }
                _ => (),
            }

            str_byte_buffer += 1;
        }

        let mut new_curr_byte_idx: usize = 0;
        let mut token = self.create_token(TokenType::UnsupportedSymbolError, String::from(""));

        match end_of_string_idx {
            None => {
                // TODO: this needs to make a bigger deal out of this.
                // This compilation should end here; unable to tokenize.
                ()
            }
            Some(new_curr_idx) => {
                token = self.create_token(token_type, string_content);
                new_curr_byte_idx = new_curr_idx;
            }
        };

        if token.token_type == TokenType::Identity {
            // check if this is a keyword, and change it accordingly
            // create function that returns the tokentype given a string keyword.
            // if it returns Some(type) then update the tokentype.
            // else None, then keep this as an Identity, since it doesn't match any keyword.
            let f = TokenType::from_str(&token.text);
            match f {
                Ok(token_type) => token.token_type = token_type,
                _ => {}
            }
        }

        (token, new_curr_byte_idx)
    }
}

#[derive(Debug)]
pub struct Token {
    pub text: String, // used for identifiers, strings, numbers
    pub token_type: TokenType,
    pub line_number: u8,
    pub col_number: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum TokenType {
    EOF = 0,
    Newline,
    Number,
    Identity,
    Str,
    // Keywords
    Label = 100, //unused
    //NumberType, // for declaring variable 'Number'
    VarDeclaration,
    FunctionDeclaration,
    UpdateKeyword, // assigning to variables
    Goto,
    Print,
    Input,
    Let,
    If,
    Then,
    EndIf,
    While,
    Do,
    EndWhile,
    Return,
    LeftParen,
    RightParen,
    Comma,
    Arrow,
    EndFunction,
    // Operators
    Equal = 200,
    Plus,
    Minus,
    Asterisk,
    Slash,
    EqualEqual, // 205
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo, // 210
    DoubleAmpersand,
    DoubleBar,
    Bang,
    Colon,
    UnsupportedSymbolError = 900,
    // Won't get through to the parser, just for processing in here.
    Space,
}
/**
 * This allows for easy matching of a keyword (as a String) to its
 * TokenType counterpart.
*/
impl FromStr for TokenType {
    type Err = ();

    fn from_str(input: &str) -> Result<TokenType, Self::Err> {
        match input {
            "label" => Ok(TokenType::Label),
            "goto" => Ok(TokenType::Goto),
            "print" => Ok(TokenType::Print),
            "input" => Ok(TokenType::Input),
            "let" => Ok(TokenType::Let),
            "if" => Ok(TokenType::If),
            "then" => Ok(TokenType::Then),
            "endIf" => Ok(TokenType::EndIf),
            "while" => Ok(TokenType::While),
            "do" => Ok(TokenType::Do),
            "endWhile" => Ok(TokenType::EndWhile),
            "Number" | "String" => Ok(TokenType::VarDeclaration),
            "update" => Ok(TokenType::UpdateKeyword),
            "function" => Ok(TokenType::FunctionDeclaration),
            "return" => Ok(TokenType::Return),
            "endFunction" => Ok(TokenType::EndFunction),
            _ => Err(()),
        }
    }
}

// TODO: this may be kindof gross having to edit both here and above.
// Look into if any workaround available.
impl TokenType {
    /// Converts a TokenType to its string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            TokenType::EOF => "EOF",
            TokenType::Newline => "\\n",
            TokenType::Number => "Number",
            TokenType::Identity => "Identity",
            TokenType::Str => "String",
            // Keywords
            TokenType::Label => "label",
            TokenType::VarDeclaration => "VarDeclaration",
            TokenType::UpdateKeyword => "update",
            TokenType::FunctionDeclaration => "FunctionDeclaration",
            TokenType::Goto => "goto",
            TokenType::Print => "print",
            TokenType::Input => "input",
            TokenType::Let => "let",
            TokenType::If => "if",
            TokenType::Then => "then",
            TokenType::EndIf => "endIf",
            TokenType::While => "while",
            TokenType::Do => "do",
            TokenType::EndWhile => "endWhile",
            // Operators
            TokenType::Equal => "=",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Asterisk => "*",
            TokenType::Slash => "/",
            TokenType::EqualEqual => "==",
            TokenType::NotEqual => "!=",
            TokenType::LessThan => "<",
            TokenType::LessThanEqualTo => "<=",
            TokenType::GreaterThan => ">",
            TokenType::GreaterThanEqualTo => ">=",
            TokenType::DoubleAmpersand => "&&",
            TokenType::DoubleBar => "||",
            TokenType::Bang => "!",
            TokenType::Colon => ":",
            TokenType::UnsupportedSymbolError => "UnsupportedSymbol",
            TokenType::Space => " ",
            TokenType::Return => "return",
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::Comma => ",",
            TokenType::Arrow => "->",
            TokenType::EndFunction => "endFunction",
        }
    }
}

#[derive(Debug)]
pub enum TokenMatch {
    Single(TokenType),
    Double(char, TokenType), // (next_char, token_if_matched)
    Comment,                 // Special case for comments
}

impl Tokenizer {
    fn get_char_token_map() -> HashMap<char, TokenMatch> {
        use TokenMatch::*;
        use TokenType::*;

        [
            ('+', Single(Plus)),
            ('*', Single(Asterisk)),
            (':', Single(Colon)),
            ('(', Single(LeftParen)),
            (')', Single(RightParen)),
            (',', Single(Comma)),
            ('-', Double('>', Arrow)),
            ('/', Comment),
            ('=', Double('=', EqualEqual)),
            ('<', Double('=', LessThanEqualTo)),
            ('>', Double('=', GreaterThanEqualTo)),
            ('!', Double('=', NotEqual)),
            ('&', Double('&', DoubleAmpersand)),
            ('|', Double('|', DoubleBar)),
        ]
        .into_iter()
        .collect()
    }

    fn get_single_char_fallback(ch: char) -> TokenType {
        match ch {
            '-' => TokenType::Minus,
            '=' => TokenType::Equal,
            '<' => TokenType::LessThan,
            '>' => TokenType::GreaterThan,
            '!' => TokenType::Bang,
            '/' => TokenType::Slash,
            _ => TokenType::UnsupportedSymbolError,
        }
    }

    fn match_character(
        &mut self,
        line_bytes: &[u8],
        curr: char,
        next: Option<char>,
    ) -> (Token, bool) {
        let token_map = Self::get_char_token_map();

        if let Some(token_match) = token_map.get(&curr) {
            match token_match {
                TokenMatch::Single(token_type) => {
                    (self.create_token(*token_type, String::from(curr)), false)
                }
                TokenMatch::Double(expected_next, double_token_type) => {
                    if matches!(next, Some(x) if x == *expected_next) {
                        self.curr_byte_index_in_line += 1;
                        let double_char = format!("{}{}", curr, expected_next);
                        (self.create_token(*double_token_type, double_char), false)
                    } else {
                        let fallback_type = Self::get_single_char_fallback(curr);
                        (self.create_token(fallback_type, String::from(curr)), false)
                    }
                }
                TokenMatch::Comment => {
                    if matches!(next, Some(x) if x == '!') {
                        // Return a dummy token and signal to skip rest of line
                        (
                            self.create_token(TokenType::UnsupportedSymbolError, String::new()),
                            true,
                        )
                    } else {
                        (
                            self.create_token(TokenType::Slash, String::from(curr)),
                            false,
                        )
                    }
                }
            }
        } else {
            // Handle strings, keywords, and numbers.
            // They are handled slightly differently depending on which, so we
            // condition the data we pass into create_token_from_text.
            let token_type = match curr {
                '"' => Some(TokenType::Str),
                x if x.is_numeric() => Some(TokenType::Number),
                x if x.is_alphabetic() => Some(TokenType::Identity),
                _ => None,
            };

            if let Some(token_type) = token_type {
                let start_index = if token_type == TokenType::Str {
                    self.curr_byte_index_in_line + 1 // Skip opening quote
                } else {
                    self.curr_byte_index_in_line
                };

                let result = self.create_token_from_text(line_bytes, start_index, token_type);
                self.curr_byte_index_in_line = result.1;
                (result.0, false)
            } else {
                (
                    self.create_token(TokenType::UnsupportedSymbolError, String::from(curr)),
                    false,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    // Helper function to create a test tokenizer with sample input
    fn create_test_tokenizer() -> Tokenizer {
        Tokenizer::new()
    }

    // Helper function to assert token matches expected type and text
    fn assert_token(
        token: &Token,
        expected_type: TokenType,
        expected_text: &str,
        line: u8,
        col: usize,
    ) {
        assert_eq!(token.token_type, expected_type);
        assert_eq!(token.text, expected_text);
        assert_eq!(token.line_number, line);
        assert_eq!(token.col_number, col);
    }

    #[test]
    fn test_tokenizer_new() {
        let tokenizer = Tokenizer::new();
        assert_eq!(tokenizer.line_number, 1);
        assert_eq!(tokenizer.curr_byte_index_in_line, 0);
    }

    #[test]
    fn test_create_token() {
        let mut tokenizer = create_test_tokenizer();
        tokenizer.curr_byte_index_in_line = 5;

        let token = tokenizer.create_token(TokenType::Plus, "+".to_string());

        assert_eq!(token.token_type, TokenType::Plus);
        assert_eq!(token.text, "+");
        assert_eq!(token.line_number, 1);
        assert_eq!(token.col_number, 6); // curr_byte_index + 1
    }

    #[test]
    fn test_get_char_token_map() {
        let map = Tokenizer::get_char_token_map();

        // Test single character tokens
        assert!(matches!(
            map.get(&'+'),
            Some(TokenMatch::Single(TokenType::Plus))
        ));
        assert!(matches!(
            map.get(&'*'),
            Some(TokenMatch::Single(TokenType::Asterisk))
        ));
        assert!(matches!(
            map.get(&':'),
            Some(TokenMatch::Single(TokenType::Colon))
        ));
        assert!(matches!(
            map.get(&'('),
            Some(TokenMatch::Single(TokenType::LeftParen))
        ));
        assert!(matches!(
            map.get(&')'),
            Some(TokenMatch::Single(TokenType::RightParen))
        ));
        assert!(matches!(
            map.get(&','),
            Some(TokenMatch::Single(TokenType::Comma))
        ));

        // Test double character tokens
        assert!(matches!(
            map.get(&'='),
            Some(TokenMatch::Double('=', TokenType::EqualEqual))
        ));
        assert!(matches!(
            map.get(&'<'),
            Some(TokenMatch::Double('=', TokenType::LessThanEqualTo))
        ));
        assert!(matches!(
            map.get(&'>'),
            Some(TokenMatch::Double('=', TokenType::GreaterThanEqualTo))
        ));
        assert!(matches!(
            map.get(&'!'),
            Some(TokenMatch::Double('=', TokenType::NotEqual))
        ));
        assert!(matches!(
            map.get(&'&'),
            Some(TokenMatch::Double('&', TokenType::DoubleAmpersand))
        ));
        assert!(matches!(
            map.get(&'|'),
            Some(TokenMatch::Double('|', TokenType::DoubleBar))
        ));
        assert!(matches!(
            map.get(&'-'),
            Some(TokenMatch::Double('>', TokenType::Arrow))
        ));

        // Test comment token
        assert!(matches!(map.get(&'/'), Some(TokenMatch::Comment)));

        // Test non-existent token
        assert!(map.get(&'@').is_none());
    }

    #[test]
    fn test_get_single_char_fallback() {
        assert_eq!(Tokenizer::get_single_char_fallback('='), TokenType::Equal);
        assert_eq!(
            Tokenizer::get_single_char_fallback('<'),
            TokenType::LessThan
        );
        assert_eq!(
            Tokenizer::get_single_char_fallback('>'),
            TokenType::GreaterThan
        );
        assert_eq!(Tokenizer::get_single_char_fallback('!'), TokenType::Bang);
        assert_eq!(Tokenizer::get_single_char_fallback('-'), TokenType::Minus);
        assert_eq!(Tokenizer::get_single_char_fallback('/'), TokenType::Slash);
        assert_eq!(
            Tokenizer::get_single_char_fallback('@'),
            TokenType::UnsupportedSymbolError
        );
    }

    #[test]
    fn test_match_character_single_tokens() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"+*:(),";

        let (token, skip) = tokenizer.match_character(line_bytes, '+', Some('*'));
        assert_token(&token, TokenType::Plus, "+", 1, 1);
        assert!(!skip);

        tokenizer.curr_byte_index_in_line = 1;
        let (token, skip) = tokenizer.match_character(line_bytes, '*', Some(':'));
        assert_token(&token, TokenType::Asterisk, "*", 1, 2);
        assert!(!skip);
    }

    #[test]
    fn test_match_character_double_tokens() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"== != <= >=";

        // Test successful double character match
        let (token, skip) = tokenizer.match_character(line_bytes, '=', Some('='));
        assert_token(&token, TokenType::EqualEqual, "==", 1, 1);
        assert!(!skip);
        assert_eq!(tokenizer.curr_byte_index_in_line, 1); // Should increment for double char

        // Reset tokenizer
        tokenizer.curr_byte_index_in_line = 0;

        // Test failed double character match (falls back to single)
        let (token, skip) = tokenizer.match_character(line_bytes, '=', Some('x'));
        assert_token(&token, TokenType::Equal, "=", 1, 1);
        assert!(!skip);
        assert_eq!(tokenizer.curr_byte_index_in_line, 0); // Should not increment for single char fallback
    }

    #[test]
    fn test_match_character_comments() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"/! comment";

        // Test comment start
        let (token, skip) = tokenizer.match_character(line_bytes, '/', Some('!'));
        assert_eq!(token.token_type, TokenType::UnsupportedSymbolError);
        assert!(skip);

        // Test division (not comment)
        let line_bytes2 = b"/ 2";
        let (token, skip) = tokenizer.match_character(line_bytes2, '/', Some(' '));
        assert_token(&token, TokenType::Slash, "/", 1, 1);
        assert!(!skip);
    }

    #[test]
    fn test_match_character_numbers() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"123";

        let (token, skip) = tokenizer.match_character(line_bytes, '1', Some('2'));
        assert_token(&token, TokenType::Number, "123", 1, 1);
        assert!(!skip);
        assert_eq!(tokenizer.curr_byte_index_in_line, 2); // Should be at end of number
    }

    #[test]
    fn test_match_character_strings() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"\"hello\"";

        let (token, skip) = tokenizer.match_character(line_bytes, '"', Some('h'));
        assert_token(&token, TokenType::Str, "hello", 1, 1);
        assert!(!skip);
        assert_eq!(tokenizer.curr_byte_index_in_line, 6); // Should be at closing quote
    }

    #[test]
    fn test_match_character_identifiers() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"variable";

        let (token, skip) = tokenizer.match_character(line_bytes, 'v', Some('a'));
        assert_token(&token, TokenType::Identity, "variable", 1, 1);
        assert!(!skip);
        assert_eq!(tokenizer.curr_byte_index_in_line, 7); // Should be at end of identifier
    }

    #[test]
    fn test_match_character_keywords() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"let";

        let (token, skip) = tokenizer.match_character(line_bytes, 'l', Some('e'));
        assert_token(&token, TokenType::Let, "let", 1, 1);
        assert!(!skip);
        assert_eq!(tokenizer.curr_byte_index_in_line, 2); // Should be at end of keyword
    }

    #[test]
    fn test_match_character_unsupported() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"@#$";

        let (token, skip) = tokenizer.match_character(line_bytes, '@', Some('#'));
        assert_token(&token, TokenType::UnsupportedSymbolError, "@", 1, 1);
        assert!(!skip);
    }

    #[test]
    fn test_create_token_from_text_number() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"12345abc";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Number);

        assert_token(&token, TokenType::Number, "12345", 1, 1);
        assert_eq!(end_idx, 4); // Should stop at 'a'
    }

    #[test]
    fn test_create_token_from_text_number_end_of_line() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"123";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Number);

        assert_token(&token, TokenType::Number, "123", 1, 1);
        assert_eq!(end_idx, 2); // Should be at last character
    }

    #[test]
    fn test_create_token_from_text_string() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"hello world\"";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Str);

        assert_token(&token, TokenType::Str, "hello world", 1, 1);
        assert_eq!(end_idx, 11); // Should stop at closing quote
    }

    #[test]
    fn test_create_token_from_text_empty_string() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"\"";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Str);

        assert_token(&token, TokenType::Str, "", 1, 1);
        assert_eq!(end_idx, 0); // Should stop immediately at closing quote
    }

    #[test]
    fn test_create_token_from_text_identifier() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"myVariable123";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Identity);

        assert_token(&token, TokenType::Identity, "myVariable", 1, 1);
        assert_eq!(end_idx, 9); // Should stop at first digit
    }

    #[test]
    fn test_create_token_from_text_identifier_end_of_line() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"identifier";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Identity);

        assert_token(&token, TokenType::Identity, "identifier", 1, 1);
        assert_eq!(end_idx, 9); // Should be at last character
    }

    #[test]
    fn test_create_token_from_text_keyword_detection() {
        let mut tokenizer = create_test_tokenizer();
        let line_bytes = b"function";

        let (token, end_idx) = tokenizer.create_token_from_text(line_bytes, 0, TokenType::Identity);

        // Should be converted from Identity to FunctionDeclaration
        assert_token(&token, TokenType::FunctionDeclaration, "function", 1, 1);
        assert_eq!(end_idx, 7);
    }

    #[test]
    fn test_tokenize_line_simple() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("let x = 42".to_string());

        assert_eq!(tokens.len(), 5); // let, x, =, 42, newline
        assert_token(&tokens[0], TokenType::Let, "let", 1, 1);
        assert_token(&tokens[1], TokenType::Identity, "x", 1, 5);
        assert_token(&tokens[2], TokenType::Equal, "=", 1, 7);
        assert_token(&tokens[3], TokenType::Number, "42", 1, 9);
        assert_token(&tokens[4], TokenType::Newline, "", 1, 11);
    }

    #[test]
    fn test_tokenize_line_with_string() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("print \"hello\"".to_string());

        assert_eq!(tokens.len(), 3); // print, "hello", newline
        assert_token(&tokens[0], TokenType::Print, "print", 1, 1);
        assert_token(&tokens[1], TokenType::Str, "hello", 1, 7);
        assert_token(&tokens[2], TokenType::Newline, "", 1, 14);
    }

    #[test]
    fn test_tokenize_line_with_comment() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("let x /! comment here".to_string());

        assert_eq!(tokens.len(), 3); // let, x, newline (comment ignored)
        assert_token(&tokens[0], TokenType::Let, "let", 1, 1);
        assert_token(&tokens[1], TokenType::Identity, "x", 1, 5);
        assert_token(&tokens[2], TokenType::Newline, "", 1, 22);
    }

    #[test]
    fn test_tokenize_line_complex_expression() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("if (x >= 10)".to_string());

        assert_eq!(tokens.len(), 7); // if, (, x, >=, 10, ), newline
        assert_token(&tokens[0], TokenType::If, "if", 1, 1);
        assert_token(&tokens[1], TokenType::LeftParen, "(", 1, 4);
        assert_token(&tokens[2], TokenType::Identity, "x", 1, 5);
        assert_token(&tokens[3], TokenType::GreaterThanEqualTo, ">=", 1, 7);
        assert_token(&tokens[4], TokenType::Number, "10", 1, 10);
        assert_token(&tokens[5], TokenType::RightParen, ")", 1, 12);
        assert_token(&tokens[6], TokenType::Newline, "", 1, 13);
    }

    #[test]
    fn test_tokenize_line_empty() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("".to_string());

        assert_eq!(tokens.len(), 1); // just newline
        assert_token(&tokens[0], TokenType::Newline, "", 1, 1);
    }

    #[test]
    fn test_tokenize_line_unsupported_characters_filtered() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("@ # $ %".to_string());

        assert_eq!(tokens.len(), 1); // just newline (unsupported chars filtered)
        assert_token(&tokens[0], TokenType::Newline, "", 1, 8);
    }

    #[test]
    fn test_token_type_from_str_keywords() {
        assert_eq!(TokenType::from_str("let"), Ok(TokenType::Let));
        assert_eq!(TokenType::from_str("goto"), Ok(TokenType::Goto));
        assert_eq!(TokenType::from_str("print"), Ok(TokenType::Print));
        assert_eq!(TokenType::from_str("input"), Ok(TokenType::Input));
        assert_eq!(TokenType::from_str("if"), Ok(TokenType::If));
        assert_eq!(TokenType::from_str("then"), Ok(TokenType::Then));
        assert_eq!(TokenType::from_str("endIf"), Ok(TokenType::EndIf));
        assert_eq!(TokenType::from_str("while"), Ok(TokenType::While));
        assert_eq!(TokenType::from_str("do"), Ok(TokenType::Do));
        assert_eq!(TokenType::from_str("endWhile"), Ok(TokenType::EndWhile));
        assert_eq!(
            TokenType::from_str("function"),
            Ok(TokenType::FunctionDeclaration)
        );
        assert_eq!(TokenType::from_str("return"), Ok(TokenType::Return));
        assert_eq!(
            TokenType::from_str("endFunction"),
            Ok(TokenType::EndFunction)
        );
        assert_eq!(TokenType::from_str("update"), Ok(TokenType::UpdateKeyword));
        assert_eq!(TokenType::from_str("label"), Ok(TokenType::Label));
    }

    #[test]
    fn test_token_type_from_str_variable_types() {
        assert_eq!(TokenType::from_str("Number"), Ok(TokenType::VarDeclaration));
        assert_eq!(TokenType::from_str("String"), Ok(TokenType::VarDeclaration));
    }

    #[test]
    fn test_token_type_from_str_unknown() {
        assert_eq!(TokenType::from_str("unknown"), Err(()));
        assert_eq!(TokenType::from_str("notakeyword"), Err(()));
        assert_eq!(TokenType::from_str(""), Err(()));
    }

    #[test]
    fn test_token_type_to_string() {
        assert_eq!(TokenType::EOF.to_string(), "EOF");
        assert_eq!(TokenType::Newline.to_string(), "\\n");
        assert_eq!(TokenType::Number.to_string(), "Number");
        assert_eq!(TokenType::Identity.to_string(), "Identity");
        assert_eq!(TokenType::Str.to_string(), "String");
        assert_eq!(TokenType::Let.to_string(), "let");
        assert_eq!(TokenType::Print.to_string(), "print");
        assert_eq!(TokenType::Plus.to_string(), "+");
        assert_eq!(TokenType::EqualEqual.to_string(), "==");
        assert_eq!(TokenType::Arrow.to_string(), "->");
        assert_eq!(TokenType::LeftParen.to_string(), "(");
        assert_eq!(TokenType::RightParen.to_string(), ")");
    }

    #[test]
    fn test_function_syntax_tokenization() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("function add(x, y) -> Number".to_string());

        assert_eq!(tokens.len(), 10); // function, add, (, x, ,, y, ), ->, Number, newline
        assert_token(&tokens[0], TokenType::FunctionDeclaration, "function", 1, 1);
        assert_token(&tokens[1], TokenType::Identity, "add", 1, 10);
        assert_token(&tokens[2], TokenType::LeftParen, "(", 1, 13);
        assert_token(&tokens[3], TokenType::Identity, "x", 1, 14);
        assert_token(&tokens[4], TokenType::Comma, ",", 1, 15);
        assert_token(&tokens[5], TokenType::Identity, "y", 1, 17);
        assert_token(&tokens[6], TokenType::RightParen, ")", 1, 18);
        assert_token(&tokens[7], TokenType::Arrow, "->", 1, 20);
        assert_token(&tokens[8], TokenType::VarDeclaration, "Number", 1, 23);
        assert_token(&tokens[9], TokenType::Newline, "", 1, 29);
    }

    #[test]
    fn test_line_number_progression() {
        // Create a temporary file with multiple lines
        let temp_path = format!("/tmp/test_multiline_{}.plank", std::process::id());
        let mut file = File::create(&temp_path).unwrap();
        write!(file, "let x = 1\nlet y = 2\nprint x").unwrap();
        drop(file);

        let mut file = File::open(temp_path).unwrap();
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize_file(&mut file);

        // Check that line numbers are correct
        let line_1_tokens: Vec<_> = tokens.iter().filter(|t| t.line_number == 1).collect();
        let line_2_tokens: Vec<_> = tokens.iter().filter(|t| t.line_number == 2).collect();
        let line_3_tokens: Vec<_> = tokens.iter().filter(|t| t.line_number == 3).collect();

        assert!(line_1_tokens.len() > 0);
        assert!(line_2_tokens.len() > 0);
        assert!(line_3_tokens.len() > 0);

        // First token of each line should have the correct line number
        assert_eq!(line_1_tokens[0].line_number, 1);
        assert_eq!(line_2_tokens[0].line_number, 2);
        assert_eq!(line_3_tokens[0].line_number, 3);
    }

    #[test]
    fn test_operators_tokenization() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("x + y - z * w / v".to_string());

        // Should tokenize: x, +, y, -, z, *, w, /, v, newline
        assert_eq!(tokens.len(), 10);
        assert_token(&tokens[0], TokenType::Identity, "x", 1, 1);
        assert_token(&tokens[1], TokenType::Plus, "+", 1, 3);
        assert_token(&tokens[2], TokenType::Identity, "y", 1, 5);
        assert_token(&tokens[3], TokenType::Minus, "-", 1, 7);
        assert_token(&tokens[4], TokenType::Identity, "z", 1, 9);
        assert_token(&tokens[5], TokenType::Asterisk, "*", 1, 11);
        assert_token(&tokens[6], TokenType::Identity, "w", 1, 13);
        assert_token(&tokens[7], TokenType::Slash, "/", 1, 15);
        assert_token(&tokens[8], TokenType::Identity, "v", 1, 17);
        assert_token(&tokens[9], TokenType::Newline, "", 1, 18);
    }

    #[test]
    fn test_comparison_operators() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("x == y != z < w <= a > b >= c".to_string());

        // Should tokenize comparison operators correctly
        let expected_types = vec![
            TokenType::Identity,           // x
            TokenType::EqualEqual,         // ==
            TokenType::Identity,           // y
            TokenType::NotEqual,           // !=
            TokenType::Identity,           // z
            TokenType::LessThan,           // <
            TokenType::Identity,           // w
            TokenType::LessThanEqualTo,    // <=
            TokenType::Identity,           // a
            TokenType::GreaterThan,        // >
            TokenType::Identity,           // b
            TokenType::GreaterThanEqualTo, // >=
            TokenType::Identity,           // c
            TokenType::Newline,            // newline
        ];

        assert_eq!(tokens.len(), expected_types.len());
        for (i, expected_type) in expected_types.iter().enumerate() {
            assert_eq!(tokens[i].token_type, *expected_type);
        }
    }

    #[test]
    fn test_logical_operators() {
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("x && y || !z".to_string());

        assert_eq!(tokens.len(), 7); // x, &&, y, ||, !, z, newline
        assert_token(&tokens[0], TokenType::Identity, "x", 1, 1);
        assert_token(&tokens[1], TokenType::DoubleAmpersand, "&&", 1, 3);
        assert_token(&tokens[2], TokenType::Identity, "y", 1, 6);
        assert_token(&tokens[3], TokenType::DoubleBar, "||", 1, 8);
        assert_token(&tokens[4], TokenType::Bang, "!", 1, 11);
        assert_token(&tokens[5], TokenType::Identity, "z", 1, 12);
        assert_token(&tokens[6], TokenType::Newline, "", 1, 13);
    }

    #[test]
    fn test_edge_cases_numbers() {
        let mut tokenizer = create_test_tokenizer();

        // Single digit
        let tokens = tokenizer.tokenize_line("5".to_string());
        assert_eq!(tokens.len(), 2); // 5, newline
        assert_token(&tokens[0], TokenType::Number, "5", 1, 1);

        // Multiple digits
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("12345".to_string());
        assert_eq!(tokens.len(), 2); // 12345, newline
        assert_token(&tokens[0], TokenType::Number, "12345", 1, 1);

        // Number followed by identifier
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("123abc".to_string());
        assert_eq!(tokens.len(), 3); // 123, abc, newline
        assert_token(&tokens[0], TokenType::Number, "123", 1, 1);
        assert_token(&tokens[1], TokenType::Identity, "abc", 1, 4);
    }

    #[test]
    fn test_edge_cases_strings() {
        let mut tokenizer = create_test_tokenizer();

        // Empty string
        let tokens = tokenizer.tokenize_line("\"\"".to_string());
        assert_eq!(tokens.len(), 2); // "", newline
        assert_token(&tokens[0], TokenType::Str, "", 1, 1);

        // String with spaces
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("\"hello world\"".to_string());
        assert_eq!(tokens.len(), 2); // "hello world", newline
        assert_token(&tokens[0], TokenType::Str, "hello world", 1, 1);

        // String with special characters
        let mut tokenizer = create_test_tokenizer();
        let tokens = tokenizer.tokenize_line("\"123!@#$%\"".to_string());
        assert_eq!(tokens.len(), 2); // "123!@#$%", newline
        assert_token(&tokens[0], TokenType::Str, "123!@#$%", 1, 1);
    }

    #[test]
    fn test_keyword_recognition() {
        let keywords = vec![
            ("let", TokenType::Let),
            ("if", TokenType::If),
            ("then", TokenType::Then),
            ("endIf", TokenType::EndIf),
            ("while", TokenType::While),
            ("do", TokenType::Do),
            ("endWhile", TokenType::EndWhile),
            ("function", TokenType::FunctionDeclaration),
            ("return", TokenType::Return),
            ("endFunction", TokenType::EndFunction),
            ("print", TokenType::Print),
            ("input", TokenType::Input),
            ("goto", TokenType::Goto),
            ("update", TokenType::UpdateKeyword),
            ("Number", TokenType::VarDeclaration),
            ("String", TokenType::VarDeclaration),
        ];

        for (keyword, expected_type) in keywords {
            let mut tokenizer = create_test_tokenizer();
            let tokens = tokenizer.tokenize_line(keyword.to_string());
            assert_eq!(tokens.len(), 2); // keyword, newline
            assert_token(&tokens[0], expected_type, keyword, 1, 1);
        }
    }

    #[test]
    fn test_file_tokenization_with_eof() {
        // Create a temporary file
        let temp_path = format!("/tmp/test_eof_{}.plank", std::process::id());
        let mut file = File::create(&temp_path).unwrap();
        write!(file, "let x = 42").unwrap();
        drop(file);

        let mut file = File::open(temp_path).unwrap();
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize_file(&mut file);

        // Should end with EOF token
        assert!(!tokens.is_empty());
        let last_token = &tokens[tokens.len() - 1];
        assert_eq!(last_token.token_type, TokenType::EOF);
    }
}
