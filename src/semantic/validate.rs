use crate::ast::{
    Comparison, ComparisonOperator, DataType, Expression, Logical, Term, Unary, Value, ValueType,
};
use crate::semantic::SemanticError;

/// Validates that the logical has a valid type, and doesn't break any rules.
/// If the returned vec is empty, then that means everything is okay.
/// Typcially called after resolving a logical's datatype.
pub fn validate_logical(logical: &Logical, line: u32) -> Vec<SemanticError> {
    let mut errors: Vec<SemanticError> = Vec::new();

    // If any of the expressions are invalid, just leave we're done here.
    // (The logical's datatype will be set to invalid appropriately)
    /* NOTE: I'm commenting this out because I think all of the other error coverage covers this,
     * and with more comprehensive errors. So for now, I'm leaving here just in case.
    if logical.data_type == DataType::Invalid {
        // TODO: add errors that say the logical is invalid.
        errors.push(SemanticError::UnexpectedStatement {
            line: line,
            explanation: "one of the comparisons in the logical is invalid.".to_string(),
        });
        return errors;
    }
    */

    // All of these must evaluate to booleans.
    // You can't say ("string" && 3); that doesn't make any sense.
    /* NOTE: this too? requires testing.

    for comparison in &mut logical.comparisons.iter() {
        if comparison.data_type != DataType::Boolean {
            // TODO: add error for this comparison here.
            errors.push(SemanticError::UnexpectedStatement {
                line: line,
                explanation: "logical is invalid; cannot have non boolean in logical expression."
                    .to_string(),
            });
            //println!("Logical is invalid: {:#?}", logical);
            return errors;
        }
    }
    */

    // make sure operations are being used correctly.
    let errors_prop_ops = get_operation_errors_logical(logical, line);
    errors.extend(errors_prop_ops);

    errors
}

fn get_operation_errors_logical(logical: &Logical, line: u32) -> Option<SemanticError> {
    for comparison in &logical.comparisons {
        if let Some(error) = get_operation_errors_comparison(comparison, line) {
            return Some(error);
        }
    }

    // Strings can't have any of these ops
    // Numbers can't have any of these ops
    // boolean who cares

    if logical.operators.is_empty() {
        return None;
    }

    // These all need to be booleans for this to be valid. Logical operators
    // are only used between booleans.
    for comparison in &logical.comparisons {
        if comparison.data_type != DataType::Boolean {
            return Some(SemanticError::UnexpectedStatement {
                line,
                explanation:
                    "logical cannot have && or || if string or number TODO make this better"
                        .to_string(),
            });
        }
    }

    None
}

fn get_operation_errors_comparison(comparison: &Comparison, line: u32) -> Option<SemanticError> {
    for expression in &comparison.expressions {
        if let Some(error) = get_operation_errors_expression(expression, line) {
            return Some(error);
        }
    }

    if comparison.operators.is_empty() {
        return None;
    }

    // strings can have == or !=
    // number can do whatever they want
    // bool can have ==, !=,
    let shared_expressions_type = &comparison.expressions[0].data_type;
    for expression in comparison.expressions.iter() {
        if expression.data_type != *shared_expressions_type {
            return Some(SemanticError::UnexpectedStatement {
                    line,
                    explanation:
                        "comparison has more than 1 datatype in it; you cannot have a comparison with multiple types. TODO: improve this."
                            .to_string(),
                });
        }
    }

    if comparison.expressions.len() > 1
        && matches!(
            shared_expressions_type,
            DataType::Boolean | DataType::String
        )
    {
        for op in comparison.operators.iter() {
            let does_use_invalid_op = !matches!(
                op,
                ComparisonOperator::equalequal | ComparisonOperator::notequal
            );
            let mut optional_err: Option<SemanticError> = None;
            if does_use_invalid_op {
                optional_err = Some(SemanticError::UnexpectedStatement {
                    line,
                    explanation:
                        "comparison of type string/bool is using inappropriate operators TODO: improve this."
                            .to_string(),
                })
            }
            return optional_err;
        }
    }

    None
}

fn get_operation_errors_expression(expression: &Expression, line: u32) -> Option<SemanticError> {
    for term in &expression.terms {
        if let Some(error) = get_operation_errors_term(term, line) {
            return Some(error);
        }
    }

    let shared_term_type = &expression.terms[0].data_type;
    for term in expression.terms.iter() {
        if term.data_type != *shared_term_type {
            return Some(SemanticError::UnexpectedStatement {
                line,
                explanation:
                    "expression contains terms of different types TODO: improve this err message."
                        .to_string(),
            });
        }
    }

    // Numbers do whatever they want
    // Booleans and Strings aren't allowed at all, just add an error.
    if expression.terms.len() > 1
        && matches!(shared_term_type, DataType::Boolean | DataType::String)
    {
        return Some(SemanticError::UnexpectedStatement {
            line,
            explanation:
                "expression cannot add/subtract from booleans or strings TODO: improve this err message"
                    .to_string(),
        });
    }

    None
}

fn get_operation_errors_term(term: &Term, line: u32) -> Option<SemanticError> {
    for unary in &term.unarys {
        if let Some(error) = get_operation_errors_unary(unary, line) {
            return Some(error);
        }
    }

    let shared_term_type = &term.unarys[0].data_type;
    for unary in term.unarys.iter() {
        if unary.data_type != *shared_term_type {
            return Some(SemanticError::UnexpectedStatement {
                line,
                explanation:
                    "term cannot have unarys of multiple data types TODO: imrpvoe this error message"
                        .to_string(),
            });
        }
    }

    // Numbers do whatever they want
    // Booleans and Strings aren't allowed at all, just add an error.
    if term.unarys.len() > 1 && matches!(shared_term_type, DataType::Boolean | DataType::String) {
        return Some(SemanticError::UnexpectedStatement {
            line,
            explanation:
                "term cannot add/subtract from booleans or strings TODO: improve this err message"
                    .to_string(),
        });
    }

    None
}

fn get_operation_errors_unary(unary: &Unary, line: u32) -> Option<SemanticError> {
    get_operation_errors_value(&unary.primary, line)
}

fn get_operation_errors_value(value: &Value, line: u32) -> Option<SemanticError> {
    // Check if this is a function call with parameters
    if value.value_type == ValueType::FunctionCall {
        for param in &value.params {
            if let Some(error) = get_operation_errors_logical(param, line) {
                return Some(error);
            }
        }
    }
    None
}
