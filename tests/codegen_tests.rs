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
    assert!(
        rust_code.contains("fn before_each()"),
        "should generate before_each fn:\n{}",
        rust_code
    );
    // The test should auto-invoke before_each()
    assert!(
        rust_code.contains("before_each();"),
        "test should call before_each():\n{}",
        rust_code
    );
    // before_each() should appear BEFORE the assertion
    let before_pos = rust_code.find("before_each();").unwrap();
    let assert_pos = rust_code.find("assert_eq!").unwrap();
    assert!(
        before_pos < assert_pos,
        "before_each() should come before assertions"
    );
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
    assert!(
        rust_code.contains("fn after_each()"),
        "should generate after_each fn:\n{}",
        rust_code
    );
    // The test should auto-invoke after_each()
    assert!(
        rust_code.contains("after_each();"),
        "test should call after_each():\n{}",
        rust_code
    );
    // after_each() should appear AFTER the assertion
    let assert_pos = rust_code.find("assert_eq!").unwrap();
    let after_pos = rust_code.find("after_each();").unwrap();
    assert!(
        after_pos > assert_pos,
        "after_each() should come after assertions"
    );
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
    assert!(
        rust_code.contains("fn before_each()"),
        "should generate before_each fn"
    );
    assert!(
        rust_code.contains("fn after_each()"),
        "should generate after_each fn"
    );

    // Count occurrences of hook calls — should be 2 each (one per test)
    let before_count = rust_code.matches("before_each();").count();
    let after_count = rust_code.matches("after_each();").count();
    assert_eq!(
        before_count, 2,
        "before_each() should be called in each test, found {}",
        before_count
    );
    assert_eq!(
        after_count, 2,
        "after_each() should be called in each test, found {}",
        after_count
    );
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
    assert!(
        rust_code.contains("fn before_each()"),
        "outer before_each fn should exist:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("fn before_each_1()"),
        "inner before_each_1 fn should exist:\n{}",
        rust_code
    );
    // The nested test should call BOTH hooks (parent first, then inner)
    assert!(
        rust_code.contains("before_each();"),
        "nested test should call parent before_each:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("before_each_1();"),
        "nested test should call inner before_each_1:\n{}",
        rust_code
    );
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
    assert!(
        !rust_code.contains("before_each"),
        "no before_each when none defined:\n{}",
        rust_code
    );
    assert!(
        !rust_code.contains("after_each"),
        "no after_each when none defined:\n{}",
        rust_code
    );
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
    assert!(
        rust_code.contains("fn before_all()"),
        "should generate before_all fn:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("fn after_all()"),
        "should generate after_all fn:\n{}",
        rust_code
    );
    // They should NOT be auto-invoked in test functions (they're module-level)
    assert!(
        !rust_code.contains("before_all();"),
        "before_all should not be auto-invoked in tests"
    );
    assert!(
        !rust_code.contains("after_all();"),
        "after_all should not be auto-invoked in tests"
    );
}

// ===== Phase 12.4: Async Test Support =====

#[test]
fn test_async_test_generates_tokio_test() {
    let source = r#"
import { describe, test, expect } from "liva/test"

fetchData(): string => "data"

describe("Async Tests", () => {
    test("fetches data", () => {
        let result = async fetchData()
        expect(result).toBe("data")
    })
})
"#;

    let rust_code = compile_and_generate(source);
    assert!(
        rust_code.contains("#[tokio::test]"),
        "should generate #[tokio::test] for async test:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("async fn test_fetches_data()"),
        "should generate async fn:\n{}",
        rust_code
    );
}

#[test]
fn test_sync_test_stays_normal() {
    let source = r#"
import { describe, test, expect } from "liva/test"

add(a: int, b: int): int => a + b

describe("Sync Tests", () => {
    test("adds numbers", () => {
        expect(add(1, 2)).toBe(3)
    })
})
"#;

    let rust_code = compile_and_generate(source);
    assert!(
        rust_code.contains("#[test]"),
        "sync test should use #[test]:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("fn test_adds_numbers()"),
        "sync test should use plain fn:\n{}",
        rust_code
    );
    assert!(
        !rust_code.contains("#[tokio::test]"),
        "sync test should NOT use tokio::test:\n{}",
        rust_code
    );
    assert!(
        !rust_code.contains("async fn test_"),
        "sync test should NOT be async:\n{}",
        rust_code
    );
}

#[test]
fn test_mixed_sync_and_async_tests() {
    let source = r#"
import { describe, test, expect } from "liva/test"

add(a: int, b: int): int => a + b
fetchData(): string => "data"

describe("Mixed Tests", () => {
    test("sync test", () => {
        expect(add(1, 2)).toBe(3)
    })

    test("async test", () => {
        let result = async fetchData()
        expect(result).toBe("data")
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // Should have both #[test] and #[tokio::test]
    assert!(
        rust_code.contains("#[test]"),
        "should have sync test:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("#[tokio::test]"),
        "should have async test:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("fn test_sync_test()"),
        "sync test should use plain fn:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("async fn test_async_test()"),
        "async test should use async fn:\n{}",
        rust_code
    );
}

#[test]
fn test_async_test_with_lifecycle_hooks() {
    let source = r#"
import { describe, test, expect, beforeEach, afterEach } from "liva/test"

fetchData(): string => "data"

describe("Async with hooks", () => {
    beforeEach(() => {
        print("setup")
    })

    afterEach(() => {
        print("teardown")
    })

    test("async fetch", () => {
        let result = async fetchData()
        expect(result).toBe("data")
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // The test should be async
    assert!(
        rust_code.contains("#[tokio::test]"),
        "async test should use tokio::test:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("async fn test_async_fetch()"),
        "should be async fn:\n{}",
        rust_code
    );
    // Sync hooks should be called without .await in async test
    assert!(
        rust_code.contains("before_each();"),
        "sync hook should be called without await:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("after_each();"),
        "sync hook should be called without await:\n{}",
        rust_code
    );
}

#[test]
fn test_async_lifecycle_hooks() {
    let source = r#"
import { describe, test, expect, beforeEach } from "liva/test"

fetchData(): string => "data"
setupDb(): string => "ready"

describe("Async hooks", () => {
    beforeEach(() => {
        let db = async setupDb()
        print(db)
    })

    test("async test with async hook", () => {
        let result = async fetchData()
        expect(result).toBe("data")
    })
})
"#;

    let rust_code = compile_and_generate(source);
    // The hook itself should be async
    assert!(
        rust_code.contains("async fn before_each()"),
        "async hook should generate async fn:\n{}",
        rust_code
    );
    // The test should be async
    assert!(
        rust_code.contains("#[tokio::test]"),
        "test should use tokio::test:\n{}",
        rust_code
    );
    // Async hook should be called with .await in async test
    assert!(
        rust_code.contains("before_each().await;"),
        "async hook should be awaited in async test:\n{}",
        rust_code
    );
}

// ============================================================
// Session 13: Edge Case Bug Fix Tests (#55-#62)
// ============================================================

#[test]
fn test_bug55_substring_expression_index() {
    // Bug #55: substring(start, maxLen - 3) should wrap in (expr) as usize
    let source = r#"
truncate(text: string, maxLen: number): string {
    if text.length > maxLen {
        return text.substring(0, maxLen - 3) + "..."
    }
    return text
}

main() {
    let result = truncate("Hello World", 8)
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    // Should wrap (max_len - 3) in parens before as usize
    assert!(
        rust_code.contains("(max_len - 3) as usize"),
        "substring expression index should be wrapped in parens:\n{}",
        rust_code
    );
    assert_snapshot!("bug55_substring_expr_index", rust_code);
}

#[test]
fn test_bug56_foreach_string_param() {
    // Bug #56: forEach on [string] function parameters should not use |&s|
    let source = r#"
printLines(lines: [string]) {
    lines.forEach(line => {
        print("> " + line)
    })
}

main() {
    let items = ["one", "two"]
    printLines(items)
}
"#;

    let rust_code = compile_and_generate(source);
    // Should NOT have |&line| because String is non-Copy (will_use_cloned = true)
    assert!(
        !rust_code.contains("|&line|"),
        "forEach on [string] should not use |&line|:\n{}",
        rust_code
    );
    assert_snapshot!("bug56_foreach_string_param", rust_code);
}

#[test]
fn test_bug57_array_literal_strings() {
    // Bug #57: ["hello", "world"] should generate vec!["hello".to_string(), ...]
    let source = r#"
getColors(): [string] {
    return ["red", "green", "blue"]
}

main() {
    let colors = getColors()
    print(colors.length)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(
        rust_code.contains(".to_string()"),
        "string array literals should have .to_string():\n{}",
        rust_code
    );
    assert_snapshot!("bug57_array_literal_strings", rust_code);
}

#[test]
fn test_bug58_char_tostring_concat() {
    // Bug #58: char.toString() + char.toString() should use format!()
    let source = r#"
getInitials(name: string): string {
    let first = name.charAt(0)
    let spaceIdx = name.indexOf(" ")
    if spaceIdx > 0 {
        let second = name.charAt(spaceIdx + 1)
        return first.toString() + second.toString()
    }
    return first.toString()
}

main() {
    let initials = getInitials("John Doe")
    print(initials)
}
"#;

    let rust_code = compile_and_generate(source);
    // Should use format! for toString() + toString() concatenation
    assert!(
        rust_code.contains("format!"),
        "toString() concatenation should use format!:\n{}",
        rust_code
    );
    assert_snapshot!("bug58_char_tostring_concat", rust_code);
}

#[test]
fn test_bug59_60_class_filter_comparison() {
    // Bug #59: this.items.filter() in class methods
    // Bug #60: filter(|&item| item == query) needs dereference
    let source = r#"
Library {
    books: [string]

    constructor() {
        this.books = []
    }

    addBook(title: string) {
        this.books.push(title)
    }

    search(query: string): [string] {
        return this.books.filter(book => book == query)
    }
}

main() {
    let lib = Library()
    lib.addBook("Liva Guide")
    let found = lib.search("Liva Guide")
    print(found.length)
}
"#;

    let rust_code = compile_and_generate(source);
    // Should have |&book| with dereference * in comparison
    assert!(
        rust_code.contains("|&book|"),
        "filter on class string field should use |&book|:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("*book == query") || rust_code.contains("*book =="),
        "filter comparison should dereference &T:\n{}",
        rust_code
    );
    assert_snapshot!("bug59_60_class_filter_comparison", rust_code);
}

#[test]
fn test_bug61_print_array_from_function() {
    // Bug #61: print(reversed) where reversed comes from array-returning function
    let source = r#"
doubleNums(nums: [number]): [number] {
    let result: [number] = []
    nums.forEach(n => {
        result.push(n * 2)
    })
    return result
}

main() {
    let nums = [1, 2, 3]
    let doubled = doubleNums(nums)
    print(doubled)
}
"#;

    let rust_code = compile_and_generate(source);
    // Should use {:?} for array variable from function return
    assert!(
        rust_code.contains("{:?}"),
        "print(array) should use Debug format {{:?}}:\n{}",
        rust_code
    );
    assert_snapshot!("bug61_print_array_from_function", rust_code);
}

#[test]
fn test_bug62_filter_result_indexing() {
    // Bug #62: found[0] on Vec<String> from filter needs .clone()
    let source = r#"
Library {
    books: [string]

    constructor() {
        this.books = []
    }

    addBook(title: string) {
        this.books.push(title)
    }

    findFirst(query: string): string {
        let found = this.books.filter(book => book == query)
        if found.length > 0 {
            return found[0]
        }
        return ""
    }
}

main() {
    let lib = Library()
    lib.addBook("Rust Book")
    let result = lib.findFirst("Rust Book")
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    // found[0] should have .clone() because it's Vec<String>
    assert!(
        rust_code.contains("found[0].clone()") || rust_code.contains("found[0 as usize].clone()"),
        "indexing Vec<String> should add .clone():\n{}",
        rust_code
    );
    assert_snapshot!("bug62_filter_result_indexing", rust_code);
}

// ============================================================================
// COMPREHENSIVE FEATURE COVERAGE TESTS
// These tests document ALL supported Liva syntax and their Rust codegen output.
// They serve as the "source of truth" for what the language supports.
// ============================================================================

// ---------------------------------------------------------------------------
// 1. Variables & Constants
// ---------------------------------------------------------------------------

#[test]
fn test_feature_variables_let_and_const() {
    let source = r#"
main() {
    // Mutable variable (let)
    let x = 10
    let name = "Alice"
    let pi = 3.14
    let active = true

    // Immutable constant (const)
    const MAX = 100
    const GREETING = "Hello"

    // Type annotations
    let count: number = 42
    let label: string = "test"
    let ratio: float = 0.5
    let flag: bool = false

    print(x)
    print(name)
    print(MAX)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_variables_let_and_const", rust_code);
}

#[test]
fn test_feature_top_level_const() {
    let source = r#"
const MAX_SIZE = 1024
const APP_NAME = "MyApp"
const PI = 3.14159

main() {
    print(APP_NAME)
    print(MAX_SIZE)
    print(PI)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_top_level_const", rust_code);
}

// ---------------------------------------------------------------------------
// 2. Types (Primitives, Rust types, Null)
// ---------------------------------------------------------------------------

#[test]
fn test_feature_primitive_types() {
    let source = r#"
main() {
    let n: number = 42
    let f: float = 3.14
    let b: bool = true
    let s: string = "hello"
    let c: char = 'A'

    print(n)
    print(f)
    print(s)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_primitive_types", rust_code);
}

#[test]
fn test_feature_rust_native_types() {
    let source = r#"
main() {
    let small: i8 = 127
    let medium: i16 = 32000
    let big: i64 = 9999999999
    let unsigned: u64 = 42
    let precise: f32 = 3.14
    let idx: usize = 0

    print(small)
    print(big)
    print(idx)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_rust_native_types", rust_code);
}

// ---------------------------------------------------------------------------
// 3. Operators
// ---------------------------------------------------------------------------

#[test]
fn test_feature_arithmetic_operators() {
    let source = r#"
main() {
    let a = 10
    let b = 3

    let sum = a + b
    let diff = a - b
    let prod = a * b
    let quot = a / b
    let rem = a % b

    print(sum)
    print(diff)
    print(prod)
    print(quot)
    print(rem)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_arithmetic_operators", rust_code);
}

#[test]
fn test_feature_comparison_operators() {
    let source = r#"
main() {
    let a = 10
    let b = 20

    if a == b {
        print("equal")
    }
    if a != b {
        print("not equal")
    }
    if a < b {
        print("less")
    }
    if a > b {
        print("greater")
    }
    if a <= b {
        print("less or equal")
    }
    if a >= b {
        print("greater or equal")
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_comparison_operators", rust_code);
}

#[test]
fn test_feature_logical_operators() {
    let source = r#"
main() {
    let x = true
    let y = false

    // Liva keywords: and, or, not
    if x and y {
        print("both")
    }
    if x or y {
        print("either")
    }
    if not x {
        print("negated")
    }

    // Also supports: &&, ||, !
    if x && y {
        print("both2")
    }
    if x || y {
        print("either2")
    }
    if !x {
        print("negated2")
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_logical_operators", rust_code);
}

// ---------------------------------------------------------------------------
// 4. Functions
// ---------------------------------------------------------------------------

#[test]
fn test_feature_function_styles() {
    let source = r#"
// One-liner with =>
double(x) => x * 2

// One-liner with =
square(x) = x * x

// Block function
greet(name: string): string {
    let msg = "Hello, " + name + "!"
    return msg
}

// Default parameters
connect(host: string, port: number = 8080): string {
    return host + ":" + port
}

main() {
    print(double(5))
    print(square(4))
    print(greet("World"))
    print(connect("localhost"))
    print(connect("localhost", 3000))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_function_styles", rust_code);
}

#[test]
fn test_feature_lambdas_closures() {
    let source = r#"
main() {
    let nums = [1, 2, 3, 4, 5]

    // Lambda with block body
    let doubled = nums.map(x => {
        return x * 2
    })

    // Lambda with expression body
    let tripled = nums.map(x => x * 3)

    // Lambda in filter
    let evens = nums.filter(x => x % 2 == 0)

    print(doubled)
    print(tripled)
    print(evens)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_lambdas_closures", rust_code);
}

// ---------------------------------------------------------------------------
// 5. Control Flow
// ---------------------------------------------------------------------------

#[test]
fn test_feature_if_else() {
    let source = r#"
classify(n: number): string {
    if n > 0 {
        return "positive"
    } else if n < 0 {
        return "negative"
    } else {
        return "zero"
    }
}

main() {
    print(classify(5))
    print(classify(-3))
    print(classify(0))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_if_else", rust_code);
}

#[test]
fn test_feature_one_liner_if() {
    let source = r#"
// One-liner with ternary (if is a statement, ternary is an expression)
abs(n: number): number => n < 0 ? -n : n

main() {
    print(abs(-5))
    print(abs(3))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_one_liner_if", rust_code);
}

#[test]
fn test_feature_ternary_operator() {
    let source = r#"
main() {
    let x = 10
    let result = x > 5 ? "big" : "small"
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_ternary_operator", rust_code);
}

// ---------------------------------------------------------------------------
// 6. Pattern Matching
// ---------------------------------------------------------------------------

#[test]
fn test_feature_switch_statement() {
    let source = r#"
classifyNum(n: number): string {
    switch n {
        case 1:
            return "one"
        case 2:
            return "two"
        case 3:
            return "three"
        default:
            return "other"
    }
}

main() {
    print(classifyNum(2))
    print(classifyNum(99))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_switch_statement", rust_code);
}

#[test]
fn test_feature_switch_expression() {
    let source = r#"
main() {
    let day = 3
    let name = switch day {
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        _ => "Unknown"
    }
    print(name)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_switch_expression", rust_code);
}

#[test]
fn test_feature_switch_advanced_patterns() {
    let source = r#"
classify(score: number): string {
    let result = switch score {
        1 | 2 | 3 => "low",
        4 | 5 | 6 => "medium",
        7 | 8 | 9 | 10 => "high",
        _ => "invalid"
    }
    return result
}

main() {
    print(classify(2))
    print(classify(5))
    print(classify(9))
    print(classify(0))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_switch_advanced_patterns", rust_code);
}

// ---------------------------------------------------------------------------
// 7. Loops
// ---------------------------------------------------------------------------

#[test]
fn test_feature_while_loop() {
    let source = r#"
main() {
    let count = 0
    while count < 5 {
        print(count)
        count = count + 1
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_while_loop", rust_code);
}

#[test]
fn test_feature_for_range_loop() {
    let source = r#"
main() {
    // Exclusive range
    for i in 0..5 {
        print(i)
    }

    // Another range
    for j in 1..11 {
        print(j)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_for_range_loop", rust_code);
}

#[test]
fn test_feature_for_array_loop() {
    let source = r#"
main() {
    let fruits = ["apple", "banana", "cherry"]
    for fruit in fruits {
        print(fruit)
    }

    let nums = [10, 20, 30]
    for n in nums {
        print(n)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_for_array_loop", rust_code);
}

#[test]
fn test_feature_one_liner_loops() {
    let source = r#"
main() {
    let items = [1, 2, 3]

    // One-liner for
    for x in items => print(x)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_one_liner_loops", rust_code);
}

// ---------------------------------------------------------------------------
// 8. Classes & Interfaces
// ---------------------------------------------------------------------------

#[test]
fn test_feature_class_basic() {
    let source = r#"
Person {
    name: string
    age: number

    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }

    greet(): string {
        return "Hi, I'm " + this.name
    }

    isAdult(): bool => this.age >= 18
}

main() {
    let p = Person("Alice", 30)
    print(p.greet())
    print(p.isAdult())
    print(p.name)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_class_basic", rust_code);
}

#[test]
fn test_feature_interface_implements() {
    let source = r#"
Printable {
    display(): string
}

Item {
    name: string

    constructor(name: string) {
        this.name = name
    }

    display(): string {
        return this.name
    }
}

main() {
    let item = Item("Widget")
    print(item.display())
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_interface_implements", rust_code);
}

// ---------------------------------------------------------------------------
// 9. Error Handling
// ---------------------------------------------------------------------------

#[test]
fn test_feature_error_handling_fail() {
    let source = r#"
divide(a: number, b: number): number {
    if b == 0 {
        fail "Division by zero"
    }
    return a / b
}

main() {
    let result, err = divide(10, 2)
    if err != "" {
        print("Error: " + err)
    } else {
        print(result)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_error_handling_fail", rust_code);
}

#[test]
fn test_feature_error_handling_or_fail() {
    let source = r#"
loadConfig(path: string): string {
    let content, err = File.read(path)
    let data = content or fail err
    return data
}

main() {
    let config, err = loadConfig("config.json")
    if err != "" {
        print("Failed: " + err)
    } else {
        print(config)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_error_handling_or_fail", rust_code);
}

#[test]
fn test_feature_error_handling_or_value() {
    let source = r#"
divide(a: number, b: number): number {
    if b == 0 => fail "Division by zero"
    return a / b
}

main() {
    let r = divide(10, 0) or 42
    print(r)

    let r2 = divide(10, 2) or 99
    print(r2)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_error_handling_or_value", rust_code);
}

#[test]
fn test_feature_try_catch() {
    let source = r#"
risky(): number {
    fail "something went wrong"
}

main() {
    try {
        let val, err = risky()
        if err == "" {
            print(val)
        }
    } catch (err) {
        print("Caught: " + err)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_try_catch", rust_code);
}

#[test]
fn test_error_binding_underscore_discard() {
    let source = r#"
divide(a: number, b: number): number {
    if b == 0 {
        fail "Division by zero"
    }
    return a / b
}

main() {
    let result, _ = divide(10, 2)
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("error_binding_underscore_discard", rust_code);
}

#[test]
fn test_error_binding_method_call() {
    // B19: Error binding for method calls should generate match { Ok/Err } pattern
    // Previously generated (self.method(), None) without destructuring Result
    let source = r#"
Calculator {
    value: number

    constructor(initial: number) {
        this.value = initial
    }

    divide(divisor: number): number {
        if divisor == 0 {
            fail "Division by zero"
        }
        return this.value / divisor
    }
}

main() {
    let calc = Calculator(100)
    let result, err = calc.divide(5)
    if err {
        print("Error: " + err)
    } else {
        print(result)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    // Should generate match pattern, not (self.method(), None)
    assert!(rust_code.contains("match calc.divide("), "Method call should use match pattern for error binding");
    assert!(rust_code.contains("Ok(v)"), "Should destructure Ok variant");
    assert!(rust_code.contains("Err(e)"), "Should destructure Err variant");
    assert_snapshot!("error_binding_method_call", rust_code);
}

#[test]
fn test_or_fail_method_call() {
    // B22: `or fail` codegen should generate match { Ok/Err } for method calls
    // Previously the method call was not recognized as fallible, so `or fail` was ignored
    let source = r#"
Validator {
    validate(input: string): string {
        if input == "" {
            fail "Input cannot be empty"
        }
        return input
    }
}

process(input: string): string {
    let v = Validator()
    let result = v.validate(input) or fail "Validation failed"
    return result
}

main() {
    let data, err = process("hello")
    if err {
        print("Error: " + err)
    } else {
        print(data)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    // or fail should generate match with Error::chain
    assert!(rust_code.contains("Error::chain"), "or fail should generate Error::chain for method calls");
    assert!(rust_code.contains("Validation failed"), "Error message should be in the generated code");
    assert_snapshot!("or_fail_method_call", rust_code);
}

#[test]
fn test_fail_string_uses_error_new() {
    // B20: `fail "msg"` should generate Error::new when error var is out of scope
    // Previously, error_binding_vars was a flat set with no scope tracking,
    // so `fail "msg"` after an error binding in a nested block would reference
    // an out-of-scope variable
    let source = r#"
Parser {
    parseItem(input: string): number {
        if input == "special" {
            let val, err = this.tryParse(input)
            if err {
                fail err
            }
            return val
        }
        // err was declared inside the if block above — NOT in scope here
        // fail "string" should use Error::new, not Error::chain
        if input == "" {
            fail "Empty input"
        }
        fail $"Unknown input: '{input}'"
    }

    tryParse(input: string): number {
        fail "Not implemented"
    }
}

process(): number {
    let p = Parser()
    let result, err = p.parseItem("test")
    if err {
        // err IS in scope here — fail "string" should chain
        fail "Processing failed"
    }
    return result
}
"#;

    let rust_code = compile_and_generate(source);
    // fail "Empty input" outside the error binding scope → Error::new
    assert!(rust_code.contains("Error::new(\"Empty input\""),
        "fail with string literal outside error scope should use Error::new");
    // fail $"..." outside scope → Error::new
    assert!(rust_code.contains("Error::new(format!"),
        "fail with interpolated string outside error scope should use Error::new");
    // fail err inside scope → Error::chain
    assert!(rust_code.contains("Error::chain(err"),
        "fail with error variable should use Error::chain");
    // fail "Processing failed" inside error scope → Error::chain
    assert!(rust_code.contains("Error::chain(\"Processing failed\""),
        "fail with string inside error scope should use Error::chain");
    assert_snapshot!("fail_string_uses_error_new", rust_code);
}

#[test]
fn test_mut_self_deep_assignment() {
    // B08: Methods that assign to this.items[idx].field should get &mut self
    let source = r#"
Task {
    name: string
    done: bool
}

TodoList {
    tasks: [Task]

    constructor() {
        this.tasks = []
    }

    addTask(name: string) {
        this.tasks.push(Task(name, false))
    }

    completeTask(idx: number) {
        this.tasks[idx].done = true
    }
}
"#;

    let rust_code = compile_and_generate(source);
    // completeTask does this.tasks[idx].done = true → should be &mut self
    assert!(rust_code.contains("fn complete_task(&mut self"), 
        "completeTask should have &mut self because it assigns to this.tasks[idx].field");
    assert_snapshot!("mut_self_deep_assignment", rust_code);
}

// ---------------------------------------------------------------------------
// 10. Concurrency
// ---------------------------------------------------------------------------

#[test]
fn test_auto_clone_map_array_args() {
    // B17: Map/Array passed to function by value should auto-clone
    // Previously only string_vars and class_instance_vars were cloned
    let source = r#"
processArray(items: [string]): number {
    return items.length
}

main() {
    let items = ["a", "b", "c"]
    let n1 = processArray(items)
    let n2 = processArray(items)
    print(n1)
    print(n2)
}
"#;

    let rust_code = compile_and_generate(source);
    // Array args should be cloned to prevent move
    assert!(rust_code.contains("items.clone()"), "Array variable should be cloned when passed to function");
    assert_snapshot!("auto_clone_map_array_args", rust_code);
}

#[test]
fn test_feature_async_await() {
    let source = r#"
fetchData(url: string): string {
    let response, err = async HTTP.get(url)
    if err != "" {
        return "error"
    }
    return response.body
}

main() {
    let data = fetchData("https://api.example.com/data")
    print(data)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_async_await", rust_code);
}

#[test]
fn test_feature_par_concurrent() {
    let source = r#"
heavyWork(n: number): number {
    return n * n
}

main() {
    let result = par heavyWork(42)
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_par_concurrent", rust_code);
}

#[test]
fn test_feature_task_and_fire() {
    let source = r#"
fetchUser(id: number): string {
    return "user_" + id
}

logEvent(msg: string) {
    print(msg)
}

main() {
    // task: get handle, auto-awaited at scope end
    let result = task async fetchUser(1)

    // fire-and-forget: async call as statement (not assigned)
    async logEvent("app started")

    // Use the task result
    print(result)
    print("done")
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_task_and_fire", rust_code);
}

// ---------------------------------------------------------------------------
// 11. Collections / Arrays
// ---------------------------------------------------------------------------

#[test]
fn test_feature_array_methods() {
    let source = r#"
main() {
    let nums = [1, 2, 3, 4, 5]

    // Functional methods
    let doubled = nums.map(x => x * 2)
    let evens = nums.filter(x => x % 2 == 0)
    let sum = nums.reduce(0, (acc, x) => acc + x)
    let found = nums.find(x => x > 3)
    let hasEvens = nums.some(x => x % 2 == 0)
    let allPositive = nums.every(x => x > 0)

    // Iteration
    nums.forEach(n => print(n))

    // Search
    let hasThree = nums.includes(3)
    let pos = nums.indexOf(3)

    // Length
    let len = nums.length

    print(doubled)
    print(evens)
    print(sum)
    print(len)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_array_methods", rust_code);
}

#[test]
fn test_feature_array_push_pop_mutation() {
    let source = r#"
main() {
    let items: [number] = []
    items.push(1)
    items.push(2)
    items.push(3)
    
    let last = items.pop()
    
    print(items.length)
    print(items)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_array_push_pop_mutation", rust_code);
}

#[test]
fn test_feature_array_chaining() {
    let source = r#"
main() {
    let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    // Method chaining
    let result = nums
        .filter(x => x % 2 == 0)
        .map(x => x * 10)

    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_array_chaining", rust_code);
}

// ---------------------------------------------------------------------------
// 12. String Features
// ---------------------------------------------------------------------------

#[test]
fn test_feature_string_templates() {
    let source = r#"
main() {
    let name = "Alice"
    let age = 30

    let msg = $"Hello, {name}! You are {age} years old."
    print(msg)

    let calc = $"Result: {2 + 3}"
    print(calc)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_string_templates", rust_code);
}

#[test]
fn test_feature_string_methods() {
    let source = r#"
main() {
    let text = "Hello, World!"

    let upper = text.toUpperCase()
    let lower = text.toLowerCase()
    let trimmed = "  spaced  ".trim()

    let parts = "a,b,c".split(",")
    let replaced = text.replace("World", "Liva")

    let starts = text.startsWith("Hello")
    let ends = text.endsWith("!")

    let sub = text.substring(0, 5)
    let ch = text.charAt(0)
    let idx = text.indexOf("World")
    let len = text.length

    print(upper)
    print(lower)
    print(trimmed)
    print(replaced)
    print(starts)
    print(sub)
    print(len)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_string_methods", rust_code);
}

// ---------------------------------------------------------------------------
// 13. Console & IO
// ---------------------------------------------------------------------------

#[test]
fn test_feature_console_methods() {
    let source = r#"
main() {
    print("basic print")
    console.log("debug info")
    console.error("error message")
    console.warn("warning message")
    console.success("success!")
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_console_methods", rust_code);
}

// ---------------------------------------------------------------------------
// 14. Math & Conversions
// ---------------------------------------------------------------------------

#[test]
fn test_feature_math_functions() {
    let source = r#"
main() {
    let sq = Math.sqrt(16.0)
    let pw = Math.pow(2.0, 3.0)
    let ab = Math.abs(-10.5)
    let fl = Math.floor(3.7)
    let cl = Math.ceil(3.2)
    let rn = Math.round(3.5)
    let mn = Math.min(10.5, 20.3)
    let mx = Math.max(10.5, 20.3)
    let rd = Math.random()

    print(sq)
    print(pw)
    print(ab)
    print(fl)
    print(cl)
    print(rn)
    print(mn)
    print(mx)
    print(rd)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_math_functions", rust_code);
}

#[test]
fn test_feature_type_conversions() {
    let source = r#"
main() {
    let num, err = parseInt("42")
    let val, err2 = parseFloat("3.14")
    let str = toString(42)

    print(num)
    print(val)
    print(str)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_type_conversions", rust_code);
}

// ---------------------------------------------------------------------------
// 15. JSON & HTTP
// ---------------------------------------------------------------------------

#[test]
fn test_feature_json_operations() {
    let source = r#"
main() {
    let data: [int], err = JSON.parse("[1, 2, 3]")
    if err == "" {
        print(data)
    }

    let json = JSON.stringify(data)
    print(json)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_json_operations", rust_code);
}

#[test]
fn test_feature_http_methods() {
    let source = r#"
main() {
    let resp, err = async HTTP.get("https://api.example.com/users")
    if err == "" {
        print(resp.body)
    }

    let body = "{\"name\": \"Alice\"}"
    let resp2, err2 = async HTTP.post("https://api.example.com/users", body)
    print(resp2.status)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_http_methods", rust_code);
}

// ---------------------------------------------------------------------------
// 16. Visibility
// ---------------------------------------------------------------------------

#[test]
fn test_feature_visibility_private() {
    let source = r#"
Counter {
    _count: number

    constructor() {
        this._count = 0
    }

    increment() {
        this._count = this._count + 1
    }

    getCount(): number {
        return this._count
    }
}

main() {
    let c = Counter()
    c.increment()
    c.increment()
    print(c.getCount())
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_visibility_private", rust_code);
}

// ---------------------------------------------------------------------------
// 17. Test Framework Matchers
// ---------------------------------------------------------------------------

#[test]
fn test_feature_test_matchers() {
    let source = r#"
import { describe, test, expect } from "liva/test"

add(a: number, b: number): number => a + b

describe("Matchers showcase", () => {
    test("equality matchers", () => {
        expect(add(2, 3)).toBe(5)
        expect(add(0, 0)).toEqual(0)
    })

    test("truthiness", () => {
        expect(true).toBeTruthy()
        expect(false).toBeFalsy()
    })

    test("comparison", () => {
        expect(10).toBeGreaterThan(5)
        expect(3).toBeLessThan(7)
        expect(5).toBeGreaterThanOrEqual(5)
        expect(3).toBeLessThanOrEqual(3)
    })

    test("negation", () => {
        expect(1).not.toBe(2)
        expect(false).not.toBeTruthy()
    })
})
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_test_matchers", rust_code);
}

// ---------------------------------------------------------------------------
// 18. Generics
// ---------------------------------------------------------------------------

#[test]
fn test_feature_generic_function() {
    let source = r#"
identity<T>(value: T): T {
    return value
}

main() {
    let x = identity(42)
    let y = identity("hello")
    print(x)
    print(y)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_generic_function", rust_code);
}

// ---------------------------------------------------------------------------
// 19. Tuples
// ---------------------------------------------------------------------------

#[test]
fn test_feature_tuples() {
    let source = r#"
getPoint(): (number, number) {
    return (10, 20)
}

main() {
    let point = getPoint()
    let x = point.0
    let y = point.1
    print(x)
    print(y)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_tuples", rust_code);
}

// ---------------------------------------------------------------------------
// 20. Type Aliases
// ---------------------------------------------------------------------------

#[test]
fn test_feature_type_alias() {
    let source = r#"
type UserId = number
type Username = string

getUser(id: UserId): Username {
    return "user_" + id
}

main() {
    let name = getUser(42)
    print(name)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_type_alias", rust_code);
}

// ---------------------------------------------------------------------------
// 21. String Concatenation Patterns
// ---------------------------------------------------------------------------

#[test]
fn test_feature_string_concat() {
    let source = r#"
main() {
    let first = "Hello"
    let second = "World"

    // String + String
    let msg = first + " " + second

    // String + Number (auto-converts)
    let info = "Count: " + 42

    print(msg)
    print(info)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_string_concat", rust_code);
}

// ---------------------------------------------------------------------------
// 22. For Parallel Loops
// ---------------------------------------------------------------------------

#[test]
fn test_feature_for_parallel() {
    let source = r#"
main() {
    let items = [1, 2, 3, 4, 5]

    for par x in items {
        print(x)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_for_parallel", rust_code);
}

// ===========================================================================
// New Language Features (v1.3.0)
// ===========================================================================

// ---------------------------------------------------------------------------
// 23. Math.PI / Math.E Constants
// ---------------------------------------------------------------------------

#[test]
fn test_feature_math_constants() {
    let source = r#"
main() {
    let pi = Math.PI
    let e = Math.E
    let circumference = 2.0 * Math.PI * 5.0
    print(pi)
    print(e)
    print(circumference)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_math_constants", rust_code);
}

// ---------------------------------------------------------------------------
// 24. [string].join(separator)
// ---------------------------------------------------------------------------

#[test]
fn test_feature_array_join() {
    let source = r#"
main() {
    let words: [string] = ["hello", "world", "foo"]
    let result = words.join(", ")
    let dashed = words.join("-")
    print(result)
    print(dashed)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_array_join", rust_code);
}

// ---------------------------------------------------------------------------
// 25. Inclusive Range ..= in For Loops
// ---------------------------------------------------------------------------

#[test]
fn test_feature_inclusive_range() {
    let source = r#"
main() {
    for i in 1..=5 {
        print(i)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_inclusive_range", rust_code);
}

// ---------------------------------------------------------------------------
// 26. Break Statement
// ---------------------------------------------------------------------------

#[test]
fn test_feature_break() {
    let source = r#"
main() {
    let i = 0
    while i < 100 {
        if i == 5 {
            break
        }
        print(i)
        i = i + 1
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_break", rust_code);
}

// ---------------------------------------------------------------------------
// 27. Continue Statement
// ---------------------------------------------------------------------------

#[test]
fn test_feature_continue() {
    let source = r#"
main() {
    for i in 0..10 {
        if i == 3 {
            continue
        }
        print(i)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_continue", rust_code);
}

// ---------------------------------------------------------------------------
// 28. Break and Continue Combined
// ---------------------------------------------------------------------------

#[test]
fn test_feature_break_continue_combined() {
    let source = r#"
main() {
    for i in 0..20 {
        if i == 15 {
            break
        }
        if i % 2 == 0 {
            continue
        }
        print(i)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_break_continue_combined", rust_code);
}

// ---------------------------------------------------------------------------
// 29. Data Class (auto-detected: fields + no constructor)
// ---------------------------------------------------------------------------

#[test]
fn test_feature_data_class() {
    let source = r#"
Point {
    x: number
    y: number
}

main() {
    let p = Point(1, 2)
    print(p)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_data_class", rust_code);
}

// ---------------------------------------------------------------------------
// 30. Data Class with Methods
// ---------------------------------------------------------------------------

#[test]
fn test_feature_data_class_with_methods() {
    let source = r#"
Color {
    r: number
    g: number
    b: number

    sum() => r + g + b
}

main() {
    let c = Color(255, 128, 0)
    print(c)
    print(c.sum())
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_data_class_with_methods", rust_code);
}

// ============================================================
// Session 15: Dogfooding regression tests
// ============================================================

#[test]
fn test_bug63_return_without_value() {
    // Bug #63: return without value followed by } caused parse error
    let source = r#"
doSomething(x: number) {
    if x < 0 {
        return
    }
    print(x)
}

main() {
    doSomething(5)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("bug63_return_without_value", rust_code);
}

#[test]
fn test_bug64_const_continue_struct_literal() {
    // Bug #64: const LIMIT used in if-condition followed by { continue }
    // was misinterpreted as struct literal LIMIT { continue }
    let source = r#"
const LIMIT = 60

main() {
    let items = [95, 88, 72, 45]
    for score in items {
        if score >= LIMIT {
            continue
        }
        print(score)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("bug64_const_continue", rust_code);
}

#[test]
fn test_bug66_data_class_display_and_constructor() {
    // Bug #66: data class Display impl had unescaped braces in format string
    // Bug #67: data class constructor didn't accept field arguments
    // Note: `data` keyword removed — auto-detected from structure
    let source = r#"
Point {
    x: number
    y: number
}

main() {
    let p = Point(10, 20)
    print(p)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("bug66_data_class_display_constructor", rust_code);
}

#[test]
fn test_bug68_switch_string_literals() {
    // Bug #68: switch expression arms with string literals returned &str
    // instead of String, causing type mismatch
    let source = r#"
classify(score: number): string => switch score {
    90..=100 => "A",
    80..=89 => "B",
    _ => "F"
}

main() {
    print(classify(95))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("bug68_switch_string_literals", rust_code);
}

#[test]
fn test_bug70_method_fail_result() {
    // Bug #70: Method using fail should generate Result return type
    let source = r#"
Finder {
    _items: [string]

    constructor() {
        this._items = ["a", "b", "c"]
    }

    findItem(name: string): string {
        for item in this._items {
            if item == name {
                return item
            }
        }
        fail "not found"
    }
}

main() {
    let f = Finder()
    let result, err = f.findItem("b")
    if err {
        print("error")
    } else {
        print("found it")
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("bug70_method_fail_result", rust_code);
}

#[test]
fn test_arrow_method_return_type_inferred() {
    // B18: Arrow method `=> expr` should infer return type when no explicit annotation
    let source = r#"
TokenStream {
    tokens: [string]
    pos: number

    constructor() {
        this.tokens = ["a", "b", "c"]
        this.pos = 0
    }

    getValue() => this.pos
    hasMore() => this.pos < this.tokens.length
    doubled() => this.pos * 2
    label() => $"pos={this.pos}"
}

main() {
    let ts = TokenStream()
    print(ts.getValue())
    print(ts.hasMore())
    print(ts.doubled())
    print(ts.label())
}
"#;

    let rust_code = compile_and_generate(source);
    // Arrow methods without explicit type should NOT generate -> ()
    assert!(!rust_code.contains("fn get_value(&self) -> ()"), "getValue should not return ()");
    assert!(!rust_code.contains("fn has_more(&self) -> ()"), "hasMore should not return ()");
    assert!(!rust_code.contains("fn doubled(&self) -> ()"), "doubled should not return ()");
    // Should infer correct types
    assert!(rust_code.contains("fn get_value(&self) -> i32"), "getValue should return i32");
    assert!(rust_code.contains("fn has_more(&self) -> bool"), "hasMore should return bool");
    assert!(rust_code.contains("fn doubled(&self) -> i32"), "doubled should return i32");
    assert!(rust_code.contains("fn label(&self) -> String"), "label should return String");
    assert_snapshot!("arrow_method_return_type_inferred", rust_code);
}

#[test]
fn test_bug74_for_loop_ownership() {
    // Bug #74: for loop consumed collection, preventing reuse
    let source = r#"
main() {
    let items = [1, 2, 3, 4, 5]
    for x in items {
        print(x)
    }
    // items should still be usable
    for x in items {
        print(x)
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("bug74_for_loop_ownership", rust_code);
}

// ====================================================================
// Enum tests
// ====================================================================

#[test]
fn test_enum_simple() {
    let source = r#"
enum Color {
    Red,
    Green,
    Blue
}

main() {
    let c = Color.Red
    print(c)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("enum_simple", rust_code);
}

#[test]
fn test_enum_with_data() {
    let source = r#"
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}

main() {
    let s = Shape.Circle(5.0)
    let r = Shape.Rectangle(10.0, 20.0)
    let p = Shape.Point
    print(s)
    print(r)
    print(p)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("enum_with_data", rust_code);
}

#[test]
fn test_enum_switch_simple() {
    let source = r#"
enum Direction {
    North,
    South,
    East,
    West
}

directionName(direction: Direction): string {
    return switch direction {
        Direction.North => "Going north"
        Direction.South => "Going south"
        Direction.East => "Going east"
        Direction.West => "Going west"
    }
}

main() {
    let d = Direction.North
    print(directionName(d))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("enum_switch_simple", rust_code);
}

#[test]
fn test_enum_switch_with_data() {
    let source = r#"
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}

area(shape: Shape): number {
    return switch shape {
        Shape.Circle(r) => 3.14159 * r * r
        Shape.Rectangle(w, h) => w * h
        Shape.Point => 0.0
    }
}

main() {
    let circle = Shape.Circle(5.0)
    let rect = Shape.Rectangle(10.0, 20.0)
    print(area(circle))
    print(area(rect))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("enum_switch_with_data", rust_code);
}

#[test]
fn test_enum_as_param_and_return() {
    let source = r#"
enum SearchResult {
    Found(value: number),
    NotFound
}

findItem(id: number): SearchResult {
    if id > 0 {
        return SearchResult.Found(id * 10)
    }
    return SearchResult.NotFound
}

main() {
    let result = findItem(5)
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("enum_as_param_return", rust_code);
}

// ---------------------------------------------------------------------------
// Array Parallel Execution Policies (Session 18)
// ---------------------------------------------------------------------------

#[test]
fn test_feature_par_find() {
    let source = r#"
main() {
    let numbers = [1, 2, 3, 4, 5]
    let found = numbers.par().find(x => x > 3)
    print(found)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_par_find", rust_code);
}

#[test]
fn test_feature_par_indexof() {
    let source = r#"
main() {
    let numbers = [10, 20, 30, 40]
    let idx = numbers.par().indexOf(30)
    print(idx)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_par_indexof", rust_code);
}

#[test]
fn test_feature_par_reduce_multiply() {
    let source = r#"
main() {
    let numbers = [1, 2, 3, 4, 5]
    let product = numbers.par().reduce(1, (acc, x) => acc * x)
    print(product)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_par_reduce_multiply", rust_code);
}

#[test]
fn test_feature_vec_map() {
    let source = r#"
main() {
    let numbers = [1, 2, 3, 4]
    let doubled = numbers.vec().map(x => x * 2)
    print(doubled)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_vec_map", rust_code);
}

#[test]
fn test_feature_parvec_filter() {
    let source = r#"
main() {
    let numbers = [1, 2, 3, 4, 5, 6]
    let evens = numbers.parvec().filter(x => x % 2 == 0)
    print(evens)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("feature_parvec_filter", rust_code);
}

#[test]
fn test_dir_list_and_isdir() {
    let source = r#"
main() {
    // Dir.isDir - returns bool (no error binding)
    let isDir = Dir.isDir("/tmp")
    print($"Is dir: {isDir}")

    // Dir.list - returns [string] with error binding
    let entries, err = Dir.list("/tmp")
    if err != "" {
        print($"Error: {err}")
        return
    }
    print($"Found {entries.length} entries")
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("dir_list_and_isdir", rust_code);
}

#[test]
fn test_string_contains() {
    let source = r#"
main() {
    let text = "Hello World"
    let hasWorld = text.contains("World")
    let hasFoo = text.contains("Foo")
    print($"Has World: {hasWorld}")
    print($"Has Foo: {hasFoo}")
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("string_contains", rust_code);
}

// ---------------------------------------------------------------------------
// Private function call + one-liner if => continue (guard clause)
// ---------------------------------------------------------------------------

#[test]
fn test_private_fn_call_in_expression() {
    let source = r#"
_isIgnored(entry: string): bool => entry == ".git"

main() {
    let entries = ["src", ".git", "README.md"]
    let results: [string] = []

    for entry in entries {
        if _isIgnored(entry) => continue
        results.push(entry)
    }

    print(results)
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("private_fn_call_expression", rust_code);
}

// ---------------------------------------------------------------------------
// Switch case with block braces
// ---------------------------------------------------------------------------

#[test]
fn test_switch_case_with_block_braces() {
    let source = r#"
main() {
    let mode = 1
    let items = ["a.txt", "b.txt"]
    switch mode {
        case 1: {
            for file in items {
                print(file)
            }
        }
        case 2: {
            print("summary")
        }
        default: {
            print("other")
        }
    }
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("switch_case_block_braces", rust_code);
}

#[test]
fn test_filter_contains_lambda() {
    let source = r#"
isBinary(path: string): bool {
    let exts = [".png", ".jpg", ".gif"]
    let found = exts.filter(ext => path.contains(ext))
    return found.length > 0
}

main() {
    print(isBinary("image.png"))
}
"#;

    let rust_code = compile_and_generate(source);
    assert_snapshot!("filter_contains_lambda", rust_code);
}

#[test]
fn test_filter_string_array_uses_cloned() {
    // B15: .filter() on string arrays should generate .cloned() not .copied()
    // String does NOT implement Copy, only Clone
    let source = r#"
getUniqueWords(words: [string]): [string] {
    let unique = words.filter(w => w.length > 3)
    return unique
}

main() {
    let words = ["hello", "hi", "world", "ok", "testing"]
    let result = getUniqueWords(words)
    print(result)
}
"#;

    let rust_code = compile_and_generate(source);
    // Must use .cloned() not .copied() for String arrays
    assert!(rust_code.contains(".cloned()"), "String filter should use .cloned() not .copied()");
    assert!(!rust_code.contains(".copied()"), "Should NOT use .copied() for String arrays");
    assert_snapshot!("filter_string_array_uses_cloned", rust_code);
}

// ============================================================
// Auto-detected data classes (no `data` keyword needed)
// ============================================================

#[test]
fn test_auto_data_class_fields_only() {
    // A class with only fields and no constructor is auto-detected as data class
    // Gets: positional constructor, PartialEq, Display
    let source = r#"
Coordinate {
    lat: float
    lon: float
}

main() {
    let c = Coordinate(40.7128, -74.0060)
    print(c)
    print(c == Coordinate(40.7128, -74.0060))
}
"#;

    let rust_code = compile_and_generate(source);
    // Should have PartialEq derive and Display impl
    assert!(rust_code.contains("PartialEq"), "auto data class should derive PartialEq");
    assert!(rust_code.contains("impl std::fmt::Display for Coordinate"), "auto data class should have Display impl");
    assert!(rust_code.contains("pub fn new(lat: f64, lon: f64)"), "auto data class should have positional constructor");
    assert_snapshot!("auto_data_class_fields_only", rust_code);
}

#[test]
fn test_auto_data_class_with_methods() {
    // A class with fields + methods but no constructor is still a data class
    let source = r#"
Vec2 {
    x: float
    y: float

    length(): float => (x * x + y * y)
}

main() {
    let v = Vec2(3.0, 4.0)
    print(v)
    print(v.length())
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("PartialEq"), "fields+methods should still be data class");
    assert!(rust_code.contains("impl std::fmt::Display for Vec2"), "should have Display");
    assert!(rust_code.contains("pub fn new(x: f64, y: f64)"), "should have positional constructor");
    assert_snapshot!("auto_data_class_with_methods", rust_code);
}

#[test]
fn test_class_with_constructor_is_not_data() {
    // A class with an explicit constructor should NOT be auto-detected as data class
    let source = r#"
User {
    name: string
    age: number

    constructor(name: string, age: number) {
        if age < 0 fail "Age must be positive"
        this.name = name
        this.age = age
    }
}

main() {
    let u = User("Alice", 25)
    print(u.name)
}
"#;

    let rust_code = compile_and_generate(source);
    // Should NOT have PartialEq on User struct or auto Display (has explicit constructor)
    assert!(!rust_code.contains("impl std::fmt::Display for User"), "should not have auto Display");
    // The derive for User should be just Debug, Clone, Default — no PartialEq
    assert!(rust_code.contains("#[derive(Debug, Clone, Default)]\npub struct User"), 
        "class with constructor should get basic derives, not PartialEq");
    assert_snapshot!("class_with_constructor_not_data", rust_code);
}

#[test]
fn test_error_trace_chaining() {
    // Test that fail generates Error::new with function name and location
    // and that or_fail generates Error::chain preserving the cause
    let source = r#"
parsePort(s: string): number {
    fail "invalid port: " + s
}

loadConfig(path: string): string {
    let port = parsePort("abc") or fail "cannot load config"
    return port.toString()
}

startServer(): string {
    let config, err = loadConfig("/etc/app.conf")
    if err => fail "server failed to start"
    return config
}

main() {
    let server, err = startServer()
    if err {
        print(err)
    }
}
"#;

    let rust_code = compile_and_generate(source);

    // Verify Error::new is used for root fail (no err in scope)
    assert!(rust_code.contains("Error::new("), "should use Error::new for root fail");

    // Verify Error::chain is used for or-fail (chaining from inner error)
    assert!(rust_code.contains("Error::chain("), "should use Error::chain for or fail");

    // Verify function names are embedded
    assert!(rust_code.contains("\"parsePort\""), "should embed function name parsePort");
    assert!(rust_code.contains("\"loadConfig\""), "should embed function name loadConfig");
    assert!(rust_code.contains("\"startServer\""), "should embed function name startServer");

    // Verify the Error struct has cause field
    assert!(rust_code.contains("cause: Option<Box<Error>>"), "Error should have cause field");

    // Verify Display shows Error Trace
    assert!(rust_code.contains("Error Trace"), "Display should show Error Trace header");

    assert_snapshot!("error_trace_chaining", rust_code);
}
// ========================================================================
// Map<K, V> tests (Phase 13)
// ========================================================================

#[test]
fn test_map_literal_empty() {
    let source = r#"
main() {
  let empty: Map<string, int> = Map {}
  print(empty.length)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("HashMap"), "should use HashMap");
    assert_snapshot!("map_literal_empty", rust_code);
}

#[test]
fn test_map_literal_with_entries() {
    let source = r#"
main() {
  let ages = Map {
    "Alice": 30,
    "Bob": 25,
    "Carlos": 35
  }
  print(ages.length)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("HashMap::from"), "should use HashMap::from");
    assert_snapshot!("map_literal_entries", rust_code);
}

#[test]
fn test_map_get_set_has_delete() {
    let source = r#"
main() {
  let scores = Map {
    "math": 95,
    "english": 88
  }
  
  let math = scores.get("math") or 0
  scores.set("science", 92)
  let hasMath = scores.has("math")
  scores.delete("english")
  print(math)
  print(hasMath)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".get("), "should generate .get()");
    assert!(rust_code.contains(".insert("), "should generate .insert()");
    assert!(rust_code.contains(".contains_key("), "should generate .contains_key()");
    assert!(rust_code.contains(".remove("), "should generate .remove()");
    assert_snapshot!("map_get_set_has_delete", rust_code);
}

#[test]
fn test_map_keys_values_entries() {
    let source = r#"
main() {
  let config = Map {
    "host": "localhost",
    "port": "8080"
  }
  
  let allKeys = config.keys()
  let allValues = config.values()
  print(allKeys.length)
  print(allValues.length)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".keys()"), "should generate .keys()");
    assert!(rust_code.contains(".values()"), "should generate .values()");
    assert_snapshot!("map_keys_values_entries", rust_code);
}

#[test]
fn test_map_foreach() {
    let source = r#"
main() {
  let prices = Map {
    "apple": 1,
    "banana": 2
  }
  
  prices.forEach((key, value) => {
    print(key)
    print(value)
  })
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".iter().for_each("), "should generate .iter().for_each()");
    assert_snapshot!("map_foreach", rust_code);
}

#[test]
fn test_map_for_loop_iteration() {
    let source = r#"
main() {
  let colors = Map {
    "red": "FF0000",
    "green": "00FF00"
  }
  
  for key, value in colors {
    print(key)
    print(value)
  }
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("for ("), "should generate tuple destructuring in for loop");
    assert!(rust_code.contains(".iter()"), "should iterate with .iter()");
    assert_snapshot!("map_for_loop", rust_code);
}

#[test]
fn test_map_clear() {
    let source = r#"
main() {
  let data = Map {
    "key1": "value1"
  }
  data.clear()
  print(data.length)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".clear()"), "should generate .clear()");
    assert_snapshot!("map_clear", rust_code);
}

#[test]
fn test_map_type_annotation() {
    let source = r#"
getDefaults(): Map<string, int> {
  return Map {
    "timeout": 30,
    "retries": 3
  }
}

main() {
  let defaults = getDefaults()
  print(defaults.length)
}
"#;

    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("HashMap<"), "should generate HashMap type");
    assert_snapshot!("map_type_annotation", rust_code);
}

// ============================================================
// Phase 14: Set<T> Collections
// ============================================================

#[test]
fn test_set_literal_empty() {
    let source = r#"
main() {
  let empty: Set<string> = Set {}
  print(empty.length)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("HashSet"), "should use HashSet");
    assert_snapshot!("set_literal_empty", rust_code);
}

#[test]
fn test_set_literal_with_values() {
    let source = r#"
main() {
  let numbers = Set { 1, 2, 3, 4, 5 }
  print(numbers.length)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("HashSet::from"), "should use HashSet::from");
    assert_snapshot!("set_literal_values", rust_code);
}

#[test]
fn test_set_add_has_delete() {
    let source = r#"
main() {
  let fruits = Set { "apple", "banana" }
  
  fruits.add("cherry")
  let hasApple = fruits.has("apple")
  fruits.delete("banana")
  print(hasApple)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".insert("), "should generate .insert() for add");
    assert!(rust_code.contains(".contains("), "should generate .contains() for has");
    assert!(rust_code.contains(".remove("), "should generate .remove() for delete");
    assert_snapshot!("set_add_has_delete", rust_code);
}

#[test]
fn test_set_values_method() {
    let source = r#"
main() {
  let tags = Set { "rust", "liva", "wasm" }
  let allValues = tags.values()
  print(allValues.length)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".iter()"), "should generate .iter() for values");
    assert_snapshot!("set_values", rust_code);
}

#[test]
fn test_set_foreach() {
    let source = r#"
main() {
  let langs = Set { "rust", "liva", "python" }
  
  langs.forEach((item) => {
    print(item)
  })
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".iter().for_each("), "should generate .iter().for_each()");
    assert_snapshot!("set_foreach", rust_code);
}

#[test]
fn test_set_for_loop_iteration() {
    let source = r#"
main() {
  let colors = Set { "red", "green", "blue" }
  
  for color in colors {
    print(color)
  }
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("for "), "should generate for loop");
    assert_snapshot!("set_for_loop", rust_code);
}

#[test]
fn test_set_clear() {
    let source = r#"
main() {
  let data = Set { "a", "b", "c" }
  data.clear()
  print(data.length)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".clear()"), "should generate .clear()");
    assert_snapshot!("set_clear", rust_code);
}

#[test]
fn test_set_type_annotation() {
    let source = r#"
getDefaults(): Set<string> {
  return Set { "verbose", "debug" }
}

main() {
  let defaults = getDefaults()
  print(defaults.length)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("HashSet<"), "should generate HashSet type");
    assert_snapshot!("set_type_annotation", rust_code);
}

#[test]
fn test_set_union_intersection_difference() {
    let source = r#"
main() {
  let a = Set { 1, 2, 3 }
  let b = Set { 2, 3, 4 }
  
  let united = a.union(b)
  let common = a.intersection(b)
  let diff = a.difference(b)
  
  print(united.length)
  print(common.length)
  print(diff.length)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".union("), "should generate .union()");
    assert!(rust_code.contains(".intersection("), "should generate .intersection()");
    assert!(rust_code.contains(".difference("), "should generate .difference()");
    assert_snapshot!("set_union_intersection_difference", rust_code);
}

// ============================================================
// Dogfooding v2 regression tests (Bugs #75-#82)
// ============================================================

#[test]
fn test_bug75_map_set_class_fields() {
    // Bug #75: Map/Set fields in classes must be recognized for correct method routing
    let source = r#"
Store {
  prices: Map<string, int>
  tags: Set<string>

  constructor() {
    this.prices = Map {}
    this.tags = Set {}
  }

  addPrice(name: string, price: int) {
    this.prices.set(name, price)
  }

  addTag(tag: string) {
    this.tags.add(tag)
  }

  hasTag(tag: string): bool {
    return this.tags.has(tag)
  }
}

main() {
  let s = Store()
  s.addPrice("apple", 1)
  s.addPrice("banana", 2)
  s.addTag("organic")
  print(s.hasTag("organic"))
}
"#;
    let rust_code = compile_and_generate(source);
    // Map.set → .insert(), Set.add → .insert(), Set.has → .contains()
    assert!(rust_code.contains(".insert("), "Map.set/Set.add should generate .insert()");
    assert!(rust_code.contains(".contains("), "Set.has should generate .contains()");
    assert!(!rust_code.contains(".set("), "should NOT generate .set() for Map");
    assert!(!rust_code.contains(".add("), "should NOT generate .add() for Set");
    assert_snapshot!("bug75_map_set_class_fields", rust_code);
}

#[test]
fn test_bug77_string_clone_in_instance_method() {
    // Bug #77: String variables passed to instance methods must be cloned
    let source = r#"
Lookup {
  data: Map<string, string>

  constructor() {
    this.data = Map {}
  }

  add(key: string, value: string) {
    this.data.set(key, value)
  }

  getName(key: string): string {
    return this.data.get(key) or "unknown"
  }

  getCount(key: string): int {
    return 1
  }
}

main() {
  let lookup = Lookup()
  lookup.add("a", "Alpha")
  let key = "a"
  let name = lookup.getName(key)
  let count = lookup.getCount(key)
  print(name)
  print(count)
}
"#;
    let rust_code = compile_and_generate(source);
    // key should be cloned when passed to instance methods
    assert!(rust_code.contains("key.clone()"), "string var should be cloned for instance method args");
    assert_snapshot!("bug77_string_clone_instance_method", rust_code);
}

#[test]
fn test_bug78_or_string_default() {
    // Bug #78: 'or "string"' in fallible call should generate .to_string()
    let source = r#"
validate(s: string): string {
  if s == "" => fail "empty"
  return s
}

main() {
  let result = validate("") or "FALLBACK"
  print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".to_string()"), "or-string should add .to_string()");
    // Verify default value has .to_string() appended
    assert_snapshot!("bug78_or_string_default", rust_code);
}

#[test]
fn test_bug79_some_every_lambda_pattern() {
    // Bug #79: some()/every() should use |&x| not |&&x| for Copy types
    let source = r#"
main() {
  let nums = [1, 2, 3, 4, 5]
  let hasEven = nums.some((n) => n % 2 == 0)
  let allPositive = nums.every((n) => n > 0)
  print(hasEven)
  print(allPositive)
}
"#;
    let rust_code = compile_and_generate(source);
    // some/every → any/all take FnMut(Self::Item) where .iter() yields &T, so |&x|
    assert!(rust_code.contains(".any(|&n|"), "some should use |&x| pattern");
    assert!(rust_code.contains(".all(|&n|"), "every should use |&x| pattern");
    assert!(!rust_code.contains("|&&"), "should NOT use |&&x| for some/every");
    assert_snapshot!("bug79_some_every_pattern", rust_code);
}

#[test]
fn test_bug80_for_in_map_clone() {
    // Bug #80: for-in-map variables are references, need cloning
    let source = r#"
main() {
  let prices = Map { "apple": 1, "banana": 2 }
  for name, price in prices {
    print(name)
    print(price)
  }
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("let name = name.clone()"), "map loop vars should be cloned");
    assert!(rust_code.contains("let price = price.clone()"), "map loop vars should be cloned");
    assert_snapshot!("bug80_for_in_map_clone", rust_code);
}

#[test]
fn test_bug81_map_get_or_default_expr() {
    // Bug #81: map.get(key) or default at expression level should use .unwrap_or()
    let source = r#"
main() {
  let config = Map { "host": "localhost", "port": "8080" }
  let timeout = config.get("timeout") or "30"
  print(timeout)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains(".unwrap_or("), "map.get or default should use .unwrap_or()");
    assert!(rust_code.contains("unwrap_or"), "should use unwrap_or for map.get or default");
    assert_snapshot!("bug81_map_get_or_default_expr", rust_code);
}

#[test]
fn test_bug82_mutating_map_set_methods() {
    // Bug #82: Map.set, Set.add, Set.delete should mark method as &mut self
    let source = r#"
Registry {
  items: Map<string, int>
  tags: Set<string>

  constructor() {
    this.items = Map {}
    this.tags = Set {}
  }

  register(name: string, count: int) {
    this.items.set(name, count)
    this.tags.add(name)
  }

  unregister(name: string) {
    this.items.delete(name)
    this.tags.delete(name)
  }
}

main() {
  let r = Registry()
  r.register("test", 5)
  r.unregister("test")
}
"#;
    let rust_code = compile_and_generate(source);
    // Methods calling .set/.add/.delete should get &mut self
    assert!(rust_code.contains("&mut self"), "mutating methods should use &mut self");
    assert_snapshot!("bug82_mutating_map_set_methods", rust_code);
}

// ============================================================
// v1.4 — Stdlib P0: String methods
// ============================================================

#[test]
fn test_v14_string_pad_repeat() {
    let source = r#"
main() {
    let s = "42"
    let padded = s.padStart(5)
    let padded2 = s.padStart(5, "0")
    let padEnd = s.padEnd(5)
    let padEnd2 = s.padEnd(5, ".")
    let repeated = s.repeat(3)
    print(padded)
    print(padded2)
    print(padEnd)
    print(padEnd2)
    print(repeated)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_pad_repeat", rust_code);
}

#[test]
fn test_v14_string_capitalize_reverse() {
    let source = r#"
main() {
    let s = "hello world"
    let cap = s.capitalize()
    let rev = s.reverse()
    let trunc = s.truncate(5)
    print(cap)
    print(rev)
    print(trunc)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_capitalize_reverse", rust_code);
}

#[test]
fn test_v14_string_blank_empty() {
    let source = r#"
main() {
    let s = "hello"
    let empty = ""
    let blank = "   "
    let a = s.isEmpty()
    let b = empty.isEmpty()
    let c = blank.isBlank()
    let d = s.isBlank()
    print(a)
    print(b)
    print(c)
    print(d)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_blank_empty", rust_code);
}

#[test]
fn test_v14_string_last_index_of() {
    let source = r#"
main() {
    let s = "hello world hello"
    let idx = s.lastIndexOf("hello")
    print(idx)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_last_index_of", rust_code);
}

#[test]
fn test_v14_string_slice() {
    let source = r#"
main() {
    let s = "hello world"
    let sliced = s.slice(0, 5)
    print(sliced)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_slice", rust_code);
}

#[test]
fn test_v14_string_replace_all() {
    let source = r#"
main() {
    let s = "aabbcc"
    let replaced = s.replaceAll("a", "x")
    print(replaced)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_replace_all", rust_code);
}

#[test]
fn test_v14_string_chars() {
    let source = r#"
main() {
    let s = "abc"
    let chars = s.chars()
    print(chars)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_chars", rust_code);
}

#[test]
fn test_v14_string_count_matches() {
    let source = r#"
main() {
    let s = "banana"
    let count = s.countMatches("an")
    print(count)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_count_matches", rust_code);
}

#[test]
fn test_v14_string_remove_prefix_suffix() {
    let source = r#"
main() {
    let s = "hello_world"
    let noPrefix = s.removePrefix("hello_")
    let noSuffix = s.removeSuffix("_world")
    print(noPrefix)
    print(noSuffix)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_string_remove_prefix_suffix", rust_code);
}

// ============================================================
// v1.4 — Stdlib P0: Math methods
// ============================================================

#[test]
fn test_v14_math_clamp_sign_log() {
    let source = r#"
main() {
    let clamped = Math.clamp(15.0, 0.0, 10.0)
    let s1 = Math.sign(5.0)
    let s2 = Math.sign(-3.0)
    let s3 = Math.sign(0.0)
    let lg = Math.log(2.718)
    print(clamped)
    print(s1)
    print(s2)
    print(s3)
    print(lg)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_math_clamp_sign_log", rust_code);
}

// ============================================================
// v1.4 — Stdlib P0: Array methods
// ============================================================

#[test]
fn test_v14_array_slice_take_drop() {
    let source = r#"
main() {
    let nums = [1, 2, 3, 4, 5]
    let sliced = nums.slice(1, 4)
    let taken = nums.take(3)
    let dropped = nums.drop(2)
    print(sliced)
    print(taken)
    print(dropped)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_slice_take_drop", rust_code);
}

#[test]
fn test_v14_array_first_last_empty() {
    let source = r#"
main() {
    let nums = [10, 20, 30]
    let f = nums.first()
    let l = nums.last()
    let empty = nums.isEmpty()
    print(f)
    print(l)
    print(empty)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_first_last_empty", rust_code);
}

#[test]
fn test_v14_array_sort_reversed_distinct() {
    let source = r#"
main() {
    let nums = [3, 1, 4, 1, 5, 9, 2, 6]
    let sorted = nums.sort()
    let rev = nums.reversed()
    let unique = nums.distinct()
    print(sorted)
    print(rev)
    print(unique)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_sort_reversed_distinct", rust_code);
}

#[test]
fn test_v14_array_flat_chunk() {
    let source = r#"
main() {
    let nested = [[1, 2], [3, 4], [5, 6]]
    let flat = nested.flat()
    let nums = [1, 2, 3, 4, 5, 6]
    let chunks = nums.chunks(2)
    print(flat)
    print(chunks)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_flat_chunk", rust_code);
}

#[test]
fn test_v14_array_zip() {
    let source = r#"
main() {
    let names = ["Alice", "Bob", "Charlie"]
    let ages = [30, 25, 35]
    let pairs = names.zip(ages)
    print(pairs)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_zip", rust_code);
}

#[test]
fn test_v14_array_sum_min_max() {
    let source = r#"
main() {
    let nums = [3, 1, 4, 1, 5]
    let total = nums.sum()
    let smallest = nums.min()
    let largest = nums.max()
    print(total)
    print(smallest)
    print(largest)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_sum_min_max", rust_code);
}

#[test]
fn test_v14_array_find_index() {
    let source = r#"
main() {
    let nums = [10, 20, 30, 40, 50]
    let idx = nums.findIndex(x => x > 25)
    print(idx)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_find_index", rust_code);
}

#[test]
fn test_v14_array_flat_map() {
    let source = r#"
main() {
    let nums = [1, 2, 3]
    let expanded = nums.flatMap(x => [x, x * 2])
    print(expanded)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_flat_map", rust_code);
}

#[test]
fn test_v14_array_count() {
    let source = r#"
main() {
    let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    let evens = nums.count(x => x % 2 == 0)
    print(evens)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v14_array_count", rust_code);
}

// =========================================================================
// v1.5 — rust {} interop tests
// =========================================================================

fn compile_and_generate_full(source: &str) -> (String, String) {
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, source).unwrap();
    let analyzed_program = analyze(program).unwrap();
    let ctx = livac::desugaring::desugar(analyzed_program.clone()).unwrap();
    generate_with_ast(&analyzed_program, ctx).unwrap()
}

#[test]
fn test_v15_rust_block_basic() {
    let source = r#"
main() {
    let result = rust {
        let x: i32 = 42;
        x * 2
    }
    print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_basic", rust_code);
}

#[test]
fn test_v15_rust_block_with_use_hoisting() {
    let source = r#"
main() {
    let hash = rust {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("key", "value");
        map.len()
    }
    print(hash)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_use_hoisting", rust_code);
}

#[test]
fn test_v15_rust_block_nested_braces() {
    let source = r#"
main() {
    let val = rust {
        let v: Vec<i32> = vec![1, 2, 3];
        let sum: i32 = v.iter().map(|x| { x * 2 }).sum();
        sum
    }
    print(val)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_nested_braces", rust_code);
}

#[test]
fn test_v15_use_rust_version_features_cargo() {
    let source = r#"
use rust "chrono" version "0.4"
use rust "uuid" version "1.0" features ["v4", "serde"]

main() {
    print("with deps")
}
"#;
    let (rust_code, cargo_toml) = compile_and_generate_full(source);
    assert_snapshot!("v15_use_rust_version_features_rs", rust_code);
    assert_snapshot!("v15_use_rust_version_features_cargo", cargo_toml);
}

#[test]
fn test_v15_use_rust_internal_features() {
    let source = r#"
use rust "tokio" features ["net", "io-util"]

main() {
    print("tokio with extra features")
}
"#;
    let (_rust_code, cargo_toml) = compile_and_generate_full(source);
    assert_snapshot!("v15_use_rust_internal_features_cargo", cargo_toml);
}

// ==========================================
// v1.5 Comprehensive Rust Interop Tests
// ==========================================

#[test]
fn test_v15_rust_block_strings_with_braces() {
    let source = r#"
main() {
    let s = rust {
        let a = "Hello {world}";
        let b = "Nested {{ double }} braces";
        let c = "Escaped \"quotes\" inside";
        let d = "Mix {of} \"everything\" {{here}}";
        format!("{} {} {} {}", a, b, c, d)
    }
    print(s)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_strings_with_braces", rust_code);
}

#[test]
fn test_v15_rust_block_comments_with_braces() {
    let source = r#"
main() {
    let a = rust {
        // Comment with { braces } that should be ignored
        let x: i32 = 42;
        x
    }
    let b = rust {
        /* Block comment with { braces }
           and more { nested { stuff } }
           across lines */
        let y: i32 = 100;
        y
    }
    print(a + b)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_comments_with_braces", rust_code);
}

#[test]
fn test_v15_rust_block_char_literals() {
    let source = r#"
main() {
    let val = rust {
        let ch: char = 'x';
        let brace_char: char = '{';
        let close_brace: char = '}';
        let count = if ch == 'x' { 1_i32 } else { 0_i32 };
        count
    }
    print(val)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_char_literals", rust_code);
}

#[test]
fn test_v15_rust_block_multiple_blocks() {
    let source = r#"
main() {
    let a = rust {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert("x", 1);
        m.len()
    }
    let b = rust {
        use std::collections::HashMap;
        use std::collections::HashSet;
        let mut m = HashMap::new();
        m.insert("y", 2);
        let mut s = HashSet::new();
        s.insert(42);
        m.len() + s.len()
    }
    let c = rust {
        let plain: i32 = 99;
        plain
    }
    print(a + b + c)
}
"#;
    let (rust_code, _) = compile_and_generate_full(source);
    // Verify use dedup: HashMap should appear only once in hoisted uses
    let hashmap_count = rust_code.matches("use std::collections::HashMap;").count();
    assert_eq!(hashmap_count, 1, "HashMap use should be deduplicated; found {} occurrences", hashmap_count);
    assert_snapshot!("v15_rust_block_multiple_blocks", rust_code);
}

#[test]
fn test_v15_rust_block_in_non_main_function() {
    let source = r#"
compute(): number {
    let val = rust {
        let data: Vec<i32> = vec![10, 20, 30];
        data.iter().sum::<i32>()
    }
    return val
}

main() {
    let result = compute()
    print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_in_non_main", rust_code);
}

#[test]
fn test_v15_rust_block_as_statement() {
    let source = r#"
main() {
    let x = 10
    rust {
        println!("Direct rust statement");
    }
    print(x)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_as_statement", rust_code);
}

#[test]
fn test_v15_rust_block_deeply_nested() {
    let source = r#"
main() {
    let val = rust {
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        let result: i32 = v.iter()
            .filter(|x| { **x > 2 })
            .map(|x| {
                let doubled = x * 2;
                if doubled > 8 {
                    doubled + 1
                } else {
                    doubled
                }
            })
            .sum();
        result
    }
    print(val)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_deeply_nested", rust_code);
}

#[test]
fn test_v15_rust_block_with_crate_decls() {
    let source = r#"
use rust "serde" version "1.0"
use rust "chrono" version "0.4"

main() {
    let val = rust {
        use std::time::SystemTime;
        let now = SystemTime::now();
        42_i32
    }
    print(val)
}
"#;
    let (rust_code, cargo_toml) = compile_and_generate_full(source);
    assert_snapshot!("v15_rust_block_with_crate_decls_rs", rust_code);
    assert_snapshot!("v15_rust_block_with_crate_decls_cargo", cargo_toml);
}

#[test]
fn test_v15_rust_block_mixed_with_liva() {
    let source = r#"
square(x: number): number = x * x

main() {
    let liva_val = square(5)
    let rust_val = rust {
        let x: i32 = 25;
        x + 1
    }
    print($"Liva: {liva_val}, Rust: {rust_val}")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_mixed_with_liva", rust_code);
}

#[test]
fn test_v15_rust_block_comment_with_rust_word() {
    // B42: Comments containing "rust {}" should NOT create phantom rust blocks
    let source = r#"
main() {
    // This uses rust {} interop for struct construction
    let x = 42
    print(x)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_comment_with_rust_word", rust_code);
}

#[test]
fn test_v15_rust_block_with_lifetimes() {
    // B43: Lifetimes like 'a should NOT confuse the brace balancer
    let source = r#"
main() {
    let val = rust {
        fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
            if x.len() > y.len() { x } else { y }
        }
        let result = longest("hello", "world!");
        result.len() as i32
    }
    print(val)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_rust_block_with_lifetimes", rust_code);
}

#[test]
fn test_v15_use_rust_combined_features() {
    let source = r#"
use rust "tokio" version "1" features ["rt-multi-thread", "macros"]
use rust "tokio" features ["net", "io-util"]

main() {
    print("combined features")
}
"#;
    let (_, cargo_toml) = compile_and_generate_full(source);
    assert_snapshot!("v15_use_rust_combined_features_cargo", cargo_toml);
}

#[test]
fn test_v15_use_rust_with_alias() {
    let source = r#"
use rust "serde_json" as json
use rust "chrono" version "0.4"

main() {
    print("aliased crate")
}
"#;
    let (_, cargo_toml) = compile_and_generate_full(source);
    assert_snapshot!("v15_use_rust_with_alias_cargo", cargo_toml);
}

// ─────────────────────────────────────────────────────
// v1.5 — Logging module tests
// ─────────────────────────────────────────────────────

#[test]
fn test_v15_log_info_basic() {
    let source = r#"
main() {
    Log.info("Server started")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_info_basic", rust_code);
}

#[test]
fn test_v15_log_all_levels() {
    let source = r#"
main() {
    Log.debug("Cache hit ratio: 0.95")
    Log.info("User login successful")
    Log.warn("Disk space low")
    Log.error("Connection failed")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_all_levels", rust_code);
}

#[test]
fn test_v15_log_with_context_map() {
    let source = r#"
main() {
    Log.info("User login", Map { "userId": 42, "ip": "10.0.0.1" })
    Log.warn("Rate limit close", Map { "current": 95, "max": 100 })
    Log.error("Connection failed", Map { "host": "db.prod" })
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_with_context_map", rust_code);
}

#[test]
fn test_v15_log_set_level() {
    let source = r#"
main() {
    Log.info("Before level change")
    Log.setLevel("warn")
    Log.info("This should be filtered")
    Log.warn("This should show")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_set_level", rust_code);
}

#[test]
fn test_v15_log_with_variable_message() {
    let source = r#"
main() {
    let msg = "Server started on port 3000"
    Log.info(msg)
    let level = "warn"
    Log.setLevel(level)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_with_variable_message", rust_code);
}

#[test]
fn test_v15_log_cargo_toml_chrono() {
    let source = r#"
main() {
    Log.info("test")
}
"#;
    let (_, cargo_toml) = compile_and_generate_full(source);
    assert!(cargo_toml.contains("chrono"), "Cargo.toml should contain chrono dependency");
    assert_snapshot!("v15_log_cargo_toml", cargo_toml);
}

#[test]
fn test_v15_log_unknown_function_error() {
    let source = r#"
main() {
    Log.trace("test")
}
"#;
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, source).unwrap();
    let analyzed_program = analyze(program).unwrap();
    let ctx = livac::desugaring::desugar(analyzed_program.clone()).unwrap();
    let result = generate_with_ast(&analyzed_program, ctx);
    assert!(result.is_err(), "Log.trace should produce an error");
}

#[test]
fn test_v15_log_in_function() {
    let source = r#"
processData(data: string) {
    Log.info("Processing", Map { "data": data })
    Log.debug("Details available")
}

main() {
    processData("test")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_in_function", rust_code);
}

#[test]
fn test_v15_log_variadic_args() {
    let source = r#"
main() {
    Log.info("Server started on port", 8080)
    Log.warn("Request from", "192.168.1.1", "method:", "GET")
    Log.error("Failed after", 3, "retries")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_variadic_args", rust_code);
}

#[test]
fn test_v15_log_table_map() {
    let source = r#"
main() {
    Log.info("Config loaded", Map { "host": "localhost", "port": 8080, "env": "prod", "debug": false })
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_table_map", rust_code);
}

#[test]
fn test_v15_log_table_array() {
    let source = r#"
main() {
    Log.info("Active users", [Map { "name": "Alice", "age": 30 }, Map { "name": "Bob", "age": 25 }])
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_table_array", rust_code);
}

#[test]
fn test_v15_log_variadic_with_table() {
    let source = r#"
main() {
    let ip = "10.0.0.1"
    Log.warn("Request from", ip, Map { "method": "GET", "path": "/api", "status": 200, "duration": "45ms" })
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_variadic_with_table", rust_code);
}

#[test]
fn test_v15_log_json_object() {
    let source = r#"
main() {
    let config, _err = JSON.parse("{\"host\":\"localhost\",\"port\":8080}")
    Log.info("Config", config)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_json_object", rust_code);
}

#[test]
fn test_v15_log_json_array() {
    let source = r#"
main() {
    let users, _err = JSON.parse("[{\"name\":\"Alice\"},{\"name\":\"Bob\"}]")
    Log.info("Users", users)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_log_json_array", rust_code);
}

// ─────────────────────────────────────────────────────
// v1.5 — Config module tests
// ─────────────────────────────────────────────────────

#[test]
fn test_v15_config_load() {
    let source = r#"
main() {
    let config, err = Config.load(".env")
    if err {
        print("Error: " + err)
        return
    }
    print("Config loaded")
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_config_load", rust_code);
}

#[test]
fn test_v15_config_get() {
    let source = r#"
main() {
    let config, err = Config.load(".env")
    if err {
        print("Error: " + err)
        return
    }
    let host, err2 = Config.get(config, "HOST")
    if err2 {
        print("Missing HOST")
    }
    print(host)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_config_get", rust_code);
}

#[test]
fn test_v15_config_get_int() {
    let source = r#"
main() {
    let config, err = Config.load(".env")
    let port, err2 = Config.getInt(config, "PORT")
    if err2 {
        print("Invalid PORT")
    }
    print(port)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_config_get_int", rust_code);
}

#[test]
fn test_v15_config_get_bool() {
    let source = r#"
main() {
    let config, _err = Config.load(".env")
    let verbose, err2 = Config.getBool(config, "VERBOSE")
    if err2 {
        print("Invalid VERBOSE")
    }
    if verbose {
        print("Verbose mode enabled")
    }
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_config_get_bool", rust_code);
}

#[test]
fn test_v15_config_get_all() {
    let source = r#"
main() {
    let config, _err = Config.load(".env")
    let all = Config.getAll(config)
    print(all)
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_config_get_all", rust_code);
}

#[test]
fn test_v15_config_unknown_function_error() {
    let source = r#"
main() {
    let x = Config.unknown("test")
}
"#;
    let tokens = tokenize(source).unwrap();
    let program = parse(tokens, source).unwrap();
    let analyzed_program = analyze(program).unwrap();
    let ctx = livac::desugaring::desugar(analyzed_program.clone()).unwrap();
    let result = generate_with_ast(&analyzed_program, ctx);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Unknown Config function"), "Error should mention unknown Config function, got: {}", err_msg);
}

#[test]
fn test_v15_config_load_and_use() {
    let source = r#"
main() {
    let config, err = Config.load("config.env")
    if err {
        print("Cannot load config: " + err)
        return
    }
    let host, e1 = Config.get(config, "HOST")
    let port, e2 = Config.getInt(config, "PORT")
    let debug, e3 = Config.getBool(config, "DEBUG")
    print("Server: " + host + ":" + port)
    if debug {
        print("Debug mode on")
    }
}
"#;
    let rust_code = compile_and_generate(source);
    assert_snapshot!("v15_config_load_and_use", rust_code);
}

#[test]
fn test_array_index_as_arg_clones() {
    // B35: arr[i] as function argument should clone for non-Copy types
    let source = r#"
process(item: string) {
    print(item)
}

main() {
    let items = ["hello", "world"]
    process(items[0])
    process(items[1])
}
"#;
    let rust_code = compile_and_generate(source);
    // Should contain .clone() for string array index access
    assert!(rust_code.contains(".clone()"), "Array index access as arg should clone: {}", rust_code);
    assert_snapshot!("array_index_as_arg_clones", rust_code);
}

#[test]
fn test_self_array_index_clones() {
    // B21: this.tokens[idx] should generate .clone() for non-Copy types
    let source = r#"
Parser {
    tokens: [string]

    constructor(t: [string]) {
        this.tokens = t
    }

    current() => this.tokens[0]
}

main() {
    let p = Parser(["hello", "world"])
    print(p.current())
}
"#;
    let rust_code = compile_and_generate(source);
    // Should contain .clone() for this.tokens[0] access
    assert!(rust_code.contains(".clone()"), "this.tokens[i] should clone: {}", rust_code);
    assert_snapshot!("self_array_index_clones", rust_code);
}

#[test]
fn test_self_field_as_arg_clones() {
    // B44: this.field passed as function argument should clone in &self methods
    let source = r#"
format_priority(p: string): string {
    return "Priority: " + p
}

Task {
    title: string
    priority: string

    constructor(t: string, p: string) {
        this.title = t
        this.priority = p
    }

    label() => format_priority(this.priority)
}

main() {
    let t = Task("Fix bug", "high")
    print(t.label())
}
"#;
    let rust_code = compile_and_generate(source);
    // The label() method is &self, so this.priority needs .clone() when passed as arg
    assert!(rust_code.contains("self.priority.clone()"), "self.field as arg should clone: {}", rust_code);
    assert_snapshot!("self_field_as_arg_clones", rust_code);
}

#[test]
fn test_for_self_field_iter_mut() {
    // B45: for item in this.items with mutation should use .iter_mut()
    let source = r#"
TodoItem {
    title: string
    done: bool

    constructor(t: string) {
        this.title = t
        this.done = false
    }
}

TodoList {
    items: [TodoItem]

    constructor() {
        this.items = []
    }

    addItem(title: string) {
        this.items.push(TodoItem(title))
    }

    completeAll() {
        for item in this.items {
            item.done = true
        }
    }

    showAll() {
        for item in this.items {
            print(item.title)
        }
    }
}

main() {
    let list = TodoList()
    list.addItem("Buy milk")
    list.completeAll()
    list.showAll()
}
"#;
    let rust_code = compile_and_generate(source);
    // completeAll() mutates loop var → should use .iter_mut()
    // showAll() doesn't mutate → should use .clone()
    assert!(rust_code.contains(".iter_mut()"), "Mutating for-loop should use iter_mut: {}", rust_code);
    assert_snapshot!("for_self_field_iter", rust_code);
}

#[test]
fn test_error_binding_mut_when_reassigned() {
    // B34: Error binding vars should be mut when reassigned later
    let source = r#"
tryParse(s: string): int {
    if s == "" {
        fail "empty string"
    }
    return 42
}

main() {
    let val, err = tryParse("42")
    if err {
        val = 0
    }
    print(val)
}
"#;
    let rust_code = compile_and_generate(source);
    // val should be declared as mut since it's reassigned
    assert!(rust_code.contains("mut val"), "Error binding var should be mut when reassigned: {}", rust_code);
    assert_snapshot!("error_binding_mut_reassigned", rust_code);
}

#[test]
fn test_transitive_mut_self() {
    // B09: method calling &mut self method should also be &mut self
    let source = r#"
Counter {
    value: int

    constructor() {
        this.value = 0
    }

    increment() {
        this.value = this.value + 1
    }

    incrementTwice() {
        this.increment()
        this.increment()
    }
}

main() {
    let c = Counter()
    c.incrementTwice()
    print(c.value)
}
"#;
    let rust_code = compile_and_generate(source);
    // incrementTwice calls increment which is &mut self, so incrementTwice should also be &mut self
    assert!(rust_code.contains("fn increment_twice(&mut self)"), "Transitive &mut self should propagate: {}", rust_code);
    assert_snapshot!("transitive_mut_self", rust_code);
}

#[test]
fn test_class_param_dot_notation() {
    // B07: Function parameter with class type should use .field not get_field()
    let source = r#"
Item {
    name: string
    price: float

    constructor(n: string, p: float) {
        this.name = n
        this.price = p
    }
}

showItem(item: Item) {
    print(item.name + " costs " + item.price)
}

main() {
    let i = Item("Book", 9.99)
    showItem(i)
}
"#;
    let rust_code = compile_and_generate(source);
    // Check user code (after liva_rt module) for dot notation instead of get_field()
    let user_code = rust_code.split("fn show_item").last().unwrap_or(&rust_code);
    assert!(!user_code.contains("get_field"), "Class param should use dot notation: {}", user_code);
    assert!(user_code.contains("item.name"), "Should access .name directly: {}", user_code);
    assert_snapshot!("class_param_dot_notation", rust_code);
}

#[test]
fn test_type_as_field_name() {
    // B37: `type` as field name should use r#type in Rust
    let source = r#"
Token {
    type: string
    value: string
    line: int

    constructor(t: string, v: string, l: int) {
        this.type = t
        this.value = v
        this.line = l
    }
}

main() {
    let tok = Token("ident", "hello", 1)
    print(tok.type)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("r#type"), "Should escape `type` keyword with r#: {}", rust_code);
    assert_snapshot!("type_as_field_name", rust_code);
}

#[test]
fn test_enum_field_default_derive() {
    // B14: Class with enum field should compile — enum must derive Default
    let source = r#"
enum Priority { High, Medium, Low }

Item {
    title: string
    priority: Priority
    done: bool

    constructor(t: string, p: Priority) {
        this.title = t
        this.priority = p
        this.done = false
    }
}

main() {
    let item = Item("Fix bug", Priority.High)
    print(item.title)
}
"#;
    let rust_code = compile_and_generate(source);
    // Enum should derive Default with #[default] on first variant
    assert!(rust_code.contains("#[derive(Debug, Clone, PartialEq, Default)]"), "Enum should derive Default: {}", rust_code);
    assert!(rust_code.contains("#[default]"), "First variant should have #[default]: {}", rust_code);
    assert_snapshot!("enum_field_default_derive", rust_code);
}

#[test]
fn test_json_stringify_triggers_serde() {
    // B46: JSON.stringify should trigger serde derives on the class
    let source = r#"
Author {
    id: int
    name: string

    constructor(id: int, name: string) {
        this.id = id
        this.name = name
    }
}

main() {
    let author = Author(1, "Alice")
    let json, err = JSON.stringify(author)
    print(json)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("serde::Serialize"), "Author should derive Serialize: {}", rust_code);
    assert!(rust_code.contains("serde::Deserialize"), "Author should derive Deserialize: {}", rust_code);
    assert_snapshot!("json_stringify_serde", rust_code);
}

#[test]
fn test_const_string_type() {
    // B31: const string should generate &str, not String
    let source = r#"
const DB_FILE: string = "data/db.json"

main() {
    print(DB_FILE)
}
"#;
    let rust_code = compile_and_generate(source);
    assert!(rust_code.contains("const DB_FILE: &str"), "Should use &str for const string: {}", rust_code);
    assert!(!rust_code.contains("const DB_FILE: String"), "Should NOT use String for const: {}", rust_code);
    // Verify the const line specifically doesn't have .to_string()
    let const_line = rust_code.lines().find(|l| l.contains("const DB_FILE")).unwrap_or("");
    assert!(!const_line.contains(".to_string()"), "Const should not have .to_string(): {}", const_line);
    assert_snapshot!("const_string_type", rust_code);
}

#[test]
fn test_console_input_template() {
    // B11: console.input with template string shouldn't nest print!(format!(...))
    let source = r#"
main() {
    let id = 42
    let name = console.input($"Enter name for #{id}: ")
    print(name)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should use print!("{}", format!(...)) not print!(format!(...))
    assert!(rust_code.contains("print!(\"{}\","), "Should use print!(\"{{}}\", ...) pattern: {}", rust_code);
    assert!(!rust_code.contains("print!(format!"), "Should NOT nest print!(format!(...)): {}", rust_code);
    assert_snapshot!("console_input_template", rust_code);
}

#[test]
fn test_array_element_assignment() {
    // B39: arr[i] = val should not generate .clone() on LHS
    let source = r#"
main() {
    let arr = ["a", "b", "c"]
    let i = 1
    arr[i] = "X"
    print(arr)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should NOT have .clone() on the LHS of assignment
    let assign_line = rust_code.lines().find(|l| l.contains("arr[") && l.contains("= \"X\"")).unwrap_or("");
    assert!(!assign_line.contains(".clone()"), "arr[i] = val should not clone on LHS: {}", assign_line);
    assert!(rust_code.contains("as usize]"), "Should cast index to usize: {}", rust_code);
    assert_snapshot!("array_element_assignment", rust_code);
}

#[test]
fn test_single_var_fallible_binding() {
    // B33: let writeErr = File.write(path, content) should extract only the error string
    let source = r#"
main() {
    let path = "test.txt"
    let content = "hello"
    let writeErr = File.write(path, content)
    if writeErr {
        console.error($"Error: {writeErr}")
    }
}
"#;
    let rust_code = compile_and_generate(source);
    // Should extract the error string (.1), not assign the raw tuple
    assert!(rust_code.contains(".1"), "Should extract error string with .1: {}", rust_code);
    // The variable should be a plain string, not a tuple — check the let line specifically
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    let let_line = main_code.lines().find(|l| l.contains("let write_err")).unwrap_or("");
    assert!(!let_line.contains(": (Option<"), "Should not expose tuple type to variable: {}", let_line);
    assert_snapshot!("single_var_fallible_binding", rust_code);
}

#[test]
fn test_parse_int_or_default() {
    // B16: parseInt(x) or 0 should generate match parse with direct value, not tuple
    let source = r#"
main() {
    let input = "42"
    let num = parseInt(input) or 0
    print(num)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should extract the value directly, not a tuple
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    assert!(main_code.contains("parse::<i32>()"), "Should use parse::<i32>(): {}", main_code);
    assert!(main_code.contains("Ok(v) => v"), "Should extract value directly with Ok(v) => v: {}", main_code);
    assert!(main_code.contains("Err(_) => 0"), "Should use user's default value: {}", main_code);
    // Should NOT contain tuple form
    assert!(!main_code.contains("(v, String::new())"), "Should NOT generate tuple form: {}", main_code);
    assert_snapshot!("parse_int_or_default", rust_code);
}

#[test]
fn test_char_at_returns_char() {
    // B25: charAt should return char, not String, so char comparisons work
    let source = r#"
main() {
    let text = "hello"
    let ch = text.charAt(0)
    if ch == 'h' {
        print("found h")
    }
}
"#;
    let rust_code = compile_and_generate(source);
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    // Should use unwrap_or('\0') not .map(|c| c.to_string()).unwrap_or_default()
    assert!(main_code.contains("unwrap_or('\\0')"), "Should return char with unwrap_or: {}", main_code);
    let ch_line = main_code.lines().find(|l| l.contains("let ch") || l.contains("charAt")).unwrap_or("");
    assert!(!ch_line.contains("c.to_string()"), "charAt should NOT convert to String: {}", ch_line);
    assert_snapshot!("char_at_returns_char", rust_code);
}

#[test]
fn test_char_escape_sequences() {
    // B26: char escape sequences like '\n', '\t', '\r' should be preserved, not truncated to '\\'
    let source = r#"
main() {
    let text = "hello world"
    let ch = text.charAt(0)
    if ch == '\n' {
        print("newline")
    }
    if ch == '\t' {
        print("tab")
    }
    if ch == '\\' {
        print("backslash")
    }
}
"#;
    let rust_code = compile_and_generate(source);
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    // Each escape should generate its correct Rust char literal
    assert!(main_code.contains("'\\n'"), "Should contain newline char literal: {}", main_code);
    assert!(main_code.contains("'\\t'"), "Should contain tab char literal: {}", main_code);
    assert!(main_code.contains("'\\\\'"), "Should contain backslash char literal: {}", main_code);
    assert_snapshot!("char_escape_sequences", rust_code);
}

#[test]
fn test_string_concat_not_extend() {
    // B28: result = result + "x" should use format!, not .extend()
    let source = r#"
main() {
    let result = ""
    result = result + "hello"
    print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    assert!(!main_code.contains(".extend("), "Should NOT use .extend() for string concat: {}", main_code);
    assert!(main_code.contains("format!"), "Should use format! for string concat: {}", main_code);
    assert_snapshot!("string_concat_not_extend", rust_code);
}

#[test]
fn test_template_mutable_var_display_format() {
    // B29: Mutable vars in template strings should use {} (Display), not {:?} (Debug)
    let source = r#"
main() {
    let result = ""
    result = "hello world"
    print($"result: {result}")
}
"#;
    let rust_code = compile_and_generate(source);
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    // Should NOT have {:?} for the result variable — that adds unwanted quotes
    assert!(!main_code.contains("{:?}"), "Mutable string var should use Display, not Debug: {}", main_code);
    assert!(main_code.contains("\"result: {}\""), "Should use {{}} format for string var: {}", main_code);
    assert_snapshot!("template_mutable_var_display", rust_code);
}

#[test]
fn test_async_http_resp_body_dot_notation() {
    // B05: async HTTP.get response should use .body dot notation, not get_field("body")
    let source = r#"
main() {
    let resp, err = async HTTP.get("https://example.com")
    if err {
        print($"Error: {err}")
    }
    print(resp.body)
}
"#;
    let rust_code = compile_and_generate(source);
    // resp.body should NOT use get_field
    assert!(!rust_code.contains("get_field(\"body\")"), "Should use .body not get_field: {}", rust_code);
    assert_snapshot!("async_http_resp_body", rust_code);
}

#[test]
fn test_class_count_method_not_array_builtin() {
    // B10: .count() on class instance should call user method, not array built-in
    let source = r#"
TaskManager {
    tasks: [string]

    count(): int => this.tasks.length
}

main() {
    let manager = TaskManager(["a", "b"])
    let n = manager.count()
    print(n)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should NOT have .iter().filter().count() (array pipeline)
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    assert!(!main_code.contains(".iter()"), "Should NOT use .iter() for class .count(): {}", main_code);
    assert!(main_code.contains("manager.count()"), "Should call method directly: {}", main_code);
    assert_snapshot!("class_count_method", rust_code);
}

#[test]
fn test_string_ordering_comparison() {
    // B40: String >= &str needs .as_str() because PartialOrd<&str> not impl for String
    let source = r#"
isDigit(c: string): bool => c >= "0" and c <= "9"

main() {
    let result = isDigit("5")
    print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should have .as_str() on the string variable for ordering comparison
    assert!(rust_code.contains(".as_str()"), "Should use .as_str() for String ordering comparison: {}", rust_code);
    assert_snapshot!("string_ordering_comparison", rust_code);
}

#[test]
fn test_cast_priority_index_arithmetic() {
    // B41: arr[pos + 1] should generate (pos + 1) as usize, not pos + 1 as usize
    let source = r#"
main() {
    let text = "hello"
    let pos = 2
    let ch = text[pos + 1]
    print(ch)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should wrap the arithmetic expression in parens before `as usize`
    assert!(rust_code.contains("(pos + 1) as usize"), "Should wrap expr in parens: {}", rust_code);
    assert!(!rust_code.contains("pos + 1 as usize"), "Should NOT have bare 'pos + 1 as usize': {}", rust_code);
    assert_snapshot!("cast_priority_index", rust_code);
}

#[test]
fn test_float_div_length_auto_cast() {
    // B32: f64 / .length should auto-cast .length to f64
    let source = r#"
main() {
    let total = 10.5
    let items = ["a", "b", "c"]
    let avg = total / items.length
    print(avg)
}
"#;
    let rust_code = compile_and_generate(source);
    let main_code = rust_code.split("fn main()").last().unwrap_or("");
    // Should cast .length to f64 for division with float
    assert!(main_code.contains("as f64"), "Should cast .length to f64: {}", main_code);
    assert_snapshot!("float_div_length", rust_code);
}

#[test]
fn test_enum_destructuring_field_name_mapping() {
    // B27: Enum destructuring with different binding names should use field_name: binding
    let source = r#"
enum Token {
    TString(value: string)
    TNumber(value: string)
}

show(t: Token): string {
    return switch t {
        Token.TString(v) => $"str({v})"
        Token.TNumber(n) => $"num({n})"
    }
}

main() {
    let t = Token.TString("hello")
    print(show(t))
}
"#;
    let rust_code = compile_and_generate(source);
    // When binding name differs from field name, should use field: binding syntax
    assert!(rust_code.contains("value: v") || rust_code.contains("value: n"),
        "Should map field name to binding name: {}", rust_code);
    assert_snapshot!("enum_destructuring_field_mapping", rust_code);
}

#[test]
fn test_main_async_when_rust_block_has_await() {
    // B24: main() should be async when rust { } block contains .await
    let source = r#"
use rust "tokio" features ["full"]

main() {
    rust {
        let result = some_async_fn().await;
        println!("{}", result);
    }
}
"#;
    let rust_code = compile_and_generate(source);
    // main should be marked async with #[tokio::main]
    assert!(rust_code.contains("async fn main()"), "main should be async: {}", rust_code);
    assert!(rust_code.contains("#[tokio::main]"), "Should have tokio::main attribute: {}", rust_code);
    assert_snapshot!("main_async_rust_block_await", rust_code);
}

#[test]
fn test_spawn_async_user_function_has_await() {
    // B04: task async userFn(arg) should generate .await inside spawn_async
    let source = r#"
fetchData(url: string): string {
    let resp, err = HTTP.get(url)
    if err {
        return ""
    }
    return resp.body
}

main() {
    let t = task async fetchData("https://example.com")
    let result = await t
    print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should have .await inside spawn_async for the user function
    assert!(rust_code.contains("fetch_data(") && rust_code.contains(".await"),
        "User async fn should have .await inside spawn: {}", rust_code);
    assert_snapshot!("spawn_async_user_fn_await", rust_code);
}

#[test]
fn test_template_string_nested_quotes() {
    // B02: Template strings with function calls containing string args
    // $"{fn("arg")}" should parse correctly — the inner "arg" should not close the template
    let source = r#"
transform(text: string): string {
    return text
}

main() {
    let result = $"output: {transform("hello")}"
    print(result)
}
"#;
    let rust_code = compile_and_generate(source);
    // Should compile successfully and contain the function call
    assert!(rust_code.contains("transform("), "Should contain the function call: {}", rust_code);
    assert_snapshot!("template_nested_quotes", rust_code);
}