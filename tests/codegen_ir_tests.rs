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
