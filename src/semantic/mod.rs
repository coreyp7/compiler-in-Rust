mod analyzer;
mod resolve_value_type;
mod semantic_error;

pub use analyzer::analyze_statements;
pub use resolve_value_type::resolve_all_value_types_in_ast;
pub use semantic_error::SemanticError;
pub use semantic_error::print_failures_message;
pub use semantic_error::print_success_message;
