use dashmap::DashMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::*;

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
