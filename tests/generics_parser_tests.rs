use insta::assert_snapshot;
use livac::{lexer::tokenize, parser::parse};
use std::fs;

/// Test helper for generic parser tests
fn test_generic_ok(test_name: &str) {
    let source = fs::read_to_string(format!("tests/parser/generics/ok_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: ok_{}.liva", test_name));
    // Normalize line endings for cross-platform snapshot consistency
    let source = source.replace("\r\n", "\n");

    let tokens = tokenize(&source).unwrap();
    let ast = parse(tokens, &source).unwrap();

    // Convert AST to JSON for snapshot
    let json = serde_json::to_string_pretty(&ast).unwrap();
    assert_snapshot!(format!("generics_ok_{}.ast", test_name), json);
}

#[test]
fn test_generic_function_simple() {
    test_generic_ok("generic_function_simple");
}

#[test]
fn test_generic_function_multiple() {
    test_generic_ok("generic_function_multiple");
}

#[test]
fn test_generic_function_constraint() {
    test_generic_ok("generic_function_constraint");
}

#[test]
fn test_generic_function_multiple_constraints() {
    test_generic_ok("generic_function_multiple_constraints");
}

#[test]
fn test_generic_class_simple() {
    test_generic_ok("generic_class_simple");
}

#[test]
fn test_generic_class_multiple() {
    test_generic_ok("generic_class_multiple");
}

#[test]
fn test_generic_class_with_constraint() {
    test_generic_ok("generic_class_with_constraint");
}

#[test]
fn test_generic_method() {
    test_generic_ok("generic_method");
}

#[test]
fn test_identity_oneliner() {
    test_generic_ok("identity_oneliner");
}

#[test]
fn test_generic_type_arguments() {
    test_generic_ok("generic_type_arguments");
}

#[test]
fn test_nested_generics() {
    test_generic_ok("nested_generics");
}
