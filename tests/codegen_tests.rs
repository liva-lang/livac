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
