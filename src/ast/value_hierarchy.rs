use crate::tokenizer::TokenType;

/// This is a lazy solution to a specific problem.
/// All of the operators in different grammars in the "value hierarchy" are
/// stored as different enums (ExpressionOp, UnaryOp, etc.)
/// However, we want to typecheck and validate Expressions.
/// It'd be really helpful if we could store all of the operators across an
/// entire expression in a single vec. So, this shit exists.
///
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum GeneralOperator {
    // expression/unary
    Plus,
    Minus,
    // term
    Multiply,
    Divide,
    // for later convenience: Comparison
    EqualEqual,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    // for later convenience: logical
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Number,
    String,
    Boolean,
    Void,
    Unknown, // Used when type needs to be resolved before semantic analysis
    Invalid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    FunctionCall,
    Expression,
    InlineNumber,
    InlineString,
    InlineBoolean,
    Variable,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub data_type: DataType,
    pub value_type: ValueType,
    pub raw_text: String, // The raw text from the source, for reference
    /// Only exists if value_type = FunctionCall; we need to record the expressions
    /// being passed in as params.
    pub param_values: Option<Vec<Expression>>,
}

impl Value {
    pub fn new(data_type: DataType, value_type: ValueType, raw_text: String) -> Self {
        Value {
            data_type,
            value_type,
            raw_text,
            param_values: None,
        }
    }

    pub fn new_with_params(
        data_type: DataType,
        value_type: ValueType,
        raw_text: String,
        param_values: Vec<Expression>,
    ) -> Self {
        Value {
            data_type,
            value_type,
            raw_text,
            param_values: Some(param_values),
        }
    }

    pub fn invalid() -> Self {
        Value::new(DataType::Invalid, ValueType::Invalid, String::new())
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operation: Option<ExpressionOperator>,
    pub primary: Value,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub unarys: Vec<Unary>,
    pub operations: Vec<TermOperator>,
    pub data_type: DataType,
}

impl Term {
    pub fn new() -> Term {
        Term {
            unarys: Vec::new(),
            operations: Vec::new(),
            data_type: DataType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TermOperator {
    Multiply,
    Divide,
    invalidop, // TODO: these should be changed to no op
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub terms: Vec<Term>,
    pub operators: Vec<ExpressionOperator>,
    pub data_type: DataType,
}

impl Expression {
    pub fn new() -> Expression {
        Expression {
            terms: Vec::new(),
            operators: Vec::new(),
            data_type: DataType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionOperator {
    Plus,
    Minus,
    invalidop,
}

/// A comparison expression (handles equality, inequality, etc.)
#[derive(Debug)]
pub struct Comparison {
    pub expressions: Vec<Expression>,
    pub operators: Vec<ComparisonOperator>,
    // does is the comparison valid and evaluates to a boolean?
    pub is_valid: bool,
}

impl Comparison {
    pub fn new() -> Comparison {
        Comparison {
            expressions: Vec::new(),
            operators: Vec::new(),
            is_valid: true,
        }
    }
}

/// Operators that can appear at the comparison level (==, !=, <, >, etc.)
#[derive(Debug)]
pub enum ComparisonOperator {
    equalequal,
    notequal,
    lessthan,
    lessthanequalto,
    greaterthan,
    greaterthanequalto,
    invalidop,
}

/// A logical expression (handles AND, OR, NOT)
#[derive(Debug)]
pub struct Logical {
    pub comparisons: Vec<Comparison>,
    pub operators: Vec<LogicalOperator>,
    pub is_valid: bool,
}

impl Logical {
    pub fn new() -> Logical {
        Logical {
            comparisons: Vec::new(),
            operators: Vec::new(),
            is_valid: true, // assume its innocent until proven guilty
        }
    }
}

/// Operators that can appear at the logical level (&&, ||, !)
#[derive(Debug, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
    invalidop,
}

// Below are helper functions for when I'm lazy and need to easily convert stuff
// between each other.

pub fn convert_token_type_to_expression_op(token_type: TokenType) -> ExpressionOperator {
    match token_type {
        TokenType::Plus => ExpressionOperator::Plus,
        TokenType::Minus => ExpressionOperator::Minus,
        _ => ExpressionOperator::invalidop,
    }
}

pub fn convert_token_type_to_term_op(token_type: TokenType) -> TermOperator {
    match token_type {
        TokenType::Asterisk => TermOperator::Multiply,
        TokenType::Slash => TermOperator::Divide,
        _ => TermOperator::invalidop,
    }
}

pub fn convert_token_type_to_comparison_op(token_type: TokenType) -> ComparisonOperator {
    match token_type {
        TokenType::EqualEqual => ComparisonOperator::equalequal,
        TokenType::NotEqual => ComparisonOperator::notequal,
        TokenType::LessThan => ComparisonOperator::lessthan,
        TokenType::LessThanEqualTo => ComparisonOperator::lessthanequalto,
        TokenType::GreaterThan => ComparisonOperator::greaterthan,
        TokenType::GreaterThanEqualTo => ComparisonOperator::greaterthanequalto,
        _ => ComparisonOperator::invalidop,
    }
}

pub fn convert_token_type_to_logical_op(token_type: TokenType) -> LogicalOperator {
    match token_type {
        TokenType::DoubleAmpersand => LogicalOperator::And,
        TokenType::DoubleBar => LogicalOperator::Or,
        TokenType::Bang => LogicalOperator::Not,
        _ => LogicalOperator::invalidop,
    }
}

// Helper functions to convert specific operator enums to GeneralOperator

pub fn convert_expression_op_to_general(op: ExpressionOperator) -> Option<GeneralOperator> {
    match op {
        ExpressionOperator::Plus => Some(GeneralOperator::Plus),
        ExpressionOperator::Minus => Some(GeneralOperator::Minus),
        ExpressionOperator::invalidop => None,
    }
}

pub fn convert_term_op_to_general(op: TermOperator) -> Option<GeneralOperator> {
    match op {
        TermOperator::Multiply => Some(GeneralOperator::Multiply),
        TermOperator::Divide => Some(GeneralOperator::Divide),
        TermOperator::invalidop => None,
    }
}

pub fn convert_comparison_op_to_general(op: ComparisonOperator) -> Option<GeneralOperator> {
    match op {
        ComparisonOperator::equalequal => Some(GeneralOperator::EqualEqual),
        ComparisonOperator::notequal => Some(GeneralOperator::NotEqual),
        ComparisonOperator::lessthan => Some(GeneralOperator::LessThan),
        ComparisonOperator::lessthanequalto => Some(GeneralOperator::LessThanEqualTo),
        ComparisonOperator::greaterthan => Some(GeneralOperator::GreaterThan),
        ComparisonOperator::greaterthanequalto => Some(GeneralOperator::GreaterThanEqualTo),
        ComparisonOperator::invalidop => None,
    }
}

pub fn convert_logical_op_to_general(op: LogicalOperator) -> Option<GeneralOperator> {
    match op {
        LogicalOperator::And => Some(GeneralOperator::And),
        LogicalOperator::Or => Some(GeneralOperator::Or),
        LogicalOperator::Not => Some(GeneralOperator::Not),
        LogicalOperator::invalidop => None,
    }
}
