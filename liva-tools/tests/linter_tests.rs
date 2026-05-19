/// Tests for the Liva linter (W001-W004)
use liva_tools::linter;

/// Helper: parse source and run linter, return warnings
fn lint_source(source: &str) -> Vec<linter::LintWarning> {
    let tokens = livac::lexer::tokenize(source).expect("tokenize failed");
    let ast = livac::parser::parse(tokens, source).expect("parse failed");
    linter::lint(&ast, "test.liva", source)
}

/// Helper: get warning codes from a source
fn lint_codes(source: &str) -> Vec<String> {
    lint_source(source).iter().map(|w| w.code.clone()).collect()
}

// ─── W001: Unused variables ─────────────────────────────────────

#[test]
fn w001_unused_variable() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
    let y = 10
    console.log(x)
}
"#,
    );
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code, "W001");
    assert!(warnings[0].message.contains("'y'"));
}

#[test]
fn w001_no_warning_when_used() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
    console.log(x)
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert!(w001.is_empty());
}

#[test]
fn w001_underscore_prefix_suppresses() {
    let warnings = lint_source(
        r#"
main() {
    let _unused = 42
    console.log("hello")
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert!(w001.is_empty());
}

#[test]
fn w001_multiple_unused() {
    let warnings = lint_source(
        r#"
main() {
    let a = 1
    let b = 2
    let c = 3
    console.log("nothing used")
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert_eq!(w001.len(), 3);
}

#[test]
fn w001_used_in_expression() {
    let warnings = lint_source(
        r#"
main() {
    let x = 10
    let y = x + 5
    console.log(y)
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert!(w001.is_empty());
}

#[test]
fn w001_for_loop_var_used() {
    let warnings = lint_source(
        r#"
main() {
    let items = [1, 2, 3]
    for item in items {
        console.log(item)
    }
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert!(w001.is_empty());
}

#[test]
fn w001_for_loop_var_unused() {
    let warnings = lint_source(
        r#"
main() {
    let items = [1, 2, 3]
    for item in items {
        console.log("hello")
    }
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert!(w001.iter().any(|w| w.message.contains("'item'")));
}

#[test]
fn w001_const_unused() {
    let warnings = lint_source(
        r#"
main() {
    const MAX = 100
    console.log("hello")
}
"#,
    );
    let w001: Vec<_> = warnings.iter().filter(|w| w.code == "W001").collect();
    assert_eq!(w001.len(), 1);
    assert!(w001[0].message.contains("'MAX'"));
}

// ─── W002: Unused imports ───────────────────────────────────────

#[test]
fn w002_unused_import() {
    let warnings = lint_source(
        r#"
import { add, subtract } from "./math.liva"

main() {
    let x = add(1, 2)
    console.log(x)
}
"#,
    );
    let w002: Vec<_> = warnings.iter().filter(|w| w.code == "W002").collect();
    assert_eq!(w002.len(), 1);
    assert!(w002[0].message.contains("'subtract'"));
}

#[test]
fn w002_no_warning_all_used() {
    let warnings = lint_source(
        r#"
import { add } from "./math.liva"

main() {
    let x = add(1, 2)
    console.log(x)
}
"#,
    );
    let w002: Vec<_> = warnings.iter().filter(|w| w.code == "W002").collect();
    assert!(w002.is_empty());
}

#[test]
fn w002_multiple_unused() {
    let warnings = lint_source(
        r#"
import { foo, bar, baz } from "./utils.liva"

main() {
    console.log("nothing imported is used")
}
"#,
    );
    let w002: Vec<_> = warnings.iter().filter(|w| w.code == "W002").collect();
    assert_eq!(w002.len(), 3);
}

// ─── W003: Unreachable code ─────────────────────────────────────

#[test]
fn w003_after_return() {
    let warnings = lint_source(
        r#"
main() {
    return "done"
    console.log("unreachable")
}
"#,
    );
    let w003: Vec<_> = warnings.iter().filter(|w| w.code == "W003").collect();
    assert_eq!(w003.len(), 1);
    assert!(w003[0].message.contains("return"));
}

#[test]
fn w003_after_fail() {
    let warnings = lint_source(
        r#"
process(): string {
    fail "error occurred"
    return "never"
}
"#,
    );
    let w003: Vec<_> = warnings.iter().filter(|w| w.code == "W003").collect();
    assert_eq!(w003.len(), 1);
    assert!(w003[0].message.contains("fail"));
}

#[test]
fn w003_no_warning_no_unreachable() {
    let warnings = lint_source(
        r#"
main() {
    console.log("hello")
    return "done"
}
"#,
    );
    let w003: Vec<_> = warnings.iter().filter(|w| w.code == "W003").collect();
    assert!(w003.is_empty());
}

#[test]
fn w003_return_in_if_ok() {
    // return in if/else branches is fine — code after the if is reachable
    // if only one branch returns
    let warnings = lint_source(
        r#"
check(x: int): string {
    if x > 0 {
        return "positive"
    }
    return "non-positive"
}
"#,
    );
    let w003: Vec<_> = warnings.iter().filter(|w| w.code == "W003").collect();
    assert!(w003.is_empty());
}

// ─── W004: Always true/false comparisons ────────────────────────

#[test]
fn w004_same_variable() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
    if x == x {
        console.log("always true")
    }
}
"#,
    );
    let w004: Vec<_> = warnings.iter().filter(|w| w.code == "W004").collect();
    assert_eq!(w004.len(), 1);
    assert!(w004[0].title.contains("always true"));
}

#[test]
fn w004_same_variable_ne() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
    if x != x {
        console.log("always false")
    }
}
"#,
    );
    let w004: Vec<_> = warnings.iter().filter(|w| w.code == "W004").collect();
    assert_eq!(w004.len(), 1);
    assert!(w004[0].title.contains("always false"));
}

#[test]
fn w004_true_equals_true() {
    let warnings = lint_source(
        r#"
main() {
    if true == true {
        console.log("always")
    }
}
"#,
    );
    let w004: Vec<_> = warnings.iter().filter(|w| w.code == "W004").collect();
    assert!(w004.len() >= 1);
    assert!(w004[0].title.contains("always true"));
}

#[test]
fn w004_different_literals_eq() {
    let warnings = lint_source(
        r#"
main() {
    if 42 == 99 {
        console.log("never")
    }
}
"#,
    );
    let w004: Vec<_> = warnings.iter().filter(|w| w.code == "W004").collect();
    assert_eq!(w004.len(), 1);
    assert!(w004[0].title.contains("always false"));
}

#[test]
fn w004_different_literals_ne() {
    let warnings = lint_source(
        r#"
main() {
    if "a" != "b" {
        console.log("always true")
    }
}
"#,
    );
    let w004: Vec<_> = warnings.iter().filter(|w| w.code == "W004").collect();
    assert_eq!(w004.len(), 1);
    assert!(w004[0].title.contains("always true"));
}

#[test]
fn w004_no_warning_normal_comparison() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
    if x == 10 {
        console.log("maybe")
    }
}
"#,
    );
    let w004: Vec<_> = warnings.iter().filter(|w| w.code == "W004").collect();
    assert!(w004.is_empty());
}

// ─── Combined: no false positives ───────────────────────────────

#[test]
fn clean_code_no_warnings() {
    let warnings = lint_source(
        r#"
greet(name: string): string {
    return "Hello, " + name
}

main() {
    let msg = greet("World")
    console.log(msg)
}
"#,
    );
    assert!(warnings.is_empty());
}

// ─── Format: JSON output ────────────────────────────────────────

#[test]
fn json_output_valid() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
}
"#,
    );
    let json = linter::format_warnings_json(&warnings);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

#[test]
fn text_output_format() {
    let warnings = lint_source(
        r#"
main() {
    let x = 42
}
"#,
    );
    let output = linter::format_warnings(&warnings);
    assert!(output.contains("W001"));
    assert!(output.contains("warning"));
}
