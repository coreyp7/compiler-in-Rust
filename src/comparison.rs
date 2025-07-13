use crate::tokenizer::TokenType;

/**
* The way this works:
* These are two lists of all of the expresisons and operators in between.
* So, it is in the order specified in code.
* that is: expressions[0], operators[0], expressions[1],
*          operators[1], expressions[2], etc.....
*/

/*
 * grammar:
 * Logical := Comparison (op) Comparison [(op) Comparison]
 */
#[derive(Debug)]
pub struct Logical {
    pub comparisons: Vec<Comparison>,
    pub operators: Vec<LogicalOperator>,
}

impl Logical {
    pub fn new() -> Logical {
        Logical {
            comparisons: Vec::new(),
            operators: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
    invalidop,
}

#[derive(Debug)]
pub struct Comparison {
    pub expressions: Vec<Expression>,
    pub operators: Vec<ComparisonOperator>,
}

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

// Either + or -
#[derive(Debug)]
pub struct Expression {
    pub terms: Vec<Term>,
    pub operators: Vec<ExpressionOperator>,
}

#[derive(Debug)]
pub enum ExpressionOperator {
    Plus,
    Minus,
    invalidop,
}

#[derive(Debug)]
pub struct Term {
    pub unarys: Vec<Unary>,
    pub operations: Vec<TermOperator>,
}

#[derive(Debug)]
pub enum TermOperator {
    Multiply,
    Divide,
    invalidop, // TODO: these should be changed to no op
}

#[derive(Debug)]
pub struct Unary {
    pub operation: Option<ExpressionOperator>,
    pub primary: Primary,
}

#[derive(Debug)]
pub enum Primary {
    Number {
        value: String, //TODO: change this to u8 and do conversions :(
    },
    Identity {
        name: String,
        line_number: u8,
    },
    Error {
        detail: String,
    },
}

// Conversion functions
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

pub fn convert_token_type_to_logical_op(token_type: TokenType) -> LogicalOperator {
    match token_type {
        TokenType::DoubleAmpersand => LogicalOperator::And,
        TokenType::DoubleBar => LogicalOperator::Or,
        TokenType::Bang => LogicalOperator::Not,
        _ => LogicalOperator::invalidop,
    }
}
