# Phase 6.5.1: Destructuring in Function Parameters

**Status:** In Progress  
**Version:** v0.10.3  
**Estimated Time:** 2.5 hours  
**Started:** 2025-10-26

---

## 🎯 Objective

Extend destructuring patterns to function parameters, enabling more ergonomic code by eliminating the need for explicit `let` bindings inside function bodies.

### Current State (v0.10.2)
```liva
users.forEach(user => {
    let {id, name, username} = user  // Extra line needed
    console.log($"User {id}: {name} (@{username})")
})
```

### Target State (v0.10.3)
```liva
users.forEach({id, name, username} => {  // Direct destructuring
    console.log($"User {id}: {name} (@{username})")
})
```

---

## 📋 Requirements

### Must Have (Phase 1)
- [x] Object destructuring in function parameters: `{id, name}`
- [x] Array destructuring in function parameters: `[x, y]`
- [x] Object field renaming: `{name: userName}`
- [x] Array rest patterns: `[head, ...tail]`
- [x] Works with lambdas/arrow functions
- [x] Works with named functions
- [x] Type annotations: `{id, name}: User`

### Nice to Have (Future)
- [ ] Nested destructuring: `{address: {city}}`
- [ ] Default values: `{name = "Unknown"}`
- [ ] Mixed parameters: `(regular, {destructured})`
- [ ] Destructuring in method parameters

---

## 🎨 Syntax Design

### Object Destructuring

```liva
// Simple field extraction
users.map({id, name} => $"User {id}: {name}")

// Field renaming
users.map({name: userName, email: userEmail} => {
    console.log($"{userName} <{userEmail}>")
})

// With type annotation
processUser({id, name}: User => {
    console.log($"Processing user {id}")
})

// Rest pattern (future)
users.map({id, ...rest} => {
    console.log($"User {id}, data: {rest}")
})
```

### Array Destructuring

```liva
// Simple element extraction
points.map([x, y] => $"Point({x}, {y})")

// Skip elements
transformPairs([first, , third] => first + third)

// Rest pattern
processArray([head, ...tail] => {
    console.log($"First: {head}, Rest: {tail}")
})

// With type annotation
sumPair([a, b]: [int, int] => a + b)
```

### Named Functions

```liva
// Object destructuring
printUser({id, name, email}: User) {
    console.log($"User {id}: {name} <{email}>")
}

// Array destructuring
distance([x1, y1], [x2, y2]: [float, float]) => {
    let dx = x2 - x1
    let dy = y2 - y1
    return sqrt(dx * dx + dy * dy)
}
```

---

## 🏗️ Implementation Plan

### 1. AST Changes (30 min)

**Current Structure:**
```rust
pub struct FunctionParam {
    pub name: String,           // Simple identifier
    pub type_ref: Option<TypeRef>,
    pub default: Option<Expr>,
    pub span: Option<Span>,
}
```

**New Structure:**
```rust
pub struct FunctionParam {
    pub pattern: BindingPattern,  // ← Changed from `name: String`
    pub type_ref: Option<TypeRef>,
    pub default: Option<Expr>,
    pub span: Option<Span>,
}

impl FunctionParam {
    /// Helper for backward compatibility
    pub fn name(&self) -> Option<&str> {
        match &self.pattern {
            BindingPattern::Identifier(name) => Some(name),
            _ => None,
        }
    }
    
    /// Check if parameter uses destructuring
    pub fn is_destructuring(&self) -> bool {
        !self.pattern.is_simple()
    }
}
```

**Migration:**
- All code using `param.name` → `param.name().unwrap()` or handle pattern
- Update ~20-30 locations in codegen.rs, semantic.rs, lowering.rs

---

### 2. Parser Changes (45 min)

**Update `parse_function_params()`:**

```rust
fn parse_function_params(&mut self) -> Result<Vec<FunctionParam>> {
    let mut params = Vec::new();
    
    self.expect(Token::LParen)?;
    
    while !self.check(Token::RParen) {
        let start_span = self.current_span();
        
        // Parse pattern (identifier, object, or array)
        let pattern = self.parse_binding_pattern()?;
        
        // Optional type annotation
        let type_ref = if self.match_token(Token::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        
        // Optional default value (future)
        let default = if self.match_token(Token::Assign) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        
        params.push(FunctionParam {
            pattern,
            type_ref,
            default,
            span: Some(start_span),
        });
        
        if !self.check(Token::RParen) {
            self.expect(Token::Comma)?;
        }
    }
    
    self.expect(Token::RParen)?;
    Ok(params)
}
```

**Reuse Existing Logic:**
- `parse_binding_pattern()` already exists (from v0.10.2)
- `parse_object_pattern()` already exists
- `parse_array_pattern()` already exists

---

### 3. Semantic Analysis (30 min)

**Update `validate_function()`:**

```rust
fn validate_function(&mut self, func: &FunctionDecl) -> Result<()> {
    self.enter_scope();
    
    // Declare all parameters in scope
    for param in &func.params {
        // Validate type annotation if present
        if let Some(type_ref) = &param.type_ref {
            self.validate_type_ref(type_ref, &HashSet::new())?;
        }
        
        // Declare variables from pattern
        self.declare_param_pattern(&param.pattern, param.type_ref.clone(), param.span)?;
    }
    
    // ... rest of function validation
}

fn declare_param_pattern(
    &mut self,
    pattern: &BindingPattern,
    param_type: Option<TypeRef>,
    span: Option<Span>
) -> Result<()> {
    match pattern {
        BindingPattern::Identifier(name) => {
            // Simple parameter
            if self.declare_symbol(name, param_type) {
                return Err(self.error(
                    "E0310",
                    &format!("Parameter '{}' already declared", name),
                    span
                ));
            }
        }
        BindingPattern::Object(obj_pattern) => {
            // Validate field existence if type is known
            if let Some(TypeRef::Simple(type_name)) = &param_type {
                if let Some(type_info) = self.types.get(type_name) {
                    for field in &obj_pattern.fields {
                        if !type_info.fields.contains_key(&field.key) {
                            return Err(self.error(
                                "E0311",
                                &format!("Field '{}' not found on type '{}'", field.key, type_name),
                                span
                            ));
                        }
                    }
                }
            }
            
            // Declare all bindings
            for field in &obj_pattern.fields {
                let field_type = self.infer_field_type(&param_type, &field.key);
                if self.declare_symbol(&field.binding, field_type) {
                    return Err(self.error(
                        "E0312",
                        &format!("Binding '{}' already declared", field.binding),
                        span
                    ));
                }
            }
        }
        BindingPattern::Array(arr_pattern) => {
            // Infer element type from array type
            let element_type = match &param_type {
                Some(TypeRef::Array(inner)) => Some((**inner).clone()),
                _ => None,
            };
            
            // Declare element bindings
            for element in &arr_pattern.elements {
                if let Some(name) = element {
                    if self.declare_symbol(name, element_type.clone()) {
                        return Err(self.error(
                            "E0312",
                            &format!("Binding '{}' already declared", name),
                            span
                        ));
                    }
                }
            }
            
            // Declare rest binding
            if let Some(rest) = &arr_pattern.rest {
                let rest_type = element_type.map(|t| TypeRef::Array(Box::new(t)));
                if self.declare_symbol(rest, rest_type) {
                    return Err(self.error(
                        "E0312",
                        &format!("Binding '{}' already declared", rest),
                        span
                    ));
                }
            }
        }
    }
    Ok(())
}
```

---

### 4. Code Generation (45 min)

**Strategy: Generate destructuring at function entry**

```rust
fn generate_function(&mut self, func: &FunctionDecl) -> Result<()> {
    // Generate function signature
    write!(self.output, "fn {}(", self.sanitize_name(&func.name))?;
    
    // Generate parameter list with temporary names for destructured params
    for (i, param) in func.params.iter().enumerate() {
        if i > 0 {
            self.output.push_str(", ");
        }
        
        if param.is_destructuring() {
            // Use temporary name for destructured parameters
            let temp_name = format!("_param_{}", i);
            write!(self.output, "{}", temp_name)?;
        } else {
            // Simple parameter
            write!(self.output, "{}", self.sanitize_name(param.name().unwrap()))?;
        }
        
        // Add type annotation
        if let Some(type_ref) = &param.type_ref {
            write!(self.output, ": {}", type_ref.to_rust_type())?;
        }
    }
    
    self.output.push_str(")");
    
    // Return type
    if let Some(ret_type) = &func.return_type {
        write!(self.output, " -> {}", ret_type.to_rust_type())?;
    }
    
    // Function body
    self.output.push_str(" {\n");
    self.indent_level += 1;
    
    // Generate destructuring code for destructured parameters
    for (i, param) in func.params.iter().enumerate() {
        if param.is_destructuring() {
            let temp_name = format!("_param_{}", i);
            self.write_indent();
            
            // Create a temporary identifier expression
            let temp_expr = Expr::Identifier(temp_name);
            
            // Generate destructuring using existing logic
            self.generate_destructuring_pattern(&param.pattern, &temp_expr)?;
        }
    }
    
    // Generate function body statements
    if let Some(body) = &func.body {
        for stmt in &body.stmts {
            self.generate_stmt(stmt)?;
        }
    }
    
    self.indent_level -= 1;
    self.write_indent();
    self.output.push_str("}\n\n");
    
    Ok(())
}
```

**Example Output:**

```liva
// Input:
users.forEach({id, name} => {
    console.log($"User {id}: {name}")
})

// Generated Rust:
users.iter().for_each(|_param_0| {
    let id = _param_0.id.clone();
    let name = _param_0.name.clone();
    println!("{}{}", "User ", id, ": ", name);
});
```

---

### 5. Testing (30 min)

**Parser Tests:**
```rust
#[test]
fn test_parse_object_destructuring_param() {
    let source = "myFunc({id, name}: User) { return id }";
    let ast = parse(source).unwrap();
    // Assert function has destructuring parameter
}

#[test]
fn test_parse_array_destructuring_param() {
    let source = "sum([a, b]: [int, int]) => a + b";
    let ast = parse(source).unwrap();
    // Assert lambda has array pattern
}

#[test]
fn test_parse_multiple_params_with_destructuring() {
    let source = "func(x: int, {y, z}, [a, b]) { ... }";
    let ast = parse(source).unwrap();
    // Assert mix of simple and destructured params
}
```

**Semantic Tests:**
```rust
#[test]
fn test_semantic_object_destructuring_param() {
    let source = r#"
        class User { id: int, name: string }
        processUser({id, name}: User) { print(id) }
    "#;
    assert!(analyze(source).is_ok());
}

#[test]
fn test_semantic_invalid_field_destructuring() {
    let source = r#"
        class User { id: int }
        processUser({id, invalid}: User) { }
    "#;
    assert!(analyze(source).is_err());
}
```

**Integration Test:**
```liva
// test_param_destructuring.liva
User {
    id: u32
    name: string
    email: string
}

printUser({id, name, email}: User) {
    console.log($"User {id}: {name} <{email}>")
}

main() {
    let users = [
        User { id: 1, name: "Alice", email: "alice@example.com" },
        User { id: 2, name: "Bob", email: "bob@example.com" }
    ]
    
    // Test with forEach
    users.forEach({id, name} => {
        console.log($"User {id}: {name}")
    })
    
    // Test with named function
    users.forEach(printUser)
    
    // Test with array destructuring
    let points = [[1, 2], [3, 4]]
    points.forEach([x, y] => {
        console.log($"Point({x}, {y})")
    })
}
```

---

## 📊 Complexity Analysis

### Parser Complexity: **Low** ⭐
- Reuse existing `parse_binding_pattern()` logic
- Just need to call it in `parse_function_params()`
- Minimal new code (~30 lines)

### Semantic Complexity: **Medium** ⭐⭐
- Need to validate patterns against parameter types
- Need to declare all bindings from patterns
- Field existence validation
- ~80 lines of new code

### Codegen Complexity: **Medium** ⭐⭐
- Generate temporary parameter names
- Insert destructuring at function entry
- Reuse existing `generate_destructuring_pattern()`
- ~60 lines of new code

### Migration Complexity: **Medium** ⭐⭐
- Change `FunctionParam.name: String` → `pattern: BindingPattern`
- Update ~25 call sites
- Most can use `.name().unwrap()` for backward compatibility

---

## ⚠️ Edge Cases & Considerations

### 1. Type Inference
```liva
// Parameter type must be inferrable or explicit
users.forEach({id, name} => ...)  // ✅ OK if `users` has known type

let items = [...]
items.forEach({x} => ...)  // ⚠️ May need type annotation
```

### 2. Shadowing
```liva
let id = 10
users.forEach({id} => {  // Shadows outer `id`
    console.log(id)  // Uses parameter, not outer variable
})
```

### 3. Mixed Parameters (Future)
```liva
// Not in Phase 1, but should be considered
func(x: int, {y, z}: Point) {  // Regular + destructured
    return x + y + z
}
```

### 4. Performance
- Destructuring adds ~1-2 extra assignments per field
- Negligible overhead (compiler may optimize)
- Same cost as explicit `let` destructuring

---

## 📚 Documentation Updates

### 1. Language Reference
- Update `docs/language-reference/functions.md`
- Add section on "Parameter Destructuring"
- Show examples with lambdas and named functions

### 2. Migration Guide
- Create `docs/MIGRATION_PARAM_DESTRUCTURING_v0.10.3.md`
- Document `FunctionParam` API changes
- Provide before/after examples

### 3. Examples
- Add `examples/param_destructuring_demo.liva`
- Show real-world use cases (forEach, map, reduce)
- Include HTTP response handling example

### 4. Changelog
- Add entry for v0.10.3
- List all new features
- Show practical examples

---

## 🎯 Success Criteria

- [ ] All parser tests pass (6+ new tests)
- [ ] All semantic tests pass (4+ new tests)
- [ ] All codegen tests pass (4+ new tests)
- [ ] Integration test runs successfully
- [ ] HTTP example works with destructured params
- [ ] Documentation complete (4 files updated)
- [ ] No regressions in existing tests
- [ ] Performance: no significant overhead vs explicit destructuring

---

## 🚀 Rollout Plan

1. **Implement AST changes** (commit 1)
2. **Update parser** (commit 2)
3. **Update semantic analyzer** (commit 3)
4. **Update codegen** (commit 4)
5. **Add tests** (commit 5)
6. **Update documentation** (commit 6)
7. **Merge to main** as v0.10.3
8. **Tag release**

---

## 📝 Notes

- This feature builds directly on v0.10.2 destructuring
- Reuses 80% of existing pattern matching logic
- High value-to-effort ratio
- Completes the destructuring feature set
- Aligns with modern language conventions (JS, TS, Rust, Python)

**Estimated Total Time:** 2.5 hours  
**Actual Time:** _TBD_

---

## 🔗 Related Documents

- [Phase 6.5 Destructuring Design](./PHASE_6.5_DESTRUCTURING_DESIGN.md)
- [AST Reference](./compiler-internals/ast.md)
- [Parser Internals](./compiler-internals/parser.md)
- [Migration Guide v0.10.2](./MIGRATION_DESTRUCTURING_v0.10.2.md)
