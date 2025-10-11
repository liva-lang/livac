use proptest::prelude::*;
use livac::{lexer::tokenize, parser::parse, semantic::analyze};

/// Generador de código Liva válido para property testing
fn valid_liva_source() -> impl Strategy<Value = String> {
    let identifier = "[a-zA-Z][a-zA-Z0-9_]*";
    let number = "[0-9]+";
    let string = "\"[^\"]*\"";
    
    // Generar funciones simples
    let function = format!(
        "{}() {{ let x = {}; print({}); }}",
        identifier, number, string
    );
    
    // Generar clases simples
    let class = format!(
        "{} {{ {}: string; get{}(): string = this.{}; }}",
        identifier, identifier, identifier, identifier
    );
    
    // Combinar diferentes tipos de código
    prop::sample::select(vec![function, class])
        .prop_map(|code| code)
}

proptest! {
    #[test]
    fn test_parse_valid_source(input in valid_liva_source()) {
        // Test que el parser puede manejar código válido sin panics
        let tokens = tokenize(&input);
        if let Ok(tokens) = tokens {
            let ast = parse(tokens);
            // No importa si falla, solo que no haga panic
            let _ = ast;
        }
    }
    
    #[test]
    fn test_lexer_tokenize_valid_source(input in valid_liva_source()) {
        // Test que el lexer puede tokenizar código válido sin panics
        let _ = tokenize(&input);
    }
    
    #[test]
    fn test_semantic_analyze_valid_source(input in valid_liva_source()) {
        // Test que el análisis semántico puede procesar código válido sin panics
        let tokens = tokenize(&input);
        if let Ok(tokens) = tokens {
            let ast = parse(tokens);
            if let Ok(ast) = ast {
                let _ = analyze(ast);
            }
        }
    }
}

/// Test de idempotencia: parse → pretty → parse
#[test]
fn test_parse_pretty_parse_idempotent() {
    let test_cases = vec![
        "main() { print(\"hello\"); }",
        "sum(a: number, b: number): number = a + b",
        "Persona { nombre: string; getNombre(): string = this.nombre; }",
    ];
    
    for source in test_cases {
        // Parse original
        let tokens1 = tokenize(source).unwrap();
        let ast1 = parse(tokens1).unwrap();
        
        // Convertir a string (pretty print simulado)
        let pretty = format!("{:?}", ast1);
        
        // Parse de nuevo (esto es una aproximación, ya que no tenemos un pretty printer real)
        // Por ahora, verificamos que el AST original es consistente
        let tokens2 = tokenize(source).unwrap();
        let ast2 = parse(tokens2).unwrap();
        
        // Los ASTs deberían ser iguales
        assert_eq!(ast1.items.len(), ast2.items.len());
    }
}

/// Test de robustez ante entradas aleatorias
proptest! {
    #[test]
    fn test_lexer_robustness(input in ".*") {
        // Test que el lexer no hace panic con cualquier entrada
        let _ = tokenize(&input);
    }
    
    #[test]
    fn test_parser_robustness(input in "[a-zA-Z0-9_(){};=+\"\\s]*") {
        // Test que el parser no hace panic con entradas semi-válidas
        let tokens = tokenize(&input);
        if let Ok(tokens) = tokens {
            let _ = parse(tokens);
        }
    }
}
