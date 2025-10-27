use std::time::Instant;
use tower_lsp::lsp_types::*;

use crate::ast::Program;
use super::symbols::SymbolTable;

/// State of an open document
pub struct DocumentState {
    /// Full text content
    pub text: String,
    
    /// Document version (increments on change)
    pub version: i32,
    
    /// Parsed AST (cached)
    pub ast: Option<Program>,
    
    /// Symbol table (extracted from AST)
    pub symbols: Option<SymbolTable>,
    
    /// Current diagnostics
    pub diagnostics: Vec<Diagnostic>,
    
    /// Last parse timestamp
    pub last_parsed: Instant,
}

impl DocumentState {
    /// Creates a new document state
    pub fn new(text: String, version: i32) -> Self {
        Self {
            text,
            version,
            ast: None,
            symbols: None,
            diagnostics: Vec::new(),
            last_parsed: Instant::now(),
        }
    }
    
    /// Gets word at cursor position
    pub fn word_at_position(&self, position: Position) -> Option<String> {
        let line_idx = position.line as usize;
        let lines: Vec<&str> = self.text.lines().collect();
        
        if line_idx >= lines.len() {
            return None;
        }
        
        let line = lines[line_idx];
        let char_idx = position.character as usize;
        
        if char_idx > line.len() {
            return None;
        }
        
        // Find word boundaries
        let start = line[..char_idx]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        let end = line[char_idx..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| char_idx + i)
            .unwrap_or(line.len());
        
        if start < end {
            Some(line[start..end].to_string())
        } else {
            None
        }
    }
}
