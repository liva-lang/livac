use std::collections::HashMap;
use tower_lsp::lsp_types::*;

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub detail: Option<String>,
}

/// Symbol table for a document
pub struct SymbolTable {
    /// All symbols by name
    symbols: HashMap<String, Vec<Symbol>>,
}

impl SymbolTable {
    /// Creates an empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
    
    /// Adds a symbol to the table
    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols
            .entry(symbol.name.clone())
            .or_insert_with(Vec::new)
            .push(symbol);
    }
    
    /// Looks up symbols by name
    pub fn lookup(&self, name: &str) -> Option<&Vec<Symbol>> {
        self.symbols.get(name)
    }
    
    /// Gets all symbols
    pub fn all(&self) -> Vec<&Symbol> {
        self.symbols.values().flatten().collect()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
