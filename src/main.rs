use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;


fn main() -> std::io::Result<()> {

    let f = File::open("log.txt")?;
    let reader = BufReader::new(f);
    
    for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
            tokenize_line(&line_str);         
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
fn tokenize_line(line: &String) {
    let mut tokens: Vec<Token> = Vec::new();
    for curr in line.chars() {
        let token = match curr {
            '+' => Token { token_type: TokenType::Plus, text: String::from(curr) },
            '-' => Token { token_type: TokenType::Minus, text: String::from(curr) },
            '*' => Token { token_type: TokenType::Asterisk, text: String::from(curr) },
            '/' => Token { token_type: TokenType::Slash, text: String::from(curr) },
            _ => Token { token_type: TokenType::UnsupportedSymbolError, text: String::from(curr) }
        };
    
       tokens.push(token); 
    }

    println!("{:#?}", tokens);
}

//fn create_token(tokenType: TokenType, text: 

#[derive(Debug)]
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
    UnsupportedSymbolError
}
