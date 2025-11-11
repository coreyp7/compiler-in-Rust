//use crate::comparison::Logical;
use crate::ast::DataType;
use crate::ast::Value;
use crate::ast::value_hierarchy::{Expression, Logical};

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration(VariableDeclarationStatement),
    VariableAssignment(VariableAssignmentStatement),
    FunctionDeclaration(FunctionDeclarationStatement),
    RawFunctionCall(RawFunctionCallStatement),
    Return(ReturnStatement),
    Print(PrintStatement),
    If(IfStatement),
    While(WhileStatement),
}

/**
 * Contains 'Statement' enum, and all of the specific Statement structs
 * that make up the AST, and is analyzed by the semantic module.
 */

#[derive(Debug)]
pub struct VariableDeclarationStatement {
    pub symbol_name: String,
    pub data_type: DataType,
    pub line_declared_on: u32,
    pub assigned_logical: Logical, // we can have bools now :)
}

#[derive(Debug)]
pub struct VariableAssignmentStatement {
    pub var_name: String,
    pub var_data_type: DataType,
    pub assigned_logical: Logical,
    pub line_var_was_declared_on: u32,
    pub line_number: u32,
}

#[derive(Debug)]
pub struct FunctionDeclarationStatement {
    pub function_name: String,
    pub return_type: DataType,
    pub line_declared_on: u32,
    pub body: Vec<Statement>,
}

/// This is pretty much a wrapper struct so you can just call a function without
/// assigning it to anything. Thus, its just a Value, since Values encompass
/// function calls.
#[derive(Debug)]
pub struct RawFunctionCallStatement {
    pub line: u32,
    pub value: Value,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub line_declared_on: u32,
    pub return_value: Logical,
}

#[derive(Debug)]
pub struct PrintStatement {
    pub line: u32,
    pub logical: Logical,
    pub is_print_ln: bool,
}

#[derive(Debug)]
pub struct IfStatement {
    pub line_declared_on: u32,
    pub condition: Logical,
    pub if_body: Vec<Statement>,
    pub else_body: Option<Vec<Statement>>,
}

#[derive(Debug)]
pub struct WhileStatement {
    pub line_declared_on: u32,
    pub condition: Logical,
    pub body: Vec<Statement>,
}
