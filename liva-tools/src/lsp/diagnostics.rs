use tower_lsp::lsp_types::*;

use livac::CompilerError;

use crate::linter::LintWarning;

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

/// Converts a linter warning to an LSP diagnostic.
///
/// The linter only reports start line/column, so the diagnostic spans either
/// from the column to the end of the source line, or the whole line when
/// no column is available.
pub fn warning_to_diagnostic(warning: &LintWarning) -> Diagnostic {
    let line = (warning.line as u32).saturating_sub(1);
    let start_char = warning.column.map(|c| c.saturating_sub(1) as u32).unwrap_or(0);
    let end_char = warning
        .source_line
        .as_ref()
        .map(|s| s.len() as u32)
        .unwrap_or(start_char + 1);
    Diagnostic {
        range: Range {
            start: Position { line, character: start_char },
            end: Position { line, character: end_char.max(start_char + 1) },
        },
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String(warning.code.clone())),
        source: Some("liva-lint".to_string()),
        message: warning.message.clone(),
        related_information: None,
        ..Default::default()
    }
}
