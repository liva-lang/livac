pub mod diagnostics;
pub mod document;
pub mod imports;
/// Language Server Protocol implementation for Liva
///
/// This module provides LSP support for IDE integration.
///
/// Architecture:
/// - `server.rs`: Main LSP server struct
/// - `handlers/`: Request handlers (lifecycle, document, features)
/// - `document.rs`: Document state management
/// - `symbols.rs`: Symbol table and indexing
/// - `diagnostics.rs`: Error to diagnostic conversion
pub mod server;
pub mod symbols;
pub mod workspace;

pub use document::DocumentState;
pub use imports::{ImportInfo, ImportResolver};
pub use server::LivaLanguageServer;
pub use symbols::{Symbol, SymbolTable};
pub use workspace::{WorkspaceIndex, WorkspaceManager};
