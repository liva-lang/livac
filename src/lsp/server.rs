use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::document::DocumentState;
use super::diagnostics::error_to_diagnostic;
use super::symbols::SymbolTable;
use super::workspace::{WorkspaceManager, WorkspaceIndex};
use super::imports::ImportResolver;
use crate::{lexer, parser, semantic};

/// Main Language Server for Liva
pub struct LivaLanguageServer {
    /// LSP client for sending notifications
    client: Client,
    
    /// Open documents indexed by URI
    documents: DashMap<Url, DocumentState>,
    
    /// Workspace file manager
    workspace: std::sync::Arc<tokio::sync::RwLock<WorkspaceManager>>,
    
    /// Workspace symbol index
    workspace_index: std::sync::Arc<WorkspaceIndex>,
    
    /// Import resolver
    import_resolver: std::sync::Arc<tokio::sync::RwLock<ImportResolver>>,
}

impl LivaLanguageServer {
    /// Creates a new language server instance
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            workspace: std::sync::Arc::new(tokio::sync::RwLock::new(WorkspaceManager::new(vec![]))),
            workspace_index: std::sync::Arc::new(WorkspaceIndex::default()),
            import_resolver: std::sync::Arc::new(tokio::sync::RwLock::new(ImportResolver::new(vec![]))),
        }
    }
    
    /// Parses a document and updates its state
    async fn parse_document(&self, uri: &Url) {
        let mut doc = match self.documents.get_mut(uri) {
            Some(doc) => doc,
            None => return,
        };
        
        // Tokenize
        let tokens = match lexer::tokenize(&doc.text) {
            Ok(tokens) => tokens,
            Err(e) => {
                // Store lexer error as diagnostic
                if let Some(diag) = error_to_diagnostic(&e) {
                    doc.diagnostics = vec![diag];
                }
                return;
            }
        };
        
        // Parse
        match parser::parse(tokens, &doc.text) {
            Ok(ast) => {
                // Run semantic analysis
                match semantic::analyze(ast.clone()) {
                    Ok(analyzed_ast) => {
                        // Build symbol table from AST (pass source text for span conversion)
                        let symbols = SymbolTable::from_ast(&analyzed_ast, &doc.text);
                        
                        // Extract imports from AST
                        let import_resolver = self.import_resolver.read().await;
                        let imports = import_resolver.extract_imports(&analyzed_ast, uri);
                        drop(import_resolver);
                        
                        // Index file in workspace index
                        self.workspace_index.index_file(uri.clone(), &analyzed_ast, &doc.text);
                        
                        doc.ast = Some(analyzed_ast);
                        doc.symbols = Some(symbols);
                        doc.imports = imports;
                        doc.diagnostics.clear();
                    }
                    Err(e) => {
                        // Store semantic error as diagnostic
                        doc.ast = Some(ast);
                        if let Some(diag) = error_to_diagnostic(&e) {
                            doc.diagnostics = vec![diag];
                        }
                    }
                }
            }
            Err(e) => {
                // Store parse error as diagnostic
                if let Some(diag) = error_to_diagnostic(&e) {
                    doc.diagnostics = vec![diag];
                }
            }
        }
    }
    
    /// Publishes diagnostics for a document
    async fn publish_diagnostics(&self, uri: &Url) {
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return,
        };
        
        self.client
            .publish_diagnostics(uri.clone(), doc.diagnostics.clone(), Some(doc.version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LivaLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Initialize workspace with root URIs
        if let Some(workspace_folders) = params.workspace_folders {
            let root_uris: Vec<Url> = workspace_folders
                .iter()
                .map(|folder| folder.uri.clone())
                .collect();
            
            // Initialize workspace manager
            let mut workspace = self.workspace.write().await;
            *workspace = WorkspaceManager::new(root_uris.clone());
            workspace.scan_workspace();
            
            let file_count = workspace.file_count();
            drop(workspace);
            
            // Initialize import resolver with workspace roots
            let mut import_resolver = self.import_resolver.write().await;
            *import_resolver = ImportResolver::new(root_uris);
            drop(import_resolver);
            
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Workspace initialized: {} .liva files found", file_count)
                )
                .await;
        }
        
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "liva-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Liva Language Server initialized")
            .await;
        
        // Index all workspace files
        let workspace = self.workspace.read().await;
        let files = workspace.list_liva_files();
        let file_count = files.len();
        
        for file_uri in files {
            if let Ok(path) = file_uri.to_file_path() {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    // Tokenize
                    if let Ok(tokens) = lexer::tokenize(&content) {
                        // Parse
                        if let Ok(ast) = parser::parse(tokens, &content) {
                            // Run semantic analysis
                            if let Ok(analyzed_ast) = semantic::analyze(ast) {
                                // Index the file
                                self.workspace_index.index_file(file_uri.clone(), &analyzed_ast, &content);
                            }
                        }
                    }
                }
            }
        }
        
        self.client
            .log_message(
                MessageType::INFO,
                format!("Indexed {} workspace files", file_count),
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("Document opened: {}", params.text_document.uri))
            .await;
        
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;
        
        // Store document state
        self.documents.insert(
            uri.clone(),
            DocumentState::new(text, version),
        );
        
        // Parse and publish diagnostics
        self.parse_document(&uri).await;
        self.publish_diagnostics(&uri).await;
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        // Update document with full text (FULL sync mode)
        if let Some(mut doc) = self.documents.get_mut(&uri) {
            for change in params.content_changes {
                // In FULL sync mode, we replace the entire document
                doc.text = change.text;
            }
            doc.version = params.text_document.version;
        }
        
        // Parse and publish diagnostics
        self.parse_document(&uri).await;
        self.publish_diagnostics(&uri).await;
    }
    
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("Document saved: {}", params.text_document.uri))
            .await;
        
        // Optionally re-parse on save
        let uri = &params.text_document.uri;
        self.parse_document(uri).await;
        self.publish_diagnostics(uri).await;
    }
    
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("Document closed: {}", params.text_document.uri))
            .await;
        
        // Remove document from cache
        self.documents.remove(&params.text_document.uri);
        
        // Clear diagnostics
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;
        
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };
        
        let mut items = Vec::new();
        
        // Keywords (priority 0 - always first)
        let keywords = vec![
            "let", "const", "fn", "return", "if", "else", "while", "for", "switch",
            "async", "await", "task", "fire", "import", "from", "export", "type",
            "true", "false", "print", "console", "Math", "JSON", "File", "HTTP",
        ];
        
        for keyword in keywords {
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword".to_string()),
                sort_text: Some(format!("0_{}", keyword)),
                ..Default::default()
            });
        }
        
        // Types (priority 1)
        let types = vec![
            "int", "float", "string", "bool", "void",
        ];
        
        for type_name in types {
            items.push(CompletionItem {
                label: type_name.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some("type".to_string()),
                sort_text: Some(format!("1_{}", type_name)),
                ..Default::default()
            });
        }
        
        // Built-in functions (priority 2)
        let builtins = vec![
            ("parseInt", "parseInt(str: string) -> (int, string)"),
            ("parseFloat", "parseFloat(str: string) -> (float, string)"),
            ("toString", "toString(value) -> string"),
        ];
        
        for (name, signature) in builtins {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(signature.to_string()),
                sort_text: Some(format!("2_{}", name)),
                ..Default::default()
            });
        }
        
        // Local file symbols (priority 3 - local symbols)
        if let Some(symbols) = &doc.symbols {
            for symbol in symbols.all() {
                if items.iter().any(|item| item.label == symbol.name) {
                    continue;
                }
                
                let completion_kind = match symbol.kind {
                    SymbolKind::FUNCTION => CompletionItemKind::FUNCTION,
                    SymbolKind::CLASS => CompletionItemKind::CLASS,
                    SymbolKind::METHOD => CompletionItemKind::METHOD,
                    SymbolKind::STRUCT => CompletionItemKind::STRUCT,
                    SymbolKind::CONSTANT => CompletionItemKind::CONSTANT,
                    SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
                    SymbolKind::TYPE_PARAMETER => CompletionItemKind::TYPE_PARAMETER,
                    _ => CompletionItemKind::TEXT,
                };
                
                items.push(CompletionItem {
                    label: symbol.name.clone(),
                    kind: Some(completion_kind),
                    detail: symbol.detail.clone(),
                    sort_text: Some(format!("3_{}", symbol.name)),
                    ..Default::default()
                });
            }
        }
        
        // Imported symbols (priority 4 - from explicit imports)
        let import_resolver = self.import_resolver.read().await;
        for import_info in &doc.imports {
            if let Some(import_uri) = &import_info.resolved_uri {
                // Get all symbols from imported file
                if let Some(imported_symbols) = self.workspace_index.get_file_symbols(import_uri) {
                    for symbol in imported_symbols {
                        // Only add explicitly imported symbols (or all if wildcard)
                        if import_info.is_wildcard || import_info.symbols.contains(&symbol.name) {
                            if items.iter().any(|item| item.label == symbol.name) {
                                continue;
                            }
                            
                            let completion_kind = match symbol.kind {
                                SymbolKind::FUNCTION => CompletionItemKind::FUNCTION,
                                SymbolKind::CLASS => CompletionItemKind::CLASS,
                                SymbolKind::METHOD => CompletionItemKind::METHOD,
                                SymbolKind::STRUCT => CompletionItemKind::STRUCT,
                                SymbolKind::CONSTANT => CompletionItemKind::CONSTANT,
                                SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
                                SymbolKind::TYPE_PARAMETER => CompletionItemKind::TYPE_PARAMETER,
                                _ => CompletionItemKind::TEXT,
                            };
                            
                            items.push(CompletionItem {
                                label: symbol.name.clone(),
                                kind: Some(completion_kind),
                                detail: Some(format!("from {}", import_info.source)),
                                sort_text: Some(format!("4_{}", symbol.name)),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
        drop(import_resolver);
        
        // Workspace symbols (priority 5 - all other workspace symbols)
        // Limit to prevent overwhelming completion list in large workspaces
        const MAX_WORKSPACE_SYMBOLS: usize = 100;
        let all_workspace_symbols = self.workspace_index.all_symbols();
        let mut workspace_count = 0;
        
        for (symbol_uri, symbol) in all_workspace_symbols {
            // Limit workspace symbols to keep completion responsive
            if workspace_count >= MAX_WORKSPACE_SYMBOLS {
                break;
            }
            
            // Skip current file (already added)
            if &symbol_uri == uri {
                continue;
            }
            
            // Skip if already added
            if items.iter().any(|item| item.label == symbol.name) {
                continue;
            }
            
            let completion_kind = match symbol.kind {
                SymbolKind::FUNCTION => CompletionItemKind::FUNCTION,
                SymbolKind::CLASS => CompletionItemKind::CLASS,
                SymbolKind::METHOD => CompletionItemKind::METHOD,
                SymbolKind::STRUCT => CompletionItemKind::STRUCT,
                SymbolKind::CONSTANT => CompletionItemKind::CONSTANT,
                SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
                SymbolKind::TYPE_PARAMETER => CompletionItemKind::TYPE_PARAMETER,
                _ => CompletionItemKind::TEXT,
            };
            
            // Extract filename from URI
            let file_name = symbol_uri
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("unknown");
            
            items.push(CompletionItem {
                label: symbol.name.clone(),
                kind: Some(completion_kind),
                detail: Some(format!("from {}", file_name)),
                sort_text: Some(format!("5_{}", symbol.name)),
                ..Default::default()
            });
            
            workspace_count += 1;
        }
        
        self.client
            .log_message(
                MessageType::INFO,
                format!("Completion: {} items (local + imported + workspace)", items.len()),
            )
            .await;
        
        Ok(Some(CompletionResponse::Array(items)))
    }
    
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };
        
        // Get the word at the cursor position
        let word = match doc.word_at_position(position) {
            Some(w) => w,
            None => return Ok(None),
        };
        
        // 1. Try current file first (fast path)
        if let Some(symbols) = &doc.symbols {
            if let Some(symbol_list) = symbols.lookup(&word) {
                // Return the first symbol's location (TODO: handle overloads)
                if let Some(symbol) = symbol_list.first() {
                    let location = Location {
                        uri: uri.clone(),
                        range: symbol.range,
                    };
                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }
        
        // 2. Check if symbol is imported and resolve to imported file
        let import_resolver = self.import_resolver.read().await;
        let import_source = import_resolver.get_import_source(&word, &doc.imports);
        drop(import_resolver);
        
        if let Some(import_uri) = import_source {
            // Look up symbol in the imported file
            if let Some(matches) = self.workspace_index.lookup_in_file(&import_uri, &word) {
                if let Some(def_symbol) = matches.first() {
                    let location = Location {
                        uri: import_uri.clone(),
                        range: def_symbol.range,
                    };
                    
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!("Imported definition found: {} from {}", word, import_uri),
                        )
                        .await;
                    
                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }
        
        // 3. Search workspace index for cross-file definitions (fallback)
        if let Some(matches) = self.workspace_index.lookup_global(&word) {
            if let Some((def_uri, def_symbol)) = matches.first() {
                let location = Location {
                    uri: def_uri.clone(),
                    range: def_symbol.range,
                };
                
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Cross-file definition found: {} in {}", word, def_uri),
                    )
                    .await;
                
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }
        
        Ok(None)
    }
    
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };
        
        // Get the word at the cursor position
        let word = match doc.word_at_position(position) {
            Some(w) => w,
            None => return Ok(None),
        };
        
        let mut all_locations = Vec::new();
        
        // 1. Search in current file
        if let Some(symbols) = &doc.symbols {
            if symbols.lookup(&word).is_some() {
                let ranges = symbols.find_references(&word, &doc.text);
                
                // Convert ranges to locations
                for range in ranges {
                    all_locations.push(Location {
                        uri: uri.clone(),
                        range,
                    });
                }
            }
        }
        
        // 2. Search in all workspace files
        let indexed_files = self.workspace_index.indexed_files();
        
        for file_uri in indexed_files {
            // Skip current file (already searched)
            if &file_uri == uri {
                continue;
            }
            
            // Check if file is open in editor
            if let Some(open_doc) = self.documents.get(&file_uri) {
                // Search in open document
                if let Some(symbols) = &open_doc.symbols {
                    if symbols.lookup(&word).is_some() {
                        let ranges = symbols.find_references(&word, &open_doc.text);
                        for range in ranges {
                            all_locations.push(Location {
                                uri: file_uri.clone(),
                                range,
                            });
                        }
                    }
                }
            } else {
                // Read file from disk
                if let Ok(path) = file_uri.to_file_path() {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        // Quick textual search (no need to parse)
                        let lines: Vec<&str> = content.lines().collect();
                        for (line_idx, line) in lines.iter().enumerate() {
                            let mut search_from = 0;
                            while let Some(pos) = line[search_from..].find(&word) {
                                let actual_pos = search_from + pos;
                                
                                // Check word boundaries
                                let is_start_boundary = actual_pos == 0 || 
                                    !line.chars().nth(actual_pos - 1).unwrap_or(' ').is_alphanumeric();
                                let end_pos = actual_pos + word.len();
                                let is_end_boundary = end_pos >= line.len() || 
                                    !line.chars().nth(end_pos).unwrap_or(' ').is_alphanumeric();
                                
                                if is_start_boundary && is_end_boundary {
                                    all_locations.push(Location {
                                        uri: file_uri.clone(),
                                        range: Range {
                                            start: Position {
                                                line: line_idx as u32,
                                                character: actual_pos as u32,
                                            },
                                            end: Position {
                                                line: line_idx as u32,
                                                character: (actual_pos + word.len()) as u32,
                                            },
                                        },
                                    });
                                }
                                
                                search_from = actual_pos + 1;
                            }
                        }
                    }
                }
            }
        }
        
        if all_locations.is_empty() {
            return Ok(None);
        }
        
        self.client
            .log_message(
                MessageType::INFO,
                format!("Found {} references to '{}' across workspace", all_locations.len(), word),
            )
            .await;
        
        Ok(Some(all_locations))
    }
    
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };
        
        // Get the word at the cursor position
        let word = match doc.word_at_position(position) {
            Some(w) => w,
            None => return Ok(None),
        };
        
        // Look up the symbol in the symbol table
        if let Some(symbols) = &doc.symbols {
            if let Some(symbol_list) = symbols.lookup(&word) {
                if let Some(symbol) = symbol_list.first() {
                    // Format hover content as Markdown
                    let mut content = String::new();
                    
                    // Add symbol kind and name
                    let kind_str = match symbol.kind {
                        SymbolKind::FUNCTION => "function",
                        SymbolKind::CLASS => "class",
                        SymbolKind::STRUCT => "interface",
                        SymbolKind::TYPE_PARAMETER => "type",
                        SymbolKind::VARIABLE => "variable",
                        SymbolKind::CONSTANT => "constant",
                        _ => "symbol",
                    };
                    
                    content.push_str(&format!("```liva\n{} {}\n```\n", kind_str, symbol.name));
                    
                    // Add detail if available
                    if let Some(detail) = &symbol.detail {
                        content.push_str(&format!("\n{}", detail));
                    }
                    
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(symbol.range),
                    }));
                }
            }
        }
        
        // Check for built-in keywords/types
        let builtin_info = match word.as_str() {
            "int" => Some("```liva\ntype int\n```\n\nSigned 32-bit integer type"),
            "float" => Some("```liva\ntype float\n```\n\n64-bit floating-point type"),
            "string" => Some("```liva\ntype string\n```\n\nUTF-8 string type"),
            "bool" => Some("```liva\ntype bool\n```\n\nBoolean type (true/false)"),
            "void" => Some("```liva\ntype void\n```\n\nVoid type (no return value)"),
            "let" => Some("```liva\nlet\n```\n\nDeclares a mutable variable"),
            "const" => Some("```liva\nconst\n```\n\nDeclares an immutable constant"),
            "fn" => Some("```liva\nfn\n```\n\nDefines a function"),
            "return" => Some("```liva\nreturn\n```\n\nReturns a value from a function"),
            "if" => Some("```liva\nif\n```\n\nConditional statement"),
            "else" => Some("```liva\nelse\n```\n\nAlternative branch for if"),
            "while" => Some("```liva\nwhile\n```\n\nLoop while condition is true"),
            "for" => Some("```liva\nfor\n```\n\nIterate over a range or collection"),
            _ => None,
        };
        
        if let Some(info) = builtin_info {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: info.to_string(),
                }),
                range: None,
            }));
        }
        
        Ok(None)
    }
}
