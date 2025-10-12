use insta::assert_snapshot;
use livac::codegen::generate_from_ir;
use livac::desugaring::desugar;
use livac::lexer::tokenize;
use livac::lowering::lower_program;
use livac::parser::parse;
use livac::semantic::analyze;

fn compile_ir(source: &str) -> (String, String) {
    let tokens = tokenize(source).expect("tokenize");
    let program = parse(tokens, source).expect("parse");
    let analyzed = analyze(program.clone()).expect("semantic");
    let ctx = desugar(analyzed.clone()).expect("desugar");
    let module = lower_program(&analyzed);

    generate_from_ir(&module, &program, ctx).expect("codegen")
}

#[test]
fn ir_codegen_simple_function() {
    let source = r#"
        add(a: number, b: number): number = a + b

        main() {
            let result = add(2, 3)
            print(result)
        }
    "#;

    let (rust_code, _cargo) = compile_ir(source);
    assert_snapshot!("ir_simple_function", rust_code);
}

#[test]
fn ir_codegen_async_helpers() {
    let source = r#"
        fetch() {
            let response = async getData()
            return response
        }

        main() {
            let value = async fetch()
            fire async fetch()
            print(value)
        }
    "#;

    let (rust_code, _cargo) = compile_ir(source);
    assert_snapshot!("ir_async_helpers", rust_code);
}

#[test]
fn ir_codegen_parallel_helpers() {
    let source = r#"
        heavy() {
            return parallel compute()
        }

        main() {
            let value = parallel heavy()
            fire parallel heavy()
            print(value)
        }
    "#;

    let (rust_code, _cargo) = compile_ir(source);
    assert_snapshot!("ir_parallel_helpers", rust_code);
}

#[test]
fn ir_codegen_try_catch_and_switch() {
    let source = r#"
        main() {
            try {
                let flag = false
                if flag == false {
                    throw "error"
                }
                print("done")
            } catch (err) {
                print(err)
            }

            switch 200 {
                case 200:
                    print("success")
                case 500:
                    print("server")
                default:
                    print("other")
            }
        }
    "#;

    let (rust_code, _cargo) = compile_ir(source);
    assert_snapshot!("ir_try_catch_switch", rust_code);
}

#[test]
fn ir_codegen_string_templates() {
    let source = r#"
        main() {
            let name = "Liva User"
            let numbers = [1, 2, 3]
            let users = [
                { name: "Alice" },
                { name: "Bob" }
            ]

            print($"Name: {name}")
            print($"Numbers: {numbers}")
            print($"First user: {users[0].name}\n")
        }
    "#;

    let (rust_code, _cargo) = compile_ir(source);

    assert!(
        rust_code.contains(r#"format!("Name: {}", name)"#),
        "expected simple template to use Display placeholder:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains(r#"format!("Numbers: {:?}", numbers)"#),
        "expected array template to use Debug placeholder:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains(r#"format!("First user: {}\n", users[0]["name"])"#),
        "expected nested access to use bracket notation with newline escape:\n{}",
        rust_code
    );
}

#[test]
fn ir_codegen_array_param_inference_and_numeric_coercion() {
    let source = r#"
        isAdult(age) => age >= 18

        calculateTotal(items) {
            let total = 0.0
            for item in items {
                total = total + item.price
            }
            return total
        }

        main() {
            let products = [
                { price: 19.99 },
                { price: 5.50 }
            ]

            let total = calculateTotal(products)
            print(isAdult(21))
            print(total)
        }
    "#;

    let (rust_code, _cargo) = compile_ir(source);

    assert!(
        rust_code.contains("fn is_adult(age: i32) -> bool"),
        "expected isAdult to infer a bool return type:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("fn calculate_total(items: Vec<serde_json::Value>)"),
        "expected calculateTotal to infer Vec<serde_json::Value> parameter:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains(".as_f64().unwrap_or(0.0)"),
        "expected numeric coercion when summing JSON values:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("let mut total = 0.0;"),
        "expected float literal to remain 0.0 and variable to be mutable:\n{}",
        rust_code
    );
}
