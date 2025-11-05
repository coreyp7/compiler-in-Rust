/*
 * Value Resolution Functions for AST Expressions
 *
 * This module provides a comprehensive set of functions to resolve value types
 * within complex expression structures. The hierarchy is:
 *
 * Logical
 *   └── Comparison(s)
 *       └── Expression(s)
 *           └── Term(s)
 *               └── Unary(s)
 *                   └── Value (primary)
 *
 * Usage Example:
 * ```rust
 * // For a simple expression
 * resolve_expression_values(&mut expression, &function_table, &symbol_table);
 *
 * // For a complex logical expression
 * resolve_logical_values(&mut logical_expr, &function_table, &symbol_table);
 *
 * // For recursive value resolution (handles nested function calls)
 * resolve_value_recursively(&mut value, &function_table, &symbol_table);
 * ```
 *
 * All functions use the base `resolve_value` function which handles:
 * - Function calls (resolves return types from FunctionTable)
 * - Variables (resolves types from SymbolTable)
 * - Inline values (numbers, strings, etc.)
 * - Expressions (marked as Expression type)
 */

use crate::ast::Statement;
use crate::ast::Term;
use crate::ast::Unary;
use crate::ast::Value;
use crate::ast::ValueType;
use crate::ast::VariableAssignmentStatement;
use crate::ast::VariableDeclarationStatement;
use crate::ast::{Comparison, DataType, Expression, FunctionTable, Logical};
use crate::symbol_table::SymbolTable;

pub fn resolve_all_value_types_in_ast(
    ast_statements: &mut Vec<Statement>,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // TODO:
    // Loop through statements and handle differently depending on statement.
    // We're resolving functions return types by looking at the FunctionTable.
    // We don't need to be doing this for variables, because the semantic phase
    // will be validating this using its symbol map.
    /*
    for statement in ast_statements {
        resolve_statement(statement, function_header_map);
    }
    */

    for i in 0..ast_statements.len() {
        let vec_stmt = &mut ast_statements[i];
        resolve_statement(vec_stmt, function_header_map, symbol_table);
    }
    //return ast;
}

fn resolve_statement(
    statement: &mut Statement,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // this is just going to be a giant match statement
    match statement {
        Statement::VariableDeclaration(var_decl_stmt) => {
            resolve_variable_declaration_types(var_decl_stmt, function_header_map, symbol_table);
        }
        _ => (),
    }
}

pub fn resolve_variable_declaration_types(
    var_decl_stmt: &mut VariableDeclarationStatement,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Check that the value_type is a function call
    // if function call: Set the assigned value data_type
    // using the function_header_map
    resolve_expression_values(
        &mut var_decl_stmt.assigned_expr,
        function_header_map,
        symbol_table,
    );
}

pub fn resolve_variable_assignment_stmt_types(
    var_ass_stmt: &mut VariableAssignmentStatement,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Also need to resolve the type of the variable being assigned to.
    let var_name = &var_ass_stmt.var_name;
    if let Some(var_def) = symbol_table.get(var_name) {
        var_ass_stmt.var_data_type = var_def.data_type.clone();
    }

    resolve_expression_values(
        &mut var_ass_stmt.assigned_expr,
        function_header_map,
        symbol_table,
    );
}

fn resolve_value(val: &mut Value, function_header_map: &FunctionTable, symbol_table: &SymbolTable) {
    let val_type = &val.value_type;
    match val_type {
        // TODO: could this be moved into a more generic 'resolve value' function?
        ValueType::FunctionCall => {
            let func_call_decl_op = function_header_map.get_func_def_using_str(&val.raw_text);

            match func_call_decl_op {
                Some(func_decl) => {
                    val.data_type = func_decl.return_type.clone();
                    // NOTE: Parameters are now expressions, not values, so their type
                    // resolution is handled by check_expression_types in the semantic analyzer
                    // rather than here.
                }
                None => {
                    // TODO: when would this error even happen? I suppose if they're
                    // calling a function that doesn't exist, then it'd happen.
                    // Thus, leaving it unknown could indicate that the function
                    // dne. But isn't that handled in semantic analyzer anyway?
                    // So maybe we don't need to do anything here. Test this.
                    println!("NONE was found when trying to set the return type :(");
                }
            }
        }
        ValueType::Variable => {
            // Now we can use the symbol_table to resolve variable types
            // You can implement variable type resolution logic here
            let var_type_op = symbol_table.get(&val.raw_text);
            match var_type_op {
                Some(var_type) => val.data_type = var_type.data_type.clone(),
                None => (), // TODO: maybe do something? nah, this hsould be handled in analysis
            }
        }
        ValueType::Expression
        | ValueType::InlineNumber
        | ValueType::InlineString
        | ValueType::Invalid => {
            // These don't need type resolution from function table or symbol table
        }
    }
}

/// Also will resolve the type of the expression
/// (based off of first value found)
/// After you call this: ensure that the expression's datatype isn't Invalid.
/// If it is, there's some incorrect type operations happening here.
/// NOTE: for now string operations are non existent; if any strings are being
/// operated together, mark as invalid.
pub fn resolve_expression_values(
    expression: &mut Expression,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    for term in &mut expression.terms {
        resolve_term_values(term, function_header_map, symbol_table);
    }

    // Resolve the datatype of the expression itself.
    // Set it to invalid if there's mixing of types.
    // In future I can modify to allow.
    expression.datatype = get_first_value_of_entire_expr(expression).data_type.clone();
    for term in &expression.terms {
        for unary in &term.unarys {
            if unary.primary.data_type != expression.datatype {
                //is_expr_err = true;

                // Set datatype to invalid to indicate the expression couldn't be
                // evaluated to a single type.
                expression.datatype = DataType::Invalid;
            }
        }
    }
}

/// This might be sketchy but leaving for now.
/// Can change to an Option later to be more thorough, but there shouldn't
/// ever be 0 elements in any of these vecs.
pub fn get_first_value_of_entire_expr(expr: &Expression) -> &Value {
    &expr.terms[0].unarys[0].primary
}

/// Resolves all values within a Term structure  
pub fn resolve_term_values(
    term: &mut Term,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Resolve values in all unary operations within the term
    for unary in &mut term.unarys {
        resolve_unary_values(unary, function_header_map, symbol_table);
    }
}

/// Resolves all values within a Unary structure
pub fn resolve_unary_values(
    unary: &mut Unary,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Resolve the primary value in the unary operation
    resolve_value(&mut unary.primary, function_header_map, symbol_table);
}

fn type_check_expression(
    expr: &mut Expression,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Call function to get Vec of Values from term
    let values: Vec<&Value> = Vec::new();
    //for term in expr.terms {}
}
