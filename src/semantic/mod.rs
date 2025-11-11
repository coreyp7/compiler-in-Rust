mod analyzer;
pub use analyzer::analyze_statements;

mod semantic_error;
pub use semantic_error::SemanticError;
pub use semantic_error::print_failures_message;
pub use semantic_error::print_success_message;

mod type_check;

mod type_resolution;

mod validate;
