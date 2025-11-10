use crate::ast::GeneralOperator;
use crate::ast::Statement;
use crate::ast::Term;
use crate::ast::Unary;
use crate::ast::Value;
use crate::ast::ValueType;
use crate::ast::VariableAssignmentStatement;
use crate::ast::VariableDeclarationStatement;
use crate::ast::{Comparison, DataType, Expression, FunctionTable, Logical};
use crate::ast::{convert_expression_op_to_general, convert_term_op_to_general};
use crate::symbol_table::SymbolTable;

pub fn resolve_logical_values(
    logical: &mut Logical,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    for (idx, comparison) in &mut logical.comparisons.iter_mut().enumerate() {
        resolve_comparison_values(comparison, function_header_map, symbol_table);
        if !comparison.is_valid {
            logical.is_valid = false;
        }
    }
}

pub fn resolve_comparison_values(
    comparison: &mut Comparison,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    let mut all_expr_type = DataType::Unknown;

    for (idx, expr) in &mut comparison.expressions.iter_mut().enumerate() {
        resolve_expression_values_and_update_data_type(expr, function_header_map, symbol_table);

        if idx == 0 {
            all_expr_type = expr.data_type.clone();
            continue;
        }

        // If there are conflicting types (which is not allowed),
        // set the datatype to invalid and return early.
        if all_expr_type != expr.data_type {
            comparison.is_valid = false; // this should be default true
            break;
        }
    }
}

pub fn resolve_expression_values_and_update_data_type(
    expression: &mut Expression,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    for (idx, term) in &mut expression.terms.iter_mut().enumerate() {
        resolve_term_values_and_update_data_type(term, function_header_map, symbol_table);

        if idx == 0 {
            expression.data_type = term.data_type.clone();
            continue;
        }

        // If there are conflicting types (which is not allowed),
        // set the datatype to invalid and return early.
        if expression.data_type != term.data_type {
            expression.data_type = DataType::Invalid;
            break;
        }
    }
}

pub fn resolve_term_values_and_update_data_type(
    term: &mut Term,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    for (idx, unary) in &mut term.unarys.iter_mut().enumerate() {
        resolve_unary_values_and_update_data_type(unary, function_header_map, symbol_table);

        if idx == 0 {
            term.data_type = unary.data_type.clone();
            continue;
        }

        // If there are conflicting types (which is not allowed),
        // set the datatype to invalid and return early.
        if term.data_type != unary.data_type {
            term.data_type = DataType::Invalid;
            break;
        }
    }
}

pub fn resolve_unary_values_and_update_data_type(
    unary: &mut Unary,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    resolve_value(&mut unary.primary, function_header_map, symbol_table);

    unary.data_type = unary.primary.data_type.clone();
}

fn resolve_value(val: &mut Value, function_header_map: &FunctionTable, symbol_table: &SymbolTable) {
    let val_type = val.value_type.clone();
    match val_type {
        // TODO: could this be moved into a more generic 'resolve value' function?
        ValueType::FunctionCall => {
            let func_call_decl_op = function_header_map.get_func_def_using_str(&val.raw_text);

            match func_call_decl_op {
                Some(func_decl) => {
                    val.data_type = func_decl.return_type.clone();
                    // TODO: we need to also resolve all of the logicals passed into the parameters of this function.
                    // How should this be done?
                    for param in val.param_values.as_mut().unwrap() {
                        // TODO: resolve logical here. Should work since we're passing a mutable reference,
                        // and only once at a time.
                    }
                }
                None => {
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
        | ValueType::InlineBoolean
        | ValueType::Invalid => {
            // These don't need type resolution from function table or symbol table
        }
    }
}

/*
pub fn get_first_value_of_entire_expr(expr: &Expression) -> &Value {
    &expr.terms[0].unarys[0].primary
}

pub fn resolve_expression_values(
    expression: &mut Expression,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    for term in &mut expression.terms {
        resolve_term_values(term, function_header_map, symbol_table);
    }

    println!("--------------------------------------");
    println!("resolving expression: {:#?}", expression);

    // Resolve the datatype of the expression itself.
    // Set it to invalid if there's mixing of types.
    // In future I can modify to allow.
    expression.datatype = get_first_value_of_entire_expr(expression).data_type.clone();

    let mut used_ops: Vec<GeneralOperator> = Vec::new();
    for op in &expression.operators {
        if let Some(general_op) = convert_expression_op_to_general(op.clone()) {
            used_ops.push(general_op);
        }
    }

    // TODO: This is lazy and stupid. Can change to be more readable and less stupid.
    for term in &expression.terms {
        for op in &term.operations {
            if let Some(general_op) = convert_term_op_to_general(op.clone()) {
                used_ops.push(general_op);
            }
        }

        for unary in &term.unarys {
            if let Some(op) = &unary.operation {
                if let Some(general_op) = convert_expression_op_to_general(op.clone()) {
                    used_ops.push(general_op);
                }
            }

            if unary.primary.data_type != expression.datatype {
                // Set datatype to invalid to indicate the expression couldn't be
                // evaluated to a single type.
                expression.datatype = DataType::Invalid;
            }
        }
    }

    if expression.datatype != DataType::String {
        return;
    }

    // Check if any operations are invalid for strings
    let has_invalid_string_ops = used_ops.iter().any(|op| match op {
        GeneralOperator::EqualEqual | GeneralOperator::NotEqual => false,
        _ => true, // All other operations are invalid for strings
    });

    if has_invalid_string_ops {
        // TODO: in the future, we should have a specific error for this that we
        // add to the errors vec.
        expression.datatype = DataType::Invalid;
    }

    // this is a string; only allow equal equal, not equal

    println!("done resolving expression: {:#?}", expression);
    println!("--------------------------------------");
}
*/
