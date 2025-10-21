use colored::*;
use std::env;
use std::fs::File;
use std::io::Write;

mod tokenizer;
use tokenizer::Token;
use tokenizer::tokenize_file;
mod ast;
use ast::Statement;
use ast::build_ast;

mod symbol_table;
use symbol_table::SymbolTable;

mod first_pass;
use first_pass::gather_declarations;

mod semantic;
use semantic::analyze_statements;
use semantic::resolve_all_value_types_in_ast;

mod code_generate;
use code_generate::generate_code_str;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    let output_path = &args[2];
    let debug = args.len() > 3 && (args[3] == "--debug");
    //let src_path: String = String::from("./example.plank"); // for testing without compiling

    // TODO: add error handler for reading the file
    let mut plank_src_file = File::open(src_path)?;

    let tokens: Vec<Token> = tokenize_file(&mut plank_src_file);
    if debug {
        debug_print_vec(&tokens, "Tokenizer output:");
    }

    // First pass: gather all function declarations. Allows file to do
    // forward declarations.
    let function_header_map = gather_declarations(&tokens);
    if debug {
        println!("---Function header map---");
        println!("{:#?}", function_header_map);
        println!("---Function header map end---");
    }

    // Second pass: generate AST given token list
    let mut ast_context = build_ast(tokens);
    if debug {
        debug_print_vec(&ast_context.statements, "Ast output:");
    }

    // Third pass: semantic analysis.
    // (There is also some type resolution done in here, since the semantic analyzer
    // keeps track of scope of available symbols/functions, so was easier to make as
    // a part of the same step)
    let semantic_errors = analyze_statements(&mut ast_context.statements, &function_header_map);
    if debug {
        debug_print_vec(
            &ast_context.statements,
            "AST (Post type resolution / semantic analysis",
        );
    }

    //if debug {
    //println!("semantic errors:\n{:#?}", semantic_errors);
    //}

    if !semantic_errors.is_empty() {
        semantic::print_failures_message(semantic_errors.len());
        for error in &semantic_errors {
            error.print_error();
        }
        return Ok(());
    } else {
        semantic::print_success_message();
    }

    // Generate c code str with ast
    let code = generate_code_str(&ast_context.statements, &function_header_map);
    if debug {
        debug_print_generated_code(&code);
    }

    let path = format!("{output_path}/main.c");
    let mut output_file = File::create(path)?;

    output_file.write_all(code.as_bytes())?;

    Ok(())
}

// Debug helper functions
fn debug_print_vec<T: std::fmt::Debug>(items: &[T], label: &str) {
    println!("{} -----------------------------------", label);
    for item in items {
        println!("{:#?}", item);
        println!("|");
    }
    println!("{} -----------------------------------", label);
}

fn debug_print_generated_code(code: &str) {
    println!(
        "{}",
        "code generated: -----------------------------------".yellow()
    );
    println!("{}", code.green());
    println!(
        "{}",
        "code generated: -----------------------------------".yellow()
    );
}
