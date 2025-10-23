use colored::Colorize;
use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorLocation {
    pub file: String,
    pub line: usize,
    pub column: Option<usize>,
    pub source_line: Option<String>,
    /// Length of the token/text to highlight (for underlining)
    pub length: Option<usize>,
    /// Lines before the error for context (up to 2)
    pub context_before: Option<Vec<String>>,
    /// Lines after the error for context (up to 2)
    pub context_after: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticErrorInfo {
    pub location: Option<ErrorLocation>,
    pub code: String,
    pub title: String,
    pub message: String,
    pub help: Option<String>,
}

impl SemanticErrorInfo {
    pub fn new(code: &str, title: &str, message: &str) -> Self {
        Self {
            location: None,
            code: code.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            help: None,
        }
    }

    pub fn from_string(msg: String) -> Self {
        Self {
            location: None,
            code: "E0000".to_string(),
            title: "Error".to_string(),
            message: msg,
            help: None,
        }
    }

    pub fn with_location(mut self, file: &str, line: usize) -> Self {
        self.location = Some(ErrorLocation {
            file: file.to_string(),
            line,
            column: None,
            source_line: None,
            length: None,
            context_before: None,
            context_after: None,
        });
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        if let Some(loc) = &mut self.location {
            loc.column = Some(column);
        }
        self
    }

    pub fn with_source_line(mut self, source_line: String) -> Self {
        if let Some(loc) = &mut self.location {
            loc.source_line = Some(source_line);
        }
        self
    }

    pub fn with_length(mut self, length: usize) -> Self {
        if let Some(loc) = &mut self.location {
            loc.length = Some(length);
        }
        self
    }

    pub fn with_context(mut self, before: Vec<String>, after: Vec<String>) -> Self {
        if let Some(loc) = &mut self.location {
            loc.context_before = Some(before);
            loc.context_after = Some(after);
        }
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn format(&self) -> String {
        let mut output = String::new();

        // Header con icono y código de error
        output.push_str(&format!("\n{} ", "●".red().bold()));
        output.push_str(&format!("{}: ", self.code.red().bold()));
        output.push_str(&format!("{}\n", self.title.bold()));

        // Separador
        output.push_str(&format!("{}\n", "─".repeat(60).bright_black()));

        // Ubicación
        if let Some(loc) = &self.location {
            output.push_str(&format!("  {} ", "→".blue().bold()));
            output.push_str(&format!("{}:", loc.file.cyan()));
            output.push_str(&format!("{}", loc.line.to_string().yellow().bold()));
            if let Some(col) = loc.column {
                output.push_str(&format!(":{}", col.to_string().yellow().bold()));
            }
            output.push_str("\n");

            // Mostrar contexto extendido si está disponible
            if let Some(source) = &loc.source_line {
                output.push_str("\n");

                let start_line = loc.line.saturating_sub(
                    loc.context_before.as_ref().map(|v| v.len()).unwrap_or(0)
                );

                // Líneas antes del error
                if let Some(before) = &loc.context_before {
                    for (idx, line) in before.iter().enumerate() {
                        let line_num = start_line + idx;
                        output.push_str(&format!(
                            "  {} {} {}\n",
                            format!("{:>4}", line_num).bright_black(),
                            "│".bright_black(),
                            line
                        ));
                    }
                }

                // Línea del error
                let trimmed = source.trim_start();
                let leading_spaces = source.len() - trimmed.len();

                output.push_str(&format!(
                    "  {} {}\n",
                    format!("{:>4}", loc.line).bright_black(),
                    "│".bright_black()
                ));
                output.push_str(&format!(
                    "  {} {} {}\n",
                    format!("{:>4}", " ").bright_black(),
                    "│".bright_black(),
                    trimmed
                ));

                // Indicador visual con longitud precisa si está disponible
                if let Some(col) = loc.column {
                    let adjusted_col = col.saturating_sub(leading_spaces + 1);
                    let underline_len = loc.length.unwrap_or(3).max(1);
                    output.push_str(&format!(
                        "  {} {} {}{}\n",
                        format!("{:>4}", " ").bright_black(),
                        "│".bright_black(),
                        " ".repeat(adjusted_col),
                        "^".repeat(underline_len).red().bold()
                    ));
                }

                // Líneas después del error
                if let Some(after) = &loc.context_after {
                    for (idx, line) in after.iter().enumerate() {
                        let line_num = loc.line + idx + 1;
                        output.push_str(&format!(
                            "  {} {} {}\n",
                            format!("{:>4}", line_num).bright_black(),
                            "│".bright_black(),
                            line
                        ));
                    }
                }

                output.push_str(&format!(
                    "  {} {}\n",
                    format!("{:>4}", " ").bright_black(),
                    "│".bright_black()
                ));
            }
        }

        // Mensaje principal
        output.push_str(&format!("\n  {} {}\n", "ⓘ".blue().bold(), self.message));

        // Ayuda si está disponible
        if let Some(help) = &self.help {
            output.push_str(&format!("\n  {} {}\n", "💡".yellow(), help.bright_white()));
        }

        // Separador final
        output.push_str(&format!("{}\n", "─".repeat(60).bright_black()));

        output
    }

    /// Convert to JSON format for IDE integration
    pub fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to pretty-printed JSON format
    pub fn to_json_pretty(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl SemanticErrorInfo {
    /// Create a simple semantic error from just a message string
    pub fn simple(message: impl Into<String>) -> Self {
        let msg = message.into();
        SemanticErrorInfo {
            location: None,
            code: String::new(),
            title: msg.clone(),
            message: msg,
            help: None,
        }
    }
}

impl From<String> for SemanticErrorInfo {
    fn from(message: String) -> Self {
        SemanticErrorInfo::simple(message)
    }
}

impl From<&str> for SemanticErrorInfo {
    fn from(message: &str) -> Self {
        SemanticErrorInfo::simple(message)
    }
}

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("{}", .0.format())]
    LexerError(SemanticErrorInfo),

    #[error("{}", .0.format())]
    ParseError(SemanticErrorInfo),

    #[error("{}", .0.format())]
    SemanticError(SemanticErrorInfo),

    #[error("{}", .0.format())]
    TypeError(SemanticErrorInfo),

    #[error("{}", .0.format())]
    CodegenError(SemanticErrorInfo),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

impl CompilerError {
    /// Check if error can be serialized to JSON (all structured errors can)
    pub fn can_serialize_json(&self) -> bool {
        !matches!(
            self,
            CompilerError::IoError(_) | CompilerError::RuntimeError(_)
        )
    }

    /// Get the underlying SemanticErrorInfo if available
    pub fn error_info(&self) -> Option<&SemanticErrorInfo> {
        match self {
            CompilerError::LexerError(info) => Some(info),
            CompilerError::ParseError(info) => Some(info),
            CompilerError::SemanticError(info) => Some(info),
            CompilerError::TypeError(info) => Some(info),
            CompilerError::CodegenError(info) => Some(info),
            _ => None,
        }
    }

    /// Convert to JSON if possible
    pub fn to_json(&self) -> Option<String> {
        self.error_info().and_then(|info| info.to_json().ok())
    }
}

pub type Result<T> = std::result::Result<T, CompilerError>;
