# Language Server Protocol (LSP) Implementation Plan
## Phase 8.1 - Liva v0.12.0

> **Status:** Planning  
> **Estimated Time:** 8-10 hours  
> **Priority:** High - Production Readiness  
> **Branch:** `feature/lsp-v0.12.0`

---

## 🎯 Overview

Implement a complete Language Server Protocol (LSP) for Liva, providing professional IDE features including:
- Intelligent autocompletion
- Go to definition
- Find all references
- Symbol renaming
- Real-time diagnostics
- Hover information
- Signature help

This will transform Liva from a compiler-only language to a fully-featured development experience.

---

## 📋 Implementation Phases

### **Phase 1: LSP Infrastructure (2 hours)**

#### 1.1 Dependencies & Setup
- Add `tower-lsp` crate (LSP framework for Rust)
- Add `tokio` for async runtime
- Add `serde_json` for JSON-RPC
- Create `src/lsp/` module structure

#### 1.2 Basic LSP Server
```
src/lsp/
├── mod.rs           # Main LSP module
├── server.rs        # LSP server implementation
├── handlers.rs      # LSP request handlers
├── document.rs      # Document management
└── utils.rs         # Helper functions
```

**Files to create:**
- `src/lsp/mod.rs` - Module exports
- `src/lsp/server.rs` - `LivaLanguageServer` struct
- `src/lsp/document.rs` - Document state management
- `src/main.rs` - Add `--lsp` flag

**Key structures:**
```rust
pub struct LivaLanguageServer {
    client: Client,
    documents: DashMap<Url, DocumentState>,
    workspace: Arc<RwLock<WorkspaceState>>,
}

pub struct DocumentState {
    text: String,
    version: i32,
    ast: Option<Program>,
    diagnostics: Vec<Diagnostic>,
}
```

---

### **Phase 2: Document Synchronization (1 hour)**

#### 2.1 Text Document Lifecycle
Implement LSP notifications:
- `textDocument/didOpen` - Document opened in editor
- `textDocument/didChange` - Document content changed
- `textDocument/didSave` - Document saved
- `textDocument/didClose` - Document closed

#### 2.2 Document Management
- Maintain document state in memory
- Parse on change (incremental)
- Update diagnostics on change
- Handle multiple open documents

**Implementation:**
```rust
#[tower_lsp::async_trait]
impl LanguageServer for LivaLanguageServer {
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // Parse document
        // Store AST
        // Generate diagnostics
        // Send diagnostics to client
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Update document content
        // Reparse
        // Update diagnostics
    }
}
```

---

### **Phase 3: Real-time Diagnostics (1.5 hours)**

#### 3.1 Error Reporting
Convert compiler errors to LSP diagnostics:
- Lexer errors → Diagnostics
- Parser errors → Diagnostics
- Semantic errors → Diagnostics

#### 3.2 Diagnostic Mapping
```rust
fn compiler_error_to_diagnostic(error: &CompilerError) -> Diagnostic {
    Diagnostic {
        range: error_to_range(error),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(error.code()),
        message: error.message(),
        source: Some("liva".to_string()),
        ..Default::default()
    }
}
```

#### 3.3 Warnings
- Unused variables
- Unused imports
- Type inference hints
- Style suggestions

---

### **Phase 4: Autocompletion (2 hours)**

#### 4.1 Completion Provider
Implement `textDocument/completion`:
- Keywords (if, else, while, for, switch, etc.)
- Built-in types (int, string, bool, float, etc.)
- Local variables in scope
- Function names
- Class/interface names
- Import suggestions

#### 4.2 Context-Aware Completion
```rust
async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    
    // Get document state
    let doc = self.documents.get(&uri)?;
    
    // Find current scope
    let scope = find_scope_at_position(&doc.ast, position);
    
    // Generate completions based on context
    let completions = vec![
        // Keywords
        completion_item("if", CompletionItemKind::KEYWORD),
        completion_item("let", CompletionItemKind::KEYWORD),
        
        // Variables in scope
        ...scope.variables.iter().map(|v| 
            completion_item(v.name, CompletionItemKind::VARIABLE)
        ),
        
        // Functions
        ...scope.functions.iter().map(|f| 
            completion_item(f.name, CompletionItemKind::FUNCTION)
        ),
    ];
    
    Ok(CompletionResponse::Array(completions))
}
```

#### 4.3 Snippet Completions
- Function templates
- Class templates
- Control flow patterns

---

### **Phase 5: Go to Definition (1 hour)**

#### 5.1 Definition Provider
Implement `textDocument/definition`:
- Variables → Declaration site
- Functions → Function definition
- Classes → Class definition
- Interfaces → Interface definition
- Type aliases → Alias definition

#### 5.2 Symbol Resolution
```rust
async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<GotoDefinitionResponse> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    // Get symbol at position
    let symbol = find_symbol_at_position(&doc.ast, position)?;
    
    // Resolve definition location
    let definition_location = resolve_definition(&symbol, &workspace)?;
    
    Ok(GotoDefinitionResponse::Scalar(definition_location))
}
```

---

### **Phase 6: Find References (1 hour)**

#### 6.1 References Provider
Implement `textDocument/references`:
- Find all usages of a symbol
- Include definition (optional)
- Cross-file references

#### 6.2 Index Building
- Build symbol index on workspace load
- Update index on file changes
- Efficient lookup by symbol name

```rust
pub struct SymbolIndex {
    definitions: HashMap<String, Location>,
    references: HashMap<String, Vec<Location>>,
}

async fn find_references(&self, params: ReferenceParams) -> Result<Vec<Location>> {
    let symbol = find_symbol_at_position(...)?;
    let refs = self.symbol_index.get_references(&symbol.name);
    
    if params.context.include_declaration {
        refs.push(self.symbol_index.get_definition(&symbol.name));
    }
    
    Ok(refs)
}
```

---

### **Phase 7: Hover Information (0.5 hours)**

#### 7.1 Hover Provider
Implement `textDocument/hover`:
- Show type information
- Show documentation
- Show function signatures

```rust
async fn hover(&self, params: HoverParams) -> Result<Hover> {
    let symbol = find_symbol_at_position(...)?;
    
    let contents = match symbol.kind {
        SymbolKind::Variable => format!("```liva\nlet {}: {}\n```", 
            symbol.name, symbol.type_info),
        SymbolKind::Function => format!("```liva\n{}\n```\n\n{}", 
            symbol.signature, symbol.documentation),
        _ => symbol.name.clone(),
    };
    
    Ok(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: contents,
        }),
        range: Some(symbol.range),
    })
}
```

---

### **Phase 8: Rename Symbol (1 hour)**

#### 8.1 Rename Provider
Implement `textDocument/rename`:
- Find all references
- Generate workspace edit
- Apply changes across all files

```rust
async fn rename(&self, params: RenameParams) -> Result<WorkspaceEdit> {
    let old_name = find_symbol_at_position(...)?;
    let new_name = params.new_name;
    
    // Find all references
    let locations = find_all_references(&old_name)?;
    
    // Create text edits
    let mut changes = HashMap::new();
    for location in locations {
        let edit = TextEdit {
            range: location.range,
            new_text: new_name.clone(),
        };
        changes.entry(location.uri).or_insert_with(Vec::new).push(edit);
    }
    
    Ok(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}
```

---

### **Phase 9: VS Code Integration (1 hour)**

#### 9.1 Extension Updates
Update `vscode-extension/`:

**package.json:**
```json
{
  "activationEvents": [
    "onLanguage:liva"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "configuration": {
      "title": "Liva",
      "properties": {
        "liva.lsp.enabled": {
          "type": "boolean",
          "default": true,
          "description": "Enable Liva Language Server"
        },
        "liva.lsp.trace.server": {
          "type": "string",
          "enum": ["off", "messages", "verbose"],
          "default": "off"
        }
      }
    }
  }
}
```

**src/extension.ts:**
```typescript
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    // Start LSP client
    client = new LanguageClient(
        'liva',
        'Liva Language Server',
        {
            command: 'livac',
            args: ['--lsp']
        },
        {
            documentSelector: [{ scheme: 'file', language: 'liva' }]
        }
    );
    
    client.start();
}

export function deactivate() {
    return client?.stop();
}
```

#### 9.2 LSP Client Dependencies
```bash
npm install vscode-languageclient
```

---

## 📊 Success Criteria

### Functional Requirements
- ✅ LSP server starts and responds to initialize
- ✅ Documents sync correctly (open, change, close)
- ✅ Diagnostics appear in real-time
- ✅ Autocompletion works for keywords, variables, functions
- ✅ Go to definition navigates correctly
- ✅ Find references shows all usages
- ✅ Hover shows type information
- ✅ Rename updates all references

### Performance Requirements
- ✅ Completion response < 100ms
- ✅ Diagnostics update < 500ms after typing
- ✅ Go to definition < 50ms
- ✅ Memory usage < 100MB for typical project

### Quality Requirements
- ✅ No crashes with invalid input
- ✅ Handles concurrent requests
- ✅ Graceful error handling
- ✅ Comprehensive logging

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_completion_keywords() {
        let completions = get_completions_at_position(...);
        assert!(completions.contains("if"));
        assert!(completions.contains("let"));
    }
    
    #[test]
    fn test_goto_definition() {
        let location = find_definition("my_var", ...);
        assert_eq!(location.line, 5);
    }
}
```

### Integration Tests
- Test with real Liva files
- Test multi-file projects
- Test error scenarios

### Manual Testing
- Open Liva file in VS Code
- Type and verify completions
- Click go to definition
- Find references
- Rename symbols
- Verify diagnostics

---

## 📦 Dependencies

### Rust Crates
```toml
[dependencies]
tower-lsp = "0.20"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
dashmap = "5.5"
```

### VS Code Packages
```json
{
  "dependencies": {
    "vscode-languageclient": "^9.0.0"
  }
}
```

---

## 🚀 Rollout Plan

### Stage 1: Internal Testing
- Implement core LSP features
- Test with example files
- Fix critical bugs

### Stage 2: Alpha Release
- Release as v0.12.0-alpha
- Get feedback from early users
- Iterate on performance

### Stage 3: Stable Release
- v0.12.0 stable
- Update documentation
- Announce on GitHub

---

## 📝 Documentation Updates

### Files to Update
1. `docs/lsp/LSP_DESIGN.md` - Architecture & design
2. `docs/lsp/LSP_API.md` - API reference
3. `docs/lsp/LSP_USER_GUIDE.md` - User guide
4. `CHANGELOG.md` - Add v0.12.0 entry
5. `ROADMAP.md` - Mark Phase 8.1 complete
6. `README.md` - Add LSP features

### Documentation Sections
- Architecture overview
- Feature descriptions with examples
- Configuration options
- Troubleshooting guide
- Performance tips

---

## ⚠️ Known Limitations (v0.12.0)

1. **Single File Focus:** Initial version focuses on single-file projects
2. **No Workspace Symbols:** Workspace-wide symbol search in later version
3. **Limited Refactoring:** Only rename; extract/inline in future
4. **No Code Actions:** Quick fixes and refactorings in v0.13.0
5. **No Semantic Highlighting:** Token-based only in v0.12.0

---

## 🔮 Future Enhancements (v0.13.0+)

### Code Actions
- Quick fixes for common errors
- Extract function/variable
- Inline function/variable
- Add missing imports

### Advanced Features
- Semantic highlighting
- Call hierarchy
- Type hierarchy
- Document symbols
- Workspace symbols
- Code lens

### Performance
- Incremental parsing
- Parallel analysis
- Symbol cache
- Smart recompilation

---

## 📈 Metrics & Success Indicators

### Adoption Metrics
- % of users enabling LSP
- Average session duration
- Feature usage statistics

### Performance Metrics
- Response time percentiles (p50, p95, p99)
- Memory usage over time
- CPU usage

### Quality Metrics
- Crash rate
- Error rate
- Completion accuracy

---

## 🎯 Next Steps

1. Review this plan
2. Set up LSP infrastructure
3. Implement document sync
4. Add diagnostics
5. Implement completion
6. Add navigation features
7. VS Code integration
8. Testing & refinement
9. Documentation
10. Release v0.12.0

---

**Estimated Total Time:** 8-10 hours  
**Target Completion:** TBD  
**Dependencies:** None (can start immediately)
