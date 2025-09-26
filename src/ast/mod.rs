mod ast_builder;

// Public facing API of AST module
pub use ast_builder::build_ast;

// TODO: temporary, should be moved into different module or something.
pub use ast_builder::Statement;
pub use ast_builder::VariableDeclarationStatement;
