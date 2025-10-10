use crate::ast::FunctionTable;
use crate::ast::{
    DataType, FunctionDeclarationStatement, Statement, Value, ValueType,
    VariableDeclarationStatement,
};
use crate::semantic::SemanticError;
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
    statements: &[Statement],
    function_table: &FunctionTable,
) -> Vec<SemanticError> {
    let mut state = AnalysisState::new();

    for statement in statements {
        analyze_statement(statement, &mut state, function_table);
    }

    state.errors
}

/// Analyze a single statement
fn analyze_statement(
    statement: &Statement,
    state: &mut AnalysisState,
    function_table: &FunctionTable,
) {
    match statement {
        Statement::VariableDeclaration(var_decl) => {
            analyze_variable_declaration(var_decl, state, function_table);
        }
        Statement::FunctionDeclaration(func_decl) => {
            analyze_function_declaration(func_decl, state, function_table);
        }
        Statement::Return(return_stmt) => {
            if let Some(ref return_value) = return_stmt.return_value {
                if let Some(error) = validate_value(
                    return_value,
                    return_stmt.line_declared_on,
                    state,
                    function_table,
                ) {
                    state.errors.push(error);
                }
            }
        }
    }
}

/// Analyze variable declaration for semantic correctness
fn analyze_variable_declaration(
    var_decl: &VariableDeclarationStatement,
    state: &mut AnalysisState,
    function_table: &FunctionTable,
) {
    // First, try to add the variable to the current scope
    if let Err(error) = add_variable_to_current_scope(
        &var_decl.symbol_name,
        &var_decl.data_type,
        var_decl.line_declared_on,
        state,
    ) {
        state.errors.push(error);
    }

    // Type checking logic
    if var_decl.data_type != var_decl.assigned_value.data_type {
        if !matches!(
            var_decl.assigned_value.data_type,
            DataType::Invalid | DataType::Unknown
        ) {
            state.errors.push(SemanticError::TypeMismatch {
                expected: var_decl.data_type.clone(),
                found: var_decl.assigned_value.data_type.clone(),
                line: var_decl.line_declared_on,
            });
        }

        // Symbol lookup for assigned value
        let current_context = state.context_stack.last().unwrap();
        let assigned_symbol_def = current_context
            .symbol_table
            .get(&var_decl.assigned_value.raw_text);

        if let Some(symbol_def) = assigned_symbol_def {
            let assigned_data_type = &symbol_def.data_type;
            if &var_decl.data_type != assigned_data_type {
                state.errors.push(SemanticError::TypeMismatch {
                    expected: var_decl.data_type.clone(),
                    found: var_decl.assigned_value.data_type.clone(),
                    line: var_decl.line_declared_on,
                });
            }
        }
    }

    // Validate the assigned value
    if let Some(error) = validate_value(
        &var_decl.assigned_value,
        var_decl.line_declared_on,
        state,
        function_table,
    ) {
        state.errors.push(error);
    }
}

/// Analyze function declaration
fn analyze_function_declaration(
    func_decl: &FunctionDeclarationStatement,
    state: &mut AnalysisState,
    function_table: &FunctionTable,
) {
    push_scope(&func_decl.function_name, state, function_table);

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
    for statement in &func_decl.body {
        analyze_statement(statement, state, function_table);
    }

    pop_scope(state);
}

/// Validate that a value reference is semantically correct
fn validate_value(
    value: &Value,
    line: u32,
    state: &AnalysisState,
    function_table: &FunctionTable,
) -> Option<SemanticError> {
    let current_context = state.context_stack.last().unwrap();

    match value.value_type {
        ValueType::Variable => {
            if !current_context.symbol_table.contains_name(&value.raw_text) {
                return Some(SemanticError::VariableNotDeclared {
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
                return Some(SemanticError::FunctionNotDeclared {
                    name: value.raw_text.clone(),
                    line,
                });
            }
        }
        ValueType::InlineNumber | ValueType::InlineString => {
            // Inline values don't need validation
        }
        ValueType::Expression => {
            // Expression validation would be more complex
        }
        ValueType::Invalid => {
            return Some(SemanticError::InvalidValueReference {
                name: value.raw_text.clone(),
                line,
            });
        }
    }

    None
}

/// Push a new scope for function analysis
fn push_scope(function_name: &str, state: &mut AnalysisState, function_table: &FunctionTable) {
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
}

/// Pop the current scope
fn pop_scope(state: &mut AnalysisState) {
    if state.context_stack.len() > 1 {
        state.context_stack.pop();
    }
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
