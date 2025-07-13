use colored::Colorize;
use std::env;
use std::fs::File;
use std::io::Write;

mod tokenizer;
use tokenizer::Token;
use tokenizer::Tokenizer;

mod ast;
use ast::AstBuilder;

mod expression_parser;

mod symbol_table;

mod semantic;
use semantic::{ScopeType, SemanticAnalyzer};

mod error;
use error::{ErrMsg, print_all_errors};

mod statement;
use statement::Statement;

mod code_generator;
use code_generator::generate_code_str;

mod comparison;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    let output_path = &args[2];
    let debug = args.len() > 3 && (args[3] == "--debug");
    //let src_path: String = String::from("./example.plank"); // for testing without compiling

    // TODO: add error handler for reading the file
    let mut f = File::open(src_path)?;

    let mut tokenizer = Tokenizer::new();
    let tokens: Vec<Token> = tokenizer.tokenize_file(&mut f);
    if debug {
        debug_print_tokens(&tokens);
    }

    // AST building
    let mut ast_builder = AstBuilder::new(tokens);
    let ast_vec = ast_builder.generate_ast();
    let mut ast_errors = ast_builder.get_error_vec().clone();

    if debug {
        debug_print_ast(&ast_vec);
        debug_print_errors_and_var_map(&ast_errors, &ast_builder);
    }

    // Semantic analysis on the AST
    let mut analyzer = SemanticAnalyzer::new(
        ast_builder.symbol_table.get_variables().clone(),
        ast_builder.symbol_table.get_functions().clone(),
    );
    analyzer.analyze_ast_vec(&ast_vec);
    ast_errors.extend(analyzer.errors);

    if !ast_errors.is_empty() {
        print_all_errors(&ast_errors);
        let error_str = "Failed:".red().bold();
        if ast_errors.len() == 1 {
            println!(
                "{} Could not compile plank file due to {} previous error.",
                error_str, 1
            );
        } else {
            println!(
                "{} Could not compile plank file due to {} previous errors.",
                error_str,
                ast_errors.len()
            );
        }
        return Ok(());
    }

    // Generate c code str with ast
    let code: String = generate_code_str(&ast_vec);
    if debug {
        debug_print_generated_code(&code);
    }

    let path = format!("{output_path}/main.c");
    let mut output_file = File::create(path)?;

    output_file.write_all(code.as_bytes())?;

    Ok(())
}

// Debug helper functions
fn debug_print_tokens(tokens: &[Token]) {
    println!("Tokenizer output: -----------------------------------");
    for token in tokens {
        println!("{:?}", token);
    }
    println!("Tokenizer output: -----------------------------------");
}

fn debug_print_ast(ast_vec: &[Statement]) {
    println!("Ast output: -----------------------------------");
    for node in ast_vec {
        println!("{:#?}", node);
    }
    println!("Ast output: -----------------------------------");
}

fn debug_print_errors_and_var_map(ast_errors: &[ErrMsg], ast_builder: &AstBuilder) {
    println!("Ast ERRORS: -----------------------------------");
    for err in ast_errors {
        println!("{:#?}", err);
    }
    println!("Ast ERRORS: -----------------------------------");
    println!("Ast map: -----------------------------------");
    ast_builder.symbol_table.debug_print();
    println!("Ast map: -----------------------------------");
}

fn debug_print_generated_code(code: &str) {
    println!("code generated: -----------------------------------");
    println!("{}", code);
    println!("code generated: -----------------------------------");
}
