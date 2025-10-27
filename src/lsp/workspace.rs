use dashmap::DashMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::*;

use super::symbols::{Symbol, SymbolTable};
use crate::ast::Program;

/// Metadata about a file in the workspace
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub uri: Url,
    pub path: PathBuf,
    pub last_modified: std::time::SystemTime,
}

/// Manages workspace files and directories
pub struct WorkspaceManager {
    /// Root URIs of the workspace folders
    root_uris: Vec<Url>,
    
    /// All discovered .liva files
    file_uris: DashMap<Url, FileMetadata>,
}

impl WorkspaceManager {
    /// Creates a new workspace manager
    pub fn new(root_uris: Vec<Url>) -> Self {
        Self {
            root_uris,
            file_uris: DashMap::new(),
        }
    }
    
    /// Scans all workspace folders for .liva files
    pub fn scan_workspace(&mut self) {
        for root_uri in &self.root_uris {
            if let Ok(root_path) = root_uri.to_file_path() {
                self.scan_directory(&root_path);
            }
        }
    }
    
    /// Recursively scans a directory for .liva files
    fn scan_directory(&self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip hidden files and directories
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                }
                
                // Skip common ignored directories
                if path.is_dir() {
                    let dir_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    
                    if matches!(dir_name, "node_modules" | "target" | ".git" | "dist" | "build") {
                        continue;
                    }
                    
                    // Recursively scan subdirectory
                    self.scan_directory(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("liva") {
                    // Found a .liva file
                    if let Ok(uri) = Url::from_file_path(&path) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                self.file_uris.insert(uri.clone(), FileMetadata {
                                    uri,
                                    path: path.clone(),
                                    last_modified: modified,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Adds a file to the workspace
    pub fn add_file(&self, uri: Url) {
        if let Ok(path) = uri.to_file_path() {
            if path.extension().and_then(|s| s.to_str()) == Some("liva") {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        self.file_uris.insert(uri.clone(), FileMetadata {
                            uri,
                            path,
                            last_modified: modified,
                        });
                    }
                }
            }
        }
    }
    
    /// Removes a file from the workspace
    pub fn remove_file(&self, uri: &Url) {
        self.file_uris.remove(uri);
    }
    
    /// Lists all .liva files in the workspace
    pub fn list_liva_files(&self) -> Vec<Url> {
        self.file_uris
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Gets file metadata
    pub fn get_metadata(&self, uri: &Url) -> Option<FileMetadata> {
        self.file_uris.get(uri).map(|entry| entry.value().clone())
    }
    
    /// Checks if a file exists in the workspace
    pub fn contains_file(&self, uri: &Url) -> bool {
        self.file_uris.contains_key(uri)
    }
    
    /// Gets the number of files in the workspace
    pub fn file_count(&self) -> usize {
        self.file_uris.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_manager_creation() {
        let root = Url::parse("file:///tmp/test").unwrap();
        let manager = WorkspaceManager::new(vec![root]);
        assert_eq!(manager.file_count(), 0);
    }
    
    #[test]
    fn test_add_remove_file() {
        let manager = WorkspaceManager::new(vec![]);
        let uri = Url::parse("file:///tmp/test.liva").unwrap();
        
        // Initially empty
        assert_eq!(manager.file_count(), 0);
        
        // Note: add_file requires actual file system, so this test is limited
        // In a real scenario, you'd need to create temp files
    }
}

/// Global symbol index across all workspace files
pub struct WorkspaceIndex {
    /// Symbol name -> List of (URI, Symbol)
    /// Allows lookup of all symbols with a given name across the workspace
    symbols: DashMap<String, Vec<(Url, Symbol)>>,
    
    /// URI -> Local symbol table for that file
    /// Caches the parsed symbol table for each file
    file_symbols: DashMap<Url, SymbolTable>,
}

impl WorkspaceIndex {
    /// Creates a new empty workspace index
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
            file_symbols: DashMap::new(),
        }
    }
    
    /// Indexes a file and adds its symbols to the global index
    pub fn index_file(&self, uri: Url, ast: &Program, source: &str) {
        // Build symbol table for this file
        let symbol_table = SymbolTable::from_ast(ast, source);
        
        // Add each symbol to the global index
        for symbol in symbol_table.all() {
            let name = symbol.name.clone();
            let symbol_clone = symbol.clone();
            
            self.symbols
                .entry(name)
                .or_insert_with(Vec::new)
                .push((uri.clone(), symbol_clone));
        }
        
        // Cache the symbol table for this file
        self.file_symbols.insert(uri, symbol_table);
    }
    
    /// Looks up a symbol globally across all files
    /// Returns all symbols with the given name and their locations
    pub fn lookup_global(&self, name: &str) -> Option<Vec<(Url, Symbol)>> {
        self.symbols.get(name).map(|entry| entry.value().clone())
    }
    
    /// Looks up a symbol in a specific file
    pub fn lookup_in_file(&self, uri: &Url, name: &str) -> Option<Vec<Symbol>> {
        self.file_symbols
            .get(uri)
            .and_then(|table| table.lookup(name).map(|v| v.clone()))
    }
    
    /// Removes a file from the index
    pub fn remove_file(&self, uri: &Url) {
        // Remove the file's symbol table
        if let Some((_, table)) = self.file_symbols.remove(uri) {
            // Remove all symbols from this file from the global index
            for symbol in table.all() {
                if let Some(mut entry) = self.symbols.get_mut(&symbol.name) {
                    entry.retain(|(file_uri, _)| file_uri != uri);
                    
                    // Remove empty entries
                    if entry.is_empty() {
                        drop(entry);
                        self.symbols.remove(&symbol.name);
                    }
                }
            }
        }
    }
    
    /// Gets all symbols in the index
    pub fn all_symbols(&self) -> Vec<(Url, Symbol)> {
        let mut all = Vec::new();
        for entry in self.symbols.iter() {
            all.extend(entry.value().clone());
        }
        all
    }
    
    /// Gets the number of indexed files
    pub fn file_count(&self) -> usize {
        self.file_symbols.len()
    }
    
    /// Gets the total number of symbols across all files
    pub fn symbol_count(&self) -> usize {
        self.symbols.iter().map(|entry| entry.value().len()).sum()
    }
    
    /// Checks if a file is indexed
    pub fn contains_file(&self, uri: &Url) -> bool {
        self.file_symbols.contains_key(uri)
    }
    
    /// Lists all indexed file URIs
    pub fn indexed_files(&self) -> Vec<Url> {
        self.file_symbols
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

impl Default for WorkspaceIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod workspace_index_tests {
    use super::*;
    
    #[test]
    fn test_workspace_index_creation() {
        let index = WorkspaceIndex::new();
        assert_eq!(index.file_count(), 0);
        assert_eq!(index.symbol_count(), 0);
    }
    
    #[test]
    fn test_empty_lookup() {
        let index = WorkspaceIndex::new();
        assert!(index.lookup_global("nonexistent").is_none());
    }
}
