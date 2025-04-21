use std::fs::File;
//use std::io::BufReader;
use std::io::prelude::*;

mod tokenizer;
use tokenizer::tokenize_file;
use tokenizer::Token;
//use tokenizer::TokenType;

mod parser;
use parser::TokenList;

use std::env;

fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let src_path: &String = &args[1];
    let output_path: &String = &args[2];

    //println!("RUST OUTPUT START:");
    //println!("src_path: '{}'", src_path);
    //println!("output_path: {}", output_path);
   
    // TODO: add error handler for reading the file
    let mut f = File::open(src_path)?;
    //println!("file openend");

    let tokenized_file: Vec<Token> = tokenize_file(&mut f); 
    //println!("1. File has been tokenized.");
    //println!("passed tokenized file");
    
    //let mut parser: TokenList = TokenList {vec: tokenized_file, curr_idx: 0};
    let mut parser: TokenList = TokenList::new(tokenized_file);
    parser.parse_tokens();
    //println!("2. File has been parsed; it abides by grammar.");

    let path = format!("{output_path}/main.c");
    let mut output_file = File::create(path)?;
    //TODO: add error handling
    let _ = output_file.write_all(parser.code_str.as_bytes());

    Ok(())
}

