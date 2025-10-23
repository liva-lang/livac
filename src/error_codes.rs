/// Error code constants for the Liva compiler
///
/// This module centralizes all error codes used throughout the compiler.
/// Error codes follow a structured numbering system:
///
/// - E0xxx: General semantic errors (0000-0999)
/// - E1xxx: Lexer errors (1000-1999)
/// - E2xxx: Parser errors (2000-2999)
/// - E3xxx: Code generation errors (3000-3999)
/// - E4xxx: Module system errors (4000-4999)
/// - E5xxx: Type system errors (5000-5999)
/// - E6xxx: Concurrency errors (6000-6999)
/// - E7xxx: Error handling errors (7000-7999)

// ============================================================================
// E0xxx: General Semantic Errors
// ============================================================================

pub const E0000_GENERIC: &str = "E0000";
pub const E0001_INTERFACE_NOT_IMPL: &str = "E0001";
pub const E0002_METHOD_SIGNATURE_MISMATCH: &str = "E0002";

// ============================================================================
// E1xxx: Lexer Errors
// ============================================================================

pub const E1000_LEXER_ERROR: &str = "E1000";

// ============================================================================
// E2xxx: Parser Errors
// ============================================================================

pub const E2000_PARSE_ERROR: &str = "E2000";
pub const E2001_INVALID_EXEC_MODIFIER: &str = "E2001";
pub const E2002_DUPLICATE_EXEC_MODIFIER: &str = "E2002";
pub const E2003_INVALID_LOOP_POLICY: &str = "E2003";

// ============================================================================
// E3xxx: Code Generation Errors
// ============================================================================

pub const E3000_CODEGEN_ERROR: &str = "E3000";
pub const E3001_COMPILATION_FAILED: &str = "E3001";
pub const E3002_BUILD_FAILED: &str = "E3002";

// ============================================================================
// E4xxx: Module System Errors
// ============================================================================

pub const E4003_INVALID_MODULE_PATH: &str = "E4003";
pub const E4004_MODULE_NOT_FOUND: &str = "E4004";
pub const E4005_MODULE_COMPILATION_FAILED: &str = "E4005";
pub const E4006_SYMBOL_NOT_FOUND: &str = "E4006";
pub const E4007_INVALID_IMPORT_SYNTAX: &str = "E4007";
pub const E4008_EMPTY_IMPORT_LIST: &str = "E4008";
pub const E4009_MODULE_NOT_EXPORTED: &str = "E4009";

// ============================================================================
// E5xxx: Type System Errors
// ============================================================================

pub const E5001_TYPE_MISMATCH: &str = "E5001";

// ============================================================================
// E6xxx: Concurrency Errors
// ============================================================================

pub const E0401_INVALID_CONCURRENT_EXEC: &str = "E0401";
pub const E0402_UNSAFE_CONCURRENT_ACCESS: &str = "E0402";
pub const E0510_NON_SEND_CAPTURE: &str = "E0510";
pub const E0511_NON_SYNC_CAPTURE: &str = "E0511";
pub const E0602_DUPLICATE_EXEC_MODIFIER: &str = "E0602";
pub const E0603_NOT_AWAITABLE: &str = "E0603";
pub const E0604_AWAIT_MULTIPLE_TIMES: &str = "E0604";
pub const E0605_AWAIT_IN_PARALLEL_LOOP: &str = "E0605";

// ============================================================================
// E7xxx: Error Handling Errors
// ============================================================================

pub const E0701_FALLIBLE_WITHOUT_BINDING: &str = "E0701";
pub const E0702_INVALID_CHUNK_SIZE: &str = "E0702";
pub const E0703_INVALID_PREFETCH_SIZE: &str = "E0703";
pub const E0704_INVALID_THREAD_COUNT: &str = "E0704";
pub const E0705_SIMD_WITHOUT_VEC: &str = "E0705";
pub const E0706_INVALID_SIMD_WIDTH: &str = "E0706";

// ============================================================================
// Error Categories
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Semantic,
    Lexer,
    Parser,
    Codegen,
    Module,
    Type,
    Concurrency,
    ErrorHandling,
}

impl ErrorCategory {
    /// Get the category from an error code
    pub fn from_code(code: &str) -> Option<Self> {
        if !code.starts_with('E') || code.len() < 5 {
            return None;
        }

        let num = &code[1..2];
        match num {
            "0" => Some(ErrorCategory::Semantic),
            "1" => Some(ErrorCategory::Lexer),
            "2" => Some(ErrorCategory::Parser),
            "3" => Some(ErrorCategory::Codegen),
            "4" => Some(ErrorCategory::Module),
            "5" => Some(ErrorCategory::Type),
            "6" => Some(ErrorCategory::Concurrency),
            "7" => Some(ErrorCategory::ErrorHandling),
            _ => None,
        }
    }

    /// Get a human-readable name for the category
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::Semantic => "Semantic",
            ErrorCategory::Lexer => "Lexer",
            ErrorCategory::Parser => "Parser",
            ErrorCategory::Codegen => "Code Generation",
            ErrorCategory::Module => "Module System",
            ErrorCategory::Type => "Type System",
            ErrorCategory::Concurrency => "Concurrency",
            ErrorCategory::ErrorHandling => "Error Handling",
        }
    }

    /// Get a short description of the category
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCategory::Semantic => "Semantic analysis and validation errors",
            ErrorCategory::Lexer => "Lexical analysis and tokenization errors",
            ErrorCategory::Parser => "Syntax parsing errors",
            ErrorCategory::Codegen => "Code generation and compilation errors",
            ErrorCategory::Module => "Module import and resolution errors",
            ErrorCategory::Type => "Type checking and inference errors",
            ErrorCategory::Concurrency => "Async and parallel execution errors",
            ErrorCategory::ErrorHandling => "Fallibility and error binding errors",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_code() {
        assert_eq!(
            ErrorCategory::from_code("E0001"),
            Some(ErrorCategory::Semantic)
        );
        assert_eq!(
            ErrorCategory::from_code("E1000"),
            Some(ErrorCategory::Lexer)
        );
        assert_eq!(
            ErrorCategory::from_code("E2000"),
            Some(ErrorCategory::Parser)
        );
        assert_eq!(
            ErrorCategory::from_code("E3000"),
            Some(ErrorCategory::Codegen)
        );
        assert_eq!(
            ErrorCategory::from_code("E4000"),
            Some(ErrorCategory::Module)
        );
        assert_eq!(ErrorCategory::from_code("E9999"), None);
        assert_eq!(ErrorCategory::from_code("invalid"), None);
    }

    #[test]
    fn test_category_names() {
        assert_eq!(ErrorCategory::Semantic.name(), "Semantic");
        assert_eq!(ErrorCategory::Lexer.name(), "Lexer");
        assert_eq!(ErrorCategory::Parser.name(), "Parser");
    }
}
