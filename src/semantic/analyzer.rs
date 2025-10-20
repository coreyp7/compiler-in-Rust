use crate::ast::FunctionTable;
use crate::ast::{
    DataType, FunctionDeclarationStatement, Statement, Value, ValueType,
    VariableDeclarationStatement,
};
use crate::semantic::SemanticError;
use crate::semantic::resolve_value_type::resolve_variable_declaration;
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
        Statement::FunctionDeclaration(func_decl) => {
            state = analyze_function_declaration(func_decl, state, function_table);
        }
        Statement::Return(return_stmt) => {
            if let Some(ref mut return_value) = return_stmt.return_value {
                state = resolve_value_type(return_value, state, function_table);
                state = validate_value(
                    return_value,
                    return_stmt.line_declared_on,
                    state,
                    function_table,
                );
            }
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
    resolve_variable_declaration(
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

    // Now do type checking with resolved types
    // Check that the value assigned matches the declared type of the var.
    if var_decl.data_type != var_decl.assigned_value.data_type {
        if !matches!(var_decl.assigned_value.data_type, DataType::Invalid) {
            state.errors.push(SemanticError::TypeMismatch {
                expected: var_decl.data_type.clone(),
                found: var_decl.assigned_value.data_type.clone(),
                line: var_decl.line_declared_on,
            });
        }
    }

    // Validate the assigned value
    state = validate_value(
        &var_decl.assigned_value,
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

            // Validate each parameter recursively and check types
            for (i, param_value) in param_values.iter().enumerate() {
                state = validate_value(param_value, line, state, function_table);

                // Type check parameter if it exists in function definition
                if let Some(expected_param) = func_def.parameters.get(i) {
                    if param_value.data_type != expected_param.data_type {
                        state.errors.push(SemanticError::TypeMismatch {
                            expected: expected_param.data_type.clone(),
                            found: param_value.data_type.clone(),
                            line,
                        });
                    }
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
            // First resolve parameter types if they exist
            if let Some(ref mut param_values) = value.param_values {
                for param_value in param_values {
                    state = resolve_value_type(param_value, state, function_table);
                }
            }

            // Then resolve the function's return type
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
