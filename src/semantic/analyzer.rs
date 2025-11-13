use crate::ast::{
    DataType, FunctionDeclarationStatement, IfStatement, PrintStatement, RawFunctionCallStatement,
    Statement, VariableAssignmentStatement, VariableDeclarationStatement, WhileStatement,
};
use crate::ast::{FunctionTable, ReturnStatement};

use crate::semantic::type_resolution::{resolve_logical_values, resolve_value};

use crate::semantic::validate::validate_logical;

use crate::symbol_table::SymbolTable;

use crate::semantic::SemanticError;

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
    let mut line_number = 0;

    resolve_logical_values(
        &mut var_ass.assigned_logical,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    // TODO: put this shit into its own function, its rlly large rn
    let var_op = state
        .context_stack
        .last()
        .unwrap()
        .symbol_table
        .get(&var_ass.var_name);

    match var_op {
        Some(var_def) => {
            // type check logical type with var being declared
            line_number = var_def.line_declared_on;
            let declared_var_type = &var_def.data_type;
            let assigned_logical_type = &var_ass.assigned_logical.data_type;
            if assigned_logical_type != declared_var_type {
                state.errors.push(SemanticError::TypeMismatch {
                    expected: declared_var_type.clone(),
                    found: assigned_logical_type.clone(),
                    line: var_def.line_declared_on,
                });
            }
        }
        None => {
            //println!("NONE found for {}", var_ass.var_name);
            state.errors.push(SemanticError::VariableNotDeclared {
                name: var_ass.var_name.clone(),
                line: var_ass.line_number,
            });
        }
    }

    let logical_err = validate_logical(&var_ass.assigned_logical, line_number);
    if logical_err.len() > 0 {
        // if there's a problem with the logical being assigned to the var,
        // we can't add it to our map.
        // Also prevents duplicate errors for the same statement.
        // Return early.
        state.errors.extend(logical_err);
        return state;
    }

    state
}

fn analyze_variable_declaration(
    var_decl: &mut VariableDeclarationStatement,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    let mut state = state;

    resolve_logical_values(
        &mut var_decl.assigned_logical,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    let logical_err = validate_logical(&var_decl.assigned_logical, var_decl.line_declared_on);
    println!("After validate_logical: {:#?}", logical_err);
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
    state = push_scope_for_function(&func_decl.function_name, state, function_table);

    // Only check return type requirement.
    // Type checking of the return is done in analyze_return function
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

    resolve_logical_values(
        &mut print_stmt.logical,
        function_table,
        current_symbol_table,
    );

    if print_stmt.logical.data_type == DataType::Invalid {
        // TODO: improve this error message
        state.errors.push(SemanticError::ExpressionInvalid {
            line: print_stmt.line,
        })
    }

    state
}

fn analyze_return_stmt(
    return_stmt: &mut ReturnStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    // TODO: check if return type is void or something
    resolve_logical_values(
        &mut return_stmt.return_value,
        function_table,
        &state.context_stack.last().unwrap().symbol_table, // TODO: make a helper function for this LOL
    );

    state = ensure_return_type_matches_function(state, function_table, return_stmt);

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
    resolve_logical_values(&mut stmt.condition, function_table, symbol_table);

    let logical_err = validate_logical(&stmt.condition, stmt.line_declared_on);
    if logical_err.len() > 0 {
        // if there's a problem with the logical being assigned to the var,
        // we can't add it to our map.
        // Also prevents duplicate errors for the same statement.
        // Return early.
        state.errors.extend(logical_err);
        return state;
    }

    // TODO: need to check all the statements inside of the if stmt, right?
    // TODO: also don't we need to alter the scope stack to account for the new
    // block of code??

    push_scope_for_new_block(&mut state);
    for statement in stmt.if_body.iter_mut() {
        state = analyze_statement(statement, state, function_table);
    }
    state = pop_scope(state);

    if let Some(else_statement_vec) = stmt.else_body.as_mut() {
        push_scope_for_new_block(&mut state);
        for statement in else_statement_vec {
            state = analyze_statement(statement, state, function_table);
        }
        state = pop_scope(state);
    }

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
    resolve_logical_values(&mut stmt.condition, function_table, symbol_table);

    let logical_err = validate_logical(&stmt.condition, stmt.line_declared_on);
    if logical_err.len() > 0 {
        // if there's a problem with the logical being assigned to the var,
        // we can't add it to our map.
        // Also prevents duplicate errors for the same statement.
        // Return early.
        state.errors.extend(logical_err);
        return state;
    }

    push_scope_for_new_block(&mut state);
    for statement in stmt.body.iter_mut() {
        state = analyze_statement(statement, state, function_table);
    }
    state = pop_scope(state);

    state
}

fn analyze_raw_func_call(
    stmt: &mut RawFunctionCallStatement,
    mut state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
    //state = resolve_value(&mut stmt.value, state, function_table);
    resolve_value(
        &mut stmt.value,
        function_table,
        &state.context_stack.last().unwrap().symbol_table,
    );

    //state = analyze_function_call(&stmt.value, stmt.line, state, function_table);
    match function_table.get_func_def_using_str(&stmt.value.raw_text) {
        None => {
            // This function doesn't exist. Add error.
            state.errors.push(SemanticError::FunctionNotDeclared {
                name: stmt.value.raw_text.clone(),
                line: stmt.line,
            })
        }
        Some(function_definition) => {
            // the types of all the parameters should be resolved by now.
            // Loop through definition, and confirm that it matches the call.
            for (expected_param_idx, expected_param) in
                function_definition.parameters.iter().enumerate()
            {
                if let Some(function_call_param) = stmt.value.params.get(expected_param_idx) {
                    if function_call_param.data_type != expected_param.data_type {
                        state.errors.push(SemanticError::UnexpectedStatement {
                            line: stmt.line,
                            explanation: format!(
                                "Provided arguments to function {} are invalid.",
                                stmt.value
                            ),
                        })
                    }
                } else {
                    // the parameter wasn't found, also add error.
                    state.errors.push(SemanticError::IncorrectParameters {
                        parameters_expected: function_definition.parameters.len(),
                        parameters_provided: stmt.value.params.len(),
                        line: stmt.line,
                    });
                }
            }
        }
    }

    state
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
        let return_stmt_value_type = &return_stmt.return_value.data_type;
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

/// TODO; move into module specific to analysis state functions
fn push_scope_for_function(
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

/// aka if/while stmts
fn push_scope_for_new_block(state: &mut AnalysisState) {
    // let mut new_symbol_table = state.context_stack.last().unwrap().symbol_table.clone();
    let mut new_symbol_table = SymbolTable::new();

    // lazy clone
    for (key, value) in state.context_stack.last().unwrap().symbol_table.iter() {
        new_symbol_table.insert(&value.identifier, &value.data_type, &value.line_declared_on);
    }

    let new_context = SemanticContext {
        symbol_table: new_symbol_table,
        scope: None, // will this break anything?
    };
    state.context_stack.push(new_context);
}

/// TODO; move into module specific to analysis state functions
fn pop_scope(state: AnalysisState) -> AnalysisState {
    let mut state = state;
    if state.context_stack.len() > 1 {
        state.context_stack.pop();
    }
    state
}

/// TODO; move into module specific to analysis state functions
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
