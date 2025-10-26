#[cfg(test)]
mod destructuring_tests {
    use livac::parser::*;
    use livac::lexer::*;
    use livac::ast::*;

    #[test]
    fn test_object_destructuring_simple() {
        let source = "myFunc() {
    let {name, age} = user
    return name
}";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse object destructuring");
        
        // Verify it parsed as a function with VarDecl containing ObjectPattern
        let program = result.unwrap();
        assert_eq!(program.items.len(), 1);
        
        if let TopLevel::Function(func) = &program.items[0] {
            assert_eq!(func.name, "myFunc");
            // Should have a let statement with object pattern
            if let Stmt::VarDecl(var_decl) = &func.body.as_ref().unwrap().stmts[0] {
                assert_eq!(var_decl.bindings.len(), 1);
                match &var_decl.bindings[0].pattern {
                    BindingPattern::Object(obj_pattern) => {
                        assert_eq!(obj_pattern.fields.len(), 2);
                        assert_eq!(obj_pattern.fields[0].key, "name");
                        assert_eq!(obj_pattern.fields[1].key, "age");
                    }
                    _ => panic!("Expected ObjectPattern"),
                }
            } else {
                panic!("Expected VarDecl statement");
            }
        } else {
            panic!("Expected Function");
        }
    }

    #[test]
    fn test_object_destructuring_with_rename() {
        let source = "myFunc() {
    let {name: userName, age: userAge} = user
    return userName
}";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse object destructuring with rename");
        
        let program = result.unwrap();
        if let TopLevel::Function(func) = &program.items[0] {
            if let Stmt::VarDecl(var_decl) = &func.body.as_ref().unwrap().stmts[0] {
                match &var_decl.bindings[0].pattern {
                    BindingPattern::Object(obj_pattern) => {
                        assert_eq!(obj_pattern.fields[0].key, "name");
                        assert_eq!(obj_pattern.fields[0].binding, "userName");
                        assert_eq!(obj_pattern.fields[1].key, "age");
                        assert_eq!(obj_pattern.fields[1].binding, "userAge");
                    }
                    _ => panic!("Expected ObjectPattern"),
                }
            }
        }
    }

    #[test]
    fn test_array_destructuring_simple() {
        let source = "myFunc() {
    let [first, second] = array
    return first
}";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse array destructuring");
        
        let program = result.unwrap();
        if let TopLevel::Function(func) = &program.items[0] {
            if let Stmt::VarDecl(var_decl) = &func.body.as_ref().unwrap().stmts[0] {
                match &var_decl.bindings[0].pattern {
                    BindingPattern::Array(arr_pattern) => {
                        assert_eq!(arr_pattern.elements.len(), 2);
                        assert_eq!(arr_pattern.elements[0], Some("first".to_string()));
                        assert_eq!(arr_pattern.elements[1], Some("second".to_string()));
                        assert_eq!(arr_pattern.rest, None);
                    }
                    _ => panic!("Expected ArrayPattern"),
                }
            }
        }
    }

    #[test]
    fn test_array_destructuring_with_skip() {
        let source = "myFunc() {
    let [first, , third] = array
    return first
}";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse array destructuring with skip");
        
        let program = result.unwrap();
        if let TopLevel::Function(func) = &program.items[0] {
            if let Stmt::VarDecl(var_decl) = &func.body.as_ref().unwrap().stmts[0] {
                match &var_decl.bindings[0].pattern {
                    BindingPattern::Array(arr_pattern) => {
                        assert_eq!(arr_pattern.elements.len(), 3);
                        assert_eq!(arr_pattern.elements[0], Some("first".to_string()));
                        assert_eq!(arr_pattern.elements[1], None); // skipped
                        assert_eq!(arr_pattern.elements[2], Some("third".to_string()));
                    }
                    _ => panic!("Expected ArrayPattern"),
                }
            }
        }
    }

    #[test]
    fn test_array_destructuring_with_rest() {
        let source = "myFunc() {
    let [head, ...tail] = items
    return head
}";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse array destructuring with rest");
        
        let program = result.unwrap();
        if let TopLevel::Function(func) = &program.items[0] {
            if let Stmt::VarDecl(var_decl) = &func.body.as_ref().unwrap().stmts[0] {
                match &var_decl.bindings[0].pattern {
                    BindingPattern::Array(arr_pattern) => {
                        assert_eq!(arr_pattern.elements.len(), 1);
                        assert_eq!(arr_pattern.elements[0], Some("head".to_string()));
                        assert_eq!(arr_pattern.rest, Some("tail".to_string()));
                    }
                    _ => panic!("Expected ArrayPattern"),
                }
            }
        }
    }

    #[test]
    fn test_destructuring_with_type_annotation() {
        let source = "myFunc() {
    let {x, y}: Point = point
    return x
}";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens, source);
        
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse destructuring with type annotation");
        
        let program = result.unwrap();
        if let TopLevel::Function(func) = &program.items[0] {
            if let Stmt::VarDecl(var_decl) = &func.body.as_ref().unwrap().stmts[0] {
                // Check that type annotation is present
                assert!(var_decl.bindings[0].type_ref.is_some());
                match &var_decl.bindings[0].pattern {
                    BindingPattern::Object(obj_pattern) => {
                        assert_eq!(obj_pattern.fields.len(), 2);
                    }
                    _ => panic!("Expected ObjectPattern"),
                }
            }
        }
    }
}
