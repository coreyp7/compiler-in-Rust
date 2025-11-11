use std::ops::DerefMut;

use crate::ast::{
    DataType, FunctionDeclarationStatement, IfStatement, PrintStatement, RawFunctionCallStatement,
    Statement, Value, ValueType, VariableAssignmentStatement, VariableDeclarationStatement,
    WhileStatement,
};
use crate::ast::{Expression, Logical, Term, Unary};
use crate::ast::{FunctionTable, ReturnStatement};
use crate::semantic::SemanticError;
use crate::semantic::resolve_value_type::resolve_variable_assignment_stmt_types;
use crate::semantic::resolve_value_type::resolve_variable_declaration_types;
use crate::semantic::resolve_value_type::{resolve_expression_values, resolve_logical_values};
use crate::semantic::type_check::add_type_check_errors_for_logical;
use crate::semantic::type_check::type_check_expression;
use crate::semantic::validate::validate_logical;
use crate::symbol_table::{self, SymbolTable};

pub struct SemanticContext {
    pub symbol_table: SymbolTable,
    pub scope: Option<u8>, // Function scope if in function, None if global
}

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

// Start of analysis phase
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
            state = analyze_return_stmt(return_stmt, state, function_table);
        }
        Statement::Print(print_stmt) => {
            state = analyze_print_statement(print_stmt, state, function_table);
        }
        Statement::If(if_stmt) => {
            state = analyze_if_stmt(if_stmt, state, function_table);
        }
        Statement::While(while_stmt) => {
            state = analyze_while_stmt(while_stmt, state, function_table);
        }
        Statement::RawFunctionCall(func_stmt) => {
            state = analyze_raw_func_call(func_stmt, state, function_table);
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

    /*
    resolve_variable_assignment_stmt_types(
        var_ass,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );
    */
    resolve_logical_values(
        &mut var_ass.assigned_logical,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    //println!("Updated statement in ast:");
    //println!("{:#?}", var_ass);

    /* BOOL REFACTOR
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
            //println!("some var def found for {}", var_ass.var_name);
            // Type check the expression with its assignment
            let expected_type = var_def.data_type.clone();
            state = type_check_expression(
                &var_ass.assigned_expr,
                &expected_type,
                var_ass.line_number,
                state,
                function_table,
            );
        }
        None => {
            //println!("NONE found for {}", var_ass.var_name);
            state.errors.push(SemanticError::VariableNotDeclared {
                name: var_ass.var_name.clone(),
                line: var_ass.line_number,
            });
        }
    }
    */

    state
}

fn analyze_variable_declaration(
    var_decl: &mut VariableDeclarationStatement,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    // Not done in AST, so we need to do it here.
    /*
    resolve_variable_declaration_types(
        var_decl,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );
    */
    resolve_logical_values(
        &mut var_decl.assigned_logical,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    let logical_err = validate_logical(&var_decl.assigned_logical, var_decl.line_declared_on);
    if logical_err.len() > 0 {
        // if there's a problem with the logical being assigned to the var,
        // we can't add it to our map.
        // Also prevents duplicate errors for the same statement.
        // Return early.
        state.errors.extend(logical_err);
        return state;
    }

    // type check logical type with var being declared
    let declared_var_type = &var_decl.data_type;
    let assigned_logical_type = &var_decl.assigned_logical.data_type;
    if assigned_logical_type != declared_var_type {
        state.errors.push(SemanticError::TypeMismatch {
            expected: declared_var_type.clone(),
            found: assigned_logical_type.clone(),
            line: var_decl.line_declared_on,
        });
    }

    if let Err(error) = add_variable_to_current_scope(
        &var_decl.symbol_name,
        &var_decl.data_type,
        var_decl.line_declared_on,
        &mut state,
    ) {
        state.errors.push(error);
    }

    // Now do comprehensive type checking of the expression.
    // Check that the expression assigned matches the declared type of the variable.

    /* DURING BOOL REFACTOR
    state = type_check_expression(
        &var_decl.assigned_expr,
        &var_decl.data_type,
        var_decl.line_declared_on,
        state,
        function_table,
    );
    */

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
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let current_symbol_table = &state.context_stack.last().unwrap().symbol_table;

    resolve_expression_values(
        &mut print_stmt.expression,
        function_table,
        current_symbol_table,
    );

    if print_stmt.expression.data_type == DataType::Invalid {
        state.errors.push(SemanticError::ExpressionInvalid {
            line: print_stmt.line_declared_on,
        })
    }

    state
}

// This is only called in type_check rn (since value is so far down the
// expression hierarchy)
pub fn analyze_value(
    value: &Value,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // this is being done in a previous phase now
    //state = resolve_value_type(value, state, function_table);

    let mut state = state;
    let current_context = state.context_stack.last().unwrap();

    /* DURING BOOL REFACTOR
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
    */
    state
}

/// Analyze function call parameters and validate them
/// NOTE IM pretty sure you need to resolve all the values before
/// you call this
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
                    state = type_check_expression(
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

fn analyze_return_stmt(
    return_stmt: &mut ReturnStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // TODO: check if return type is void or something
    if let Some(ref mut return_expr) = return_stmt.return_value {
        resolve_expression_values(
            return_expr,
            function_table,
            &state.context_stack.last().unwrap().symbol_table, // TODO: make a helper function for this LOL
        );
        state = ensure_return_type_matches_function(state, function_table, &return_stmt);
    } else {
        state.errors.push(SemanticError::UnexpectedStatement {
            line: return_stmt.line_declared_on,
            explanation: "Return statement found outside of function".to_string(),
        })
    }

    state
}

fn analyze_if_stmt(
    stmt: &mut IfStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // need to go through the logical of the if statement, and resolve all expressions.
    // We still need to ensure that the types are legit
    let symbol_table = &state.context_stack.last().unwrap().symbol_table; // TODO: make a helper function for this LOL
    resolve_logical(&mut stmt.condition, function_table, symbol_table);

    // what needs to be checked?
    // Well, maybe the Logical should be checked to ensure that the types are correct.
    // There's a function that does this kind of thing for expressions ...
    // look for it and mimick behavior
    state = add_type_check_errors_for_logical(
        state,
        &mut stmt.condition,
        function_table,
        stmt.line_declared_on,
    );

    state
}

/// RN this is pretty identical to if stmt, but don't feel like taking the effort
/// to make generic. Its whatever
fn analyze_while_stmt(
    stmt: &mut WhileStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // need to go through the logical of the if statement, and resolve all expressions.
    // We still need to ensure that the types are legit
    let symbol_table = &state.context_stack.last().unwrap().symbol_table; // TODO: make a helper function for this LOL
    resolve_logical(&mut stmt.condition, function_table, symbol_table);

    // what needs to be checked?
    // Well, maybe the Logical should be checked to ensure that the types are correct.
    // There's a function that does this kind of thing for expressions ...
    // look for it and mimick behavior
    state = add_type_check_errors_for_logical(
        state,
        &mut stmt.condition,
        function_table,
        stmt.line_declared_on,
    );

    state
}

fn resolve_logical(
    logical: &mut Logical,
    function_table: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Resolve all expressions inside this logical.
    for comparison in logical.comparisons.iter_mut() {
        for expr in comparison.expressions.iter_mut() {
            resolve_expression_values(expr, function_table, symbol_table);
        }
    }
}

fn ensure_return_type_matches_function(
    mut state: AnalysisState,
    function_table: &FunctionTable,
    return_stmt: &ReturnStatement,
) -> AnalysisState {
    let current_analysis_context = state.context_stack.last().unwrap();
    let current_function_context = current_analysis_context.scope.unwrap();
    let current_function = function_table.get_using_id(current_function_context);

    if let Some(current_function) = current_function {
        let return_stmt_value_type = &return_stmt.return_value.as_ref().unwrap().data_type;
        if return_stmt_value_type == &DataType::Invalid {
            // This should mean that the expression cannot evaluate to a
            // single type, since they're adding different types together.
            // Could improve in future to be more grandular and specific.
            // (This is set in resolve_expression_values, which is unintuitive)
            state
                .errors
                .push(SemanticError::ExpressionInvalidExpectingSpecificType {
                    line: return_stmt.line_declared_on,
                    expected_type: current_function.return_type.clone(),
                });
        } else if &current_function.return_type != return_stmt_value_type {
            state.errors.push(SemanticError::ReturnTypeIncorrect {
                func_def: current_function.clone(),
                got_type: return_stmt_value_type.clone(),
                line: return_stmt.line_declared_on,
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

fn analyze_raw_func_call(
    stmt: &mut RawFunctionCallStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    state = resolve_value_type(&mut stmt.value, state, function_table);

    state = analyze_function_call(&stmt.value, stmt.line, state, function_table);

    state
}
