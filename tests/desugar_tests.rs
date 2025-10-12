use insta::assert_snapshot;
use livac::{desugaring::desugar, lexer::tokenize, parser::parse, semantic::analyze};
use std::fs;

/// Test helper para casos correctos del desugaring
fn test_desugar_ok(test_name: &str) {
    let source = fs::read_to_string(format!("tests/desugar/ok_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: ok_{}.liva", test_name));

    let tokens = tokenize(&source).unwrap();
    let ast = parse(tokens, &source).unwrap();
    let analyzed = analyze(ast).unwrap();
    let ctx = desugar(analyzed).unwrap();

    // Convertir contexto de desugaring a JSON para snapshot
    let json = serde_json::to_string_pretty(&ctx).unwrap();
    assert_snapshot!(format!("ok_{}.ctx", test_name), json);
}

#[test]
fn test_functions_oneliner() {
    test_desugar_ok("functions_oneliner");
}

#[test]
fn test_classes() {
    test_desugar_ok("classes");
}

#[test]
fn test_async_parallel_fire() {
    test_desugar_ok("async_parallel_fire");
}

#[test]
fn test_string_templates() {
    test_desugar_ok("string_templates");
}

#[test]
fn test_rust_crates() {
    test_desugar_ok("rust_crates");
}
