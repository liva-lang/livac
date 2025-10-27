# Liva Language Server Protocol - API Reference

> **Version:** 0.12.0  
> **Audience:** LSP contributors and maintainers

---

## 📋 Table of Contents

1. [Server API](#server-api)
2. [Handler APIs](#handler-apis)
3. [Data Structures](#data-structures)
4. [Symbol Management](#symbol-management)
5. [Document Management](#document-management)
6. [Diagnostic System](#diagnostic-system)
7. [Utility Functions](#utility-functions)
8. [Extension Points](#extension-points)

---

## 🖥️ Server API

### `LivaLanguageServer`

Main LSP server implementation.

```rust
pub struct LivaLanguageServer {
    client: Client,
    documents: DashMap<Url, DocumentState>,
    workspace: Arc<RwLock<WorkspaceState>>,
    config: Arc<RwLock<LspConfig>>,
}

impl LivaLanguageServer {
    /// Creates a new language server instance
    pub fn new(client: Client) -> Self;
    
    /// Gets document state (read-only)
    pub fn get_document(&self, uri: &Url) -> Option<Ref<Url, DocumentState>>;
    
    /// Gets mutable document state
    pub fn get_document_mut(&self, uri: &Url) -> Option<RefMut<Url, DocumentState>>;
    
    /// Parses a document and updates AST
    pub async fn parse_document(&self, uri: &Url) -> Result<(), LspError>;
    
    /// Publishes diagnostics to client
    pub async fn publish_diagnostics(&self, uri: &Url);
}
```

**Initialization:**
```rust
let (service, socket) = LspService::build(|client| {
    LivaLanguageServer::new(client)
})
.finish();

Server::new(stdin(), stdout(), socket).serve(service).await;
```

---

## 🔧 Handler APIs

### Lifecycle Handlers

#### `initialize`
```rust
async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult>;
```

**Parameters:**
- `params.root_uri`: Workspace root
- `params.capabilities`: Client capabilities
- `params.initialization_options`: Custom config

**Returns:**
```rust
InitializeResult {
    capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(Full)),
        completion_provider: Some(CompletionOptions { ... }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Left(true)),
        ...
    },
    server_info: Some(ServerInfo {
        name: "liva-language-server".to_string(),
        version: Some("0.12.0".to_string()),
    }),
}
```

#### `initialized`
```rust
async fn initialized(&self, _params: InitializedParams);
```

Called after initialization. Use for:
- Loading workspace configuration
- Registering dynamic capabilities
- Starting background tasks

#### `shutdown`
```rust
async fn shutdown(&self) -> Result<()>;
```

Cleanup before server exits.

---

### Document Synchronization Handlers

#### `did_open`
```rust
async fn did_open(&self, params: DidOpenTextDocumentParams);
```

**Parameters:**
```rust
pub struct DidOpenTextDocumentParams {
    pub text_document: TextDocumentItem {
        uri: Url,
        language_id: String,
        version: i32,
        text: String,
    }
}
```

**Implementation:**
```rust
async fn did_open(&self, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri;
    let text = params.text_document.text;
    let version = params.text_document.version;
    
    // Store document
    self.documents.insert(uri.clone(), DocumentState {
        text: text.clone(),
        version,
        ast: None,
        symbols: None,
        diagnostics: vec![],
        last_parsed: Instant::now(),
    });
    
    // Parse and publish diagnostics
    self.parse_document(&uri).await;
    self.publish_diagnostics(&uri).await;
}
```

#### `did_change`
```rust
async fn did_change(&self, params: DidChangeTextDocumentParams);
```

**Parameters:**
```rust
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

pub struct TextDocumentContentChangeEvent {
    pub range: Option<Range>,  // None = full document
    pub text: String,
}
```

**Incremental vs Full Sync:**
```rust
if let Some(range) = change.range {
    // Incremental: Apply diff
    doc.apply_change(range, &change.text);
} else {
    // Full: Replace everything
    doc.text = change.text.clone();
}
```

#### `did_save`
```rust
async fn did_save(&self, params: DidSaveTextDocumentParams);
```

Trigger full reparse on save (optional).

#### `did_close`
```rust
async fn did_close(&self, params: DidCloseTextDocumentParams);
```

Clean up document state.

---

### Completion Handler

#### `completion`
```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>>;
```

**Parameters:**
```rust
pub struct CompletionParams {
    pub text_document_position: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier,
        position: Position,
    },
    pub context: Option<CompletionContext>,
}
```

**Returns:**
```rust
pub enum CompletionResponse {
    Array(Vec<CompletionItem>),
    List(CompletionList {
        is_incomplete: bool,
        items: Vec<CompletionItem>,
    }),
}

pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,  // Variable, Function, Class, etc.
    pub detail: Option<String>,
    pub documentation: Option<Documentation>,
    pub insert_text: Option<String>,
    pub insert_text_format: Option<InsertTextFormat>,  // PlainText or Snippet
    ...
}
```

**Example:**
```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    
    let doc = self.get_document(uri).ok_or(LspError::DocumentNotFound)?;
    let symbols = doc.symbols.as_ref().ok_or(LspError::NotParsed)?;
    
    let mut items = vec![];
    
    // Keywords
    for keyword in KEYWORDS {
        items.push(CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }
    
    // Variables in scope
    for (name, symbol) in symbols.in_scope(position) {
        items.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(format!("{}", symbol.type_)),
            ..Default::default()
        });
    }
    
    Ok(Some(CompletionResponse::Array(items)))
}
```

---

### Definition Handler

#### `goto_definition`
```rust
async fn goto_definition(&self, params: GotoDefinitionParams) 
    -> Result<Option<GotoDefinitionResponse>>;
```

**Returns:**
```rust
pub enum GotoDefinitionResponse {
    Scalar(Location),
    Array(Vec<Location>),
    Link(Vec<LocationLink>),
}

pub struct Location {
    pub uri: Url,
    pub range: Range,
}
```

**Example:**
```rust
async fn goto_definition(&self, params: GotoDefinitionParams) 
    -> Result<Option<GotoDefinitionResponse>> 
{
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    let doc = self.get_document(uri).ok_or(LspError::DocumentNotFound)?;
    let symbols = doc.symbols.as_ref().ok_or(LspError::NotParsed)?;
    
    // Find symbol at position
    if let Some(symbol_name) = doc.word_at_position(position) {
        if let Some(symbol) = symbols.lookup(&symbol_name) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: symbol.definition_uri.clone(),
                range: symbol.definition_range,
            })));
        }
    }
    
    Ok(None)
}
```

---

### References Handler

#### `references`
```rust
async fn references(&self, params: ReferenceParams) 
    -> Result<Option<Vec<Location>>>;
```

**Parameters:**
```rust
pub struct ReferenceParams {
    pub text_document_position: TextDocumentPositionParams,
    pub context: ReferenceContext {
        include_declaration: bool,
    },
}
```

**Example:**
```rust
async fn references(&self, params: ReferenceParams) 
    -> Result<Option<Vec<Location>>> 
{
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let include_decl = params.context.include_declaration;
    
    let doc = self.get_document(uri).ok_or(LspError::DocumentNotFound)?;
    let symbols = doc.symbols.as_ref().ok_or(LspError::NotParsed)?;
    
    if let Some(symbol_name) = doc.word_at_position(position) {
        let mut locations = vec![];
        
        // Search all documents
        for entry in self.documents.iter() {
            let (doc_uri, doc_state) = entry.pair();
            if let Some(doc_symbols) = &doc_state.symbols {
                let refs = doc_symbols.find_references(&symbol_name);
                
                for ref_range in refs {
                    // Skip definition if not requested
                    if !include_decl && is_definition(ref_range) {
                        continue;
                    }
                    
                    locations.push(Location {
                        uri: doc_uri.clone(),
                        range: ref_range,
                    });
                }
            }
        }
        
        return Ok(Some(locations));
    }
    
    Ok(None)
}
```

---

### Hover Handler

#### `hover`
```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>>;
```

**Returns:**
```rust
pub struct Hover {
    pub contents: HoverContents,
    pub range: Option<Range>,
}

pub enum HoverContents {
    Scalar(MarkedString),
    Array(Vec<MarkedString>),
    Markup(MarkupContent),
}

pub struct MarkupContent {
    pub kind: MarkupKind,  // PlainText or Markdown
    pub value: String,
}
```

**Example:**
```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;
    
    let doc = self.get_document(uri).ok_or(LspError::DocumentNotFound)?;
    let symbols = doc.symbols.as_ref().ok_or(LspError::NotParsed)?;
    
    if let Some(symbol_name) = doc.word_at_position(position) {
        if let Some(symbol) = symbols.lookup(&symbol_name) {
            let hover_text = match &symbol.kind {
                SymbolKind::Variable => {
                    format!("```liva\nlet {}: {}\n```", symbol_name, symbol.type_)
                }
                SymbolKind::Function { params, return_type } => {
                    format!(
                        "```liva\nfn {}({}) -> {}\n```\n\n{}",
                        symbol_name,
                        params.join(", "),
                        return_type,
                        symbol.documentation.as_deref().unwrap_or("")
                    )
                }
                _ => format!("{}: {}", symbol_name, symbol.type_),
            };
            
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: Some(symbol.range),
            }));
        }
    }
    
    Ok(None)
}
```

---

### Rename Handler

#### `rename`
```rust
async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>>;
```

**Parameters:**
```rust
pub struct RenameParams {
    pub text_document_position: TextDocumentPositionParams,
    pub new_name: String,
}
```

**Returns:**
```rust
pub struct WorkspaceEdit {
    pub changes: Option<HashMap<Url, Vec<TextEdit>>>,
    pub document_changes: Option<Vec<DocumentChanges>>,
}

pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}
```

**Example:**
```rust
async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let new_name = params.new_name;
    
    // Validate new name
    if !is_valid_identifier(&new_name) {
        return Err(LspError::InvalidIdentifier);
    }
    
    let doc = self.get_document(uri).ok_or(LspError::DocumentNotFound)?;
    
    if let Some(symbol_name) = doc.word_at_position(position) {
        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        
        // Find all references across workspace
        for entry in self.documents.iter() {
            let (doc_uri, doc_state) = entry.pair();
            if let Some(symbols) = &doc_state.symbols {
                let refs = symbols.find_references(&symbol_name);
                
                let edits: Vec<TextEdit> = refs.into_iter()
                    .map(|range| TextEdit {
                        range,
                        new_text: new_name.clone(),
                    })
                    .collect();
                
                if !edits.is_empty() {
                    changes.insert(doc_uri.clone(), edits);
                }
            }
        }
        
        return Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
        }));
    }
    
    Ok(None)
}
```

---

## 📦 Data Structures

### `DocumentState`

```rust
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
    /// Gets text in a specific range
    pub fn text_in_range(&self, range: Range) -> &str;
    
    /// Gets word at cursor position
    pub fn word_at_position(&self, position: Position) -> Option<String>;
    
    /// Applies incremental text change
    pub fn apply_change(&mut self, range: Range, text: &str);
    
    /// Converts position to byte offset
    pub fn position_to_offset(&self, position: Position) -> usize;
    
    /// Converts byte offset to position
    pub fn offset_to_position(&self, offset: usize) -> Position;
}
```

---

### `SymbolTable`

```rust
pub struct SymbolTable {
    /// All symbols by name
    symbols: HashMap<String, Symbol>,
    
    /// Position index for fast lookup
    position_index: BTreeMap<Position, String>,
    
    /// Scope hierarchy
    scopes: Vec<Scope>,
}

pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub type_: Type,
    pub definition_uri: Url,
    pub definition_range: Range,
    pub references: Vec<Location>,
    pub documentation: Option<String>,
    pub scope_id: ScopeId,
}

pub enum SymbolKind {
    Variable,
    Constant,
    Function { params: Vec<Param>, return_type: Type },
    Class { fields: Vec<Field>, methods: Vec<Method> },
    Interface { methods: Vec<Method> },
    TypeAlias { aliased_type: Type },
    Parameter,
}

impl SymbolTable {
    /// Creates empty symbol table
    pub fn new() -> Self;
    
    /// Adds a symbol
    pub fn insert(&mut self, symbol: Symbol);
    
    /// Looks up symbol by name
    pub fn lookup(&self, name: &str) -> Option<&Symbol>;
    
    /// Gets all symbols in scope at position
    pub fn in_scope(&self, position: Position) -> Vec<&Symbol>;
    
    /// Finds all references to a symbol
    pub fn find_references(&self, name: &str) -> Vec<Range>;
    
    /// Gets symbol at exact position
    pub fn symbol_at_position(&self, position: Position) -> Option<&Symbol>;
}
```

---

### `WorkspaceState`

```rust
pub struct WorkspaceState {
    /// Root directory
    pub root: PathBuf,
    
    /// All Liva files in workspace
    pub files: HashSet<PathBuf>,
    
    /// Cross-file symbol index
    pub global_symbols: SymbolTable,
    
    /// Module dependencies
    pub dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

impl WorkspaceState {
    /// Scans workspace for Liva files
    pub async fn scan(&mut self) -> Result<()>;
    
    /// Resolves import path
    pub fn resolve_import(&self, from: &Path, import: &str) -> Option<PathBuf>;
    
    /// Gets all symbols exported by a module
    pub fn exported_symbols(&self, module: &Path) -> Vec<&Symbol>;
}
```

---

### `LspConfig`

```rust
pub struct LspConfig {
    /// Maximum diagnostics per file
    pub max_problems: usize,
    
    /// Debounce time for diagnostics (ms)
    pub diagnostics_debounce_ms: u64,
    
    /// Enable/disable features
    pub features: FeatureFlags,
    
    /// Trace level
    pub trace: TraceLevel,
}

pub struct FeatureFlags {
    pub completion: bool,
    pub hover: bool,
    pub definition: bool,
    pub references: bool,
    pub rename: bool,
}

pub enum TraceLevel {
    Off,
    Messages,
    Verbose,
}
```

---

## 🔍 Symbol Management

### Building Symbol Table

```rust
pub fn build_symbol_table(ast: &Program, uri: &Url) -> SymbolTable {
    let mut table = SymbolTable::new();
    let mut visitor = SymbolVisitor::new(&mut table, uri);
    visitor.visit_program(ast);
    table
}

struct SymbolVisitor<'a> {
    table: &'a mut SymbolTable,
    uri: &'a Url,
    current_scope: ScopeId,
}

impl<'a> SymbolVisitor<'a> {
    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, type_, value, span } => {
                self.table.insert(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Variable,
                    type_: type_.clone(),
                    definition_uri: self.uri.clone(),
                    definition_range: span_to_range(*span),
                    references: vec![],
                    documentation: None,
                    scope_id: self.current_scope,
                });
            }
            Statement::Function { name, params, return_type, body, span } => {
                // Add function symbol
                // Enter new scope
                // Visit parameters
                // Visit body
            }
            // ... other statement types
        }
    }
}
```

---

## 🚨 Diagnostic System

### Converting Errors

```rust
pub fn error_to_diagnostic(error: &CompileError, uri: &Url) -> Diagnostic {
    Diagnostic {
        range: span_to_range(error.span),
        severity: Some(match error.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(error.code.to_string())),
        source: Some("liva".to_string()),
        message: error.message.clone(),
        related_information: error.related.as_ref().map(|related| {
            related.iter().map(|r| DiagnosticRelatedInformation {
                location: Location {
                    uri: uri.clone(),
                    range: span_to_range(r.span),
                },
                message: r.message.clone(),
            }).collect()
        }),
        ..Default::default()
    }
}
```

### Publishing Diagnostics

```rust
async fn publish_diagnostics(&self, uri: &Url) {
    if let Some(doc) = self.get_document(uri) {
        self.client.publish_diagnostics(
            uri.clone(),
            doc.diagnostics.clone(),
            Some(doc.version),
        ).await;
    }
}
```

---

## 🛠️ Utility Functions

### Position/Range Conversion

```rust
/// Converts compiler Span to LSP Range
pub fn span_to_range(span: Span) -> Range {
    Range {
        start: Position {
            line: span.start.line as u32 - 1,  // LSP is 0-indexed
            character: span.start.column as u32 - 1,
        },
        end: Position {
            line: span.end.line as u32 - 1,
            character: span.end.column as u32 - 1,
        },
    }
}

/// Converts LSP Position to byte offset
pub fn position_to_offset(text: &str, position: Position) -> usize {
    let mut offset = 0;
    let mut current_line = 0;
    
    for (i, ch) in text.char_indices() {
        if current_line == position.line as usize {
            if i >= position.character as usize {
                return offset;
            }
        }
        
        if ch == '\n' {
            current_line += 1;
        }
        
        offset = i;
    }
    
    offset
}
```

### Identifier Validation

```rust
pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    
    // Must start with letter or underscore
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    
    // Rest can be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}
```

---

## 🔌 Extension Points

### Custom Capabilities

```rust
pub trait LspCapability {
    fn register(&self, server: &mut LivaLanguageServer);
    fn handle_request(&self, method: &str, params: serde_json::Value) 
        -> Result<serde_json::Value>;
}

// Example: Code Lens
pub struct CodeLensCapability;

impl LspCapability for CodeLensCapability {
    fn register(&self, server: &mut LivaLanguageServer) {
        server.register_capability("textDocument/codeLens", self);
    }
    
    fn handle_request(&self, _method: &str, params: serde_json::Value) 
        -> Result<serde_json::Value> 
    {
        // Implement code lens logic
        Ok(serde_json::json!([]))
    }
}
```

### Custom Diagnostics

```rust
pub trait DiagnosticProvider {
    fn name(&self) -> &str;
    fn diagnose(&self, doc: &DocumentState) -> Vec<Diagnostic>;
}

// Example: Unused variable checker
pub struct UnusedVariableDiagnostic;

impl DiagnosticProvider for UnusedVariableDiagnostic {
    fn name(&self) -> &str {
        "unused-variable"
    }
    
    fn diagnose(&self, doc: &DocumentState) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        
        if let Some(symbols) = &doc.symbols {
            for symbol in symbols.all() {
                if symbol.references.is_empty() {
                    diagnostics.push(Diagnostic {
                        range: symbol.definition_range,
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: format!("Unused variable '{}'", symbol.name),
                        ..Default::default()
                    });
                }
            }
        }
        
        diagnostics
    }
}
```

---

## 📚 Complete Example

```rust
use tower_lsp::{LspService, Server};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::build(|client| {
        LivaLanguageServer::new(client)
    })
    .finish();
    
    Server::new(stdin, stdout, socket).serve(service).await;
}

impl LivaLanguageServer {
    async fn full_workflow_example(&self, uri: &Url, text: &str) -> Result<()> {
        // 1. Open document
        self.documents.insert(uri.clone(), DocumentState {
            text: text.to_string(),
            version: 1,
            ast: None,
            symbols: None,
            diagnostics: vec![],
            last_parsed: Instant::now(),
        });
        
        // 2. Parse document
        let tokens = tokenize(&text);
        let ast = parse(tokens)?;
        
        // 3. Build symbol table
        let symbols = build_symbol_table(&ast, uri);
        
        // 4. Run semantic analysis
        let errors = analyze(&ast);
        
        // 5. Convert to diagnostics
        let diagnostics: Vec<Diagnostic> = errors.iter()
            .map(|e| error_to_diagnostic(e, uri))
            .collect();
        
        // 6. Update document state
        if let Some(mut doc) = self.get_document_mut(uri) {
            doc.ast = Some(ast);
            doc.symbols = Some(symbols);
            doc.diagnostics = diagnostics.clone();
        }
        
        // 7. Publish diagnostics
        self.client.publish_diagnostics(
            uri.clone(),
            diagnostics,
            Some(1),
        ).await;
        
        Ok(())
    }
}
```

---

**Version:** 0.12.0  
**Last Updated:** 2025-10-27  
**Maintainer:** Liva Core Team
