mod analyzer;
mod semantic_error;

pub use analyzer::SemanticAnalyzer;
pub use semantic_error::SemanticError;
pub use semantic_error::print_failures_message;
pub use semantic_error::print_success_message;
