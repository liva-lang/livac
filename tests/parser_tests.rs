use insta::assert_snapshot;
use livac::{lexer::tokenize, parser::parse};
use std::fs;

/// Test helper para casos correctos del parser
fn test_parser_ok(test_name: &str) {
    let source = fs::read_to_string(format!("tests/parser/ok_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: ok_{}.liva", test_name));

    let tokens = tokenize(&source).unwrap();
    let ast = parse(tokens, &source).unwrap();

    // Convertir AST a JSON para snapshot
    let json = serde_json::to_string_pretty(&ast).unwrap();
    assert_snapshot!(format!("ok_{}.ast", test_name), json);
}

/// Test helper para casos de error del parser
fn test_parser_err(test_name: &str) {
    let source = fs::read_to_string(format!("tests/parser/err_{}.liva", test_name))
        .unwrap_or_else(|_| panic!("Failed to read test file: err_{}.liva", test_name));

    let tokens = tokenize(&source).unwrap();
    let result = parse(tokens, &source);
    assert!(
        result.is_err(),
        "Expected parser error for test: {}",
        test_name
    );

    let error_msg = result.unwrap_err().to_string();
    assert_snapshot!(format!("err_{}.diag", test_name), error_msg);
}

#[test]
fn test_functions_oneliner() {
    test_parser_ok("functions_oneliner");
}

#[test]
fn test_functions_block() {
    test_parser_ok("functions_block");
}

#[test]
fn test_classes() {
    test_parser_ok("classes");
}

#[test]
fn test_control_flow() {
    test_parser_ok("control_flow");
}

#[test]
fn test_expressions() {
    test_parser_ok("expressions");
}

#[test]
#[ignore = "TODO: Fix import std syntax parsing"]
fn test_imports() {
    test_parser_ok("imports");
}

#[test]
fn test_types_advanced() {
    test_parser_ok("types_advanced");
}

#[test]
fn test_expressions_advanced() {
    test_parser_ok("expressions_advanced");
}

#[test]
fn test_concurrency_advanced() {
    test_parser_ok("concurrency_advanced");
}

#[test]
fn test_error_handling_advanced() {
    test_parser_ok("error_handling_advanced");
}

#[test]
fn test_generics_advanced() {
    test_parser_ok("generics_advanced");
}

#[test]
fn test_constants_advanced() {
    test_parser_ok("constants_advanced");
}

#[test]
fn test_complex_expressions() {
    test_parser_err("complex_expressions");
}

#[test]
fn test_type_errors() {
    test_parser_err("type_errors");
}

#[test]
fn test_unclosed_paren() {
    test_parser_err("unclosed_paren");
}

#[test]
fn test_unclosed_brace() {
    test_parser_err("unclosed_brace");
}

#[test]
fn test_case_without_switch() {
    test_parser_err("case_without_switch");
}

#[test]
fn test_return_outside_function() {
    test_parser_err("return_outside_function");
}

#[test]
fn test_duplicate_default() {
    test_parser_err("duplicate_default");
}

#[test]
fn test_modifier_on_function_decl() {
    test_parser_err("invalid_modifier_decl");
}

#[test]
fn test_modifier_on_class_member() {
    test_parser_err("invalid_modifier_member");
}

#[test]
fn test_param_destructuring() {
    test_parser_ok("param_destructuring");
}

#[test]
fn test_union_patterns() {
    test_parser_ok("union_patterns");
}

#[test]
fn test_type_aliases() {
    test_parser_ok("type_aliases");
}

#[test]
fn test_union_types() {
    test_parser_ok("union_types");
}

#[test]
fn test_or_fail() {
    test_parser_ok("or_fail");
}

#[test]
fn test_oneliner_arrow() {
    test_parser_ok("oneliner_arrow");
}

#[test]
fn test_method_references() {
    test_parser_ok("method_references");
}
