use std::env;
use std::fs::File;
use std::io::Write;

mod tokenizer;
use tokenizer::Token;
use tokenizer::Tokenizer;

mod ast;
use ast::AstBuilder;

mod error;
use error::print_all_errors;

mod statement;

mod code_generator;
use code_generator::generate_code_str;

mod comparison;

fn main() -> std::io::Result<()> {
    /* When running from command line, not used when testing */
    let args: Vec<String> = env::args().collect();
    let src_path: &String = &args[1];
    let output_path: &String = &args[2];
    let debug = true; // TODO: add parsing of this shit in args

    //let src_path: String = String::from("./example.plank");

    // TODO: add error handler for reading the file
    let mut f = File::open(src_path)?;

    //let tokenized_file: Vec<Token> = tokenize_file(&mut f);
    let mut tokenizer = Tokenizer::new();
    let tokens: Vec<Token> = tokenizer.tokenize_file(&mut f);
    if debug {
        println!("Tokenizer output: -----------------------------------");
        for token in &tokens {
            println!("{:?}", token);
        }
        println!("Tokenizer output: -----------------------------------");
    }

    // build ast with tokens
    let mut ast_builder = AstBuilder::new(tokens);
    let ast_vec = ast_builder.generate_ast();
    let ast_errors = ast_builder.get_error_vec();

    if debug {
        println!("Ast output: -----------------------------------");
        for node in &ast_vec {
            println!("{:#?}", node);
        }
        println!("Ast output: -----------------------------------");

        println!("Ast ERRORS: -----------------------------------");
        for err in ast_errors {
            println!("{:#?}", err);
        }
        println!("Ast ERRORS: -----------------------------------");
        println!("Ast map: -----------------------------------");
        println!("{:#?}", ast_builder.var_map);
        println!("Ast map: -----------------------------------");
    }

    if ast_errors.len() > 0 {
        print_all_errors(&ast_errors);
        return Ok(());
    }

    // generate c code str with ast
    let code: String = generate_code_str(&ast_vec);
    if debug {
        println!("code generated: -----------------------------------");
        println!("{}", code);
        println!("code generated: -----------------------------------");
    }

    let path = format!("{output_path}/main.c");
    let mut output_file = File::create(path)?;

    //TODO: add error handling
    let _ = output_file.write_all(code.as_bytes());

    Ok(())
}
