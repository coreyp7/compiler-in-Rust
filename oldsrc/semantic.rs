use std::collections::HashMap;

// Import types from other modules
use crate::comparison::{Comparison, Expression, Logical, Primary, Term, Unary};
use crate::error::ErrMsg;
use crate::statement::{
    AssignmentStatement, FunctionCallStatement, FunctionInstantiationStatement, IfStatement,
    PrintStatement, Statement, VarInstantiationStatement, WhileStatement,
};

// Types that will remain in ast.rs but need to be imported
use crate::ast::{FunctionHeader, Var, VarType};

#[derive(Debug, Clone)]
pub struct ScopeContext {
    pub variables: HashMap<String, Var>,
    pub scope_type: ScopeType,
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Global,
    Function(String), // Function name
    Block,            // For future if/while block scoping
}

pub struct SemanticAnalyzer {
    pub function_map: HashMap<String, FunctionHeader>,
    pub scope_stack: Vec<ScopeContext>,
    pub errors: Vec<ErrMsg>,
}

impl SemanticAnalyzer {
    pub fn new(
        var_map: HashMap<String, Var>,
        function_map: HashMap<String, FunctionHeader>,
    ) -> Self {
        let mut analyzer = SemanticAnalyzer {
            function_map,
            scope_stack: Vec::new(),
            errors: Vec::new(),
        };
        // Setup scope to be global and include the variables passed in.
        analyzer.push_scope(ScopeType::Global);
        analyzer.scope_stack[0].variables = var_map;

        analyzer
    }

    pub fn analyze_ast_vec(&mut self, ast_vec: &[Statement]) {
        self.errors.clear();
        for statement in ast_vec {
            if let Err(err) = self.analyze_statement(statement) {
                self.errors.push(err);
            }
        }
    }

    pub fn push_scope(&mut self, scope_type: ScopeType) {
        self.scope_stack.push(ScopeContext {
            variables: HashMap::new(),
            scope_type,
        });
    }

    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn lookup_variable(&self, name: &str) -> Option<&Var> {
        // Search from innermost scope outward
        for scope in self.scope_stack.iter().rev() {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub fn declare_variable(&mut self, name: String, var: Var) -> Result<(), ErrMsg> {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            if let Some(existing_var) = current_scope.variables.get(&name) {
                return Err(ErrMsg::VariableAlreadyDeclared {
                    identity: name,
                    first_declared_line: existing_var.line_declared_on,
                    redeclared_line: var.line_declared_on,
                });
            }
            current_scope.variables.insert(name, var);
            Ok(())
        } else {
            // For now, return a generic error - we might need to add this to ErrMsg enum
            Err(ErrMsg::VariableAlreadyDeclared {
                identity: "No active scope".to_string(),
                first_declared_line: 0,
                redeclared_line: var.line_declared_on,
            })
        }
    }

    pub fn validate_function_call(
        &mut self,
        call: &FunctionCallStatement,
    ) -> Result<VarType, ErrMsg> {
        // 1. Check if function exists
        let function_info = self.function_map.get(&call.function_name).ok_or_else(|| {
            ErrMsg::VariableNotDeclared {
                identity: call.function_name.clone(),
                attempted_assignment_line: call.line_number,
            }
        })?;

        // 2. Check argument count
        if call.arguments.len() != function_info.parameters.len() {
            // For now, use existing error types - we can enhance ErrMsg later
            return Err(ErrMsg::VariableNotDeclared {
                identity: format!(
                    "Function {} expects {} arguments, got {}",
                    call.function_name,
                    function_info.parameters.len(),
                    call.arguments.len()
                ),
                attempted_assignment_line: call.line_number,
            });
        }

        // 3. Check each argument: existence and type compatibility
        for (i, arg_name) in call.arguments.iter().enumerate() {
            // Check if the argument variable exists
            let arg_var =
                self.lookup_variable(arg_name)
                    .ok_or_else(|| ErrMsg::VariableNotDeclared {
                        identity: arg_name.clone(),
                        attempted_assignment_line: call.line_number,
                    })?;

            // Check if we have a corresponding parameter
            if i < function_info.parameters.len() {
                let expected_param = &function_info.parameters[i];

                // Type check: ensure argument type matches parameter type
                if arg_var.var_type != expected_param.param_type {
                    return Err(ErrMsg::new_incorrect_type_assignment(
                        expected_param.param_type.clone(),
                        arg_var.var_type.clone(),
                        call.line_number,
                    ));
                }
            }
        }

        // Return the function's return type for expression type checking
        Ok(function_info.return_type.clone())
    }

    pub fn analyze_statement(&mut self, statement: &Statement) -> Result<(), ErrMsg> {
        match statement {
            Statement::FunctionCall(call) => {
                self.validate_function_call(call)?;
            }
            Statement::VariableDeclarationStatement(var_inst) => {
                // Type checking - ensure the declared type matches the assigned value type
                if var_inst.var_type != var_inst.assigned_value_type {
                    return Err(ErrMsg::IncorrectTypeAssignment {
                        expected_type: var_inst.var_type.clone(),
                        got_type: var_inst.assigned_value_type.clone(),
                        line_number: var_inst.line_number,
                    });
                }
            }
            Statement::Assignment(assignment) => {
                // Check if the variable being assigned to exists in scope
                let target_var = self.lookup_variable(&assignment.identity).ok_or_else(|| {
                    ErrMsg::VariableNotDeclared {
                        identity: assignment.identity.clone(),
                        attempted_assignment_line: assignment.line_number,
                    }
                })?;

                // TODO: If the assignment value is a variable reference, validate it exists
                // TODO: Add type checking - ensure assigned value type matches variable type
                // For now, we'd need to parse the assignment.value to determine if it's a variable reference

                // Basic type checking
                if target_var.var_type != assignment.assigned_value_type {
                    return Err(ErrMsg::IncorrectTypeAssignment {
                        expected_type: target_var.var_type.clone(),
                        got_type: assignment.assigned_value_type.clone(),
                        line_number: assignment.line_number,
                    });
                }
            }
            Statement::If(if_stmt) => {
                // Analyze the conditional expression
                self.analyze_logical(&if_stmt.logical)?;

                // Analyze statements in if block
                for stmt in &if_stmt.statements {
                    self.analyze_statement(stmt)?;
                }
            }
            Statement::While(while_stmt) => {
                // Analyze the conditional expression
                self.analyze_logical(&while_stmt.logical)?;

                // Analyze statements in while block
                for stmt in &while_stmt.statements {
                    self.analyze_statement(stmt)?;
                }
            }
            Statement::FunctionInstantiation(func) => {
                self.analyze_function(func)?;
            }
            Statement::Print(print_stmt) => {
                // If the print statement references a variable, validate it's in scope
                if print_stmt.is_content_identity_name {
                    self.lookup_variable(&print_stmt.content).ok_or_else(|| {
                        ErrMsg::VariableNotDeclared {
                            identity: print_stmt.content.clone(),
                            attempted_assignment_line: print_stmt.line_number,
                        }
                    })?;
                }
            }
            Statement::Newline | Statement::TestStub | Statement::Return(_) => {
                // These don't need semantic analysis for now
            }
        }
        Ok(())
    }

    pub fn analyze_function(
        &mut self,
        func: &FunctionInstantiationStatement,
    ) -> Result<(), ErrMsg> {
        // Push function scope
        self.push_scope(ScopeType::Function(func.header.function_name.clone()));

        // Add parameters to function scope (for now, empty since we don't parse them yet)
        // TODO: When we implement parameter parsing, add them here
        for parameter in &func.header.parameters {
            // TODO: this is bootlegged
            let _ = self.declare_variable(
                parameter.name.clone(),
                Var {
                    var_type: parameter.param_type.clone(),
                    identity: parameter.name.clone(),
                    line_declared_on: func.line_number,
                },
            );
        }

        // Analyze function body
        for statement in &func.statements {
            if let Err(err) = self.analyze_statement(statement) {
                self.errors.push(err);
            }
        }

        // Pop function scope
        self.pop_scope();
        Ok(())
    }

    pub fn analyze_logical(&mut self, logical: &Logical) -> Result<(), ErrMsg> {
        // Analyze all comparisons in the logical expression
        for comparison in &logical.comparisons {
            self.analyze_comparison(comparison)?;
        }
        Ok(())
    }

    pub fn analyze_comparison(&mut self, comparison: &Comparison) -> Result<(), ErrMsg> {
        // Analyze all expressions in the comparison
        for expression in &comparison.expressions {
            self.analyze_expression(expression)?;
        }
        Ok(())
    }

    pub fn analyze_expression(&mut self, expression: &Expression) -> Result<(), ErrMsg> {
        // Analyze all terms in the expression
        for term in &expression.terms {
            self.analyze_term(term)?;
        }
        Ok(())
    }

    pub fn analyze_term(&mut self, term: &Term) -> Result<(), ErrMsg> {
        // Analyze all unary expressions in the term
        for unary in &term.unarys {
            self.analyze_unary(unary)?;
        }
        Ok(())
    }

    pub fn analyze_unary(&mut self, unary: &Unary) -> Result<(), ErrMsg> {
        // Analyze the primary expression
        self.analyze_primary(&unary.primary)
    }

    pub fn analyze_primary(&mut self, primary: &Primary) -> Result<(), ErrMsg> {
        match primary {
            Primary::Identity { name, line_number } => {
                // Check if the variable is declared in scope
                self.lookup_variable(name)
                    .ok_or_else(|| ErrMsg::VariableNotDeclared {
                        identity: name.clone(),
                        attempted_assignment_line: *line_number,
                    })?;
            }
            Primary::Number { .. } => {
                // Numbers don't need scope validation
            }
            Primary::String { .. } => {
                // Strings have no validation
            }
            Primary::Error { .. } => {
                // Error primaries indicate parsing issues, skip validation
            }
            Primary::FunctionCall {
                name,
                arguments,
                line_number,
            } => {
                // TODO: need to do some type of checking and validation
                // in here when a primary is a function call.
            }
        }
        Ok(())
    }
}
