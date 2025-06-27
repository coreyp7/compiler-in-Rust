use std::io::prelude::*;
use std::str::FromStr;
use std::io::BufReader;
use std::fs::File;


pub struct Tokenizer {
    line_number: u8,
    /*
    Okay, so having this here is pretty wack, but allows easy tracking of the
    current column in the current line being parsed.

    So, this is exclusively used in tokenize_line. And purely for informational
    purposes to show as error messages to the user.
    */
    curr_byte_index_in_line: usize
}

impl Tokenizer<> {
    
    pub fn new() -> Tokenizer {
        Tokenizer {
            line_number: 1,
            curr_byte_index_in_line: 0
        }
    }

    pub fn tokenize_file(&mut self, src_file: &mut File) -> Vec<Token> {
        let reader = BufReader::new(src_file);

        let mut token_vec: Vec<Token> = Vec::new();

        for line_result in reader.lines() {
            if let Ok(line_str) = line_result {
                let mut tokens = self.tokenize_line(line_str);         
                token_vec.append(&mut tokens);
                self.line_number = self.line_number + 1
            }
        }
        token_vec.push(self.create_token(TokenType::EOF, String::new()));

        return token_vec;
    }

    fn create_token(&mut self, token_type_param: TokenType, text_param: String) -> Token {
        Token {
            token_type: token_type_param,
            text: text_param,
            line_number: self.line_number,
            col_number: self.curr_byte_index_in_line+1
        }
    }

    // TODO: return result indicating successor
    // Here's our tokenizer. Function for now, can change later if necessary.
    fn tokenize_line(&mut self, line: String) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let line_bytes: &[u8] = line.as_bytes();
        //let mut curr_byte_index = 0;
        self.curr_byte_index_in_line = 0;

        while self.curr_byte_index_in_line < line_bytes.len() {
            let curr = line_bytes[self.curr_byte_index_in_line];
            let curr = curr as char;
            let mut next: Option<char> = None;
            if self.curr_byte_index_in_line < line_bytes.len()-1 {
                next = Some(line_bytes[self.curr_byte_index_in_line+1] as char);
            }

            //let mut token: Token = Token { token_type: TokenType::UnsupportedSymbolError, text: String::from(curr) };
            let mut token: Token = self.create_token(TokenType::UnsupportedSymbolError, String::from(curr));;
            let mut skip_rest_of_line = false;


            match curr {
                '+' => token = self.create_token(TokenType::Plus, String::from(curr)),
                '-' => token = self.create_token(TokenType::Minus, String::from(curr)),
                '*' => token = self.create_token(TokenType::Asterisk, String::from(curr)),
                ':' => token = self.create_token(TokenType::Colon, String::from(curr)),
                '/' => {
                    if matches!(next, Some(x) if x == '!') {
                        // skip rest of line, return early; rest of line is coment
                        skip_rest_of_line = true; 
                    } else {
                        token = self.create_token(TokenType::Slash, String::from(curr));
                    }
                },
                '=' => {
                    // if next isn't None and next char makes self a double equals
                    if matches!(next, Some(x) if x == '=') {
                        self.curr_byte_index_in_line += 1;
                        token = self.create_token(TokenType::EqualEqual, String::from("=="));
                    } else {
                        token = self.create_token(TokenType::Equal, String::from("="));
                    }
                },
                '<' => {
                    if matches!(next, Some(x) if x == '=') {
                        self.curr_byte_index_in_line += 1;
                        token = self.create_token(TokenType::LessThanEqualTo, String::from("<="));
                    } else {
                        token = self.create_token(TokenType::LessThan, String::from("<"));
                    }
                },
                '>' => {
                    if matches!(next, Some(x) if x == '=') {
                        self.curr_byte_index_in_line += 1;
                        token = self.create_token(TokenType::GreaterThanEqualTo, String::from(">="));
                    } else {
                        token = self.create_token(TokenType::GreaterThan, String::from(">"));
                    }
                },
                '!' => {
                    if matches!(next, Some(x) if x == '=') {
                        self.curr_byte_index_in_line += 1;
                        token = self.create_token(TokenType::NotEqual, String::from("!="))
                    } else if matches!(next, Some(x) if x != ' ') {
                        // ! alone isn't supported in this lanugage
                        println!("TOKENIZER: Found something after a ! that isn't =. Invalid operator.");
                        std::process::exit(0);
                    }
                },
                '"' => {
                    let result = self.get_end_of_token(&line_bytes, self.curr_byte_index_in_line+1, TokenType::Str);
                    let str_token = result.0;
                    let new_curr_byte_idx = result.1; 
                    
                    token = str_token;
                    self.curr_byte_index_in_line = new_curr_byte_idx;
                },
                x if x.is_numeric() => {
                    let result = self.get_end_of_token(&line_bytes, self.curr_byte_index_in_line, TokenType::Number);
                    let num_token = result.0;
                    let new_curr_byte_idx = result.1; 
                    
                    token = num_token;
                    self.curr_byte_index_in_line = new_curr_byte_idx;
                },
                x if x.is_alphabetic() => {
                    let result = self.get_end_of_token(&line_bytes, self.curr_byte_index_in_line, TokenType::Identity);
                    let num_token = result.0;
                    let new_curr_byte_idx = result.1; 
                    
                    token = num_token;
                    self.curr_byte_index_in_line = new_curr_byte_idx;
                },
                _ => ()
            };

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
        tokens.push(
            self.create_token(TokenType::Newline, String::new())
        ); 


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
    fn get_end_of_token(
        &mut self,        
        line_bytes: &[u8],
        token_start: usize,
        token_type: TokenType
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
                        end_of_string_idx = Some(str_byte_buffer-1);
                        str_byte_buffer = line_bytes.len();
                    } else if str_byte_buffer == line_bytes.len()-1 {
                        string_content.push(curr_char);
                        end_of_string_idx = Some(str_byte_buffer);
                        str_byte_buffer = line_bytes.len();
                    } else {
                        string_content.push(curr_char);
                    }
                },
                TokenType::Str => {
                    if curr_char == '"' {
                        end_of_string_idx = Some(str_byte_buffer);
                        str_byte_buffer = line_bytes.len();
                    } else {
                        string_content.push(curr_char);
                    }
                },
                TokenType::Identity => {
                    /*
                    Identities follow the same parsing rules as keywords.
                    So, they are both processed in here, and then we figure out
                    later if this token is a keyword instead of an Identity.
                    */
                    if curr_char.is_alphabetic() == false {
                        end_of_string_idx = Some(str_byte_buffer-1);
                        str_byte_buffer = line_bytes.len();
                    } else if str_byte_buffer == line_bytes.len()-1 {
                        string_content.push(curr_char);
                        end_of_string_idx = Some(str_byte_buffer);
                        str_byte_buffer = line_bytes.len();
                    } else {
                        string_content.push(curr_char);
                    }
                },
                _ => () 
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
            },
            Some(new_curr_idx) => {
                token = self.create_token(
                    token_type,
                    string_content
                );
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

        (
            token,
            new_curr_byte_idx
        )
    }
}


#[derive(Debug)]
pub struct Token {
    pub text: String, // used for identifiers, strings, numbers
    pub token_type: TokenType,
    pub line_number: u8,
    pub col_number: usize
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[allow(dead_code)]
pub enum TokenType {
    EOF = 0,
    Newline,
    Number,
    Identity,
    Str,
    // Keywords
    Label = 100,//unused
    //NumberType, // for declaring variable 'Number'
    VarDeclaration,
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
    Colon,
    UnsupportedSymbolError = 900,
    // Won't get through to the parser, just for processing in here.
    Space
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
            _ => Err(())
        }
    }
}
