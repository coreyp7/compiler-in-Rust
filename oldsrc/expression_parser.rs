use crate::comparison::*;
use crate::error::ErrMsg;
use crate::tokenizer::{Token, TokenType};

/// Expression parser that can be used by the main AST builder
/// This is a pretty wack solution since we have refs for everything with lifetimes.
/// This was due to the coupled nature of the AST builder.
pub struct ExpressionParser<'a> {
    tokens: &'a [Token],
    current: &'a mut usize,
    errors: &'a mut Vec<ErrMsg>,
}

impl<'a> ExpressionParser<'a> {
    pub fn new(tokens: &'a [Token], current: &'a mut usize, errors: &'a mut Vec<ErrMsg>) -> Self {
        ExpressionParser {
            tokens,
            current,
            errors,
        }
    }

    pub fn get_current(&self) -> usize {
        *self.current
    }

    fn get_curr_token(&self) -> &Token {
        &self.tokens[*self.current]
    }

    fn next_token(&mut self) {
        *self.current += 1;
    }

    fn is_curr_token_logical_operator(&self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::DoubleAmpersand => true,
            TokenType::DoubleBar => true,
            TokenType::Bang => true,
            _ => false,
        }
    }

    fn is_curr_token_comparison_operator(&self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::EqualEqual => true,
            TokenType::NotEqual => true,
            TokenType::LessThan => true,
            TokenType::LessThanEqualTo => true,
            TokenType::GreaterThan => true,
            TokenType::GreaterThanEqualTo => true,
            _ => false,
        }
    }

    fn is_curr_token_expression_operator(&self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::Plus => true,
            TokenType::Minus => true,
            _ => false,
        }
    }

    fn is_curr_token_term_operator(&self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::Asterisk => true,
            TokenType::Slash => true,
            _ => false,
        }
    }

    pub fn logical(&mut self) -> Logical {
        let mut logical = Logical::new();

        // check for optional bang
        if self.get_curr_token().token_type == TokenType::Bang {
            let op = LogicalOperator::from(TokenType::Bang);
            logical.operators.push(op);
            self.next_token();
        }

        let comp1 = self.comparison();
        logical.comparisons.push(comp1);

        let op1 = LogicalOperator::from(self.get_curr_token().token_type.clone());

        if op1 == LogicalOperator::Invalid {
            // No logical operators, there's just 1 comparison to process.
            // return the struct as is, with no operations
            println!("skipping this because its not logical op");
            println!("{:?}", self.get_curr_token());
            return logical;
        }

        while self.is_curr_token_logical_operator() {
            let op = LogicalOperator::from(self.get_curr_token().token_type.clone());
            logical.operators.push(op);
            self.next_token();

            // check for optional bang
            if self.get_curr_token().token_type == TokenType::Bang {
                let bang = LogicalOperator::from(TokenType::Bang);
                logical.operators.push(bang);
                self.next_token();
            }

            let comp: Comparison = self.comparison();
            logical.comparisons.push(comp);
        }

        logical
    }

    pub fn comparison(&mut self) -> Comparison {
        let mut comparison = Comparison {
            expressions: Vec::new(),
            operators: Vec::new(),
        };
        let expr1: Expression = self.expression();
        comparison.expressions.push(expr1);

        while self.is_curr_token_comparison_operator() {
            let op = ComparisonOperator::from(self.get_curr_token().token_type.clone());
            comparison.operators.push(op);
            self.next_token();

            let expr: Expression = self.expression();
            comparison.expressions.push(expr);
        }

        comparison
    }

    pub fn expression(&mut self) -> Expression {
        let mut expr = Expression {
            terms: Vec::new(),
            operators: Vec::new(),
        };

        let term1 = self.term();
        expr.terms.push(term1);

        while self.is_curr_token_expression_operator() {
            let op = ExpressionOperator::from(self.get_curr_token().token_type.clone());
            expr.operators.push(op);
            self.next_token();

            let term = self.term();
            expr.terms.push(term);
        }

        expr
    }

    pub fn term(&mut self) -> Term {
        let mut term = Term {
            unarys: Vec::new(),
            operations: Vec::new(),
        };

        let unary1 = self.unary();
        term.unarys.push(unary1);

        while self.is_curr_token_term_operator() {
            let op = TermOperator::from(self.get_curr_token().token_type.clone());
            term.operations.push(op);
            self.next_token();

            let unary = self.unary();
            term.unarys.push(unary);
        }

        term
    }

    pub fn unary(&mut self) -> Unary {
        let mut unary = Unary {
            operation: None,
            primary: Primary::Error {
                detail: String::new(),
            },
        };

        if self.is_curr_token_expression_operator() {
            unary.operation = Some(ExpressionOperator::from(
                self.get_curr_token().token_type.clone(),
            ));
            self.next_token();
        }

        unary.primary = self.primary();

        unary
    }

    pub fn primary(&mut self) -> Primary {
        let primary = match self.get_curr_token().token_type {
            TokenType::Number => Primary::Number {
                value: self.get_curr_token().text.clone(),
            },
            TokenType::Str => Primary::String {
                value: self.get_curr_token().text.clone(),
            },
            TokenType::Identity => {
                let identity_name = self.get_curr_token().text.clone();
                let line_number = self.get_curr_token().line_number;

                // Check if next token is '(' which indicates a function call
                let next_idx = *self.current + 1;
                if next_idx < self.tokens.len()
                    && self.tokens[next_idx].token_type == TokenType::LeftParen
                {
                    // This is a function call expression
                    self.next_token(); // Move to '('
                    self.next_token(); // Move past '('

                    let mut arguments = Vec::new();
                    while self.get_curr_token().token_type != TokenType::RightParen {
                        if self.get_curr_token().token_type == TokenType::Identity {
                            arguments.push(self.get_curr_token().text.clone());
                            self.next_token();

                            if self.get_curr_token().token_type == TokenType::Comma {
                                self.next_token();
                            }
                        } else {
                            self.errors.push(ErrMsg::UnexpectedToken {
                                expected: TokenType::Identity,
                                got: self.get_curr_token().token_type.clone(),
                                line_number: self.get_curr_token().line_number,
                            });
                            self.next_token();
                        }
                    }
                    // Don't advance past ')' here since next_token() is called at the end

                    Primary::FunctionCall {
                        name: identity_name,
                        arguments,
                        line_number,
                    }
                } else {
                    // This is just a variable reference
                    Primary::Identity {
                        name: identity_name,
                        line_number,
                    }
                }
            }
            _ => Primary::Error {
                detail: String::new(),
            },
        };
        self.next_token();

        primary
    }
}
