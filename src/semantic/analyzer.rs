use crate::ast::FunctionTable;
use crate::ast::{DataType, Statement, Value, ValueType, VariableDeclarationStatement};
use crate::semantic::SemanticError;
use crate::symbol_table::SymbolTable;

/// Semantic analyzer context for managing scope
pub struct SemanticContext {
    pub symbol_table: SymbolTable,
    pub function_table: FunctionTable,
    pub scope: Option<u8>, // Function scope if in function, None if global
}

/// Main semantic analyzer
pub struct SemanticAnalyzer {
    context_stack: Vec<SemanticContext>,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer with global scope
    pub fn new(function_table: FunctionTable) -> Self {
        let mut context_stack = Vec::new();
        context_stack.push(SemanticContext {
            symbol_table: SymbolTable::new(),
            function_table,
            scope: None,
        });

        Self {
            context_stack,
            errors: Vec::new(),
        }
    }

    /// Analyze a list of statements
    pub fn analyze(&mut self, statements: &[Statement]) -> Vec<SemanticError> {
        self.errors.clear();

        for statement in statements {
            self.analyze_statement(statement);
        }

        self.errors.clone()
    }

    // Analyze a single statement
    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VariableDeclaration(var_decl) => {
                self.analyze_variable_declaration(var_decl);
            }
            Statement::FunctionDeclaration(_func_decl) => {
                // Function declarations are handled in the first pass
                // Here we might validate the function body when we implement it
            }
            Statement::Return(return_stmt) => {
                // TODO: improve this to check the type and symbol lookup validation
                if let Some(ref return_value) = return_stmt.return_value {
                    if let Some(error) =
                        self.validate_value(return_value, return_stmt.line_declared_on)
                    {
                        self.errors.push(error);
                    }
                }
            }
        }
    }

    /// Analyze variable declaration for semantic correctness
    fn analyze_variable_declaration(&mut self, var_decl: &VariableDeclarationStatement) {
        // First, try to add the variable to the current scope to check for redeclaration
        if let Err(error) = self.add_variable_to_current_scope(
            &var_decl.symbol_name,
            &var_decl.data_type,
            var_decl.line_declared_on,
        ) {
            self.errors.push(error);
        }

        // Check for type mismatch between declared type and assigned value type
        if var_decl.data_type != var_decl.assigned_value.data_type {
            // Only report error if the assigned value type is not Invalid or Unknown.
            // If Invalid/Unknown, could be an undeclared variable, reference,
            // which is a different error reported later.
            if !matches!(
                var_decl.assigned_value.data_type,
                (DataType::Invalid | DataType::Unknown)
            ) {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: var_decl.data_type.clone(),
                    found: var_decl.assigned_value.data_type.clone(),
                    line: var_decl.line_declared_on,
                });
            }

            // TODO: if this is a variable call, then the assigned DataType is Unknown.
            // Do a symbol lookup and see if the assigned variable type matches.
            // If not, throw error here.
            // Also need to check that this variable even exists.
            let assigned_symbol_def = &self
                .get_current_symbol_context()
                .get(&var_decl.assigned_value.raw_text);

            match assigned_symbol_def {
                // If the var being called exists, ensure that the types match.
                Some(symbol_def) => {
                    let assigned_data_type = &symbol_def.data_type;
                    if &var_decl.data_type != assigned_data_type {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: var_decl.data_type.clone(),
                            found: var_decl.assigned_value.data_type.clone(),
                            line: var_decl.line_declared_on,
                        });
                    }
                }
                // If this is not found, then we add an error that the assigned var
                // doesn't exist. (This is done when 'validate_value' is called below,
                // so here we won't be adding anything.)
                // Leaving previous code for convenience.
                None => {
                    /*
                    if !self
                        .get_current_symbol_context()
                        .contains_name(&var_decl.assigned_value.raw_text)
                    {
                        self.errors.push(SemanticError::InvalidValueReference {
                            name: var_decl.assigned_value.raw_text.clone(),
                            line: var_decl.line_declared_on,
                        });
                    };
                    */
                }
            }
        }

        // Validate the assigned value itself
        if let Some(error) =
            self.validate_value(&var_decl.assigned_value, var_decl.line_declared_on)
        {
            self.errors.push(error);
        }
    }

    /// Validate that a value reference is semantically correct
    pub fn validate_value(&self, value: &Value, line: u32) -> Option<SemanticError> {
        let current_context = self.context_stack.last().unwrap();

        match value.value_type {
            ValueType::Variable => {
                // Check if the variable exists by name
                if !current_context.symbol_table.contains_name(&value.raw_text) {
                    return Some(SemanticError::VariableNotDeclared {
                        name: value.raw_text.clone(),
                        line,
                    });
                }
            }
            ValueType::FunctionCall => {
                // Check if the function exists by name
                if current_context
                    .function_table
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
                // For now, we'll skip it
            }
            ValueType::Invalid => {
                // Invalid values indicate parsing errors
                return Some(SemanticError::InvalidValueReference {
                    name: value.raw_text.clone(),
                    line,
                });
            }
        }

        None
    }

    /// Push a new scope (for function analysis)
    pub fn push_scope(&mut self, function_name: &str) {
        let current_context = self.context_stack.last().unwrap();

        // Get function information for the new scope
        let function_name_string = function_name.to_string();
        if let Some(function_id) = current_context
            .function_table
            .get_id_with_function_name(&function_name_string)
        {
            if let Some(function_def) = current_context
                .function_table
                .get_func_def_using_str(&function_name_string)
            {
                let mut new_symbol_table = SymbolTable::new();

                // Add function parameters to the new scope's symbol table
                for parameter in &function_def.parameters {
                    new_symbol_table.insert(
                        &parameter.name,
                        &parameter.data_type,
                        &function_def.line_declared_on,
                    );
                }

                let new_context = SemanticContext {
                    symbol_table: new_symbol_table,
                    function_table: FunctionTable::new(),
                    scope: Some(function_id),
                };

                self.context_stack.push(new_context);
            }
        }
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if self.context_stack.len() > 1 {
            self.context_stack.pop();
        }
    }

    /// Get the current symbol context
    pub fn get_current_symbol_context(&self) -> &SymbolTable {
        &self.context_stack.last().unwrap().symbol_table
    }

    /// Get the current function context
    pub fn get_current_function_context(&self) -> &FunctionTable {
        &self.context_stack.last().unwrap().function_table
    }

    /// Add a variable to the current scope (used during semantic analysis)
    pub fn add_variable_to_current_scope(
        &mut self,
        name: &str,
        data_type: &DataType,
        line: u32,
    ) -> Result<u8, SemanticError> {
        let current_context = self.context_stack.last_mut().unwrap();

        // Check if variable already exists in current scope
        if current_context.symbol_table.contains_name(name) {
            println!("{} exists in the table already:", name);
            println!("{:#?}", current_context.symbol_table);
            if let Some(existing_var) = current_context.symbol_table.get(name) {
                return Err(SemanticError::VariableAlreadyDeclared {
                    name: name.to_string(),
                    first_line: existing_var.line_declared_on,
                    redeclaration_line: line,
                });
            } else {
                println!("So the name exists in the table, but trying to get it results in None.");
            }
        } else {
            println!("------------");
            println!(
                "The symbol '{}' doesn't exist yet in the following table.",
                name
            );
            println!("{:#?}", current_context.symbol_table);
            println!("------------");
        }

        // Add to symbol table
        let name_string = name.to_string();
        if let Some(key) = current_context
            .symbol_table
            .insert(&name_string, data_type, &line)
        {
            Ok(key)
        } else {
            Err(SemanticError::VariableAlreadyDeclared {
                name: name.to_string(),
                first_line: 0, // We don't have the original line in this case
                redeclaration_line: line,
            })
        }
    }

    /// Get all errors found during analysis
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Print all errors
    pub fn print_errors(&self) {
        for error in &self.errors {
            error.print_error();
        }
    }
}
