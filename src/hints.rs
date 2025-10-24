/// Error hints and helpful messages
///
/// This module provides contextual help messages for common errors in Liva.
/// Hints guide developers toward correct solutions with actionable advice.

use crate::error_codes::*;

/// Get a helpful hint for a specific error code
pub fn get_hint(error_code: &str) -> Option<&'static str> {
    match error_code {
        // Parser Errors
        E2000_PARSE_ERROR => Some("Check for missing semicolons, parentheses, or keywords"),
        
        // Module Errors
        E4003_INVALID_MODULE_PATH => Some("Module paths should be relative (e.g., './module') or from the standard library"),
        E4004_MODULE_NOT_FOUND => Some("Make sure the file exists and the path is correct"),
        E4006_SYMBOL_NOT_FOUND => Some("Check the module's exports or look for typos in the symbol name"),
        E4007_INVALID_IMPORT_SYNTAX => Some("Use: import { symbol1, symbol2 } from 'module'"),
        E4008_EMPTY_IMPORT_LIST => Some("Specify at least one symbol to import, or remove the import statement"),
        
        // Concurrency Errors
        E0602_DUPLICATE_EXEC_MODIFIER => Some("Use only one execution modifier: async, par, task async, task par, fire async, or fire par"),
        E0603_NOT_AWAITABLE => Some("Only async and task async expressions can be awaited"),
        E0604_AWAIT_MULTIPLE_TIMES => Some("Each async operation can only be awaited once. Store the result in a variable if needed"),
        E0605_AWAIT_IN_PARALLEL_LOOP => Some("Parallel loops execute synchronously. Use 'for async' for asynchronous iteration"),
        
        // Error Handling
        E0701_FALLIBLE_WITHOUT_BINDING => Some("Use error binding: let result, err = fallibleFunc(...)"),
        E0702_INVALID_CHUNK_SIZE => Some("Chunk size must be a positive integer, e.g., 'chunk 100'"),
        E0703_INVALID_PREFETCH_SIZE => Some("Prefetch size must be a positive integer, e.g., 'prefetch 10'"),
        E0704_INVALID_THREAD_COUNT => Some("Thread count must be a positive integer, e.g., 'threads 4'"),
        E0705_SIMD_WITHOUT_VEC => Some("SIMD width requires vectorized execution: use 'for vec' or 'for parvec'"),
        E0706_INVALID_SIMD_WIDTH => Some("SIMD width must be a positive integer (typically 4, 8, 16, or 32)"),
        
        // Semantic Errors
        E0001_INTERFACE_NOT_IMPL => Some("Implement all required methods or remove the interface declaration"),
        E0002_METHOD_SIGNATURE_MISMATCH => Some("Method signature must exactly match the interface definition"),
        
        _ => None,
    }
}

/// Get an example fix for a specific error
pub fn get_example(error_code: &str) -> Option<&'static str> {
    match error_code {
        E4007_INVALID_IMPORT_SYNTAX => Some(
            "// Correct:\nimport { add, subtract } from './math'\n\n// Incorrect:\nimport add from './math'"
        ),
        E0701_FALLIBLE_WITHOUT_BINDING => Some(
            "// Correct:\nlet result, err = divide(10, 2)\nif err == \"\" {\n  print(result)\n}\n\n// Incorrect:\ndivide(10, 2)"
        ),
        E0603_NOT_AWAITABLE => Some(
            "// Correct:\nlet result = await asyncFunc()\n\n// Incorrect:\nlet result = await parFunc()  // par completes eagerly"
        ),
        E0605_AWAIT_IN_PARALLEL_LOOP => Some(
            "// Correct:\nfor async item in items {\n  let data = await fetchData(item)\n}\n\n// Incorrect:\nfor par item in items {\n  await fetchData(item)  // Cannot await in parallel loop\n}"
        ),
        _ => None,
    }
}

/// Get related documentation URL for an error code
pub fn get_doc_link(error_code: &str) -> Option<String> {
    // Guard against empty error codes
    if error_code.len() < 2 {
        return None;
    }
    
    // Extract category from error code
    let category = match &error_code[1..2] {
        "0" => "semantic",
        "1" => "lexer",
        "2" => "parser",
        "3" => "codegen",
        "4" => "modules",
        "5" => "types",
        "6" => "concurrency",
        "7" => "error-handling",
        _ => return None,
    };
    
    Some(format!(
        "https://liva-lang.org/docs/errors/{}#{}",
        category,
        error_code.to_lowercase()
    ))
}

/// Get a list of common fixes for an error category
pub fn get_common_fixes(error_code: &str) -> Vec<&'static str> {
    match error_code {
        E2000_PARSE_ERROR => vec![
            "Add missing semicolons",
            "Check for unclosed parentheses or braces",
            "Verify keyword spelling",
            "Remove extra commas or operators",
        ],
        E4004_MODULE_NOT_FOUND => vec![
            "Check file path spelling",
            "Verify file extension (.liva)",
            "Use relative paths (./module or ../module)",
            "Ensure file exists in the expected location",
        ],
        E0701_FALLIBLE_WITHOUT_BINDING => vec![
            "Use error binding: let result, err = func(...)",
            "Check for error: if err != \"\" { ... }",
            "Or ignore error with: let result, _ = func(...)",
        ],
        _ => vec![],
    }
}

/// Get a quick tip for improving code based on the error
pub fn get_tip(error_code: &str) -> Option<&'static str> {
    match error_code {
        E0701_FALLIBLE_WITHOUT_BINDING => Some(
            "💡 Tip: Always handle errors from fallible functions to avoid runtime surprises"
        ),
        E4006_SYMBOL_NOT_FOUND => Some(
            "💡 Tip: Use your IDE's autocomplete to see available exports from modules"
        ),
        E0603_NOT_AWAITABLE => Some(
            "💡 Tip: Use 'async' for I/O operations and 'par' for CPU-bound tasks"
        ),
        E0605_AWAIT_IN_PARALLEL_LOOP => Some(
            "💡 Tip: Parallel loops execute synchronously for performance. Use 'for async' for concurrent iteration"
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hint() {
        assert!(get_hint(E2000_PARSE_ERROR).is_some());
        assert!(get_hint(E4006_SYMBOL_NOT_FOUND).is_some());
        assert!(get_hint(E0701_FALLIBLE_WITHOUT_BINDING).is_some());
        assert!(get_hint("E9999").is_none());
    }

    #[test]
    fn test_get_example() {
        assert!(get_example(E4007_INVALID_IMPORT_SYNTAX).is_some());
        assert!(get_example(E0701_FALLIBLE_WITHOUT_BINDING).is_some());
        assert!(get_example(E2000_PARSE_ERROR).is_none());
    }

    #[test]
    fn test_get_doc_link() {
        let link = get_doc_link(E2000_PARSE_ERROR);
        assert!(link.is_some());
        assert!(link.unwrap().contains("parser"));
        
        let link = get_doc_link(E4006_SYMBOL_NOT_FOUND);
        assert!(link.is_some());
        assert!(link.unwrap().contains("modules"));
    }

    #[test]
    fn test_get_common_fixes() {
        let fixes = get_common_fixes(E2000_PARSE_ERROR);
        assert!(!fixes.is_empty());
        
        let fixes = get_common_fixes(E0701_FALLIBLE_WITHOUT_BINDING);
        assert!(!fixes.is_empty());
        
        let fixes = get_common_fixes("E9999");
        assert!(fixes.is_empty());
    }

    #[test]
    fn test_get_tip() {
        assert!(get_tip(E0701_FALLIBLE_WITHOUT_BINDING).is_some());
        assert!(get_tip(E4006_SYMBOL_NOT_FOUND).is_some());
        assert!(get_tip(E0603_NOT_AWAITABLE).is_some());
        assert!(get_tip("E9999").is_none());
    }
}
