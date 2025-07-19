use crate::comparison::Logical;

// Re-export types for convenience
pub use crate::ast::{FunctionHeader, Var, VarType};

#[derive(Debug)]
pub struct PrintStatement {
    pub content: String,
    pub is_content_identity_name: bool,
    pub variable_type: Option<VarType>, // Type of the variable being printed (if it's a variable)
    pub line_number: u8,
}

#[derive(Debug)]
pub struct IfStatement {
    pub logical: Logical,
    pub statements: Vec<Statement>,
    pub line_number: u8,
}

#[derive(Debug)]
pub struct WhileStatement {
    pub logical: Logical,
    pub statements: Vec<Statement>,
    pub line_number: u8,
}

#[derive(Debug)]
pub struct AssignmentStatement {
    pub identity: String,
    pub value: String,
    pub assigned_value_type: VarType,
    pub line_number: u8,
}

#[derive(Debug)]
pub struct VarInstantiationStatement {
    pub identity: String,
    pub value: String,
    pub var_type: VarType,
    pub assigned_value_type: VarType,
    pub line_number: u8,
}

#[derive(Debug)]
pub struct FunctionInstantiationStatement {
    pub header: FunctionHeader,
    pub statements: Vec<Statement>,
    pub line_number: u8,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub return_type: VarType,
    pub return_value: Option<Var>,
    pub line_number: u8,
    pub is_identity: bool,
}

#[derive(Debug)]
pub struct FunctionCallStatement {
    pub function_name: String,
    pub arguments: Vec<String>,
    pub line_number: u8,
}

#[derive(Debug)]
pub enum Statement {
    Print(PrintStatement),
    If(IfStatement),
    While(WhileStatement),
    Assignment(AssignmentStatement),
    VarInstantiation(VarInstantiationStatement),
    FunctionInstantiation(FunctionInstantiationStatement),
    FunctionCall(FunctionCallStatement),
    Return(ReturnStatement),
    Newline,
    TestStub,
}
