use crate::ast::DataType;
use crate::ast::{Comparison, Expression, Logical, Term, Unary};
use crate::semantic::SemanticError;

// this needs to be given state so we can add errors.
// Or maybe it can return a vec of errors? Idk.
pub fn validate_logical(logical: &Logical, line: u32) -> Vec<SemanticError> {
    let mut errors: Vec<SemanticError> = Vec::new();

    // If any of the expressions are invalid, just leave we're done here.
    if !logical.is_valid {
        // TODO: add errors that say the logical is invalid.
        errors.push(SemanticError::UnexpectedStatement {
            line: line,
            explanation: "one of the comparisons in the logical is invalid.".to_string(),
        });
        return errors;
    }

    if logical.operators.len() == 0 {
        // if this is the case, we don't need the comparison(s) to evaluate to
        // booleans, since that means this may just be a normal expression.
        return errors;
    }

    println!("corey made it here");

    // All of these must evaluate to booleans.
    // You can't say ("string" && 3); that doesn't make any sense.
    for comparison in &mut logical.comparisons.iter() {
        if comparison.data_type != DataType::Boolean {
            // TODO: add error for this comparison here.
            errors.push(SemanticError::UnexpectedStatement {
                line: line,
                explanation: "logical is invalid; cannot have non boolean in logical expression."
                    .to_string(),
            });
            return errors;
        }
    }

    errors
}
