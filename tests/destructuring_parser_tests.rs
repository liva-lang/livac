#[cfg(test)]
mod destructuring_tests {
    use livac::parser::*;
    use livac::lexer::*;
    use livac::ast::*;

    #[test]
    fn test_object_destructuring_simple() {
        let source = "
            test() {
                let {name, age} = user
            }
        ";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_object_destructuring_with_rename() {
        let source = "
            test() {
                let {name: userName, age: userAge} = user
            }
        ";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_destructuring_simple() {
        let source = "
            test() {
                let [first, second] = array
            }
        ";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_destructuring_with_skip() {
        let source = "
            test() {
                let [first, , third] = array
            }
        ";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_destructuring_with_rest() {
        let source = "
            test() {
                let [head, ...tail] = items
            }
        ";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_destructuring_with_type_annotation() {
        let source = "
            test() {
                let {x, y}: Point = point
            }
        ";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
