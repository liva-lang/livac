use std::collections::HashMap;
use tower_lsp::lsp_types::*;

use crate::ast::{Program, TopLevel, FunctionDecl, ClassDecl, TypeDecl, TypeAliasDecl};

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
    
    /// Builds a symbol table from an AST
    pub fn from_ast(program: &Program) -> Self {
        let mut table = Self::new();
        table.visit_program(program);
        table
    }
    
    fn visit_program(&mut self, program: &Program) {
        for item in &program.items {
            self.visit_top_level(item);
        }
    }
    
    fn visit_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function(func) => {
                self.visit_function(func);
            }
            TopLevel::Class(cls) => {
                self.visit_class(cls);
            }
            TopLevel::Type(type_decl) => {
                self.visit_type_decl(type_decl);
            }
            TopLevel::TypeAlias(type_alias) => {
                self.visit_type_alias(type_alias);
            }
            TopLevel::Import(_) | TopLevel::UseRust(_) | TopLevel::Test(_) => {
                // Skip for now
            }
        }
    }
    
    fn visit_function(&mut self, func: &FunctionDecl) {
        self.insert(Symbol {
            name: func.name.clone(),
            kind: SymbolKind::FUNCTION,
            range: Range::default(),
            detail: Some(format!("fn {}(...)", func.name)),
        });
    }
    
    fn visit_class(&mut self, cls: &ClassDecl) {
        self.insert(Symbol {
            name: cls.name.clone(),
            kind: SymbolKind::CLASS,
            range: Range::default(),
            detail: Some(format!("class {}", cls.name)),
        });
    }
    
    fn visit_type_decl(&mut self, type_decl: &TypeDecl) {
        self.insert(Symbol {
            name: type_decl.name.clone(),
            kind: SymbolKind::STRUCT,
            range: Range::default(),
            detail: Some("interface".to_string()),
        });
    }
    
    fn visit_type_alias(&mut self, type_alias: &TypeAliasDecl) {
        self.insert(Symbol {
            name: type_alias.name.clone(),
            kind: SymbolKind::TYPE_PARAMETER,
            range: Range::default(),
            detail: Some("type alias".to_string()),
        });
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
