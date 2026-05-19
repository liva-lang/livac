//! Liva developer tools — formatter, linter, language server.
//!
//! This crate hosts `livac fmt`, `livac lint`, and `livac lsp` implementations,
//! living outside the (eventually frozen) bootstrap compiler crate.

pub mod formatter;
pub mod linter;
pub mod lsp;
