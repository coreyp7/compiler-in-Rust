use crate::comparison::Logical;

// Re-export VarType for convenience
pub use crate::ast::VarType;

#[derive(Debug)]
pub struct PrintStatement {
    pub content: String,
    pub is_content_identity_name: bool,
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
pub enum Statement {
    Print(PrintStatement),
    If(IfStatement),
    While(WhileStatement),
    Assignment(AssignmentStatement),
    VarInstantiation(VarInstantiationStatement),
    Newline,
    TestStub,
}
