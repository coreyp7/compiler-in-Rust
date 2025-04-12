use std::fs::File;
use std::io::BufReader;

mod tokenizer;
use tokenizer::tokenize_file;
use tokenizer::Token;
use tokenizer::TokenType;

mod parser;
use parser::TokenList;

fn main() -> std::io::Result<()> {

    // TODO: add error handler for reading the file
    let mut f = File::open("log.txt")?;

    let tokenized_file: Vec<Token> = tokenize_file(&mut f); 
    println!("1. File has been tokenized.");
    
    //let mut parser: TokenList = TokenList {vec: tokenized_file, curr_idx: 0};
    let mut parser: TokenList = TokenList::new(tokenized_file);
    parser.parse_tokens();
    println!("2. File has been parsed; it abides by grammar.");

    Ok(())
}

