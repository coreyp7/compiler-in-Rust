use std::fs::File;
//use std::io::BufReader;
use std::io::prelude::*;

mod tokenizer;
//use tokenizer::tokenize_file;
use tokenizer::Token;
use tokenizer::Tokenizer;

//mod parser;
//use parser::TokenList;

use std::env;


fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let src_path: &String = &args[1];
    let output_path: &String = &args[2];

    // TODO: add error handler for reading the file
    let mut f = File::open(src_path)?;

    //let tokenized_file: Vec<Token> = tokenize_file(&mut f); 
    let tokenizer = Tokenizer::new();
    //tokenizer.tokenize_file(&mut f);

    
    
    /*
    let mut parser: TokenList = TokenList::new(tokenized_file);
    parser.parse_tokens();

    let path = format!("{output_path}/main.c");
    let mut output_file = File::create(path)?;

    //TODO: add error handling
    let _ = output_file.write_all(parser.code_str.as_bytes());
    */

    Ok(())
}

