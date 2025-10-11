use thiserror::Error;

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

    #[error("Semantic error: {0}")]
    SemanticError(String),

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
