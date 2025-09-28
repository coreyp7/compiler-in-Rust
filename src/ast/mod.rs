mod ast_builder;
mod function_table;

// Public facing API of AST module
pub use ast_builder::build_ast;

// TODO: temporary, should be moved into different module or something.
pub use ast_builder::DataType;
pub use ast_builder::FunctionDeclarationStatement;
pub use ast_builder::Statement;
pub use ast_builder::VariableDeclarationStatement;
pub use ast_builder::VariableSymbol;
pub use function_table::{FunctionSymbol, FunctionTable, Parameter};
