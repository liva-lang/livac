use insta::assert_snapshot;
use livac::lexer::tokenize;
use std::fs;

/// Strip ANSI escape codes for deterministic snapshots
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ESC [ ... m sequences
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc == 'm' {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Test helper para casos correctos del lexer
fn test_lexer_ok(test_name: &str) {
    let source = fs::read_to_string(format!("tests/lexer/ok_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: ok_{}.liva", test_name));
    // Normalize line endings for cross-platform snapshot consistency
    let source = source.replace("\r\n", "\n");

    let tokens = tokenize(&source).unwrap();

    // Convertir tokens a formato legible para snapshot
    let token_strings: Vec<String> = tokens.iter().map(|t| format!("{:?}", t.token)).collect();

    let output = format!("Tokens: {:#?}", token_strings);
    assert_snapshot!(format!("ok_{}.tokens", test_name), output);
}

/// Test helper para casos de error del lexer
fn test_lexer_err(test_name: &str) {
    let source = fs::read_to_string(format!("tests/lexer/err_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: err_{}.liva", test_name));
    // Normalize line endings for cross-platform snapshot consistency
    let source = source.replace("\r\n", "\n");

    let result = tokenize(&source);
    assert!(
        result.is_err(),
        "Expected lexer error for test: {}",
        test_name
    );

    let error_msg = strip_ansi(&result.unwrap_err().to_string());
    assert_snapshot!(format!("err_{}.diag", test_name), error_msg);
}

#[test]
fn test_identifiers() {
    test_lexer_ok("identifiers");
}

#[test]
fn test_literals() {
    test_lexer_ok("literals");
}

#[test]
fn test_operators() {
    test_lexer_ok("operators");
}

#[test]
fn test_keywords() {
    test_lexer_ok("keywords");
}

#[test]
fn test_comments() {
    test_lexer_ok("comments");
}

#[test]
fn test_unknown_token() {
    test_lexer_err("unknown_token");
}

#[test]
fn test_unclosed_string() {
    test_lexer_err("unclosed_string");
}

#[test]
fn test_unclosed_char() {
    test_lexer_err("unclosed_char");
}

#[test]
fn test_unclosed_comment() {
    test_lexer_err("unclosed_comment");
}
