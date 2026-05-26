use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::{GotoImplementationParams, GotoImplementationResponse};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::diagnostics::error_to_diagnostic;
use super::document::DocumentState;
use super::imports::ImportResolver;
use super::symbols::SymbolTable;
use super::workspace::{WorkspaceIndex, WorkspaceManager};
use livac::{lexer, parser, semantic};

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
            import_resolver: std::sync::Arc::new(tokio::sync::RwLock::new(ImportResolver::new(
                vec![],
            ))),
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
                        self.workspace_index
                            .index_file(uri.clone(), &analyzed_ast, &doc.text);

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
                    format!("Workspace initialized: {} .liva files found", file_count),
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
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: Default::default(),
                }),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
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
                                self.workspace_index.index_file(
                                    file_uri.clone(),
                                    &analyzed_ast,
                                    &content,
                                );
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
            .log_message(
                MessageType::INFO,
                format!("Document opened: {}", params.text_document.uri),
            )
            .await;

        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Store document state
        self.documents
            .insert(uri.clone(), DocumentState::new(text, version));

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
            .log_message(
                MessageType::INFO,
                format!("Document saved: {}", params.text_document.uri),
            )
            .await;

        // Optionally re-parse on save
        let uri = &params.text_document.uri;
        self.parse_document(uri).await;
        self.publish_diagnostics(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Document closed: {}", params.text_document.uri),
            )
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
            "let", "const", "fn", "return", "if", "else", "while", "for", "switch", "async",
            "await", "task", "import", "from", "export", "type", "true", "false", "print",
            "console", "Math", "JSON", "File", "HTTP", "break", "continue",
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
        let types = vec!["int", "float", "string", "bool", "void"];

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
                format!(
                    "Completion: {} items (local + imported + workspace)",
                    items.len()
                ),
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

    async fn goto_implementation(
        &self,
        params: GotoImplementationParams,
    ) -> Result<Option<GotoImplementationResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let word = match doc.word_at_position(position) {
            Some(w) => w,
            None => return Ok(None),
        };

        // Look up all classes that implement an interface named `word` or
        // declare a method named `word` while implementing some interface.
        if let Some(impls) = self.workspace_index.lookup_implementations(&word) {
            if !impls.is_empty() {
                let locations: Vec<Location> = impls
                    .into_iter()
                    .map(|(impl_uri, sym)| Location {
                        uri: impl_uri,
                        range: sym.range,
                    })
                    .collect();

                self.client
                    .log_message(
                        MessageType::INFO,
                        format!(
                            "Found {} implementation(s) for `{}`",
                            locations.len(),
                            word
                        ),
                    )
                    .await;

                return Ok(Some(GotoImplementationResponse::Array(locations)));
            }
        }

        // No implementations recorded \u2014 fall back to definition lookup so
        // the editor still navigates somewhere reasonable.
        let fallback = self
            .goto_definition(GotoDefinitionParams {
                text_document_position_params: params.text_document_position_params,
                work_done_progress_params: params.work_done_progress_params,
                partial_result_params: params.partial_result_params,
            })
            .await?;

        Ok(fallback.map(|resp| match resp {
            GotoDefinitionResponse::Scalar(loc) => GotoImplementationResponse::Scalar(loc),
            GotoDefinitionResponse::Array(locs) => GotoImplementationResponse::Array(locs),
            GotoDefinitionResponse::Link(links) => GotoImplementationResponse::Link(links),
        }))
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
                                let is_start_boundary = actual_pos == 0
                                    || !line
                                        .chars()
                                        .nth(actual_pos - 1)
                                        .unwrap_or(' ')
                                        .is_alphanumeric();
                                let end_pos = actual_pos + word.len();
                                let is_end_boundary = end_pos >= line.len()
                                    || !line.chars().nth(end_pos).unwrap_or(' ').is_alphanumeric();

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
                format!(
                    "Found {} references to '{}' across workspace",
                    all_locations.len(),
                    word
                ),
            )
            .await;

        Ok(Some(all_locations))
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let word = match doc.word_at_position(position) {
            Some(w) => w,
            None => return Ok(None),
        };

        // Re-use the textual word-boundary search already used by `references`
        // to scope highlights to the current document only \u2014 that's the
        // contract of `textDocument/documentHighlight`.
        let highlights: Vec<DocumentHighlight> = doc
            .symbols
            .as_ref()
            .map(|s| s.find_references(&word, &doc.text))
            .unwrap_or_default()
            .into_iter()
            .map(|range| DocumentHighlight {
                range,
                // We don't currently distinguish reads from writes; emit
                // TEXT for all occurrences. LSP clients render this as a
                // single highlight color, matching most editors' defaults.
                kind: Some(DocumentHighlightKind::TEXT),
            })
            .collect();

        if highlights.is_empty() {
            return Ok(None);
        }
        Ok(Some(highlights))
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let uri = &params.text_document.uri;
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Build a chain of expanding ranges per requested position:
        //   word \u2192 line \u2192 whole document. This is a lightweight
        // syntax-agnostic fallback; a proper implementation would walk
        // the AST and emit token/expression/block/function nesting.
        let mut out = Vec::with_capacity(params.positions.len());
        let lines: Vec<&str> = doc.text.lines().collect();
        let total_lines = lines.len() as u32;
        let last_line_len = lines.last().map(|l| l.len() as u32).unwrap_or(0);

        for pos in params.positions {
            let line_text = lines.get(pos.line as usize).copied().unwrap_or("");
            let word_range = expand_word_range(line_text, pos);

            let line_range = Range {
                start: Position { line: pos.line, character: 0 },
                end: Position {
                    line: pos.line,
                    character: line_text.len() as u32,
                },
            };
            let doc_range = Range {
                start: Position { line: 0, character: 0 },
                end: Position {
                    line: total_lines.saturating_sub(1),
                    character: last_line_len,
                },
            };

            let doc_sel = SelectionRange {
                range: doc_range,
                parent: None,
            };
            let line_sel = SelectionRange {
                range: line_range,
                parent: Some(Box::new(doc_sel)),
            };
            let word_sel = SelectionRange {
                range: word_range.unwrap_or(line_range),
                parent: Some(Box::new(line_sel)),
            };
            out.push(word_sel);
        }
        Ok(Some(out))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Validate the new name is a sensible identifier.
        if new_name.is_empty()
            || new_name
                .chars()
                .next()
                .map(|c| !(c.is_ascii_alphabetic() || c == '_'))
                .unwrap_or(true)
            || !new_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                "`{}` is not a valid Liva identifier",
                new_name
            )));
        }

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let word = match doc.word_at_position(position) {
            Some(w) => w,
            None => return Ok(None),
        };
        if word == new_name {
            return Ok(None);
        }

        // Compute edits using the same textual word-boundary search the
        // references handler uses. This is intra-file rename; cross-file
        // rename will require resolving the symbol's defining scope, which
        // is a v3.x item.
        let ranges = doc
            .symbols
            .as_ref()
            .map(|s| s.find_references(&word, &doc.text))
            .unwrap_or_default();
        if ranges.is_empty() {
            return Ok(None);
        }

        let edits: Vec<TextEdit> = ranges
            .into_iter()
            .map(|range| TextEdit {
                range,
                new_text: new_name.clone(),
            })
            .collect();

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), edits);

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn document_link(
        &self,
        params: DocumentLinkParams,
    ) -> Result<Option<Vec<DocumentLink>>> {
        let uri = &params.text_document.uri;
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let resolver = self.import_resolver.read().await;
        let mut links: Vec<DocumentLink> = Vec::new();

        // Scan each line for `import ... "PATH"` or `import "PATH"`.
        // The Liva parser already validated these as imports, so we don't
        // need to be strict \u2014 we just need the quoted path range.
        for (line_idx, line) in doc.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("import ") && !trimmed.starts_with("import\"") {
                continue;
            }
            // Find first quoted string on the line.
            let Some(open) = line.find('"') else { continue };
            let Some(close_rel) = line[open + 1..].find('"') else { continue };
            let close = open + 1 + close_rel;
            let source = &line[open + 1..close];

            let target = resolver.resolve_import(source, uri);
            links.push(DocumentLink {
                range: Range {
                    start: Position {
                        line: line_idx as u32,
                        character: (open + 1) as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: close as u32,
                    },
                },
                target,
                tooltip: Some(format!("Open {}", source)),
                data: None,
            });
        }

        if links.is_empty() {
            return Ok(None);
        }
        Ok(Some(links))
    }

    async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> Result<Option<Vec<FoldingRange>>> {
        let uri = &params.text_document.uri;
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };
        let ranges = compute_folding_ranges(&doc.text);
        if ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ranges))
        }
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

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get formatting options from the request
        let indent_size = params.options.tab_size as usize;

        let options = crate::formatter::FormatOptions {
            indent_size,
            ..Default::default()
        };

        // Format the document
        match crate::formatter::format_source(&doc.text, &options) {
            Ok(formatted) => {
                if formatted == doc.text {
                    // Already formatted
                    return Ok(None);
                }

                // Replace the entire document
                let line_count = doc.text.lines().count();
                let last_line = doc.text.lines().last().unwrap_or("");
                let edit = TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: line_count as u32,
                            character: last_line.len() as u32,
                        },
                    },
                    new_text: formatted,
                };

                self.client
                    .log_message(MessageType::INFO, format!("Formatted {}", uri))
                    .await;

                Ok(Some(vec![edit]))
            }
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Format failed for {}: {}", uri, e),
                    )
                    .await;
                Ok(None)
            }
        }
    }

    /// Document symbols — populates VS Code's Outline view and breadcrumbs
    /// with all top-level functions, classes, type aliases, and methods.
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let symbols = match self.workspace_index.get_file_symbols(uri) {
            Some(s) => s,
            None => return Ok(None),
        };

        #[allow(deprecated)]
        let infos: Vec<SymbolInformation> = symbols
            .into_iter()
            .map(|sym| SymbolInformation {
                name: sym.name,
                kind: sym.kind,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: uri.clone(),
                    range: sym.range,
                },
                container_name: None,
            })
            .collect();

        Ok(Some(DocumentSymbolResponse::Flat(infos)))
    }

    /// Workspace symbols — Ctrl+T (Go to Symbol in Workspace) populated
    /// from every indexed file. Filters by the user's query substring.
    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let all = self.workspace_index.all_symbols();

        #[allow(deprecated)]
        let results: Vec<SymbolInformation> = all
            .into_iter()
            .filter(|(_, sym)| query.is_empty() || sym.name.to_lowercase().contains(&query))
            .take(500) // protect against huge result sets
            .map(|(uri, sym)| SymbolInformation {
                name: sym.name,
                kind: sym.kind,
                tags: None,
                deprecated: None,
                location: Location {
                    uri,
                    range: sym.range,
                },
                container_name: None,
            })
            .collect();

        Ok(Some(results))
    }
}

/// Expand `pos` to the surrounding identifier-like word range on `line`.
/// Returns None when the cursor is not over an identifier character.
fn expand_word_range(line: &str, pos: tower_lsp::lsp_types::Position) -> Option<tower_lsp::lsp_types::Range> {
    let col = pos.character as usize;
    let bytes = line.as_bytes();
    if col > bytes.len() {
        return None;
    }
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    // The cursor can sit just past the last char of a word; check both sides.
    let on_word = (col < bytes.len() && is_ident(bytes[col]))
        || (col > 0 && is_ident(bytes[col - 1]));
    if !on_word {
        return None;
    }
    let mut start = col;
    while start > 0 && is_ident(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = col;
    while end < bytes.len() && is_ident(bytes[end]) {
        end += 1;
    }
    Some(tower_lsp::lsp_types::Range {
        start: tower_lsp::lsp_types::Position { line: pos.line, character: start as u32 },
        end: tower_lsp::lsp_types::Position { line: pos.line, character: end as u32 },
    })
}

/// Compute folding ranges for `text` by matching `{`/`}` pairs and collapsing
/// consecutive `import` lines and `//` line-comment blocks.
///
/// Strings and comments are skipped while scanning braces so they don't throw
/// the matcher off. Single-line braces (`{ }` on the same line) are not folded.
fn compute_folding_ranges(text: &str) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    // 1. Brace-based regions.
    let mut stack: Vec<(u32, u32)> = Vec::new(); // (line, character)
    for (line_idx, line) in lines.iter().enumerate() {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            match b {
                b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => break, // line comment
                b'"' => {
                    // Skip string literal, honoring escapes.
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        if bytes[i] == b'\\' && i + 1 < bytes.len() {
                            i += 2;
                            continue;
                        }
                        i += 1;
                    }
                }
                b'{' => stack.push((line_idx as u32, i as u32)),
                b'}' => {
                    if let Some((start_line, start_char)) = stack.pop() {
                        if (line_idx as u32) > start_line {
                            ranges.push(FoldingRange {
                                start_line,
                                start_character: Some(start_char),
                                end_line: line_idx as u32,
                                end_character: Some(i as u32),
                                kind: Some(FoldingRangeKind::Region),
                                collapsed_text: None,
                            });
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    // 2. Consecutive `import` lines.
    let is_import = |s: &str| s.trim_start().starts_with("import ");
    let mut idx = 0;
    while idx < lines.len() {
        if is_import(lines[idx]) {
            let start = idx;
            while idx < lines.len() && is_import(lines[idx]) {
                idx += 1;
            }
            let end = idx - 1;
            if end > start {
                ranges.push(FoldingRange {
                    start_line: start as u32,
                    start_character: None,
                    end_line: end as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Imports),
                    collapsed_text: None,
                });
            }
        } else {
            idx += 1;
        }
    }

    // 3. Consecutive `//` comment lines (3+ lines).
    let is_line_comment = |s: &str| s.trim_start().starts_with("//");
    let mut idx = 0;
    while idx < lines.len() {
        if is_line_comment(lines[idx]) {
            let start = idx;
            while idx < lines.len() && is_line_comment(lines[idx]) {
                idx += 1;
            }
            let end = idx - 1;
            if end >= start + 2 {
                ranges.push(FoldingRange {
                    start_line: start as u32,
                    start_character: None,
                    end_line: end as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Comment),
                    collapsed_text: None,
                });
            }
        } else {
            idx += 1;
        }
    }

    ranges
}

#[cfg(test)]
mod folding_tests {
    use super::compute_folding_ranges;
    use tower_lsp::lsp_types::FoldingRangeKind;

    #[test]
    fn folds_function_body() {
        let src = "greet(name) {\n    print(\"hi\")\n    print(name)\n}\n";
        let ranges = compute_folding_ranges(src);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 3);
    }

    #[test]
    fn does_not_fold_single_line_braces() {
        let src = "f() { 42 }\n";
        assert!(compute_folding_ranges(src).is_empty());
    }

    #[test]
    fn ignores_braces_in_strings() {
        let src = "f() {\n    let s = \"{not a brace}\"\n}\n";
        let ranges = compute_folding_ranges(src);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn folds_import_block() {
        let src = "import \"std/io\"\nimport \"std/math\"\nimport \"std/str\"\n\nmain() { }\n";
        let ranges = compute_folding_ranges(src);
        let imports: Vec<_> = ranges
            .iter()
            .filter(|r| r.kind == Some(FoldingRangeKind::Imports))
            .collect();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].start_line, 0);
        assert_eq!(imports[0].end_line, 2);
    }

    #[test]
    fn folds_comment_block_of_three_or_more() {
        let src = "// a\n// b\n// c\ncode\n";
        let ranges = compute_folding_ranges(src);
        let comments: Vec<_> = ranges
            .iter()
            .filter(|r| r.kind == Some(FoldingRangeKind::Comment))
            .collect();
        assert_eq!(comments.len(), 1);
    }

    #[test]
    fn does_not_fold_two_comment_lines() {
        let src = "// a\n// b\ncode\n";
        let ranges = compute_folding_ranges(src);
        let comments: Vec<_> = ranges
            .iter()
            .filter(|r| r.kind == Some(FoldingRangeKind::Comment))
            .collect();
        assert!(comments.is_empty());
    }
}
