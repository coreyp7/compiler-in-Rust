use crate::ast::DataType;
use crate::ast::VariableSymbol;
use std::collections::HashMap;

/// A wrapper around HashMap for managing symbol identifiers in the compiler.
/// Automatically assigns unique u8 keys when symbols are inserted and maintains
/// ownership of all symbol strings in one centralized location.
#[derive(Debug)]
pub struct SymbolTable {
    /// Internal HashMap storing the actual symbol data
    symbols: HashMap<u8, VariableSymbol>,
    /// Counter to generate unique keys for new symbols
    next_key: u8,
}

impl SymbolTable {
    /// Creates a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            next_key: 0,
        }
    }

    /// Inserts a new symbol into the table and returns the assigned key.
    /// Returns None if the table is full (reached u8::MAX symbols).
    pub fn insert(
        &mut self,
        symbol_name: &String,
        data_type: &DataType,
        line_declared_on: &u32,
    ) -> Option<u8> {
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

        // Increment for next insertion, wrapping around if needed
        self.next_key = self.next_key.wrapping_add(1);

        Some(key)
    }

    /// Retrieves a symbol by its key
    pub fn get(&self, key: u8) -> Option<&VariableSymbol> {
        self.symbols.get(&key)
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
    /*
    pub fn find_by_name(&self, name: &str) -> Option<u8> {
        for (key, symbol_name) in &self.symbols {
            if symbol_name == name {
                return Some(*key);
            }
        }
        None
    }
    */

    /// Returns an iterator over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&u8, &VariableSymbol)> {
        self.symbols.iter()
    }

    /// Removes a symbol by key and returns it if it existed
    pub fn remove(&mut self, key: u8) -> Option<VariableSymbol> {
        self.symbols.remove(&key)
    }

    /// Clears all symbols from the table and resets the key counter
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.next_key = 0;
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
