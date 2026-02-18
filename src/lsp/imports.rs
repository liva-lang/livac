use tower_lsp::lsp_types::Url;

use crate::ast::{Program, TopLevel};

/// Information about a single import statement
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Imported symbol names: ["add", "multiply"]
    pub symbols: Vec<String>,

    /// Source path from import statement: "./math.liva"
    pub source: String,

    /// Whether this is a wildcard import: import *
    pub is_wildcard: bool,

    /// Alias for wildcard imports: import * as name
    pub alias: Option<String>,

    /// Resolved absolute URI if path was resolved successfully
    pub resolved_uri: Option<Url>,
}

/// Import resolver for Liva modules
pub struct ImportResolver {
    /// Workspace root URIs for resolving absolute imports
    workspace_roots: Vec<Url>,
}

impl ImportResolver {
    /// Create a new import resolver
    pub fn new(workspace_roots: Vec<Url>) -> Self {
        Self { workspace_roots }
    }

    /// Extract imports from an AST
    pub fn extract_imports(&self, ast: &Program, current_file_uri: &Url) -> Vec<ImportInfo> {
        let mut imports = Vec::new();

        for item in &ast.items {
            if let TopLevel::Import(import_decl) = item {
                let mut info = ImportInfo {
                    symbols: import_decl.imports.clone(),
                    source: import_decl.source.clone(),
                    is_wildcard: import_decl.is_wildcard,
                    alias: import_decl.alias.clone(),
                    resolved_uri: None,
                };

                // Try to resolve the import path
                info.resolved_uri = self.resolve_import(&import_decl.source, current_file_uri);

                imports.push(info);
            }
        }

        imports
    }

    /// Resolve an import path to an absolute URI
    ///
    /// Handles:
    /// - Relative paths: "./math.liva", "../utils/helper.liva"
    /// - Absolute paths: "std/math", "lib/utils"
    pub fn resolve_import(&self, source: &str, current_file_uri: &Url) -> Option<Url> {
        // Handle relative paths (start with ./ or ../)
        if source.starts_with("./") || source.starts_with("../") {
            return self.resolve_relative_import(source, current_file_uri);
        }

        // Handle absolute paths (workspace-relative or standard library)
        self.resolve_absolute_import(source)
    }

    /// Resolve relative import path
    fn resolve_relative_import(&self, source: &str, current_file_uri: &Url) -> Option<Url> {
        // Get the directory of the current file
        let current_path = current_file_uri.to_file_path().ok()?;
        let current_dir = current_path.parent()?;

        // Resolve relative path
        let mut resolved_path = current_dir.to_path_buf();

        // Parse the relative path
        for component in source.split('/') {
            match component {
                "." => continue,
                ".." => {
                    resolved_path.pop();
                }
                part => resolved_path.push(part),
            }
        }

        // Add .liva extension if not present
        if resolved_path.extension().is_none() {
            resolved_path.set_extension("liva");
        }

        // Convert to URI
        Url::from_file_path(&resolved_path).ok()
    }

    /// Resolve absolute import path
    ///
    /// Tries to find the module in workspace roots
    fn resolve_absolute_import(&self, source: &str) -> Option<Url> {
        // Try each workspace root
        for root_uri in &self.workspace_roots {
            if let Ok(root_path) = root_uri.to_file_path() {
                let mut module_path = root_path.clone();

                // Build path from components
                for component in source.split('/') {
                    module_path.push(component);
                }

                // Add .liva extension if not present
                if module_path.extension().is_none() {
                    module_path.set_extension("liva");
                }

                // Check if file exists
                if module_path.exists() {
                    if let Ok(uri) = Url::from_file_path(&module_path) {
                        return Some(uri);
                    }
                }
            }
        }

        None
    }

    /// Check if a symbol is imported in a list of imports
    pub fn is_symbol_imported(&self, symbol_name: &str, imports: &[ImportInfo]) -> bool {
        for import in imports {
            if import.is_wildcard {
                return true;
            }
            if import.symbols.contains(&symbol_name.to_string()) {
                return true;
            }
        }
        false
    }

    /// Get the source file URI for an imported symbol
    pub fn get_import_source(&self, symbol_name: &str, imports: &[ImportInfo]) -> Option<Url> {
        for import in imports {
            if import.is_wildcard || import.symbols.contains(&symbol_name.to_string()) {
                return import.resolved_uri.clone();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)] // file:///workspace/... URIs require Unix-style paths
    fn test_relative_import_resolution() {
        let workspace_roots = vec![];
        let resolver = ImportResolver::new(workspace_roots);

        let current = Url::parse("file:///workspace/src/main.liva").unwrap();
        let resolved = resolver.resolve_import("./math.liva", &current);

        assert!(resolved.is_some());
        let resolved_uri = resolved.unwrap();
        assert_eq!(resolved_uri.path(), "/workspace/src/math.liva");
    }

    #[test]
    #[cfg(unix)] // file:///workspace/... URIs require Unix-style paths
    fn test_parent_directory_import() {
        let workspace_roots = vec![];
        let resolver = ImportResolver::new(workspace_roots);

        let current = Url::parse("file:///workspace/src/utils/helper.liva").unwrap();
        let resolved = resolver.resolve_import("../math.liva", &current);

        assert!(resolved.is_some());
        let resolved_uri = resolved.unwrap();
        assert_eq!(resolved_uri.path(), "/workspace/src/math.liva");
    }

    #[test]
    #[cfg(unix)] // file:///workspace/... URIs require Unix-style paths
    fn test_auto_extension() {
        let workspace_roots = vec![];
        let resolver = ImportResolver::new(workspace_roots);

        let current = Url::parse("file:///workspace/main.liva").unwrap();
        let resolved = resolver.resolve_import("./math", &current);

        assert!(resolved.is_some());
        let resolved_uri = resolved.unwrap();
        assert_eq!(resolved_uri.path(), "/workspace/math.liva");
    }
}
