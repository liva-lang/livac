use dashmap::DashMap;
use livac::ast::{Member, Program, TopLevel};
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::*;

use super::symbols::{Symbol, SymbolTable};

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
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if matches!(
                        dir_name,
                        "node_modules" | "target" | ".git" | "dist" | "build"
                    ) {
                        continue;
                    }

                    // Recursively scan subdirectory
                    self.scan_directory(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("liva") {
                    // Found a .liva file
                    if let Ok(uri) = Url::from_file_path(&path) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                self.file_uris.insert(
                                    uri.clone(),
                                    FileMetadata {
                                        uri,
                                        path: path.clone(),
                                        last_modified: modified,
                                    },
                                );
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
                        self.file_uris.insert(
                            uri.clone(),
                            FileMetadata {
                                uri,
                                path,
                                last_modified: modified,
                            },
                        );
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

    /// Interface / type name -> list of classes implementing it.
    /// Populated from `ClassDecl.implements` during `index_file`.
    /// Used by `goto_implementation` to jump from an interface or method
    /// declaration to all concrete implementors in the workspace.
    implementations: DashMap<String, Vec<(Url, Symbol)>>,
}

impl WorkspaceIndex {
    /// Creates a new empty workspace index
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
            file_symbols: DashMap::new(),
            implementations: DashMap::new(),
        }
    }

    /// Indexes a file and adds its symbols to the global index.
    ///
    /// If the file was already indexed, its previous symbols are removed
    /// from the global index before the new ones are added. This keeps
    /// re-indexing on edit (did_change / did_save) idempotent — without
    /// this step, every keystroke would duplicate the file's symbols in
    /// `self.symbols` and corrupt workspace/symbol results.
    pub fn index_file(&self, uri: Url, ast: &Program, source: &str) {
        // Drop any stale entries from a previous index of this file.
        if self.file_symbols.contains_key(&uri) {
            self.remove_file(&uri);
        }

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
        self.file_symbols.insert(uri.clone(), symbol_table);

        // Index interface implementations declared in this file. For every
        // `Class : Interface1, Interface2 { ... }` we record an entry for
        // each interface, plus one entry per method (so go-to-implementation
        // on a method name also resolves). The class name lookups go through
        // the regular symbol table.
        self.index_implementations(&uri, ast, source);
    }

    fn index_implementations(&self, uri: &Url, ast: &Program, source: &str) {
        for item in &ast.items {
            let TopLevel::Class(cls) = item else { continue };
            if cls.implements.is_empty() {
                continue;
            }

            // Resolve the class's display range from the source by finding
            // the first occurrence of the class name on a line that looks
            // like a class header (e.g. `Cat :` or `Cat {`).
            let range = locate_class_header(&cls.name, source).unwrap_or_default();
            let class_sym = Symbol {
                name: cls.name.clone(),
                kind: SymbolKind::CLASS,
                range,
                detail: Some(format!(
                    "class {} : {}",
                    cls.name,
                    cls.implements.join(", ")
                )),
                definition_span: None,
            };

            for iface in &cls.implements {
                self.implementations
                    .entry(iface.clone())
                    .or_insert_with(Vec::new)
                    .push((uri.clone(), class_sym.clone()));
            }

            // Per-method entries: methodName -> all classes that declare
            // a method by that name AND implement at least one interface.
            // This is conservative — it doesn't check that the method is
            // actually required by the interface, but the LSP filter on
            // the call site already constrains the context.
            for member in &cls.members {
                if let Member::Method(method) = member {
                    let method_range =
                        locate_method(&cls.name, &method.name, source).unwrap_or_default();
                    let method_sym = Symbol {
                        name: method.name.clone(),
                        kind: SymbolKind::METHOD,
                        range: method_range,
                        detail: Some(format!("{}::{}", cls.name, method.name)),
                        definition_span: None,
                    };
                    self.implementations
                        .entry(method.name.clone())
                        .or_insert_with(Vec::new)
                        .push((uri.clone(), method_sym));
                }
            }
        }
    }

    /// Looks up implementations for an interface or interface method name.
    pub fn lookup_implementations(&self, name: &str) -> Option<Vec<(Url, Symbol)>> {
        self.implementations
            .get(name)
            .map(|entry| entry.value().clone())
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

    /// Gets all symbols from a specific file
    pub fn get_file_symbols(&self, uri: &Url) -> Option<Vec<Symbol>> {
        self.file_symbols
            .get(uri)
            .map(|table| table.all().iter().map(|s| (*s).clone()).collect())
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
        // Clean implementations contributed by this file.
        let keys_to_clean: Vec<String> = self
            .implementations
            .iter()
            .map(|e| e.key().clone())
            .collect();
        for key in keys_to_clean {
            if let Some(mut entry) = self.implementations.get_mut(&key) {
                entry.retain(|(file_uri, _)| file_uri != uri);
                if entry.is_empty() {
                    drop(entry);
                    self.implementations.remove(&key);
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

/// Find the LSP range for a class header by scanning the source for the
/// class name followed by `:` or `{`. Returns `None` if the class header
/// cannot be located (in that case the caller falls back to a default range).
fn locate_class_header(name: &str, source: &str) -> Option<Range> {
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with(name) {
            continue;
        }
        let after = &trimmed[name.len()..];
        // Class header: `Name {` or `Name :` or `Name<...>`.
        let next = after.chars().next();
        let looks_like_header = matches!(next, Some('{') | Some(':') | Some('<') | Some(' '));
        if !looks_like_header {
            continue;
        }
        let col = line.len() - trimmed.len();
        return Some(Range {
            start: Position {
                line: line_idx as u32,
                character: col as u32,
            },
            end: Position {
                line: line_idx as u32,
                character: (col + name.len()) as u32,
            },
        });
    }
    None
}

/// Find the LSP range for a method declaration `methodName(` inside a class.
/// Best-effort: scans the source after the class header for the first
/// `methodName(` occurrence and returns its range. Returns `None` if not
/// found. Does not attempt to handle nested classes (Liva has none).
fn locate_method(class_name: &str, method_name: &str, source: &str) -> Option<Range> {
    let mut in_class = false;
    let needle = format!("{}(", method_name);
    for (line_idx, line) in source.lines().enumerate() {
        if !in_class {
            let trimmed = line.trim_start();
            if trimmed.starts_with(class_name) {
                let after = &trimmed[class_name.len()..];
                let next = after.chars().next();
                if matches!(next, Some('{') | Some(':') | Some('<') | Some(' ')) {
                    in_class = true;
                }
            }
            continue;
        }
        if let Some(pos) = line.find(&needle) {
            // Skip if `methodName` is part of a longer identifier.
            let before_ok = pos == 0
                || !line
                    .as_bytes()
                    .get(pos - 1)
                    .map(|b| b.is_ascii_alphanumeric() || *b == b'_')
                    .unwrap_or(false);
            if before_ok {
                return Some(Range {
                    start: Position {
                        line: line_idx as u32,
                        character: pos as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (pos + method_name.len()) as u32,
                    },
                });
            }
        }
    }
    None
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

    #[test]
    fn test_reindex_does_not_duplicate_symbols() {
        // Re-indexing the same file twice should leave the global symbol
        // table in the same state as a single index pass. Before the
        // invalidation fix, every keystroke would duplicate symbols.
        let source = "greet() { print(\"hi\") }\n";
        let tokens = livac::lexer::tokenize(source).expect("lex");
        let ast = livac::parser::parse(tokens, source).expect("parse");

        let index = WorkspaceIndex::new();
        let uri = Url::parse("file:///tmp/reindex.liva").unwrap();

        index.index_file(uri.clone(), &ast, source);
        let first_count = index.symbol_count();
        let first_files = index.file_count();
        assert!(
            first_count >= 1,
            "expected at least one symbol after first index"
        );
        assert_eq!(first_files, 1);

        // Re-index the same file: should not grow the symbol count.
        index.index_file(uri.clone(), &ast, source);
        assert_eq!(index.symbol_count(), first_count);
        assert_eq!(index.file_count(), 1);

        // workspace/symbol must return each symbol exactly once.
        let greet = index
            .lookup_global("greet")
            .expect("greet should be globally indexed");
        assert_eq!(
            greet.len(),
            1,
            "duplicate workspace-symbol entries after reindex"
        );
    }
}
