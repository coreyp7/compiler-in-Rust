use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;


fn main() -> std::io::Result<()> {

    let f = File::open("log.txt")?;
    let reader = BufReader::new(f);
    
    for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
            tokenize_line(line_str);         
        }
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


        println!("{}", curr);
        match curr {
            '+' => token = Token { token_type: TokenType::Plus, text: String::from(curr) },
            '-' => token = Token { token_type: TokenType::Minus, text: String::from(curr) },
            '*' => token = Token { token_type: TokenType::Asterisk, text: String::from(curr) },
            '/' => {
                if matches!(next, Some(x) if x == '!') {
                    // skip rest of line, return early; rest of line is coment
                    skip_rest_of_line = true; 
                } else {
                    token = Token { token_type: TokenType::Slash, text: String::from(curr) }
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

    println!("{:#?}", tokens);
}

fn testing() {
    let test = 1;
    let mut other_variable = 3;
    match test {
        3 => { 
            println!("not returning anything");
        },
        1 => {
            other_variable = 777;
        },
        _ => {
            other_variable = 999;
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[allow(dead_code)]
enum TokenType {
    EOF,
    Newline,
    Number,
    Identity,
    Str,
    // Keywords
    Label,
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
    Equal,  
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
    UnsupportedSymbolError,
    // Won't get through to the parser, just for processing in here.
    Space
}
