use std::fs::File;
//use std::io::BufReader;
use std::io::prelude::*;

mod tokenizer;
use tokenizer::Token;
use tokenizer::Tokenizer;

mod ast;
use ast::AstBuilder;
use ast::ErrMsg;
use ast::Statement;

mod code_generator;
use code_generator::generate_code_str;

mod comparison;

//mod parser;
//use parser::TokenList;

use std::env;

fn main() -> std::io::Result<()> {
    /* When running from command line, not used when testing
    let args: Vec<String> = env::args().collect();
    let src_path: &String = &args[1];
    let output_path: &String = &args[2];
    */
    let debug = true; // TODO: add parsing of this shit in args

    let src_path: String = String::from("./example.plank");

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

    if (ast_errors.len() > 0) {
        crate::ast::print_all_errors(&ast_errors);
        return Ok(());
    }

    // generate c code str with ast
    let code: String = generate_code_str(&ast_vec);
    if debug {
        println!("code generated: -----------------------------------");
        println!("{}", code);
        println!("code generated: -----------------------------------");
    }

    /*
    let mut parser: TokenList = TokenList::new(tokenized_file);
    parser.parse_tokens();

    let path = format!("{output_path}/main.c");
    let mut output_file = File::create(path)?;

    //TODO: add error handling
    let _ = output_file.write_all(parser.code_str.as_bytes());
    */

    // Okay, some tests of things for the refactor.
    /*
    What do I want to test?
    - can a global variable exist in a module? that the struct can just access?
    - how do we resolve havinfg tokens in self? Would it be better to just
    pass the vector's permission around? Would that even work?

    I want to be able to give the ability for each function to be able to
    take ownership of the dynamically allocated variables in each token,
    since it won't be needed by the token going forward.

    So, how do we do that?
    */
    //let mut text: String = String::from("here's the text");
    /*
    let mut text: String = String::from("here's the text");
    layer_one(&mut text);
    println!("{}", text);
    */

    /*
    let mut tree: tree = tree::new();
    tree.tokens.push(
        node {
            text: String::from("test")
        }
    );
    tree.start_test();
    */
    Ok(())
}

#[derive(Debug)]
struct node {
    pub text: String,
}

struct tree {
    pub tokens: Vec<node>,
    index: usize,
}

impl tree {
    pub fn new() -> tree {
        tree {
            tokens: Vec::new(),
            index: 0,
        }
    }

    pub fn start_test(&mut self) {
        println!("start_test");
        self.run();
    }

    fn get_curr(&mut self) -> &mut node {
        &mut self.tokens[self.index]
    }

    fn run(&mut self) {
        println!("run");
        let mut token: &node = self.get_curr();
        println!("{:?}", token);
        //let taken_string: String = token.text;
        /*
        let mut string_borrow: String = self.get_curr();
        string_borrow.push_str("test string changed");
        println!("{}", string_borrow);
        */
    }
}

/*
fn test(mut text: String) -> mut String{
    text.push_str("in test");
}
*/

fn layer_one(text: &mut String) {
    layer_two(text);
}
fn layer_two(text: &mut String) {
    layer_three(text);
}
fn layer_three(text: &mut String) {
    text.clear();
    text.push_str("text changed!");
}
