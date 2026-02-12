use insta::assert_snapshot;
use livac::codegen::generate_with_ast;
use livac::lexer::tokenize;
use livac::parser::parse;
use livac::semantic::analyze;

fn compile_and_generate(source: &str) -> String {
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, source).unwrap();
    let analyzed_program = analyze(program).unwrap();
    let ctx = livac::desugaring::desugar(analyzed_program.clone()).unwrap();
    let (rust_code, _cargo_toml) = generate_with_ast(&analyzed_program, ctx).unwrap();
    rust_code
}

#[test]
fn test_async_main_generation() {
    let source = r#"
main() {
  print("Hello from async main!")
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("async_main", rust_code);
}

#[test]
fn test_function_name_generation() {
    let source = r#"
sum(a, b) => a + b
multiply(x, y) => x * y
greet(name) => "Hello " + name

main() {
  let result1 = sum(2, 3)
  let result2 = multiply(4, 5)
  let message = greet("World")
  
  print(result1)
  print(result2)
  print(message)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("function_names", rust_code);
}

#[test]
fn test_return_type_inference() {
    let source = r#"
add(a, b) => a + b
subtract(x, y) = x - y
multiply(n, m) = n * m

main() {
  let sum = add(10, 5)
  let diff = subtract(10, 3)
  let prod = multiply(4, 6)
  
  print(sum)
  print(diff)
  print(prod)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("return_type_inference", rust_code);
}

#[test]
fn test_mixed_function_types() {
    let source = r#"
simpleAdd(a, b) = a + b

complexAdd(x, y) {
  let result = x + y
  return result
}

main() {
  let simple = simpleAdd(1, 2)
  let complex = complexAdd(3, 4)
  
  print(simple)
  print(complex)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("mixed_functions", rust_code);
}

#[test]
fn test_explicit_return_types() {
    let source = r#"
add(a: number, b: number): number => a + b
greet(name: string): string => "Hello " + name
isEven(n: number): bool => n % 2 == 0

main() {
  let sum = add(5, 3)
  let message = greet("Liva")
  let even = isEven(4)
  
  print(sum)
  print(message)
  print(even)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("explicit_return_types", rust_code);
}

#[test]
fn test_comprehensive_codegen() {
    let source = r#"
// Simple expression-bodied functions
add(a, b) => a + b
multiply(x, y) = x * y

// Function with explicit return type
power(base: number, exp: number): number = base * exp

// Block function
complexCalculation(n: number) {
  let doubled = n * 2
  let squared = doubled * doubled
  return squared
}

// Async main function
main() {
  let sum = add(10, 5)
  let product = multiply(3, 4)
  let powered = power(2, 3)
  let complex = complexCalculation(5)
  
  print("Sum: " + sum)
  print("Product: " + product)
  print("Power: " + powered)
  print("Complex: " + complex)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("comprehensive_codegen", rust_code);
}

#[test]
fn test_file_io_operations() {
    let source = r#"
main() {
    // Test File.read with error binding
    let content, err = File.read("test.txt")
    if err {
        print("Read error: " + err.message)
    } else {
        print("Content: " + content)
    }
    
    // Test File.write
    let success, writeErr = File.write("output.txt", "Hello, Liva!")
    
    // Test File.append
    let ok, appendErr = File.append("log.txt", "New entry")
    
    // Test File.exists (no error binding)
    if File.exists("config.json") {
        print("Config exists")
    }
    
    // Test File.delete
    let deleted, delErr = File.delete("temp.txt")
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("file_io_operations", rust_code);
}

#[test]
fn test_point_free_function_references() {
    let source = r#"
double(x) => x * 2
isPositive(n) => n > 0

main() {
    let items = [1, 2, 3, 4, 5]
    
    // Point-free: pass function name directly
    items.forEach(print)
    
    let doubled = items.map(double)
    let positives = items.filter(isPositive)
    
    let names = ["Alice", "Bob", "Charlie"]
    names.forEach(print)
    
    let strs = items.map(toString)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("point_free_function_refs", rust_code);
}

#[test]
fn test_point_free_for_loop() {
    let source = r#"
showItem(n: number) {
    print($"Item: {n}")
}

main() {
    let items = [1, 2, 3, 4, 5]
    
    // Point-free in for loop with print
    for item in items => print
    
    let items2 = [10, 20, 30]
    
    // Point-free in for loop with user function
    for item in items2 => showItem
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("point_free_for_loop", rust_code);
}

#[test]
fn test_method_ref_double_colon() {
    let source = r#"
Formatter {
    prefix: string
    constructor(prefix: string) { this.prefix = prefix }
    format(s: string) => $"{this.prefix}: {s}"
}

main() {
    let names = ["Alice", "Bob", "Charlie"]
    let formatter = Formatter("Hello")

    // Method reference with :: in map
    let formatted = names.map(formatter::format)
    formatted.forEach(print)

    // Method reference with :: in forEach
    let greeter = Formatter("Hi")
    names.forEach(greeter::format)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("method_ref_double_colon", rust_code);
}

// ─── Phase 12.3: Lifecycle Hooks Auto-Invocation ──────────────────────

#[test]
fn test_lifecycle_before_each_auto_invocation() {
    let source = r#"
import { describe, test, expect, beforeEach } from "liva/test"

add(a: int, b: int): int => a + b

describe("Math", () => {
    beforeEach(() => {
        print("setup")
    })

    test("addition", () => {
        expect(add(1, 2)).toBe(3)
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // Should generate a before_each function
    assert!(rust_code.contains("fn before_each()"), "should generate before_each fn:\n{}", rust_code);
    // The test should auto-invoke before_each()
    assert!(rust_code.contains("before_each();"), "test should call before_each():\n{}", rust_code);
    // before_each() should appear BEFORE the assertion
    let before_pos = rust_code.find("before_each();").unwrap();
    let assert_pos = rust_code.find("assert_eq!").unwrap();
    assert!(before_pos < assert_pos, "before_each() should come before assertions");
}

#[test]
fn test_lifecycle_after_each_auto_invocation() {
    let source = r#"
import { describe, test, expect, afterEach } from "liva/test"

add(a: int, b: int): int => a + b

describe("Math", () => {
    afterEach(() => {
        print("teardown")
    })

    test("addition", () => {
        expect(add(1, 2)).toBe(3)
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // Should generate an after_each function
    assert!(rust_code.contains("fn after_each()"), "should generate after_each fn:\n{}", rust_code);
    // The test should auto-invoke after_each()
    assert!(rust_code.contains("after_each();"), "test should call after_each():\n{}", rust_code);
    // after_each() should appear AFTER the assertion
    let assert_pos = rust_code.find("assert_eq!").unwrap();
    let after_pos = rust_code.find("after_each();").unwrap();
    assert!(after_pos > assert_pos, "after_each() should come after assertions");
}

#[test]
fn test_lifecycle_both_hooks() {
    let source = r#"
import { describe, test, expect, beforeEach, afterEach } from "liva/test"

add(a: int, b: int): int => a + b

describe("Calculator", () => {
    beforeEach(() => {
        print("setup")
    })

    afterEach(() => {
        print("cleanup")
    })

    test("add works", () => {
        expect(add(2, 3)).toBe(5)
    })

    test("add negatives", () => {
        expect(add(-1, 1)).toBe(0)
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // Both hooks should exist
    assert!(rust_code.contains("fn before_each()"), "should generate before_each fn");
    assert!(rust_code.contains("fn after_each()"), "should generate after_each fn");
    
    // Count occurrences of hook calls — should be 2 each (one per test)
    let before_count = rust_code.matches("before_each();").count();
    let after_count = rust_code.matches("after_each();").count();
    assert_eq!(before_count, 2, "before_each() should be called in each test, found {}", before_count);
    assert_eq!(after_count, 2, "after_each() should be called in each test, found {}", after_count);
}

#[test]
fn test_lifecycle_nested_describe_inherits_hooks() {
    let source = r#"
import { describe, test, expect, beforeEach } from "liva/test"

add(a: int, b: int): int => a + b

describe("Outer", () => {
    beforeEach(() => {
        print("outer setup")
    })

    describe("Inner", () => {
        beforeEach(() => {
            print("inner setup")
        })

        test("nested test", () => {
            expect(add(1, 1)).toBe(2)
        })
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // Should have both hook functions (with depth-based naming)
    assert!(rust_code.contains("fn before_each()"), "outer before_each fn should exist:\n{}", rust_code);
    assert!(rust_code.contains("fn before_each_1()"), "inner before_each_1 fn should exist:\n{}", rust_code);
    // The nested test should call BOTH hooks (parent first, then inner)
    assert!(rust_code.contains("before_each();"), "nested test should call parent before_each:\n{}", rust_code);
    assert!(rust_code.contains("before_each_1();"), "nested test should call inner before_each_1:\n{}", rust_code);
}

#[test]
fn test_lifecycle_no_hooks_no_calls() {
    let source = r#"
import { describe, test, expect } from "liva/test"

add(a: int, b: int): int => a + b

describe("Simple", () => {
    test("basic", () => {
        expect(add(1, 1)).toBe(2)
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // No hook calls should be generated when no hooks are defined
    assert!(!rust_code.contains("before_each"), "no before_each when none defined:\n{}", rust_code);
    assert!(!rust_code.contains("after_each"), "no after_each when none defined:\n{}", rust_code);
}

#[test]
fn test_lifecycle_before_all_after_all() {
    let source = r#"
import { describe, test, expect, beforeAll, afterAll } from "liva/test"

add(a: int, b: int): int => a + b

describe("Suite", () => {
    beforeAll(() => {
        print("suite setup")
    })

    afterAll(() => {
        print("suite teardown")
    })

    test("first", () => {
        expect(add(1, 1)).toBe(2)
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // beforeAll and afterAll should generate functions  
    assert!(rust_code.contains("fn before_all()"), "should generate before_all fn:\n{}", rust_code);
    assert!(rust_code.contains("fn after_all()"), "should generate after_all fn:\n{}", rust_code);
    // They should NOT be auto-invoked in test functions (they're module-level)
    assert!(!rust_code.contains("before_all();"), "before_all should not be auto-invoked in tests");
    assert!(!rust_code.contains("after_all();"), "after_all should not be auto-invoked in tests");
}
