use crate::ast::{FunctionHeader, Var, VarType};
use crate::error::ErrMsg;
use std::collections::HashMap;

/// Centralized symbol table for managing variables and functions
/// This replaces the scattered variable and function maps in AST builder and semantic analyzer
pub struct SymbolTable {
    /// Global variables (and function-local variables in current scope)
    variables: HashMap<String, Var>,
    /// All declared functions
    functions: HashMap<String, FunctionHeader>,
    /// Stack of scopes for nested contexts (functions, blocks, etc.)
    scope_stack: Vec<Scope>,
}

#[derive(Debug, Clone)]
struct Scope {
    /// Variables declared in this scope
    variables: HashMap<String, Var>,
    /// Type of scope (Global, Function, Block)
    scope_type: ScopeType,
}

#[derive(Debug, Clone)]
pub enum ScopeType {
    Global,
    Function(String), // Function name
    Block,            // For if/while blocks
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = SymbolTable {
            variables: HashMap::new(),
            functions: HashMap::new(),
            scope_stack: Vec::new(),
        };

        // Always start with global scope
        table.push_scope(ScopeType::Global);
        table
    }

    /// Create a new symbol table with pre-populated variables and functions
    /// (for compatibility with existing code)
    pub fn with_maps(
        variables: HashMap<String, Var>,
        functions: HashMap<String, FunctionHeader>,
    ) -> Self {
        let mut table = SymbolTable {
            variables: variables.clone(),
            functions,
            scope_stack: Vec::new(),
        };

        // Start with global scope containing the variables
        table.scope_stack.push(Scope {
            variables,
            scope_type: ScopeType::Global,
        });

        table
    }

    // === Scope Management ===

    pub fn push_scope(&mut self, scope_type: ScopeType) {
        self.scope_stack.push(Scope {
            variables: HashMap::new(),
            scope_type,
        });
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        // Don't allow popping the global scope
        if self.scope_stack.len() <= 1 {
            return None;
        }
        self.scope_stack.pop()
    }

    pub fn current_scope(&self) -> Option<&Scope> {
        self.scope_stack.last()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut Scope> {
        self.scope_stack.last_mut()
    }

    // === Variable Management ===

    /// Declare a variable in the current scope
    pub fn declare_variable(&mut self, name: String, var: Var) -> Result<(), ErrMsg> {
        if let Some(current_scope) = self.current_scope_mut() {
            // Check if variable already exists in current scope
            if let Some(existing_var) = current_scope.variables.get(&name) {
                return Err(ErrMsg::VariableAlreadyDeclared {
                    identity: name,
                    first_declared_line: existing_var.line_declared_on,
                    redeclared_line: var.line_declared_on,
                });
            }

            // Add to current scope and global variables map
            current_scope.variables.insert(name.clone(), var.clone());
            self.variables.insert(name, var);
            Ok(())
        } else {
            Err(ErrMsg::VariableAlreadyDeclared {
                identity: "No active scope".to_string(),
                first_declared_line: 0,
                redeclared_line: var.line_declared_on,
            })
        }
    }

    /// Look up a variable, searching from current scope up to global
    pub fn lookup_variable(&self, name: &str) -> Option<&Var> {
        // Search from current scope backwards to global scope
        for scope in self.scope_stack.iter().rev() {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }
        None
    }

    /// Get all variables (mainly for compatibility with existing code)
    pub fn get_variables(&self) -> &HashMap<String, Var> {
        &self.variables
    }

    /// Get a mutable reference to variables (for compatibility)
    pub fn get_variables_mut(&mut self) -> &mut HashMap<String, Var> {
        &mut self.variables
    }

    // === Function Management ===

    /// Declare a function
    pub fn declare_function(&mut self, name: String, header: FunctionHeader) -> Result<(), ErrMsg> {
        // For now, allow function redeclaration (forward declarations)
        // In the future, we might want to check for conflicts
        self.functions.insert(name, header);
        Ok(())
    }

    /// Look up a function by name
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionHeader> {
        self.functions.get(name)
    }

    /// Get all functions (mainly for compatibility with existing code)
    pub fn get_functions(&self) -> &HashMap<String, FunctionHeader> {
        &self.functions
    }

    /// Get a mutable reference to functions (for compatibility)
    pub fn get_functions_mut(&mut self) -> &mut HashMap<String, FunctionHeader> {
        &mut self.functions
    }

    // === Utility Methods ===

    /// Check if a variable exists in the current scope
    pub fn variable_exists_in_current_scope(&self, name: &str) -> bool {
        if let Some(scope) = self.current_scope() {
            scope.variables.contains_key(name)
        } else {
            false
        }
    }

    /// Check if a function exists
    pub fn function_exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get the current scope type
    pub fn current_scope_type(&self) -> Option<&ScopeType> {
        self.current_scope().map(|scope| &scope.scope_type)
    }

    /// Debug method to print current symbol table state
    pub fn debug_print(&self) {
        println!("=== Symbol Table Debug ===");
        println!(
            "Functions: {:#?}",
            self.functions.keys().collect::<Vec<_>>()
        );
        println!(
            "Global variables: {:#?}",
            self.variables.keys().collect::<Vec<_>>()
        );
        println!("Scope stack ({} levels):", self.scope_stack.len());
        for (i, scope) in self.scope_stack.iter().enumerate() {
            println!(
                "  Scope {}: {:?} with {} variables",
                i,
                scope.scope_type,
                scope.variables.len()
            );
        }
        println!("========================");
    }
}

// === Helper methods for type checking ===

impl SymbolTable {
    /// Validate variable assignment (type checking)
    pub fn validate_assignment(
        &self,
        var_name: &str,
        assigned_type: &VarType,
    ) -> Result<(), ErrMsg> {
        if let Some(var) = self.lookup_variable(var_name) {
            if var.var_type != *assigned_type {
                return Err(ErrMsg::new_incorrect_type_assignment(
                    var.var_type.clone(),
                    assigned_type.clone(),
                    0, // TODO: pass line number
                ));
            }
            Ok(())
        } else {
            Err(ErrMsg::VariableNotDeclared {
                identity: var_name.to_string(),
                attempted_assignment_line: 0, // TODO: pass line number
            })
        }
    }

    /// Get variable type (convenience method)
    pub fn get_variable_type(&self, name: &str) -> Option<VarType> {
        self.lookup_variable(name).map(|var| var.var_type.clone())
    }

    /// Get function return type (convenience method)
    pub fn get_function_return_type(&self, name: &str) -> Option<VarType> {
        self.lookup_function(name)
            .map(|func| func.return_type.clone())
    }
}
