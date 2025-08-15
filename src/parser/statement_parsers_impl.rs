use super::{ParserContext, StatementParser};
use crate::ast::{FunctionHeader, FunctionParameter, Var, VarType};
use crate::comparison::*;
use crate::error::ErrMsg;
use crate::expression_parser::ExpressionParser;
use crate::statement::*;
use crate::tokenizer::TokenType;

/// Parser for print statements
pub struct PrintStatementParser;

impl StatementParser for PrintStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::Print)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        context.next_token(); // Skip 'print'

        let string_content = context.get_curr_token().text.clone();
        let mut is_identity = false;

        match &context.get_curr_token().token_type {
            TokenType::Str => {}
            TokenType::Identity => {
                is_identity = true;
            }
            _ => {
                context.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::Str, // We expect either Str or Identity
                    got: context.get_curr_token().token_type.clone(),
                    line_number: context.get_curr_token().line_number,
                });
            }
        }

        // Look up variable type if it's a variable reference
        let variable_type = if is_identity {
            context.symbol_table.get_variable_type(&string_content)
        } else {
            None
        };

        Statement::Print(PrintStatement {
            content: string_content,
            line_number: context.get_curr_token().line_number,
            is_content_identity_name: is_identity,
            variable_type,
        })
    }
}

/// Parser for if statements
pub struct IfStatementParser;

impl StatementParser for IfStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::If)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        context.next_token(); // Skip 'if'

        // Parse the logical condition
        let mut expr_parser =
            ExpressionParser::new(&context.tokens, &mut context.current, &mut context.errors);
        let conditional = expr_parser.logical();

        if let Err(_err) = context.expect_and_consume(TokenType::Then) {
            // Error already added to context.errors
        }

        let mut statements = Vec::new();
        while !context.is_curr_token_type(&TokenType::EndIf) && !context.at_end() {
            statements.push(parse_single_statement(context));
        }

        if !context.at_end() {
            context.next_token(); // Skip 'endIf'
        }

        Statement::If(IfStatement {
            logical: conditional,
            statements,
            line_number: context.get_curr_token().line_number,
        })
    }
}

/// Parser for while statements
pub struct WhileStatementParser;

impl StatementParser for WhileStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::While)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        context.next_token(); // Skip 'while'

        // Parse the logical condition
        let mut expr_parser =
            ExpressionParser::new(&context.tokens, &mut context.current, &mut context.errors);
        let conditional = expr_parser.logical();

        if let Err(_err) = context.expect_and_consume(TokenType::Do) {
            // Error already added to context.errors
        }

        let mut statements = Vec::new();
        while !context.is_curr_token_type(&TokenType::EndWhile) && !context.at_end() {
            statements.push(parse_single_statement(context));
        }

        if !context.at_end() {
            context.next_token(); // Skip 'endWhile'
        }

        Statement::While(WhileStatement {
            logical: conditional,
            statements,
            line_number: context.get_curr_token().line_number,
        })
    }
}

/// Parser for return statements
pub struct ReturnStatementParser;

impl StatementParser for ReturnStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::Return)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        context.next_token(); // Skip 'return'

        let mut return_value = None;
        let mut return_type = VarType::Unrecognized;
        let mut is_identity = false;

        // Check if there's a return value (not just a bare return)
        if !context.is_curr_token_type(&TokenType::Newline) && !context.at_end() {
            match context.get_curr_token().token_type {
                TokenType::Number => {
                    return_value = Some(Var {
                        identity: context.get_curr_token().text.clone(),
                        var_type: VarType::Num,
                        line_declared_on: context.get_curr_token().line_number,
                    });
                    return_type = VarType::Num;
                }
                TokenType::Str => {
                    return_value = Some(Var {
                        identity: context.get_curr_token().text.clone(),
                        var_type: VarType::Str,
                        line_declared_on: context.get_curr_token().line_number,
                    });
                    return_type = VarType::Str;
                }
                TokenType::Identity => {
                    let var_name = context.get_curr_token().text.clone();
                    if let Some(var_info) = context.symbol_table.lookup_variable(&var_name) {
                        return_value = Some(Var {
                            identity: var_name.clone(),
                            var_type: var_info.var_type.clone(),
                            line_declared_on: context.get_curr_token().line_number,
                        });
                        return_type = var_info.var_type.clone();
                    } else {
                        // NOTE: This isn't in the analyzer since we have to populate the return
                        // value struct anyway, so we need to lookup in the map.
                        // But perhaps we should just be returning the symbol name and then having
                        // the analyzer figure that shit out later? I think that'd make sense.
                        // But what if a function returns an inline variable? So,
                        // we'd probably be best to still use the struct with info.
                        // But it'd be better to use a struct that's actually appropriate for this situation.
                        /*
                        Struct new:
                        is_identity: bool,
                        content: String (either symbol identifier or string/num inline)

                        Then, we'd actually include helpful info for the code generator.
                        Would we regret this? I'm not sure what would be important to include later.
                         */
                        context.errors.push(ErrMsg::VariableNotDeclared {
                            identity: var_name,
                            attempted_assignment_line: context.get_curr_token().line_number,
                        });
                    }
                    is_identity = true;
                }
                _ => {
                    context.errors.push(ErrMsg::UnexpectedToken {
                        expected: TokenType::Identity,
                        got: context.get_curr_token().token_type.clone(),
                        line_number: context.get_curr_token().line_number,
                    });
                }
            }
        }

        Statement::Return(ReturnStatement {
            return_type,
            return_value,
            line_number: context.get_curr_token().line_number,
            is_identity,
        })
    }
}

/// Parser for Identity-based statements (function calls and variable assignments)
/// This handles the complex logic of distinguishing between function calls and assignments
pub struct IdentityStatementParser;

impl StatementParser for IdentityStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::Identity)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        let identity = context.get_curr_token().text.clone();
        context.next_token();

        // Look ahead to determine if this is a function call or assignment
        if context.is_curr_token_type(&TokenType::LeftParen) {
            self.parse_function_call(context, identity)
        } else {
            self.parse_variable_assignment(context, identity)
        }
    }
}

impl IdentityStatementParser {
    fn parse_function_call(
        &self,
        context: &mut ParserContext<'_>,
        function_name: String,
    ) -> Statement {
        context.next_token(); // Skip '('

        let mut arguments = Vec::new();
        while !context.is_curr_token_type(&TokenType::RightParen) && !context.at_end() {
            if context.is_curr_token_type(&TokenType::Identity) {
                arguments.push(context.get_curr_token().text.clone());
                context.next_token();

                if context.is_curr_token_type(&TokenType::Comma) {
                    context.next_token();
                }
            } else {
                context.errors.push(ErrMsg::UnexpectedToken {
                    expected: TokenType::Identity,
                    got: context.get_curr_token().token_type.clone(),
                    line_number: context.get_curr_token().line_number,
                });
                context.next_token();
            }
        }

        if let Err(_) = context.expect_and_consume(TokenType::RightParen) {
            // Error already added to context.errors
        }

        Statement::FunctionCall(FunctionCallStatement {
            function_name,
            arguments,
            line_number: context.get_curr_token().line_number,
        })
    }

    fn parse_variable_assignment(
        &self,
        context: &mut ParserContext<'_>,
        identity: String,
    ) -> Statement {
        if let Err(_) = context.expect_and_consume(TokenType::LessThanEqualTo) {
            // Error already added
        }

        let assignment_token_type = context.get_curr_token().token_type.clone();
        let assignment_value_text = context.get_curr_token().text.clone();
        // Just in case this is a function assignment, check if next tokens are parentheses.
        // A peek function should prolly exist TODO
        let mut isFunctionCall = false;
        if context.tokens.get(context.current + 1).unwrap().token_type == TokenType::LeftParen {
            // This is a function call because we can see the parentheses.
            // TODO: might have to improve this shitty behavior.
            isFunctionCall = true;
        }

        // Check if we're assigning a function reference
        let assignment_var_type = match assignment_token_type {
            TokenType::Identity => {
                if isFunctionCall {
                    let mut functionReturnType = VarType::Unrecognized;
                    let functionHeaderOption =
                        context.symbol_table.lookup_function(&assignment_value_text);

                    if let Some(functionHeader) = functionHeaderOption {
                        functionReturnType = functionHeader.return_type.clone();
                    }

                    // If the function doesn't return anything, or other unexpected behavior
                    // reading this, this will be unexpected. (or if its void)
                    functionReturnType
                } else {
                    // This means its just a normal variable lookup
                    // Regular variable reference - get its type
                    context
                        .symbol_table
                        .get_variable_type(&assignment_value_text)
                        .unwrap_or(VarType::Unrecognized)
                }
            }
            _ => VarType::from(assignment_token_type),
        };

        Statement::Assignment(AssignmentStatement {
            identity,
            value: assignment_value_text,
            assigned_value_type: assignment_var_type,
            line_number: context.get_curr_token().line_number,
            is_function: isFunctionCall,
        })
    }
}

/// Parser for variable declarations
pub struct VarDeclarationStatementParser;

impl StatementParser for VarDeclarationStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::VarDeclaration)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        let var_type = VarType::from(context.get_curr_token().text.as_str());
        context.next_token();

        let identity = context.get_curr_token().text.clone();
        context.next_token();

        if let Err(_) = context.expect_and_consume(TokenType::Colon) {
            // Error already added
        }

        // Parse the assignment value using primary()
        let mut expr_parser =
            ExpressionParser::new(&context.tokens, &mut context.current, &mut context.errors);
        let assignment_primary = expr_parser.primary();
        let (assignment_value_text, assignment_var_type) =
            extract_value_and_type_from_primary(&assignment_primary, &context.symbol_table);

        // Type checking (could be moved to semantic analysis)
        if var_type != assignment_var_type && assignment_var_type != VarType::Unrecognized {
            // TODO: move into semantic analyzer
            context.errors.push(ErrMsg::new_incorrect_type_assignment(
                var_type.clone(),
                assignment_var_type.clone(),
                context.get_curr_token().line_number,
            ));
        }

        Statement::VarInstantiation(VarInstantiationStatement {
            identity,
            value: assignment_value_text,
            var_type,
            assigned_value_type: assignment_var_type,
            line_number: context.get_curr_token().line_number,
        })
    }
}

/// Parser for function declarations
pub struct FunctionDeclarationStatementParser;

impl StatementParser for FunctionDeclarationStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::FunctionDeclaration)
    }

    fn parse(&mut self, context: &mut ParserContext<'_>) -> Statement {
        context.next_token(); // Skip 'function'

        let function_name = context.get_curr_token().text.clone();

        // Look up the function header that was already parsed in Phase 1
        let function_header = if let Some(header) = context
            .symbol_table
            .lookup_function(&function_name)
            .cloned()
        {
            header
        } else {
            context.errors.push(ErrMsg::UnexpectedToken {
                expected: TokenType::FunctionDeclaration,
                got: context.get_curr_token().token_type.clone(),
                line_number: context.get_curr_token().line_number,
            });
            return Statement::TestStub;
        };

        // Skip past the function signature since we already have it
        self.skip_function_signature(context);

        // Parse the function body statements
        let mut function_statements = Vec::new();
        while !context.is_curr_token_type(&TokenType::EndFunction) && !context.at_end() {
            let statement = parse_single_statement(context);
            if !matches!(statement, Statement::Newline) {
                function_statements.push(statement);
            }
        }

        if !context.at_end() {
            context.next_token(); // Skip 'endFunction'
        }

        Statement::FunctionInstantiation(FunctionInstantiationStatement {
            header: function_header,
            statements: function_statements,
            line_number: context.get_curr_token().line_number,
        })
    }
}

impl FunctionDeclarationStatementParser {
    fn skip_function_signature(&self, context: &mut ParserContext<'_>) {
        context.next_token(); // Skip function name

        // Skip '(parameters)'
        if context.is_curr_token_type(&TokenType::LeftParen) {
            let mut paren_depth = 1;
            context.next_token();
            while paren_depth > 0 && !context.at_end() {
                match context.get_curr_token().token_type {
                    TokenType::LeftParen => paren_depth += 1,
                    TokenType::RightParen => paren_depth -= 1,
                    _ => {}
                }
                context.next_token();
            }
        }

        // Skip return type if present (-> Type)
        if context.is_curr_token_type(&TokenType::Arrow) {
            context.next_token(); // Skip '->'
            context.next_token(); // Skip type
        }

        // Skip ':'
        if context.is_curr_token_type(&TokenType::Colon) {
            context.next_token();
        }
    }
}

/// Parser for newline statements
pub struct NewlineStatementParser;

impl StatementParser for NewlineStatementParser {
    fn can_parse(&self, token_type: &TokenType) -> bool {
        matches!(token_type, TokenType::Newline)
    }

    fn parse(&mut self, _context: &mut ParserContext<'_>) -> Statement {
        Statement::Newline
    }
}

// Helper functions

fn parse_single_statement(context: &mut ParserContext<'_>) -> Statement {
    let mut coordinator = super::StatementParserCoordinator::new();
    coordinator.parse_statement(context)
}

/// Helper function extracted from the original AST builder
fn extract_value_and_type_from_primary(
    primary: &Primary,
    symbol_table: &crate::symbol_table::SymbolTable,
) -> (String, VarType) {
    match primary {
        Primary::Number { value } => (value.clone(), VarType::Num),
        Primary::String { value } => (value.clone(), VarType::Str),
        Primary::Identity {
            name,
            line_number: _,
        } => {
            if let Some(var_info) = symbol_table.lookup_variable(name) {
                (name.clone(), var_info.var_type.clone())
            } else {
                (name.clone(), VarType::Unrecognized)
            }
        }
        Primary::FunctionCall {
            name,
            arguments,
            line_number: _,
        } => {
            if let Some(function_info) = symbol_table.lookup_function(name) {
                let mut call_text = name.clone();
                call_text.push_str("(");
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        call_text.push_str(", ");
                    }
                    call_text.push_str(arg);
                }
                call_text.push_str(")");
                (call_text, function_info.return_type.clone())
            } else {
                (name.clone(), VarType::Unrecognized)
            }
        }
        Primary::Error { detail: _ } => ("/* error */".to_string(), VarType::Unrecognized),
    }
}
