use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::document::DocumentState;
use super::diagnostics::error_to_diagnostic;
use super::symbols::SymbolTable;
use crate::{lexer, parser, semantic};

/// Main Language Server for Liva
pub struct LivaLanguageServer {
    /// LSP client for sending notifications
    client: Client,
    
    /// Open documents indexed by URI
    documents: DashMap<Url, DocumentState>,
}

impl LivaLanguageServer {
    /// Creates a new language server instance
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
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
                        // Build symbol table from AST
                        let symbols = SymbolTable::from_ast(&analyzed_ast);
                        doc.ast = Some(analyzed_ast);
                        doc.symbols = Some(symbols);
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
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
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
        let position = params.text_document_position.position;
        
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };
        
        let mut items = Vec::new();
        
        // Keywords
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
                ..Default::default()
            });
        }
        
        // Types
        let types = vec![
            "int", "float", "string", "bool", "void",
        ];
        
        for type_name in types {
            items.push(CompletionItem {
                label: type_name.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some("type".to_string()),
                ..Default::default()
            });
        }
        
        // Built-in functions
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
                ..Default::default()
            });
        }
        
        // Add symbols from AST (variables, functions, classes)
        if let Some(symbols) = &doc.symbols {
            for symbol in symbols.all() {
                // Skip if already added (avoid duplicates with keywords)
                if items.iter().any(|item| item.label == symbol.name) {
                    continue;
                }
                
                // Convert SymbolKind to CompletionItemKind
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
                    ..Default::default()
                });
            }
        }
        
        Ok(Some(CompletionResponse::Array(items)))
    }
}
