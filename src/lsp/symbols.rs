use std::collections::HashMap;
use tower_lsp::lsp_types::*;

use crate::ast::{Program, TopLevel, FunctionDecl, ClassDecl, TypeDecl, TypeAliasDecl};
use crate::span::{Span, SourceMap};

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub detail: Option<String>,
    pub definition_span: Option<Span>,  // Byte span in source
}

/// Convert a Span to an LSP Range using a SourceMap
fn span_to_range(span: Span, source_map: &SourceMap) -> Range {
    let (start_line, start_col) = span.start_position(source_map);
    let (end_line, end_col) = span.end_position(source_map);
    
    Range {
        start: Position {
            line: (start_line - 1) as u32,  // LSP is 0-indexed
            character: (start_col - 1) as u32,
        },
        end: Position {
            line: (end_line - 1) as u32,
            character: (end_col - 1) as u32,
        },
    }
}

/// Symbol table for a document
pub struct SymbolTable {
    /// All symbols by name
    symbols: HashMap<String, Vec<Symbol>>,
    /// Source map for position conversion
    source_map: SourceMap,
}

impl SymbolTable {
    /// Creates an empty symbol table
    pub fn new(source: &str) -> Self {
        Self {
            symbols: HashMap::new(),
            source_map: SourceMap::new(source),
        }
    }
    
    /// Builds a symbol table from an AST
    pub fn from_ast(program: &Program, source: &str) -> Self {
        let mut table = Self::new(source);
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
        // FunctionDecl doesn't have span field yet - use default range
        self.insert(Symbol {
            name: func.name.clone(),
            kind: SymbolKind::FUNCTION,
            range: Range::default(),
            detail: Some(format!("fn {}(...)", func.name)),
            definition_span: None,
        });
    }
    
    fn visit_class(&mut self, cls: &ClassDecl) {
        // ClassDecl doesn't have span field yet - use default range
        self.insert(Symbol {
            name: cls.name.clone(),
            kind: SymbolKind::CLASS,
            range: Range::default(),
            detail: Some(format!("class {}", cls.name)),
            definition_span: None,
        });
    }
    
    fn visit_type_decl(&mut self, type_decl: &TypeDecl) {
        // TypeDecl doesn't have span yet, use default
        self.insert(Symbol {
            name: type_decl.name.clone(),
            kind: SymbolKind::STRUCT,
            range: Range::default(),
            detail: Some("interface".to_string()),
            definition_span: None,
        });
    }
    
    fn visit_type_alias(&mut self, type_alias: &TypeAliasDecl) {
        let range = type_alias.span
            .map(|s| span_to_range(s, &self.source_map))
            .unwrap_or_default();
        
        self.insert(Symbol {
            name: type_alias.name.clone(),
            kind: SymbolKind::TYPE_PARAMETER,
            range,
            detail: Some("type alias".to_string()),
            definition_span: type_alias.span,
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
    
    /// Finds all textual references to a symbol name in the source
    /// Returns a list of ranges where the symbol appears
    pub fn find_references(&self, name: &str, source: &str) -> Vec<Range> {
        let mut references = Vec::new();
        
        for (line_idx, line) in source.lines().enumerate() {
            let mut search_start = 0;
            
            while let Some(pos) = line[search_start..].find(name) {
                let actual_pos = search_start + pos;
                
                // Check if it's a word boundary (not part of a larger identifier)
                let before_ok = actual_pos == 0 || 
                    !line.chars().nth(actual_pos - 1).unwrap().is_alphanumeric();
                let after_pos = actual_pos + name.len();
                let after_ok = after_pos >= line.len() || 
                    !line.chars().nth(after_pos).unwrap().is_alphanumeric();
                
                if before_ok && after_ok {
                    references.push(Range {
                        start: Position {
                            line: line_idx as u32,
                            character: actual_pos as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (actual_pos + name.len()) as u32,
                        },
                    });
                }
                
                search_start = actual_pos + 1;
            }
        }
        
        references
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new("")
    }
}
