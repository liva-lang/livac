use insta::assert_snapshot;
use livac::{lexer::tokenize, parser::parse, semantic::analyze};
use std::fs;

/// Test helper para casos correctos del análisis semántico
fn test_semantics_ok(test_name: &str) {
    let source = fs::read_to_string(format!("tests/semantics/ok_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: ok_{}.liva", test_name));

    let tokens = tokenize(&source).unwrap();
    let ast = parse(tokens, &source).unwrap();
    let analyzed = analyze(ast).unwrap();

    // Convertir AST analizado a JSON para snapshot
    let json = serde_json::to_string_pretty(&analyzed).unwrap();
    assert_snapshot!(format!("ok_{}.sem", test_name), json);
}

/// Test helper para casos de error del análisis semántico
fn test_semantics_err(test_name: &str) {
    let source = fs::read_to_string(format!("tests/semantics/err_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: err_{}.liva", test_name));

    let tokens = tokenize(&source).unwrap();
    let ast = parse(tokens, &source).unwrap();
    let result = analyze(ast);
    assert!(
        result.is_err(),
        "Expected semantic error for test: {}",
        test_name
    );

    let error_msg = result.unwrap_err().to_string();
    assert_snapshot!(format!("err_{}.diag", test_name), error_msg);
}

#[test]
fn test_type_inference() {
    test_semantics_ok("type_inference");
}

#[test]
fn test_async_inference() {
    test_semantics_ok("async_inference");
}

#[test]
fn test_visibility() {
    test_semantics_ok("visibility");
}

#[test]
fn test_rust_types() {
    test_semantics_ok("rust_types");
}

#[test]
fn test_number_plus_float() {
    test_semantics_ok("number_plus_float");
}

#[test]
fn test_private_access() {
    test_semantics_ok("private_access");
}

#[test]
fn test_protected_access() {
    test_semantics_ok("protected_access");
}

#[test]
fn test_undefined_type() {
    test_semantics_ok("undefined_type");
}

#[test]
fn test_async_without_await() {
    test_semantics_ok("async_without_await");
}

#[test]
fn test_undefined_variable() {
    test_semantics_ok("undefined_variable");
}

#[test]
fn test_len_function_call() {
    test_semantics_err("len_function_call");
}

#[test]
fn test_length_misuse() {
    test_semantics_err("length_invalid");
}

#[test]
fn test_task_never_awaited() {
    test_semantics_err("task_never_awaited");
}

#[test]
fn test_task_double_await() {
    test_semantics_err("task_double_await");
}

#[test]
fn test_invalid_await_call() {
    test_semantics_err("invalid_await_call");
}

#[test]
fn test_for_par_await() {
    test_semantics_err("for_par_await");
}

#[test]
fn test_for_seq_simd_invalid() {
    test_semantics_err("for_seq_simd");
}

#[test]
fn test_for_par_chunk_invalid() {
    test_semantics_err("for_par_chunk_invalid");
}
