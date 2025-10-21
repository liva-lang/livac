//! Module System - Phase 3: Module Resolution
//! 
//! Handles loading, resolving, and validating multi-file Liva projects.

use crate::ast::{TopLevel, Program, ImportDecl};
use crate::error::{CompilerError, Result, SemanticErrorInfo};
use crate::parser::parse;
use crate::lexer::tokenize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::fs;

/// Represents a single Liva module (file)
#[derive(Debug, Clone)]
pub struct Module {
    /// Canonical path to the file
    pub path: PathBuf,
    
    /// Parsed AST
    pub ast: Program,
    
    /// Public symbols (exported) - functions, classes, constants without '_' prefix
    pub public_symbols: HashSet<String>,
    
    /// Private symbols (not exported) - functions, classes, constants with '_' prefix
    pub private_symbols: HashSet<String>,
    
    /// Import declarations in this module
    pub imports: Vec<ImportDecl>,
    
    /// Source code (for error reporting)
    pub source: String,
}

impl Module {
    /// Create a new module from a file path
    pub fn from_file(path: &Path) -> Result<Self> {
        // Read file
        let source = fs::read_to_string(path).map_err(|e| {
            CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4004",
                &format!("Cannot read module file: {}", path.display()),
                &format!("{}\nHint: Check that the file exists and you have read permissions.", e.to_string()),
            ))
        })?;
        
        // Lex and parse
        let tokens = tokenize(&source)?;
        let ast = parse(tokens, &source)?;
        
        // Extract symbols and imports
        let mut public_symbols = HashSet::new();
        let mut private_symbols = HashSet::new();
        let mut imports = Vec::new();
        
        for item in &ast.items {
            match item {
                TopLevel::Function(func) => {
                    if func.name.starts_with('_') {
                        private_symbols.insert(func.name.clone());
                    } else {
                        public_symbols.insert(func.name.clone());
                    }
                }
                TopLevel::Class(class) => {
                    if class.name.starts_with('_') {
                        private_symbols.insert(class.name.clone());
                    } else {
                        public_symbols.insert(class.name.clone());
                    }
                }
                TopLevel::Import(import_decl) => {
                    imports.push(import_decl.clone());
                }
                _ => {
                    // Type, UseRust, Test - ignore for now
                }
            }
        }
        
        Ok(Module {
            path: path.to_path_buf(),
            ast,
            public_symbols,
            private_symbols,
            imports,
            source,
        })
    }
    
    /// Get all symbol names (public + private)
    pub fn all_symbols(&self) -> HashSet<String> {
        self.public_symbols.union(&self.private_symbols).cloned().collect()
    }
}

/// Dependency graph for detecting cycles
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Edges: module_path -> [imported_module_paths]
    edges: HashMap<PathBuf, Vec<PathBuf>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }
    
    /// Add an edge from `from` module to `to` module
    pub fn add_edge(&mut self, from: PathBuf, to: PathBuf) {
        self.edges.entry(from).or_insert_with(Vec::new).push(to);
    }
    
    /// Detect if there's a cycle starting from `start_path`
    /// Returns Some(cycle_path) if cycle detected, None otherwise
    pub fn detect_cycle(&self, start_path: &Path) -> Option<Vec<PathBuf>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        if self.has_cycle_dfs(start_path, &mut visited, &mut path) {
            Some(path)
        } else {
            None
        }
    }
    
    fn has_cycle_dfs(
        &self,
        current: &Path,
        visited: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
    ) -> bool {
        // If already in current path, we found a cycle
        if path.contains(&current.to_path_buf()) {
            path.push(current.to_path_buf());
            return true;
        }
        
        // If already fully visited, no cycle here
        if visited.contains(&current.to_path_buf()) {
            return false;
        }
        
        path.push(current.to_path_buf());
        
        // Check all dependencies
        if let Some(deps) = self.edges.get(&current.to_path_buf()) {
            for dep in deps {
                if self.has_cycle_dfs(dep, visited, path) {
                    return true;
                }
            }
        }
        
        path.pop();
        visited.insert(current.to_path_buf());
        false
    }
    
    /// Get topological order for compilation (dependencies first)
    pub fn topological_sort(&self) -> Result<Vec<PathBuf>> {
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
        let mut result = Vec::new();
        
        // Initialize in-degrees
        for (from, deps) in &self.edges {
            in_degree.entry(from.clone()).or_insert(0);
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }
        
        // Queue of nodes with in-degree 0
        let mut queue: VecDeque<PathBuf> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(path, _)| path.clone())
            .collect();
        
        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            
            if let Some(deps) = self.edges.get(&node) {
                for dep in deps {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }
        
        // If we haven't visited all nodes, there's a cycle
        if result.len() != in_degree.len() {
            return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4003",
                "Circular dependency detected",
                "Cannot compile modules with circular dependencies.\nHint: Check your import statements to find the circular reference chain.",
            )));
        }
        
        Ok(result)
    }
}

/// Module Resolver - Loads and resolves multi-file projects
#[derive(Debug)]
pub struct ModuleResolver {
    /// All loaded modules: canonical_path -> Module
    modules: HashMap<PathBuf, Module>,
    
    /// Entry point file
    entry_point: PathBuf,
    
    /// Root directory of the project
    root_dir: PathBuf,
    
    /// Dependency graph
    dependency_graph: DependencyGraph,
}

impl ModuleResolver {
    /// Create a new resolver for a project
    pub fn new(entry_point: &Path) -> Result<Self> {
        let entry_point = entry_point.canonicalize().map_err(|e| {
            CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4004",
                &format!("Cannot find entry point: {}", entry_point.display()),
                &format!("{}\nHint: Verify the file path is correct.", e.to_string()),
            ))
        })?;
        
        let root_dir = entry_point
            .parent()
            .ok_or_else(|| {
                CompilerError::CodegenError(SemanticErrorInfo::new(
                    "E4004",
                    "Cannot determine project root directory",
                    "Entry point file has no parent directory.",
                ))
            })?
            .to_path_buf();
        
        Ok(Self {
            modules: HashMap::new(),
            entry_point,
            root_dir,
            dependency_graph: DependencyGraph::new(),
        })
    }
    
    /// Resolve all modules starting from entry point
    pub fn resolve_all(&mut self) -> Result<Vec<&Module>> {
        // Load entry point
        self.load_module_recursive(&self.entry_point.clone())?;
        
        // Check for cycles
        if let Some(cycle) = self.dependency_graph.detect_cycle(&self.entry_point) {
            let cycle_str = cycle
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  → ");
            
            return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4003",
                "Circular dependency detected",
                &format!("Import chain forms a cycle:\n  {}\n\nHint: Remove one of the imports in this chain to break the circular dependency.", cycle_str),
            )));
        }
        
        // Get compilation order (topological sort)
        let order = self.dependency_graph.topological_sort()?;
        
        // Return modules in compilation order
        let modules: Vec<&Module> = order
            .iter()
            .filter_map(|path| self.modules.get(path))
            .collect();
        
        Ok(modules)
    }
    
    /// Load a module and all its dependencies recursively
    fn load_module_recursive(&mut self, path: &Path) -> Result<()> {
        // Canonicalize path
        let canonical_path = path.canonicalize().map_err(|e| {
            CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4004",
                &format!("Cannot find module: {}", path.display()),
                &format!("{}\nHint: Check that the import path is correct and the file exists.", e.to_string()),
            ))
        })?;
        
        // Check if already loaded (avoid re-parsing)
        if self.modules.contains_key(&canonical_path) {
            return Ok(());
        }
        
        // Load the module
        let module = Module::from_file(&canonical_path)?;
        
        // Process imports
        let imports = module.imports.clone();
        self.modules.insert(canonical_path.clone(), module);
        
        // Recursively load imported modules
        for import in imports {
            let imported_path = self.resolve_import_path(&canonical_path, &import.source)?;
            
            // Add edge to dependency graph
            self.dependency_graph.add_edge(canonical_path.clone(), imported_path.clone());
            
            // Recursively load
            self.load_module_recursive(&imported_path)?;
        }
        
        Ok(())
    }
    
    /// Resolve an import path relative to the current file
    fn resolve_import_path(&self, current_file: &Path, import_path: &str) -> Result<PathBuf> {
        // Get the directory of the current file
        let current_dir = current_file.parent().ok_or_else(|| {
            CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4004",
                "Cannot resolve import path",
                &format!("Current file has no parent directory: {}", current_file.display()),
            ))
        })?;
        
        // Resolve relative path
        let resolved = if import_path.starts_with("./") || import_path.starts_with("../") {
            // Relative path
            current_dir.join(import_path)
        } else {
            // For now, treat as relative to current directory
            current_dir.join(import_path)
        };
        
        // Check if file exists
        if !resolved.exists() {
            return Err(CompilerError::CodegenError(SemanticErrorInfo::new(
                "E4004",
                &format!("Module not found: '{}'", import_path),
                &format!("File does not exist: {}\nHint: Check the import path. Relative paths should start with './' or '../'.", resolved.display()),
            )));
        }
        
        Ok(resolved)
    }
    
    /// Get a module by path
    pub fn get_module(&self, path: &Path) -> Option<&Module> {
        self.modules.get(path)
    }
    
    /// Get all modules
    pub fn modules(&self) -> &HashMap<PathBuf, Module> {
        &self.modules
    }
    
    /// Get entry point module
    pub fn entry_module(&self) -> Option<&Module> {
        self.modules.get(&self.entry_point)
    }
    
    /// Get compilation order (topological sort)
    pub fn compilation_order(&self) -> Result<Vec<PathBuf>> {
        self.dependency_graph.topological_sort()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dependency_graph_no_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(PathBuf::from("a.liva"), PathBuf::from("b.liva"));
        graph.add_edge(PathBuf::from("b.liva"), PathBuf::from("c.liva"));
        
        assert!(graph.detect_cycle(&PathBuf::from("a.liva")).is_none());
    }
    
    #[test]
    fn test_dependency_graph_direct_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(PathBuf::from("a.liva"), PathBuf::from("b.liva"));
        graph.add_edge(PathBuf::from("b.liva"), PathBuf::from("a.liva"));
        
        assert!(graph.detect_cycle(&PathBuf::from("a.liva")).is_some());
    }
    
    #[test]
    fn test_dependency_graph_indirect_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(PathBuf::from("a.liva"), PathBuf::from("b.liva"));
        graph.add_edge(PathBuf::from("b.liva"), PathBuf::from("c.liva"));
        graph.add_edge(PathBuf::from("c.liva"), PathBuf::from("a.liva"));
        
        assert!(graph.detect_cycle(&PathBuf::from("a.liva")).is_some());
    }
}
