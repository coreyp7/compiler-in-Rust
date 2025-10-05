use super::pure_builder::DataType;
use std::collections::HashMap;

// Function-related structures
#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub identifier: String,
    pub parameters: Vec<Parameter>,
    pub return_type: DataType,
    pub line_declared_on: u32,
    // A FunctionSymbol is "assumed guilty until proven innocent".
    // When the ast parses this function and sees its return type is valid,
    // we can mark this as true.
    pub properly_returns: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub data_type: DataType,
}

/// Function table for managing function symbols
#[derive(Debug, Clone)]
pub struct FunctionTable {
    functions: HashMap<u8, FunctionSymbol>,
    name_to_id: HashMap<String, u8>,
    next_id: u8,
}

impl FunctionTable {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            name_to_id: HashMap::new(),
            next_id: 0,
        }
    }

    /// Inserts a new function into the table and returns the assigned key.
    /// Returns None if the table is full or if a function with the same name already exists.
    pub fn insert(
        &mut self,
        name: &str,
        parameters: Vec<Parameter>,
        return_type: DataType,
        line: &u32,
    ) -> Option<u8> {
        // Check for duplicate names
        if self.name_to_id.contains_key(name) {
            return None; // Function already exists
        }

        if self.next_id == u8::MAX && self.functions.contains_key(&u8::MAX) {
            return None; // Table is full
        }

        let id = self.next_id;
        let function_symbol = FunctionSymbol {
            identifier: name.to_string(),
            parameters,
            return_type,
            line_declared_on: *line,
            properly_returns: false,
        };

        self.functions.insert(id, function_symbol);
        self.name_to_id.insert(name.to_string(), id);
        self.next_id = self.next_id.wrapping_add(1);

        Some(id)
    }

    /// Retrieves a function by its key
    pub fn get_using_id(&self, id: u8) -> Option<&FunctionSymbol> {
        self.functions.get(&id)
    }

    /// Gets function ID by name
    pub fn get_id_with_function_name(&self, name: &str) -> Option<u8> {
        self.name_to_id.get(name).copied()
    }

    pub fn get_func_def_using_str(&self, function_name: &String) -> Option<&FunctionSymbol> {
        // TODO need better error handling
        let id_optional = self.get_id_with_function_name(function_name);
        match id_optional {
            None => {
                println!(
                    "HEY: the id lookup for the function named {} failed.",
                    function_name
                );
                return None;
            }
            _ => (),
        }
        let function_def = self.get_using_id(id_optional.unwrap());
        function_def
    }

    /// Checks if a function with the given name exists
    pub fn contains_name(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    /// Returns the number of functions in the table
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Checks if the function table is empty
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    /// Returns an iterator over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&u8, &FunctionSymbol)> {
        self.functions.iter()
    }
}
