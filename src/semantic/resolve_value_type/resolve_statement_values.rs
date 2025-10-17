use crate::ast::DataType;
use crate::ast::FunctionTable;
use crate::ast::Statement;
use crate::ast::ValueType;

pub fn resolve_all_value_types_in_ast(
    ast_statements: &mut Vec<Statement>,
    function_header_map: &FunctionTable,
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
        resolve_statement(vec_stmt, function_header_map);
    }
    //return ast;
}

fn resolve_statement(statement: &mut Statement, function_header_map: &FunctionTable) {
    // this is just going to be a giant match statement
    match statement {
        Statement::VariableDeclaration(var_decl_stmt) => {
            // Check that the value_type is a function call
            // if function call: Set the assigned value data_type
            // using the function_header_map
            if var_decl_stmt.assigned_value.value_type == ValueType::FunctionCall {
                let func_call_decl_op = function_header_map
                    .get_func_def_using_str(&var_decl_stmt.assigned_value.raw_text);

                match func_call_decl_op {
                    Some(func_decl) => {
                        var_decl_stmt.assigned_value.data_type = func_decl.return_type.clone();
                        println!("we set the return type!");
                        println!("func_decl type: {:?}", func_decl.return_type);
                        println!(
                            "assigned value new type: {:?}",
                            var_decl_stmt.assigned_value.data_type
                        );
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
        }
        _ => (),
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
