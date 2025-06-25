use crate::tokenizer::Token;
use crate::tokenizer::TokenType;
//use colored::Colorize;
use std::collections::HashSet;
use std::io;
use std::io::Write;
use colored::Colorize;

pub struct Node {
   value: u8 
}

pub struct AstBuilder<> {
    pub head: Node,
    pub tokens: Vec<Token>,
    curr_idx: usize,
    statements: Vec<Statement> 
}

impl AstBuilder<> {
    pub fn new(token_vec: Vec<Token>) -> AstBuilder {
        let node: Node = Node {value: 1};

        AstBuilder {
            head: node,
            tokens: token_vec,
            curr_idx: 0,
            statements: Vec::new()
        }
    }


    pub fn generate_ast(&mut self){
        self.program() 
    }

    fn get_curr_token(&mut self) -> &Token {
        &self.tokens[self.curr_idx]
    }


    fn next_token(&mut self){
        self.curr_idx = self.curr_idx + 1;
    }

    fn program(&mut self){
        println!("program() start");  
        
        while self.get_curr_token().token_type != TokenType::EOF {
            self.statement();
        }
    }

    fn is_curr_token_type(&mut self, t_type: &TokenType) -> bool{
        return self.get_curr_token().token_type == *t_type;
    }

    fn is_curr_token_comparison_operator(&mut self) -> bool {
        match &self.get_curr_token().token_type {
            TokenType::EqualEqual => true,
            TokenType::NotEqual => true,
            TokenType::LessThan => true,
            TokenType::LessThanEqualTo => true,
            TokenType::GreaterThan => true,
            TokenType::GreaterThanEqualTo => true,
            _ => false
        }
    }


    fn statement(&mut self) -> Statement {
        let mut statement = Statement::TestStub;
        //let curr_token = self.get_curr_token();
        match self.get_curr_token().token_type {
            TokenType::Print => {
                //println!("{:?}", curr_token);                
                self.next_token();
                let string_content: String = self.get_curr_token().text.clone();
                
                if self.get_curr_token().token_type != TokenType::Str {
                    println!("ERROR: expecting string, got {:#?}", 
                        self.get_curr_token().token_type
                    );
                }

                statement = Statement::Print{
                        content: string_content,
                        line_number: self.get_curr_token().line_number
                    };
            },
            TokenType::If => {
                /**

                Notes for future corey.

                So, the way the structs are written are a bit naive.
                A comparison MUST be 2 expressions with a comparison operator
                in between them.

                However, optionally, a comparison can have 0 or more expressions
                prefixed with another operator.

                How will this data look? How should this be organized?

                Idea:
                A vector (or two?) could keep these in order.
                So, a vector<Expression>
                and a vector<ComparisonOperator>.

                As we navigate through each token we add them to these lists.
                Keep looping through tokens until there isn't a comparison operator next.
                Then, condition the data into what makes sense for the
                comparison struct.

                Idea 2:
                Have this be a linkedlist of structs, and have the node pointer
                be an optional. Then the links to the next expression go
                until the optional is None.
                */
                self.next_token();

                // parse comparison
                let comparison = self.comparison();

                // TODO: assert that the next keyword is 'Then'
                self.next_token();

                let mut statements: Vec<Statement> = Vec::new();

                while !self.is_curr_token_type(&TokenType::EndIf){
                    statements.push(self.statement());
                }

                // TODO: assert the keyword is EndIf
                self.next_token();

                statement = Statement::If{
                        comparison: comparison,
                        statements: statements 
                };
            },
            //TokenType::Newline => {
                // I don't think I have to do anything here.

            //},
            _ => {
            }
        };

/*
        match statement {
            !Statement::TestStub => {
                println!("{:?}", value);                
                self.statements.push(value);
            },
            _ => {
                */
        let line_number = self.get_curr_token().line_number;
        let col_number = self.get_curr_token().col_number;
        let token_type = &self.get_curr_token().token_type;
        println!("{:#?} at line number {}", 
            statement,
            line_number
        );
        /*
        if *token_type != tokentype::newline {
            println!("skipping {:?} at {},{}", 
                token_type,
                line_number,
                col_number
            );
        }
        */


        self.next_token();
        return statement;
    }

    fn comparison(&mut self) -> Comparison {
        let mut comparison = Comparison {
            expressions: Vec::new(),
            operators: Vec::new()
        };
        let expr1: Expression = self.expression(); // have this emulate an ouput
        comparison.expressions.push(expr1);

        let op1: ComparisonOperator = convert_token_type_to_comparison_op(
            self.get_curr_token().token_type.clone()
        );
        comparison.operators.push(op1);
        self.next_token();

        let expr2: Expression = self.expression(); // TODO: get current token as comparison operator
        comparison.expressions.push(expr2);

        while self.is_curr_token_comparison_operator() {
            let op: ComparisonOperator = convert_token_type_to_comparison_op(
                self.get_curr_token().token_type.clone()
            );
            comparison.operators.push(op);
            self.next_token();
            
            let expr: Expression = self.expression();
            comparison.expressions.push(expr);
        }

        return comparison;
    }

    fn expression(&mut self) -> Expression {
        // this is test shit rn
        let expr = Expression {
            term_left: String::from("left"),
            operation: Operation::Plus,
            term_right: String::from("right")
        };
        self.next_token();
        return expr;
    }
}

fn convert_token_type_to_comparison_op(token_type: TokenType) -> ComparisonOperator {
    match token_type {
        TokenType::EqualEqual => ComparisonOperator::equalequal,
        TokenType::NotEqual => ComparisonOperator::notequal,
        TokenType::LessThan => ComparisonOperator::lessthan,
        TokenType::LessThanEqualTo => ComparisonOperator::lessthanequalto,
        TokenType::GreaterThan => ComparisonOperator::greaterthan,
        TokenType::GreaterThanEqualTo => ComparisonOperator::greaterthanequalto,
        _ => ComparisonOperator::invalidop
    }    
}

#[derive(Debug)]
enum Operation {
    Plus,
    Minus,
    Multiply,
    Divide
}

#[derive(Debug)]
enum Statement {
    Print {
        content: String,
        line_number: u8
    },
    If {
        comparison: Comparison,
        statements: Vec<Statement>
    },
    TestStub
}


#[derive(Debug)]
enum ComparisonOperator {
    equalequal,
    notequal,
    lessthan,
    lessthanequalto, 
    greaterthan,
    greaterthanequalto,
    invalidop
}


/** 
* The way this works:
* These are two lists of all of the expresisons and operators in between.
* So, it is in the order specified in code.
* that is: expressions[0], operators[0], expressions[1], 
*          operators[1], expressions[2], etc.....
*/
#[derive(Debug)]
struct Comparison {
    expressions: Vec<Expression>,
    operators: Vec<ComparisonOperator>
}

// Either + or -
#[derive(Debug)]
struct Expression {
    /*
    term_left: Term,
    operation: Operation,
    term_right: Term
    */
    term_left: String,
    operation: Operation,
    term_right: String
}

#[derive(Debug)]
struct Term {
    unary_left: Unary,
    operation: Operation,
    unary_right: Unary
}

#[derive(Debug)]
struct Unary {
    operation: Operation,
    primary: Primary
}

#[derive(Debug)]
enum Primary {
    Number {
       value: u8 
    },
    Identity {
        name: String
    },
    Error {
        detail: String
    }
}
