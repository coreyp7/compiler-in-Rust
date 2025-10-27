use crate::ast::FunctionTable;
use crate::ast::{
    DataType, FunctionDeclarationStatement, PrintStatement, Statement, Value, ValueType,
    VariableAssignmentStatement, VariableDeclarationStatement,
};
use crate::ast::{Expression, Term, Unary};
use crate::semantic::SemanticError;
use crate::semantic::resolve_value_type::resolve_expression_values;
use crate::semantic::resolve_value_type::resolve_variable_assignment_stmt_types;
use crate::semantic::resolve_value_type::resolve_variable_declaration_types;
use crate::symbol_table::SymbolTable;

/// Semantic analyzer context for managing scope
pub struct SemanticContext {
    pub symbol_table: SymbolTable,
    pub scope: Option<u8>, // Function scope if in function, None if global
}

/// Analysis state passed through functions
pub struct AnalysisState {
    pub context_stack: Vec<SemanticContext>,
    pub errors: Vec<SemanticError>,
}

impl AnalysisState {
    pub fn new() -> Self {
        let mut context_stack = Vec::new();
        context_stack.push(SemanticContext {
            symbol_table: SymbolTable::new(),
            scope: None,
        });

        Self {
            context_stack,
            errors: Vec::new(),
        }
    }
}

// Main analysis entry point
pub fn analyze_statements(
    statements: &mut [Statement],
    function_table: &FunctionTable,
) -> Vec<SemanticError> {
    let mut state = AnalysisState::new();

    for statement in statements {
        state = analyze_statement(statement, state, function_table);
    }

    state.errors
}

/// Analyze a single statement
fn analyze_statement(
    statement: &mut Statement,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;
    match statement {
        Statement::VariableDeclaration(var_decl) => {
            state = analyze_variable_declaration(var_decl, state, function_table);
        }
        Statement::VariableAssignment(var_ass) => {
            state = analyze_variable_assignment(var_ass, state, function_table);
        }
        Statement::FunctionDeclaration(func_decl) => {
            state = analyze_function_declaration(func_decl, state, function_table);
        }
        Statement::Return(return_stmt) => {
            if let Some(ref mut return_value) = return_stmt.return_value {
                state = resolve_value_type(return_value, state, function_table);
                // TODO: if incorrect return statement not in function, this will crash.
                // NEED TO HANDLE IF THIS IS THE CASE.
                /*
                let curr_func_id = state.context_stack.last().unwrap().scope.unwrap();
                let curr_func_def = function_table.get_using_id(curr_func_id).unwrap();
                if return_value.data_type != curr_func_def.return_type {
                    state.errors.push(SemanticError::ReturnTypeIncorrect {
                        func_def: curr_func_def.clone(),
                        line: return_stmt.line_declared_on,
                    })
                }
                */

                let current_analysis_context = state.context_stack.last().unwrap();
                let current_function_context = current_analysis_context.scope.unwrap();
                let current_function = function_table.get_using_id(current_function_context);
                if let Some(current_function) = current_function {
                    if current_function.return_type != return_value.data_type {
                        // If return type doesn't match the function return type, create an error.
                        // TODO: need to change return statements to have expressions.
                        state.errors.push(SemanticError::ReturnTypeIncorrect {
                            func_def: current_function.clone(),
                            line: return_stmt.line_declared_on,
                        })
                    }
                } else {
                    state.errors.push(SemanticError::UnexpectedStatement {
                        line: return_stmt.line_declared_on,
                        explanation: "Return statement found outside of function".to_string(),
                    })
                }
            }
        }
        Statement::Print(print_stmt) => {
            state = analyze_print_statement(print_stmt, state, function_table);
        }
        _ => (),
    }
    state
}

fn analyze_variable_assignment(
    var_ass: &mut VariableAssignmentStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // What things do we need to validate here?
    // - type check
    // I can't think of anything else rn so just do these

    resolve_variable_assignment_stmt_types(
        var_ass,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    println!("Updated statement in ast:");
    println!("{:#?}", var_ass);

    // Check the variable being assigned to exists in this scope
    // TODO: helper functions for this shit needs to be made lol
    let var_op = state
        .context_stack
        .last()
        .unwrap()
        .symbol_table
        .get(&var_ass.var_name);

    match var_op {
        Some(var_def) => {
            println!("some var def found for {}", var_ass.var_name);
            // Type check the expression with its assignment
            let expected_type = var_def.data_type.clone();
            state = check_expression_types(
                &var_ass.assigned_expr,
                &expected_type,
                var_ass.line_number,
                state,
                function_table,
            );
        }
        None => {
            println!("NONE found for {}", var_ass.var_name);
            state.errors.push(SemanticError::VariableNotDeclared {
                name: var_ass.var_name.clone(),
                line: var_ass.line_number,
            });
        }
    }

    state
}

/// Analyze variable declaration for semantic correctness
fn analyze_variable_declaration(
    var_decl: &mut VariableDeclarationStatement,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // Not done in AST, so we need to do it here.
    //state = resolve_value_type(&mut var_decl.assigned_value, state, function_table);
    resolve_variable_declaration_types(
        var_decl,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    println!("Updated statement in ast:");
    println!("{:#?}", var_decl);

    // First, try to add the variable to the current scope
    if let Err(error) = add_variable_to_current_scope(
        &var_decl.symbol_name,
        &var_decl.data_type,
        var_decl.line_declared_on,
        &mut state,
    ) {
        state.errors.push(error);
    }

    // Now do comprehensive type checking of the expression
    // Check that the expression assigned matches the declared type of the variable
    state = check_expression_types(
        &var_decl.assigned_expr,
        &var_decl.data_type,
        var_decl.line_declared_on,
        state,
        function_table,
    );

    state
}

/// Analyze function declaration
fn analyze_function_declaration(
    func_decl: &mut FunctionDeclarationStatement,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;
    state = push_scope(&func_decl.function_name, state, function_table);

    // Check return statement requirement
    if let Some(last_statement_in_body) = func_decl.body.last() {
        let does_return_exist = matches!(last_statement_in_body, Statement::Return(_));
        let is_function_return_type_void = func_decl.return_type == DataType::Void;

        if !is_function_return_type_void && !does_return_exist {
            state.errors.push(SemanticError::ReturnMissing {
                funct_name: func_decl.function_name.clone(),
                func_declared_on_line: func_decl.line_declared_on,
            });
        }
    }

    // Analyze each statement in function body
    for statement in &mut func_decl.body {
        state = analyze_statement(statement, state, function_table);
    }

    state = pop_scope(state);
    state
}

/// Analyze print statement
fn analyze_print_statement(
    print_stmt: &mut PrintStatement,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let current_symbol_table = &state.context_stack.last().unwrap().symbol_table;

    resolve_expression_values(
        &mut print_stmt.expression,
        function_table,
        current_symbol_table,
    );

    state
}

// Validate that a value reference is semantically correct
fn validate_value(
    value: &Value,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // this is being done in a previous phase now
    //state = resolve_value_type(value, state, function_table);

    let mut state = state;
    let current_context = state.context_stack.last().unwrap();

    match value.value_type {
        ValueType::Variable => {
            if !current_context.symbol_table.contains_name(&value.raw_text) {
                state.errors.push(SemanticError::VariableNotDeclared {
                    name: value.raw_text.clone(),
                    line,
                });
            }
        }
        ValueType::FunctionCall => {
            if function_table
                .get_id_with_function_name(&value.raw_text)
                .is_none()
            {
                state.errors.push(SemanticError::FunctionNotDeclared {
                    name: value.raw_text.clone(),
                    line,
                });
            } else {
                state = analyze_function_call(value, line, state, function_table);
            }
        }
        ValueType::InlineNumber | ValueType::InlineString => {
            // Inline values don't need validation
        }
        ValueType::Expression => {
            // Expression validation would be more complex
        }
        ValueType::Invalid => {
            state.errors.push(SemanticError::InvalidValueReference {
                name: value.raw_text.clone(),
                line,
            });
        }
    }

    state
}

/// Analyze function call parameters and validate them
fn analyze_function_call(
    value: &Value,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // Validate function call parameters
    if let Some(func_def) = function_table.get_func_def_using_str(&value.raw_text) {
        if let Some(ref param_values) = value.param_values {
            // Check parameter count
            if param_values.len() != func_def.parameters.len() {
                state.errors.push(SemanticError::IncorrectParameters {
                    parameters_expected: func_def.parameters.len(),
                    parameters_provided: param_values.len(),
                    line,
                });
            }

            // Validate each parameter expression and check types
            for (i, param_expr) in param_values.iter().enumerate() {
                // Type check the entire expression against the expected parameter type
                if let Some(expected_param) = func_def.parameters.get(i) {
                    state = check_expression_types(
                        param_expr,
                        &expected_param.data_type,
                        line,
                        state,
                        function_table,
                    );
                }
            }
        } else if !func_def.parameters.is_empty() {
            // Function expects parameters but none provided
            state.errors.push(SemanticError::IncorrectParameters {
                parameters_expected: func_def.parameters.len(),
                parameters_provided: 0,
                line,
            });
        }
    }

    state
}

// The datatype of values (variables/function calls) is incomplete, since the AST
// shouldn't be having to worry about type enforcement.
// This function is used when we need to update the value struct with the datatype
// of whatever is being called.
fn resolve_value_type(
    value: &mut Value,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    if value.data_type != DataType::Unknown {
        return state; // Already resolved
    }

    let mut state = state;
    let current_context = state.context_stack.last().unwrap();

    match value.value_type {
        ValueType::Variable => {
            if let Some(symbol) = current_context.symbol_table.get(&value.raw_text) {
                value.data_type = symbol.data_type.clone();
            }
        }
        ValueType::FunctionCall => {
            // For function calls, the parameter expressions will be type-checked
            // separately by check_expression_types in analyze_function_call

            // Resolve the function's return type
            if let Some(func_def) = function_table.get_func_def_using_str(&value.raw_text) {
                value.data_type = func_def.return_type.clone();
            }
        }
        _ => {} // Other types should already have correct data_type
    }

    state
}

/// Push a new scope for function analysis
fn push_scope(
    function_name: &str,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;
    if let Some(function_id) = function_table.get_id_with_function_name(function_name) {
        if let Some(function_def) =
            function_table.get_func_def_using_str(&function_name.to_string())
        {
            let mut new_symbol_table = SymbolTable::new();

            // Add function parameters to the new scope
            for parameter in &function_def.parameters {
                new_symbol_table.insert(
                    &parameter.name,
                    &parameter.data_type,
                    &function_def.line_declared_on,
                );
            }

            let new_context = SemanticContext {
                symbol_table: new_symbol_table,
                scope: Some(function_id),
            };

            state.context_stack.push(new_context);
        }
    }
    state
}

/// Pop the current scope
fn pop_scope(state: AnalysisState) -> AnalysisState {
    let mut state = state;
    if state.context_stack.len() > 1 {
        state.context_stack.pop();
    }
    state
}

/// Add a variable to the current scope
fn add_variable_to_current_scope(
    name: &str,
    data_type: &DataType,
    line: u32,
    state: &mut AnalysisState,
) -> Result<u8, SemanticError> {
    let current_context = state.context_stack.last_mut().unwrap();

    if current_context.symbol_table.contains_name(name) {
        if let Some(existing_var) = current_context.symbol_table.get(name) {
            return Err(SemanticError::VariableAlreadyDeclared {
                name: name.to_string(),
                first_line: existing_var.line_declared_on,
                redeclaration_line: line,
            });
        }
    }

    let name_string = name.to_string();
    if let Some(key) = current_context
        .symbol_table
        .insert(&name_string, data_type, &line)
    {
        Ok(key)
    } else {
        Err(SemanticError::VariableAlreadyDeclared {
            name: name.to_string(),
            first_line: 0,
            redeclaration_line: line,
        })
    }
}

/// Comprehensive type checking for expressions
/// This function navigates the entire expression grammar hierarchy to validate all types
///
/// The expression hierarchy in the grammar is:
/// Expression -> Term -> Unary -> Value
///
/// This function:
/// 1. Checks each term in the expression
/// 2. For each term, checks each unary expression
/// 3. For each unary, validates the primary value type
/// 4. Reports type mismatches when values don't match expected types
/// 5. Validates that variables and functions exist and are accessible
///
/// Similar to the original commented code at line 141, but handles the full expression grammar
fn check_expression_types(
    expr: &Expression,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // For now, we'll check each term in the expression
    // In a more complete implementation, we'd need to validate that
    // all terms are compatible with the operations being performed
    for term in &expr.terms {
        state = check_term_types(term, expected_type, line, state, function_table);
    }

    // TODO: Check that the operators make sense with the data types
    // For example, you can't add strings and numbers in most type systems

    state
}

/// Check types for a term (multiplication/division level)
fn check_term_types(
    term: &Term,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    for unary in &term.unarys {
        state = check_unary_types(unary, expected_type, line, state, function_table);
    }

    state
}

/// Check types for a unary expression (with optional unary operator)
fn check_unary_types(
    unary: &Unary,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // Check the primary value
    state = check_value_type(&unary.primary, expected_type, line, state, function_table);

    state
}

/// Check that a value matches the expected type
/// This is the core type checking function that validates individual values
fn check_value_type(
    value: &Value,
    expected_type: &DataType,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // First validate the value itself (check if variables/functions exist)
    state = validate_value(value, line, state, function_table);

    // Then check type compatibility - this is similar to the original code at line 141
    // but now works with the resolved type system
    if value.data_type != *expected_type
        && !matches!(value.data_type, DataType::Unknown | DataType::Invalid)
    {
        state.errors.push(SemanticError::TypeMismatch {
            expected: expected_type.clone(),
            found: value.data_type.clone(),
            line,
        });
    }

    state
}
