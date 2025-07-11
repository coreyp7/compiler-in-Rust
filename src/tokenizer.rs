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
            match curr {
                '"' => {
                    let result = self.create_token_from_text(
                        line_bytes,
                        self.curr_byte_index_in_line + 1,
                        TokenType::Str,
                    );
                    self.curr_byte_index_in_line = result.1;
                    (result.0, false)
                }
                x if x.is_numeric() => {
                    let result = self.create_token_from_text(
                        line_bytes,
                        self.curr_byte_index_in_line,
                        TokenType::Number,
                    );
                    self.curr_byte_index_in_line = result.1;
                    (result.0, false)
                }
                x if x.is_alphabetic() => {
                    let result = self.create_token_from_text(
                        line_bytes,
                        self.curr_byte_index_in_line,
                        TokenType::Identity,
                    );
                    self.curr_byte_index_in_line = result.1;
                    (result.0, false)
                }
                _ => (
                    self.create_token(TokenType::UnsupportedSymbolError, String::from(curr)),
                    false,
                ),
            }
        }
    }
}
