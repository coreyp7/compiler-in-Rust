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
    Invalid,
}

impl From<TokenType> for LogicalOperator {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::DoubleAmpersand => LogicalOperator::And,
            TokenType::DoubleBar => LogicalOperator::Or,
            TokenType::Bang => LogicalOperator::Not,
            _ => LogicalOperator::Invalid,
        }
    }
}

#[derive(Debug)]
pub struct Comparison {
    pub expressions: Vec<Expression>,
    pub operators: Vec<ComparisonOperator>,
}

#[derive(Debug)]
pub enum ComparisonOperator {
    EqualEqual,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    Invalid,
}

impl From<TokenType> for ComparisonOperator {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::EqualEqual => ComparisonOperator::EqualEqual,
            TokenType::NotEqual => ComparisonOperator::NotEqual,
            TokenType::LessThan => ComparisonOperator::LessThan,
            TokenType::LessThanEqualTo => ComparisonOperator::LessThanEqualTo,
            TokenType::GreaterThan => ComparisonOperator::GreaterThan,
            TokenType::GreaterThanEqualTo => ComparisonOperator::GreaterThanEqualTo,
            _ => ComparisonOperator::Invalid,
        }
    }
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
    Invalid,
}

impl From<TokenType> for ExpressionOperator {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Plus => ExpressionOperator::Plus,
            TokenType::Minus => ExpressionOperator::Minus,
            _ => ExpressionOperator::Invalid,
        }
    }
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
    Invalid, // TODO: these should be changed to no op
}

impl From<TokenType> for TermOperator {
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Asterisk => TermOperator::Multiply,
            TokenType::Slash => TermOperator::Divide,
            _ => TermOperator::Invalid,
        }
    }
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
    FunctionCall {
        name: String,
        arguments: Vec<String>,
        line_number: u8,
    },
    String {
        value: String,
    },
    Error {
        detail: String,
    },
}
