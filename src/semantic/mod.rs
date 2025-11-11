mod analyzer;
mod resolve_value_type;
mod semantic_error;
mod type_check;
mod validate;

pub use analyzer::analyze_statements;
pub use semantic_error::SemanticError;
pub use semantic_error::print_failures_message;
pub use semantic_error::print_success_message;
