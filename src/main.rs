use std::fs::File;
use std::io::BufReader;

mod tokenizer;
use tokenizer::tokenize_file;
use tokenizer::Token;
use tokenizer::TokenType;

mod parser;
use parser::TokenList;

use std::env;

fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let src_path: &String = &args[1];
    let output_path: &String = &args[2];

    println!("RUST OUTPUT START:");
    println!("src_path: '{}'", src_path);
    println!("output_path: {}", output_path);
   

    /*
    args from vec:
    index 1: plank src code
    index 2: output dir that the bash script created for the c stuff
    
    Then, when we get the c code from the parser, we need to save it to
    a file that the bash script should've already created.
    */

    // TODO: add error handler for reading the file
    //let mut f = File::open("log.txt")?;
    let mut f = File::open(src_path)?;
    println!("file openend");

    let tokenized_file: Vec<Token> = tokenize_file(&mut f); 
    //println!("1. File has been tokenized.");
    println!("passed tokenized file");
    
    //let mut parser: TokenList = TokenList {vec: tokenized_file, curr_idx: 0};
    let mut parser: TokenList = TokenList::new(tokenized_file);
    parser.parse_tokens();
    //println!("2. File has been parsed; it abides by grammar.");

    Ok(())
}

