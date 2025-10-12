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
sum(a, b) = a + b
multiply(x, y) = x * y
greet(name) = "Hello " + name

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
add(a, b) = a + b
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
add(a: number, b: number): number = a + b
greet(name: string): string = "Hello " + name
isEven(n: number): bool = n % 2 == 0

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
add(a, b) = a + b
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
