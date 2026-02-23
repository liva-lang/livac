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

// ---------------------------------------------------------------------------
// 10. Concurrency
// ---------------------------------------------------------------------------

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

    // fire: no handle, fire and forget
    fire async logEvent("app started")

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
// 29. Data Class
// ---------------------------------------------------------------------------

#[test]
fn test_feature_data_class() {
    let source = r#"
data Point {
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
data Color {
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
    let source = r#"
data Point {
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
