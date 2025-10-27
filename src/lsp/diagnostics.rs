use tower_lsp::lsp_types::*;

use crate::CompilerError;

/// Converts a compiler error to an LSP diagnostic
pub fn error_to_diagnostic(error: &CompilerError) -> Option<Diagnostic> {
    let error_info = error.error_info()?;
    let location = error_info.location.as_ref()?;
    
    Some(Diagnostic {
        range: Range {
            start: Position {
                line: (location.line as u32).saturating_sub(1),
                character: location.column.unwrap_or(0) as u32,
            },
            end: Position {
                line: (location.line as u32).saturating_sub(1),
                character: (location.column.unwrap_or(0) + location.length.unwrap_or(1)) as u32,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error_info.code.clone())),
        source: Some("liva".to_string()),
        message: error_info.message.clone(),
        related_information: None,
        ..Default::default()
    })
}
