use std::fs::File;
use std::io::BufReader;

mod tokenizer;
use tokenizer::tokenize_file;
use tokenizer::Token;
use tokenizer::TokenType;

fn main() -> std::io::Result<()> {

    // TODO: add error handler for reading the file
    let mut f = File::open("log.txt")?;

    let tokenized_file: Vec<Token> = tokenize_file(&mut f); 
    println!("File has been tokenized.");
    

    println!("End");
    Ok(())
}

