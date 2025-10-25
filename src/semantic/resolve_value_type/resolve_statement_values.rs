use crate::ast::DataType;
use crate::ast::FunctionTable;
use crate::ast::Statement;
use crate::ast::Value;
use crate::ast::ValueType;
use crate::ast::VariableAssignmentStatement;
use crate::ast::VariableDeclarationStatement;
use crate::symbol_table::SymbolTable;

pub fn resolve_all_value_types_in_ast(
    ast_statements: &mut Vec<Statement>,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // TODO:
    // Loop through statements and handle differently depending on statement.
    // We're resolving functions return types by looking at the FunctionTable.
    // We don't need to be doing this for variables, because the semantic phase
    // will be validating this using its symbol map.
    /*
    for statement in ast_statements {
        resolve_statement(statement, function_header_map);
    }
    */

    for i in 0..ast_statements.len() {
        let vec_stmt = &mut ast_statements[i];
        resolve_statement(vec_stmt, function_header_map, symbol_table);
    }
    //return ast;
}

fn resolve_statement(
    statement: &mut Statement,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // this is just going to be a giant match statement
    match statement {
        Statement::VariableDeclaration(var_decl_stmt) => {
            resolve_variable_declaration_types(var_decl_stmt, function_header_map, symbol_table);
        }
        _ => (),
    }
}

pub fn resolve_variable_declaration_types(
    var_decl_stmt: &mut VariableDeclarationStatement,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Check that the value_type is a function call
    // if function call: Set the assigned value data_type
    // using the function_header_map

    /* TESTING AST RN
    resolve_value(
        &mut var_decl_stmt.assigned_value,
        function_header_map,
        symbol_table,
    );
    */
}

pub fn resolve_variable_assignment_stmt_types(
    var_ass_stmt: &mut VariableAssignmentStatement,
    function_header_map: &FunctionTable,
    symbol_table: &SymbolTable,
) {
    // Also need to resolve the type of the variable being assigned to.
    let var_name = &var_ass_stmt.var_name;
    if let Some(var_def) = symbol_table.get(var_name) {
        var_ass_stmt.var_data_type = var_def.data_type.clone();
    }

    resolve_value(
        &mut var_ass_stmt.assigned_value,
        function_header_map,
        symbol_table,
    );
}

fn resolve_value(val: &mut Value, function_header_map: &FunctionTable, symbol_table: &SymbolTable) {
    let val_type = &val.value_type;
    match val_type {
        // TODO: could this be moved into a more generic 'resolve value' function?
        ValueType::FunctionCall => {
            let func_call_decl_op = function_header_map.get_func_def_using_str(&val.raw_text);

            match func_call_decl_op {
                Some(func_decl) => {
                    val.data_type = func_decl.return_type.clone();
                    // TODO: NOTE: This assumes all the passed in values are variables.
                    // This isn't generic at all, and if some value that isn't a variable
                    // is here, it won't be handled correctly.
                    // This needs to be resolved.
                    if let Some(params) = val.param_values.as_mut() {
                        for param in params {
                            if let Some(param_var_def) = symbol_table.get(&param.raw_text) {
                                param.data_type = param_var_def.data_type.clone();
                            }
                        }
                    }
                }
                None => {
                    // TODO: when would this error even happen? I suppose if they're
                    // calling a function that doesn't exist, then it'd happen.
                    // Thus, leaving it unknown could indicate that the function
                    // dne. But isn't that handled in semantic analyzer anyway?
                    // So maybe we don't need to do anything here. Test this.
                    println!("NONE was found when trying to set the return type :(");
                }
            }
        }
        ValueType::Variable => {
            // Now we can use the symbol_table to resolve variable types
            // You can implement variable type resolution logic here
            let var_type_op = symbol_table.get(&val.raw_text);
            match var_type_op {
                Some(var_type) => val.data_type = var_type.data_type.clone(),
                None => (), // TODO: maybe do something? nah, this hsould be handled in analysis
            }
        }
        ValueType::Expression
        | ValueType::InlineNumber
        | ValueType::InlineString
        | ValueType::Invalid => {
            // These don't need type resolution from function table or symbol table
        }
    }
}

// Validate that a value reference is semantically correct
/*
fn validate_value(
    value: &Value,
    line: u32,
    state: AnalysisState,
    function_table: &FunctionTable,
) -> AnalysisState {
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
    */
