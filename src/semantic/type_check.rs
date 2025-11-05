use crate::ast::DataType;
use crate::ast::FunctionTable;
use crate::ast::Value;
use crate::ast::{Expression, Logical, Term, Unary};
use crate::semantic::SemanticError;
use crate::semantic::analyzer::AnalysisState;
use crate::semantic::analyzer::analyze_value;
use crate::symbol_table;

pub fn add_type_check_errors_for_logical(
    mut state: AnalysisState,
    logical: &Logical,
    function_table: &FunctionTable,
    line_number: u32,
) -> AnalysisState {
    for comparison in &logical.comparisons {
        for expr in &comparison.expressions {
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

            //NOTE: can only ever be number right now
            state =
                type_check_expression(&expr, &DataType::Number, line_number, state, function_table);
        }
    }
    state
}

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
