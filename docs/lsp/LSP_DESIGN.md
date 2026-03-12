# Liva Language Server Protocol (LSP) - Architecture & Design
## Version 0.12.0

> **Document Status:** Design Phase  
> **Last Updated:** 2025-10-27  
> **Authors:** Liva Core Team

---

## 📐 Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────┐
│                    VS Code Editor                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │         Liva VS Code Extension                  │   │
│  │  - Language client                              │   │
│  │  - UI integration                               │   │
│  │  - Configuration                                │   │
│  └──────────────────┬──────────────────────────────┘   │
│                     │ JSON-RPC over stdio               │
└─────────────────────┼─────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│           Liva Language Server (livac lsp)             │
│  ┌─────────────────────────────────────────────────┐   │
│  │  LSP Protocol Handler (tower-lsp)               │   │
│  │  - Initialize                                   │   │
│  │  - Document lifecycle                           │   │
│  │  - Feature requests                             │   │
│  └──────────────────┬──────────────────────────────┘   │
│                     │                                   │
│  ┌──────────────────▼──────────────────────────────┐   │
│  │  Document Manager                               │   │
│  │  - In-memory document store                     │   │
│  │  - Version tracking                             │   │
│  │  - Change buffering                             │   │
│  └──────────────────┬──────────────────────────────┘   │
│                     │                                   │
│  ┌──────────────────▼──────────────────────────────┐   │
│  │  Liva Compiler Pipeline                         │   │
│  │  ┌────────┐  ┌─────────┐  ┌────────────────┐  │   │
│  │  │ Lexer  │→ │ Parser  │→ │ Semantic       │  │   │
│  │  └────────┘  └─────────┘  │ Analyzer       │  │   │
│  │                            └────────────────┘  │   │
│  └──────────────────┬──────────────────────────────┘   │
│                     │                                   │
│  ┌──────────────────▼──────────────────────────────┐   │
│  │  Feature Providers                              │   │
│  │  - Diagnostics     - Completion                 │   │
│  │  - Go to Def       - Find Refs                  │   │
│  │  - Hover           - Rename                     │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## 🏗️ Core Design Principles

### 1. **Separation of Concerns**
- LSP layer handles protocol communication
- Compiler layer handles language analysis
- Clean interfaces between layers

### 2. **Performance First**
- Incremental updates
- Lazy evaluation
- Efficient caching
- Parallel processing where possible

### 3. **Robustness**
- Never crash on invalid input
- Graceful degradation
- Comprehensive error handling
- Defensive programming

### 4. **Extensibility**
- Plugin-ready architecture
- Feature flags
- Modular design
- Easy to add new capabilities

---

## 📦 Module Structure

```
src/
├── main.rs                 # Entry point, --lsp flag handling
├── lsp/
│   ├── mod.rs             # LSP module exports
│   ├── server.rs          # Main LSP server struct
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── lifecycle.rs   # initialize, shutdown
│   │   ├── document.rs    # didOpen, didChange, didSave
│   │   ├── completion.rs  # textDocument/completion
│   │   ├── definition.rs  # textDocument/definition
│   │   ├── references.rs  # textDocument/references
│   │   ├── hover.rs       # textDocument/hover
│   │   └── rename.rs      # textDocument/rename
│   ├── document.rs        # Document state management
│   ├── diagnostics.rs     # Diagnostic conversion
│   ├── symbols.rs         # Symbol index
│   └── utils.rs           # Helper functions
├── lexer.rs               # Existing lexer
├── parser.rs              # Existing parser
├── semantic.rs            # Existing semantic analyzer
└── codegen.rs             # Existing code generator
```

---

## 🔄 Request Flow

### Document Change Flow
```
1. User types in editor
2. VS Code sends textDocument/didChange
3. LSP server receives notification
4. Update document in memory
5. Lex → Parse → Analyze
6. Generate diagnostics
7. Send diagnostics to client
8. Client displays errors/warnings
```

### Completion Request Flow
```
1. User triggers completion (Ctrl+Space)
2. VS Code sends textDocument/completion
3. LSP server receives request
4. Find cursor position in AST
5. Determine context (scope, expected type)
6. Generate completion items
7. Send response to client
8. Client displays completion menu
```

### Go to Definition Flow
```
1. User clicks "Go to Definition"
2. VS Code sends textDocument/definition
3. LSP server receives request
4. Find symbol at cursor position
5. Look up definition in symbol table
6. Return location (file + position)
7. Client navigates to location
```

---

## 💾 Data Structures

### LivaLanguageServer
```rust
pub struct LivaLanguageServer {
    /// LSP client handle
    client: Client,
    
    /// Open documents: URI → DocumentState
    documents: DashMap<Url, DocumentState>,
    
    /// Workspace-wide state
    workspace: Arc<RwLock<WorkspaceState>>,
    
    /// Configuration
    config: Arc<RwLock<LspConfig>>,
}
```

### DocumentState
```rust
pub struct DocumentState {
    /// Current document text
    text: String,
    
    /// Document version (increments on change)
    version: i32,
    
    /// Parsed AST (cached)
    ast: Option<Program>,
    
    /// Semantic analysis results
    symbols: Option<SymbolTable>,
    
    /// Current diagnostics
    diagnostics: Vec<Diagnostic>,
    
    /// Last parse timestamp
    last_parsed: Instant,
}
```

### SymbolTable
```rust
pub struct SymbolTable {
    /// Symbol name → Symbol info
    symbols: HashMap<String, Symbol>,
    
    /// Position → Symbol (for quick lookup)
    position_map: BTreeMap<Position, String>,
    
    /// Scopes (nested)
    scopes: Vec<Scope>,
}

pub struct Symbol {
    name: String,
    kind: SymbolKind,
    type_info: Option<String>,
    definition_location: Location,
    references: Vec<Location>,
    documentation: Option<String>,
}

pub enum SymbolKind {
    Variable,
    Function,
    Class,
    Interface,
    TypeAlias,
    Parameter,
    Field,
}
```

### WorkspaceState
```rust
pub struct WorkspaceState {
    /// Root path
    root_uri: Option<Url>,
    
    /// All workspace files
    files: HashMap<Url, FileInfo>,
    
    /// Global symbol index
    global_symbols: SymbolIndex,
    
    /// Dependency graph
    dependencies: HashMap<Url, Vec<Url>>,
}
```

---

## 🔌 LSP Capabilities

### Supported in v0.12.0

| Capability | Status | Priority |
|------------|--------|----------|
| **Text Synchronization** | ✅ Full | P0 |
| textDocument/didOpen | ✅ | P0 |
| textDocument/didChange | ✅ Incremental | P0 |
| textDocument/didSave | ✅ | P0 |
| textDocument/didClose | ✅ | P0 |
| **Diagnostics** | ✅ Full | P0 |
| publishDiagnostics | ✅ | P0 |
| **Completion** | ✅ Full | P0 |
| textDocument/completion | ✅ | P0 |
| completionItem/resolve | ⏳ v0.12.1 | P1 |
| **Navigation** | ✅ Full | P0 |
| textDocument/definition | ✅ | P0 |
| textDocument/references | ✅ | P0 |
| **Information** | ✅ Partial | P1 |
| textDocument/hover | ✅ | P0 |
| textDocument/signatureHelp | ⏳ v0.12.1 | P1 |
| **Refactoring** | ✅ Partial | P1 |
| textDocument/rename | ✅ | P0 |
| textDocument/codeAction | ⏳ v0.13.0 | P2 |

### Planned for Future Versions

| Capability | Version | Priority |
|------------|---------|----------|
| Semantic highlighting | v0.12.1 | P1 |
| Document symbols | v0.12.1 | P1 |
| Workspace symbols | v0.13.0 | P2 |
| Code actions | v0.13.0 | P2 |
| Code lens | v0.13.0 | P2 |
| Call hierarchy | v0.14.0 | P3 |
| Type hierarchy | v0.14.0 | P3 |

---

## ⚡ Performance Considerations

### Caching Strategy
1. **AST Caching:** Cache parsed AST per document
2. **Symbol Caching:** Cache symbol table after analysis
3. **Incremental Updates:** Only reparse changed regions
4. **Lazy Loading:** Load files on demand

### Optimization Techniques
- Use `DashMap` for concurrent document access
- Parallel document parsing where possible
- Debounce rapid changes (300ms)
- Background thread for heavy operations

### Memory Management
- Limit cached ASTs (max 50 documents)
- Clear diagnostics on close
- Periodic garbage collection
- Smart pruning of old data

---

## 🛡️ Error Handling Strategy

### Error Categories
1. **Protocol Errors:** Invalid JSON-RPC
2. **Parse Errors:** Invalid Liva syntax
3. **Semantic Errors:** Type errors, undefined symbols
4. **Internal Errors:** Server crashes, OOM

### Error Responses
```rust
// Always return graceful responses
async fn completion(&self, params: CompletionParams) 
    -> Result<Option<CompletionResponse>> 
{
    match self.get_completions(params) {
        Ok(items) => Ok(Some(CompletionResponse::Array(items))),
        Err(e) => {
            error!("Completion error: {}", e);
            Ok(None)  // Return empty instead of error
        }
    }
}
```

### Logging
- `TRACE`: Detailed protocol messages
- `DEBUG`: Internal operations
- `INFO`: Major events (initialize, document open)
- `WARN`: Recoverable errors
- `ERROR`: Critical failures

---

## 🔐 Security Considerations

### Input Validation
- Validate all URIs
- Sanitize file paths
- Limit document size (10MB max)
- Timeout long operations (5s max)

### Resource Limits
- Max open documents: 100
- Max completion items: 1000
- Max references: 10,000
- Memory limit: 500MB

### Sandboxing
- No arbitrary code execution
- No file system access outside workspace
- No network access
- Safe Rust (no unsafe blocks in LSP code)

---

## 📊 Monitoring & Telemetry

### Metrics to Track
- Request counts by type
- Response times (p50, p95, p99)
- Error rates
- Memory usage
- Cache hit rates

### Health Checks
- Periodic self-test
- Resource usage monitoring
- Crash detection & recovery
- Automatic restart on failure

---

## 🧪 Testing Strategy

### Unit Tests
- Individual feature providers
- Symbol resolution
- Diagnostic conversion
- Position mapping

### Integration Tests
- Full LSP request/response cycle
- Multi-document scenarios
- Error recovery
- Performance benchmarks

### End-to-End Tests
- VS Code extension testing
- Real-world project testing
- User acceptance testing

---

## 🚀 Deployment Strategy

### Build Process
```bash
# Release build with optimizations
cargo build --release --bin livac

# Copy to extension
cp target/release/livac vscode-extension/bin/
```

### Distribution
- Bundle with VS Code extension
- Platform-specific binaries (Linux, macOS, Windows)
- Auto-update mechanism
- Fallback to manual download

---

## 📝 Configuration

### Server Configuration
```json
{
  "liva.lsp.enabled": true,
  "liva.lsp.trace.server": "off",
  "liva.lsp.maxNumberOfProblems": 100,
  "liva.lsp.completionTriggerCharacters": [".", ":"],
  "liva.lsp.diagnostics.debounceMs": 300
}
```

### Initialization Options
```json
{
  "workspaceFolders": [...],
  "capabilities": {
    "textDocument": {
      "completion": { "dynamicRegistration": true },
      "hover": { "contentFormat": ["markdown"] }
    }
  }
}
```

---

## 🔮 Future Directions

### v0.13.0: Advanced Refactoring
- Extract function/variable
- Inline function/variable
- Move to new file
- Organize imports

### v0.14.0: Advanced Navigation
- Call hierarchy
- Type hierarchy
- Document outline
- Breadcrumb navigation

### v0.15.0: AI-Assisted Features
- Smart completions
- Code generation
- Refactoring suggestions
- Documentation generation

---

## 📚 References

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp Documentation](https://docs.rs/tower-lsp/)
- [VS Code Extension API](https://code.visualstudio.com/api)
- [Rust Analyzer LSP](https://github.com/rust-lang/rust-analyzer) (reference implementation)

---

**Document Version:** 1.0  
**Target Release:** v0.12.0  
**Status:** Ready for Implementation
