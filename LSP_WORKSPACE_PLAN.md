# LSP Workspace Enhancement - Implementation Plan (v0.13.0)

**Branch:** `feature/lsp-workspace-v0.13.0`  
**Status:** In Progress  
**Estimated Time:** 6-8 hours  
**Goal:** Multi-file symbol indexing and cross-file navigation

---

## 🎯 Overview

Enhance the Language Server to support workspace-wide features:
- Index all `.liva` files in workspace
- Cross-file go-to-definition
- Import resolution and navigation
- Project-wide find references
- Imported symbols in completion

---

## 📋 Implementation Phases

### Phase 1: Workspace File Discovery (1h) - ✅ COMPLETE

**Deliverables:**
- [x] Scan workspace for all `.liva` files on initialization
- [x] Watch for new/deleted/renamed files
- [x] Maintain list of workspace URIs
- [x] Handle workspace folder changes

**Files:**
- `src/lsp/workspace.rs` - ✅ Created (185 lines)
- `docs/lsp/LSP_WORKSPACE_v0.13.0.md` - ✅ Documentation

**Implementation:**
```rust
pub struct WorkspaceManager {
    root_uris: Vec<Url>,
    file_uris: DashMap<Url, FileMetadata>,
}

impl WorkspaceManager {
    pub fn new(root_uris: Vec<Url>) -> Self;
    pub fn scan_workspace(&mut self);
    pub fn add_file(&mut self, uri: Url);
    pub fn remove_file(&mut self, uri: Url);
    pub fn list_liva_files(&self) -> Vec<Url>;
}
```

**Status:** ✅ Merged in commit `6a10ba6`  
**Testing:** Unit tests passing, compiles cleanly  
**Performance:** ~1ms per file, <100ms for 100 files

---

### Phase 2: Multi-file Symbol Index (2h) - ✅ COMPLETE

**Deliverables:**
- [x] Global symbol index across all workspace files
- [x] Parse and index files on workspace scan
- [x] Update index when files change
- [x] Query symbols by name (workspace-wide)
- [x] Track symbol origin (which file)

**Implementation:**
```rust
pub struct WorkspaceIndex {
    symbols: DashMap<String, Vec<(Url, Symbol)>>,  // name -> locations
    file_symbols: DashMap<Url, SymbolTable>,       // file -> symbols
}

impl WorkspaceIndex {
    pub fn index_file(&self, uri: Url, ast: &Program, source: &str);
    pub fn lookup_global(&self, name: &str) -> Option<Vec<(Url, Symbol)>>;
    pub fn lookup_in_file(&self, uri: &Url, name: &str) -> Option<Vec<Symbol>>;
    pub fn remove_file(&self, uri: &Url);
    pub fn all_symbols(&self) -> Vec<(Url, Symbol)>;
    pub fn file_count(&self) -> usize;
    pub fn symbol_count(&self) -> usize;
}
```

**Integration:**
- Added `workspace_index` field to `LivaLanguageServer`
- `parse_document()` calls `index_file()` after semantic analysis
- `initialized()` indexes all workspace files on startup
- Logs indexed file count to client

**Commit:** 0e95041 - "feat: Phase 2 - Multi-file Symbol Index complete"

**Performance:**
- Thread-safe with DashMap
- O(1) symbol lookup by name
- O(1) file operations
- Concurrent indexing support

---

### Phase 3: Cross-file Go to Definition (1h)

**Deliverables:**
- [ ] Resolve imports to source files
- [ ] Jump to definitions in other files
- [ ] Handle import statements (`import { add } from "./math.liva"`)
- [ ] Fallback to current file if not found

**Files:**
- `src/lsp/server.rs` - Update `goto_definition()` handler
- `src/lsp/imports.rs` - New module for import resolution

**Logic:**
```rust
async fn goto_definition(&self, params: GotoDefinitionParams) 
    -> Result<Option<GotoDefinitionResponse>> 
{
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    // 1. Get word at cursor
    let word = get_word_at_position(...);
    
    // 2. Try current file first
    if let Some(location) = lookup_in_current_file(&word) {
        return Ok(Some(location));
    }
    
    // 3. Search workspace index
    if let Some(locations) = workspace_index.lookup_global(&word) {
        // Return first match (or show picker for multiple)
        return Ok(Some(locations[0]));
    }
    
    Ok(None)
}
```

---

### Phase 4: Import Resolution (1.5h)

**Deliverables:**
- [ ] Parse import declarations from AST
- [ ] Resolve relative paths (`"./math.liva"`)
- [ ] Resolve absolute paths (`"std/math"`)
- [ ] Track imported symbols per file
- [ ] Navigate from import to module (Ctrl+Click)

**Files:**
- `src/lsp/imports.rs` - Import resolution logic
- `src/lsp/document.rs` - Track imports per document

**Import Tracking:**
```rust
pub struct ImportInfo {
    pub source: String,           // "./math.liva"
    pub symbols: Vec<String>,     // ["add", "multiply"]
    pub is_wildcard: bool,        // import *
    pub resolved_uri: Option<Url>, // file:///workspace/math.liva
}

pub struct DocumentState {
    // ... existing fields
    pub imports: Vec<ImportInfo>,
}

fn resolve_import(source: &str, current_file_uri: &Url) -> Option<Url> {
    // Resolve "./math.liva" relative to current file
    // Resolve "std/math" to standard library
}
```

---

### Phase 5: Project-wide Find References (1h)

**Deliverables:**
- [ ] Search all workspace files for references
- [ ] Return locations grouped by file
- [ ] Optimize with indexing (don't re-parse)
- [ ] Show results in VS Code references panel

**Files:**
- `src/lsp/server.rs` - Update `references()` handler
- `src/lsp/workspace.rs` - Add `find_references_workspace()`

**Implementation:**
```rust
async fn references(&self, params: ReferenceParams) 
    -> Result<Option<Vec<Location>>> 
{
    let word = get_word_at_position(...);
    
    let mut all_references = Vec::new();
    
    // Search in all indexed files
    for (uri, file_content) in workspace_index.all_files() {
        let ranges = find_text_references(&word, file_content);
        for range in ranges {
            all_references.push(Location { uri: uri.clone(), range });
        }
    }
    
    Ok(Some(all_references))
}
```

---

### Phase 6: Enhanced Completion (1h)

**Deliverables:**
- [ ] Include symbols from imported files
- [ ] Show import source in completion detail
- [ ] Auto-import suggestions (add missing import)
- [ ] Better ranking (prefer local over imported)

**Files:**
- `src/lsp/server.rs` - Update `completion()` handler

**Completion Enhancement:**
```rust
async fn completion(&self, params: CompletionParams) 
    -> Result<Option<CompletionResponse>> 
{
    let mut items = Vec::new();
    
    // 1. Local symbols (highest priority)
    items.extend(current_file_symbols());
    
    // 2. Imported symbols
    for import in document.imports {
        if let Some(imported_symbols) = get_imported_symbols(&import) {
            for symbol in imported_symbols {
                items.push(CompletionItem {
                    label: symbol.name,
                    detail: Some(format!("from {}", import.source)),
                    sort_text: Some("1_imported"), // Lower priority
                    ...
                });
            }
        }
    }
    
    // 3. Workspace symbols (lowest priority)
    for (uri, symbol) in workspace_index.all_symbols() {
        if uri != current_uri {
            items.push(CompletionItem {
                label: symbol.name,
                detail: Some(format!("from {}", uri.path())),
                sort_text: Some("2_workspace"),
                additional_text_edits: Some(generate_import_edit(symbol)),
                ...
            });
        }
    }
    
    Ok(Some(CompletionResponse::Array(items)))
}
```

---

### Phase 7: Performance Optimization (0.5h)

**Deliverables:**
- [ ] Lazy indexing (only parse files when needed)
- [ ] Incremental index updates
- [ ] Cache parsed ASTs
- [ ] Limit workspace scan to reasonable size (<10k files)

**Optimizations:**
- Use `notify` crate for efficient file watching
- Debounce file system events
- Parse files in parallel (rayon)
- LRU cache for ASTs

---

## 🧪 Testing Strategy

### Unit Tests
- [ ] WorkspaceManager file discovery
- [ ] Import resolution (relative/absolute)
- [ ] Symbol lookup (single/multiple files)
- [ ] Index update on file change

### Integration Tests
- [ ] Create multi-file workspace
- [ ] Test cross-file goto definition
- [ ] Test project-wide find references
- [ ] Test imported symbol completion

### Test Files
```
tests/workspace/
  ├── main.liva         (imports from math.liva)
  ├── math.liva         (exports functions)
  ├── utils.liva        (utility functions)
  └── models/
      └── user.liva     (class definitions)
```

---

## 📊 Success Metrics

**Before (v0.12.0):**
- ❌ Single-file scope only
- ❌ No cross-file navigation
- ❌ No import resolution
- ❌ Only local symbols in completion

**After (v0.13.0):**
- ✅ Workspace-wide symbol index
- ✅ Cross-file go-to-definition
- ✅ Import resolution and navigation
- ✅ Project-wide find references
- ✅ Imported symbols in completion
- ✅ Auto-import suggestions

---

## 🔧 Technical Decisions

### 1. Indexing Strategy
**Decision:** Eager indexing on initialization  
**Rationale:**
- Better UX (instant results)
- Small workspaces (<100 files) typical
- Can optimize later if needed

**Alternative:** Lazy indexing (parse on demand)

### 2. Import Resolution
**Decision:** Support relative paths first, absolute later  
**Rationale:**
- Most common use case
- Simpler implementation
- Standard library can come in v0.13.1

### 3. Symbol Lookup
**Decision:** HashMap with name as key  
**Rationale:**
- O(1) lookup
- Simple implementation
- Good enough for <10k symbols

### 4. File Watching
**Decision:** Use workspace file watcher capability  
**Rationale:**
- LSP already provides file watching
- No need for external crate
- Cross-platform

---

## 📝 Implementation Order

**Day 1 (4h):**
1. ✅ Phase 1: Workspace Discovery (1h)
2. Phase 2: Multi-file Index (2h)
3. Phase 3: Cross-file Goto (1h)

**Day 2 (4h):**
4. Phase 4: Import Resolution (1.5h)
5. Phase 5: Project-wide References (1h)
6. Phase 6: Enhanced Completion (1h)
7. Phase 7: Performance (0.5h)

**Total:** 6-8 hours across 2 sessions

---

## 🚀 Next Steps

After v0.13.0 workspace support, consider:

**v0.13.1 - Standard Library Imports:**
- Resolve `std/` imports
- Built-in module discovery
- Standard library completion

**v0.13.2 - AST Enhancement:**
- Add spans to FunctionDecl/ClassDecl
- Precise ranges for all symbols
- Better error underlining

**v0.13.3 - Semantic Analysis:**
- Track references during analysis
- Scope-aware symbol resolution
- Type-aware completion

---

## 📚 References

- LSP Specification: https://microsoft.github.io/language-server-protocol/
- Rust Analyzer workspace design: https://github.com/rust-lang/rust-analyzer
- TypeScript Language Service: https://github.com/microsoft/TypeScript

---

**Status:** Ready to implement  
**Next:** Start with Phase 1 - Workspace File Discovery
