use thiserror::Error;
use colored::Colorize;

#[derive(Debug, Clone)]
pub struct ErrorLocation {
    pub file: String,
    pub line: usize,
    pub column: Option<usize>,
    pub source_line: Option<String>,
}

#[derive(Debug, Clone)]
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
        });
        self
    }

    pub fn with_source_line(mut self, source_line: String) -> Self {
        if let Some(loc) = &mut self.location {
            loc.source_line = Some(source_line);
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
            output.push_str(&format!("{}\n", loc.line.to_string().yellow().bold()));
            
            // Línea de código fuente si está disponible
            if let Some(source) = &loc.source_line {
                output.push_str("\n");
                output.push_str(&format!("  {} {}\n", 
                    format!("{:>4}", loc.line).bright_black(), 
                    "│".bright_black()
                ));
                output.push_str(&format!("  {} {} {}\n", 
                    format!("{:>4}", " ").bright_black(),
                    "│".bright_black(),
                    source.trim()
                ));
                output.push_str(&format!("  {} {}\n", 
                    format!("{:>4}", " ").bright_black(),
                    "│".bright_black()
                ));
            }
        }
        
        // Mensaje principal
        output.push_str(&format!("\n  {} {}\n", 
            "ⓘ".blue().bold(),
            self.message
        ));
        
        // Ayuda si está disponible
        if let Some(help) = &self.help {
            output.push_str(&format!("\n  {} {}\n", 
                "💡".yellow(),
                help.bright_white()
            ));
        }
        
        // Separador final
        output.push_str(&format!("{}\n", "─".repeat(60).bright_black()));
        
        output
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
    #[error("Lexer error: {0}")]
    LexerError(String),

    #[error("Parse error at line {line}, column {col}: {msg}")]
    ParseError {
        line: usize,
        col: usize,
        msg: String,
    },

    #[error("{}", .0.format())]
    SemanticError(SemanticErrorInfo),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Code generation error: {0}")]
    CodegenError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

pub type Result<T> = std::result::Result<T, CompilerError>;
