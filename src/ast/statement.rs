//use crate::comparison::Logical;
use crate::ast::DataType;
use crate::ast::Value;
use crate::ast::comparison::Expression;

/**
 * Contains 'Statement' enum, and all of the specific Statement structs
 * that make up the AST, and is analyzed by the semantic module.
 */

#[derive(Debug)]
pub struct VariableDeclarationStatement {
    pub symbol_name: String,
    pub data_type: DataType,
    pub line_declared_on: u32,
    //pub assigned_value: Value,
    pub assigned_expr: Expression,
}

#[derive(Debug)]
pub struct VariableAssignmentStatement {
    pub var_name: String,
    pub var_data_type: DataType,
    //pub assigned_value: Value,
    pub assigned_expr: Expression,
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

#[derive(Debug)]
pub struct ReturnStatement {
    pub line_declared_on: u32,
    pub return_value: Option<Value>,
}

#[derive(Debug)]
pub struct PrintStatement {
    pub line_declared_on: u32,
    pub expression: Expression,
}

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration(VariableDeclarationStatement),
    VariableAssignment(VariableAssignmentStatement),
    FunctionDeclaration(FunctionDeclarationStatement),
    Return(ReturnStatement),
    Print(PrintStatement),
}
