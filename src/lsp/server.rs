use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use super::document::DocumentState;
use super::diagnostics::error_to_diagnostic;
use crate::{lexer, parser};

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
                doc.ast = Some(ast);
                doc.diagnostics.clear();
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
}
