use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::str::FromStr;


fn main() -> std::io::Result<()> {

    let f = File::open("log.txt")?;
    let reader = BufReader::new(f);

    let mut i = 1;
    
    for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
            println!("Line {}", i);
            tokenize_line(line_str);         
        }
        i += 1;
    }

    println!("End");
    Ok(())
}

#[derive(Debug)]
struct Token {
    text: String, // used for identifiers, strings, numbers
    token_type: TokenType 
}

// TODO: return result indicating successor
// Here's our tokenizer. Function for now, can change later if necessary.
fn tokenize_line(line: String) {
    let mut tokens: Vec<Token> = Vec::new();
    let line_bytes: &[u8] = line.as_bytes();
    let mut curr_byte_index = 0;

    while curr_byte_index < line_bytes.len() {
        let curr = line_bytes[curr_byte_index];
        let curr = curr as char;
        let mut next: Option<char> = None;
        if curr_byte_index < line_bytes.len()-1 {
            next = Some(line_bytes[curr_byte_index+1] as char);
        }

        let mut token: Token = Token { token_type: TokenType::UnsupportedSymbolError, text: String::from(curr) };
        let mut skip_rest_of_line = false;


        match curr {
            '+' => token = Token { token_type: TokenType::Plus, text: String::from(curr) },
            '-' => token = Token { token_type: TokenType::Minus, text: String::from(curr) },
            '*' => token = Token { token_type: TokenType::Asterisk, text: String::from(curr) },
            '/' => {
                if matches!(next, Some(x) if x == '!') {
                    // skip rest of line, return early; rest of line is coment
                    skip_rest_of_line = true; 
                } else {
                    token = Token { token_type: TokenType::Slash, text: String::from(curr) };
                }
            },
            '=' => {
                if (
                    // if next isn't None and next char makes this a double equals
                    matches!(next, Some(x) if x == '=')
                ) {
                    curr_byte_index += 1;
                    token = Token { token_type: TokenType::EqualEqual, text: String::from("==") }
                } else {
                    token = Token { token_type: TokenType::Equal, text: String::from("=") }
                }
            },
            '<' => {
                if (
                    matches!(next, Some(x) if x == '=')
                ) {
                    curr_byte_index += 1;
                    token = Token { token_type: TokenType::LessThanEqualTo, text: String::from("<=") }
                } else {
                    token = Token { token_type: TokenType::Equal, text: String::from("=") }
                }
            },
            '>' => {
                if (
                    matches!(next, Some(x) if x == '=')
                ) {
                    curr_byte_index += 1;
                    token = Token { token_type: TokenType::GreaterThanEqualTo, text: String::from(">=") }
                } else {
                    token = Token { token_type: TokenType::Equal, text: String::from("=") }
                }
            },
            '!' => {
                if (
                    matches!(next, Some(x) if x == '=')
                ) {
                    curr_byte_index += 1;
                    token = Token { token_type: TokenType::GreaterThanEqualTo, text: String::from("!=") }
                } else {
                    // ! alone isn't supported in this lanugage
                    token = Token { token_type: TokenType::UnsupportedSymbolError, text: String::from("") }
                }
            },
            '"' => {
                let result = get_end_of_token(&line_bytes, curr_byte_index+1, TokenType::Str);
                let str_token = result.0;
                let new_curr_byte_idx = result.1; 
                
                token = str_token;
                curr_byte_index = new_curr_byte_idx;
            },
            x if x.is_numeric() => {
                let result = get_end_of_token(&line_bytes, curr_byte_index, TokenType::Number);
                let num_token = result.0;
                let new_curr_byte_idx = result.1; 
                
                token = num_token;
                curr_byte_index = new_curr_byte_idx;
            },
            x if x.is_alphabetic() => {
                let result = get_end_of_token(&line_bytes, curr_byte_index, TokenType::Identity);
                let num_token = result.0;
                let new_curr_byte_idx = result.1; 
                
                token = num_token;
                curr_byte_index = new_curr_byte_idx;
            },
            _ => ()
        };

        curr_byte_index += 1;
   
        if token.token_type != TokenType::UnsupportedSymbolError {  
            tokens.push(token); 
        }

        if skip_rest_of_line {
            // This is a comment; we're skipping the rest of the line.
            curr_byte_index = line_bytes.len();
        }
    }

    // End tokens list with new line, since this is the end of the line
    tokens.push(
        Token {
            token_type: TokenType::Newline,
            text: String::new() 
        }
    ); 

    println!("{:#?}", tokens);
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
    line_bytes: &[u8],
    token_start: usize,
    token_type: TokenType
) -> (Token, usize) {
        
    //let mut str_byte_buffer = curr_byte_index+1;
    let mut str_byte_buffer = token_start;
    let mut end_of_string_idx: Option<usize> = None;
    let mut string_content: String = String::new();
    
    let mut i = 0;
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

                // This can either be an identity, or any keyword.
                // First, check if it is a keyword.
                // if not, create an identity out of this.
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
    let mut token = Token { 
        token_type: TokenType::UnsupportedSymbolError, 
        text: String::from("")
    };
        
    
    match end_of_string_idx {
        None => {
            // TODO: this needs to make a bigger deal out of this.
            // This compilation should end here; unable to tokenize.
            ()
        },
        Some(new_curr_idx) => {
            token = Token {
                //token_type: TokenType::Str,
                token_type: token_type,
                text: string_content
            };
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
            _ => ()
        }
    }

    (
        token,
        new_curr_byte_idx
    )

    /*
    (
        Token { token_type: TokenType::UnsupportedSymbolError, text: String::new() },
        0
    )
    */
}

#[derive(Debug)]
#[derive(PartialEq)]
#[allow(dead_code)]
enum TokenType {
    EOF = 0,
    Newline,
    Number,
    Identity,
    Str,
    // Keywords
    Label = 100,
    Goto,
    Print,
    Input,
    Let,
    If,
    Then,
    EndIf,
    While,
    Repeat,
    EndWhile,
    // Operators
    Equal = 200,  
    Plus,
    Minus,
    Asterisk,
    Slash,
    EqualEqual,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    UnsupportedSymbolError = 900,
    // Won't get through to the parser, just for processing in here.
    Space
}

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
            "endif" => Ok(TokenType::EndIf),
            "while" => Ok(TokenType::While),
            "repeat" => Ok(TokenType::Repeat),
            "endWhile" => Ok(TokenType::EndWhile),
            _ => Err(())
        }
    }
}
