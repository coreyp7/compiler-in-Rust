use crate::ast::Comparison;
use crate::ast::DataType;
use crate::ast::FunctionTable;
use crate::ast::Value;
use crate::ast::{Expression, Logical, Term, Unary};
use crate::semantic::SemanticError;
use crate::semantic::analyzer::AnalysisState;
use crate::semantic::analyzer::analyze_value;
use crate::semantic::type_check;
use crate::symbol_table;

/// NOTE: all of these functions assume that you have resolved the values of all
/// of the passed in structs.
/// These functions WILL NOT resolve any unknown values.

pub fn add_type_check_errors_for_logical(
    mut state: AnalysisState,
    logical: &mut Logical,
    function_table: &FunctionTable,
    line_number: u32,
) -> AnalysisState {
    /*
    for comparison in &logical.comparisons {
        for expr in &comparison.expressions {
            /*
            if expr.datatype == DataType::Invalid {
                state
                    .errors
                    .push(SemanticError::ExpressionInvalid { line: line_number })
            } else if expr.datatype == DataType::String {
                // NOTE: this should be temporary, and improved in the future.
                state.errors.push(SemanticError::UnexpectedStatement {
                    line: line_number,
                    explanation: "You cannot compare Strings.".to_string(),
                });
            }
            */

            //NOTE: can only ever be number right now
            //state = type_check_expression(&expr, &DataType::Number, line_number, state, function_table);
        }
    }first_expr_datatype
    */
    for comparison in logical.comparisons.iter_mut() {
        state = type_check_comparison(state, comparison, function_table, line_number);
        // TODO: ensure that each comparison is valid and actually resolves to
        // a boolean. (where we'd add a boolean type)
    }

    state
}

pub fn type_check_comparison(
    mut state: AnalysisState,
    comparison: &mut Comparison,
    function_table: &FunctionTable,
    line_number: u32,
) -> AnalysisState {
    // We need to check the types of the expressions being compared, and ensure
    // that they all match.
    // All we need to worry about is what's in this comparison.
    let first_expresion_datatype = &comparison.expressions[0].datatype; // ALERT: little risky
    for expr in comparison.expressions.iter() {
        // get the data type of the expression and ensure they all match.
        // the semantics of comparing strings and shit can be done later.
        if &expr.datatype != first_expresion_datatype {
            // add error that this comparison is invalid.
            // edit the comparison so it's data type is invalid
            // TODO: may want to update the comparison to have a datatype associated with it
            // in the struct. This'll prolly have to be done.
            state.errors.push(SemanticError::ComparisonInvalid {
                line: line_number,
                first_expr_datatype: first_expresion_datatype.clone(),
                got: expr.datatype.clone(),
            })
        }
    }

    state
}

// TODO: rename/recontextualize that this is specifically for variables, NOT
// in an if/while conditional.
// Current highest level is expression, so only this will be public for now.
pub fn type_check_expression(
    expr: &Expression,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // For now, we'll check each term in the expression
    // TODO: we need to validate that all terms are compatible with the operations being performed
    for term in &expr.terms {
        state = type_check_term(term, expected_type, line, state, function_table);
    }

    state
}

/// Check types for a term (multiplication/division level)
fn type_check_term(
    term: &Term,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    for unary in &term.unarys {
        state = type_check_unary(unary, expected_type, line, state, function_table);
    }

    state
}

/// Check types for a unary expression (with optional unary operator)
fn type_check_unary(
    unary: &Unary,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // Check the primary value
    state = type_check_value(&unary.primary, expected_type, line, state, function_table);

    state
}

/// Check that a value matches the expected type
/// This is the core type checking function that validates individual values
fn type_check_value(
    value: &Value,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // First validate the value itself (check if variables/functions exist)
    state = analyze_value(value, line, state, function_table);

    // Then check type compatibility - this is similar to the original code at line 141
    // but now works with the resolved type system
    if value.data_type != *expected_type
        && !matches!(value.data_type, DataType::Unknown | DataType::Invalid)
    {
        state.errors.push(SemanticError::TypeMismatch {
            expected: expected_type.clone(),
            found: value.data_type.clone(),
            line,
        });
    }

    state
}
