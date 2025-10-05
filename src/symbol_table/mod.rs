use crate::ast::DataType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VariableSymbol {
    pub identifier: String,
    pub data_type: DataType,
    pub line_declared_on: u32,
}

/// A wrapper around HashMap for managing symbol identifiers in the compiler.
/// Automatically assigns unique u8 keys when symbols are inserted and maintains
/// ownership of all symbol strings in one centralized location.
#[derive(Debug)]
pub struct SymbolTable {
    // This setup is gross. Created symbols map first, then needed to be able to
    // easily lookup by name. Impl is weird here but is easy for the caller.
    symbols: HashMap<u8, VariableSymbol>,
    name_to_key: HashMap<String, u8>,
    // Counter to generate unique keys for new symbols
    next_key: u8,
}

impl SymbolTable {
    /// Creates a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            name_to_key: HashMap::new(),
            next_key: 0,
        }
    }

    /// Inserts a new symbol into the table and returns the assigned key.
    /// Returns None if the table is full (reached u8::MAX symbols) or if a symbol
    /// with the same name already exists.
    pub fn insert(
        &mut self,
        symbol_name: &String,
        data_type: &DataType,
        line_declared_on: &u32,
    ) -> Option<u8> {
        // Check for duplicate names
        if self.name_to_key.contains_key(symbol_name) {
            return None; // Symbol already exists
        }

        if self.next_key == u8::MAX && self.symbols.contains_key(&u8::MAX) {
            // Table is full
            return None;
        }

        let key = self.next_key;
        self.symbols.insert(
            key,
            VariableSymbol {
                identifier: symbol_name.clone(),
                data_type: data_type.clone(),
                line_declared_on: *line_declared_on,
            },
        );
        self.name_to_key.insert(symbol_name.clone(), key);

        // Increment for next insertion, wrapping around if needed
        self.next_key = self.next_key.wrapping_add(1);

        Some(key)
    }

    /// Retrieves a symbol by its key
    pub fn get_using_id(&self, key: u8) -> Option<&VariableSymbol> {
        self.symbols.get(&key)
    }

    pub fn get(&self, symbol_name: &str) -> Option<&VariableSymbol> {
        if !self.contains_name(symbol_name) {
            return None;
        }

        if let Some(symbol_id) = self.find_by_name(symbol_name) {
            return self.get_using_id(symbol_id);
        }

        None
    }

    pub fn get_id_with_symbol_name(&self, symbol_name: &str) -> Option<u8> {
        self.name_to_key.get(symbol_name).cloned()
    }

    /// Checks if a symbol with the given key exists
    pub fn contains_key(&self, key: u8) -> bool {
        self.symbols.contains_key(&key)
    }

    /// Returns the number of symbols in the table
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Checks if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Finds a symbol by name and returns its key if found
    pub fn find_by_name(&self, name: &str) -> Option<u8> {
        self.name_to_key.get(name).copied()
    }

    /// Checks if a variable with the given name has been declared
    pub fn contains_name(&self, name: &str) -> bool {
        self.name_to_key.contains_key(name)
    }

    /// Returns an iterator over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&u8, &VariableSymbol)> {
        self.symbols.iter()
    }

    /// Removes a symbol by key and returns it if it existed
    pub fn remove(&mut self, key: u8) -> Option<VariableSymbol> {
        if let Some(symbol) = self.symbols.remove(&key) {
            self.name_to_key.remove(&symbol.identifier);
            Some(symbol)
        } else {
            None
        }
    }

    /// Clears all symbols from the table and resets the key counter
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.name_to_key.clear();
        self.next_key = 0;
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
