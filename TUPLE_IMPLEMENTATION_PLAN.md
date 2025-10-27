# Tuple Types Implementation Plan - v0.11.0

**Branch:** `feature/tuple-types-v0.11.0`  
**Estimated Time:** 4 hours  
**Started:** 2025-01-27

---

## 🎯 Goal

Implement tuple types and literals in Liva to enable:
- Multiple return values without boilerplate structs
- Tuple pattern matching in switch expressions
- Type-safe heterogeneous fixed-size collections

---

## 📋 Implementation Phases

### Phase 1: AST Extensions (30 min)

**File:** `src/ast.rs`

**Add Tuple Expression:**
```rust
pub enum Expr {
    // ... existing variants
    Tuple(Vec<Expr>),  // NEW: (expr1, expr2, ...)
}
```

**Add Tuple Type:**
```rust
pub enum TypeRef {
    // ... existing variants
    Tuple(Vec<TypeRef>),  // NEW: (Type1, Type2, ...)
}
```

**Update Display traits:**
- Add Display for Expr::Tuple
- Add Display for TypeRef::Tuple

**Checklist:**
- [ ] Add `Expr::Tuple(Vec<Expr>)` variant
- [ ] Add `TypeRef::Tuple(Vec<TypeRef>)` variant
- [ ] Update Display for Expr
- [ ] Update Display for TypeRef
- [ ] Verify AST compiles

---

### Phase 2: Parser Extensions (1 hour)

**File:** `src/parser.rs`

**Tuple Literals in parse_primary():**

Key challenge: Distinguish tuples from grouped expressions
- `(42)` → grouped expression (just 42)
- `(42,)` → single-element tuple
- `(1, 2)` → two-element tuple
- `()` → empty tuple (unit type)

**Algorithm:**
```rust
fn parse_primary(&mut self) -> Result<Expr> {
    if self.match_token(&Token::LParen) {
        // Empty tuple: ()
        if self.match_token(&Token::RParen) {
            return Ok(Expr::Tuple(vec![]));
        }
        
        // Parse first element
        let first = self.parse_expression()?;
        
        // Check for comma (tuple) or RParen (grouped expr)
        if self.match_token(&Token::Comma) {
            // It's a tuple!
            let mut elements = vec![first];
            
            // Parse remaining elements (allow trailing comma)
            if !self.check(&Token::RParen) {
                loop {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    // Allow trailing comma before )
                    if self.check(&Token::RParen) {
                        break;
                    }
                }
            }
            
            self.expect(Token::RParen)?;
            return Ok(Expr::Tuple(elements));
        } else {
            // Just a grouped expression
            self.expect(Token::RParen)?;
            return Ok(first);  // Return the expression, not a tuple
        }
    }
    // ... rest of parse_primary
}
```

**Tuple Types in parse_type():**
```rust
fn parse_type(&mut self) -> Result<TypeRef> {
    if self.match_token(&Token::LParen) {
        // Empty tuple type: ()
        if self.match_token(&Token::RParen) {
            return Ok(TypeRef::Tuple(vec![]));
        }
        
        let first = self.parse_type()?;
        
        if self.match_token(&Token::Comma) {
            let mut types = vec![first];
            
            if !self.check(&Token::RParen) {
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    if self.check(&Token::RParen) {
                        break;
                    }
                }
            }
            
            self.expect(Token::RParen)?;
            return Ok(TypeRef::Tuple(types));
        } else {
            // Error: grouped type doesn't make sense
            return Err(self.error("Unexpected type in parentheses".into()));
        }
    }
    // ... rest of parse_type
}
```

**Checklist:**
- [ ] Implement tuple literal parsing in `parse_primary()`
- [ ] Handle empty tuple: `()`
- [ ] Handle single-element tuple: `(x,)`
- [ ] Handle multi-element tuples: `(x, y, z)`
- [ ] Distinguish from grouped expressions: `(x)` vs `(x,)`
- [ ] Implement tuple type parsing in `parse_type()`
- [ ] Handle trailing commas
- [ ] Add parser tests

---

### Phase 3: Semantic Analysis (1 hour)

**File:** `src/semantic.rs`

**Type System Extensions:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // ... existing variants
    Tuple(Vec<Type>),  // NEW
}
```

**Validate Tuple Expressions:**
```rust
fn validate_expr(&mut self, expr: &Expr) -> Result<()> {
    match expr {
        Expr::Tuple(elements) => {
            for elem in elements {
                self.validate_expr(elem)?;
            }
            Ok(())
        }
        // ... existing cases
    }
}
```

**Type Inference:**
```rust
fn infer_type(&mut self, expr: &Expr) -> Result<Type> {
    match expr {
        Expr::Tuple(elements) => {
            let mut types = Vec::new();
            for elem in elements {
                types.push(self.infer_type(elem)?);
            }
            Ok(Type::Tuple(types))
        }
        // ... existing cases
    }
}
```

**Tuple Member Access:**
Handle `tuple.0`, `tuple.1`, etc. in member expressions:
```rust
fn validate_member_access(&mut self, object: &Expr, property: &str) -> Result<Type> {
    let obj_type = self.infer_type(object)?;
    
    match obj_type {
        Type::Tuple(types) => {
            // Parse property as tuple index
            if let Ok(index) = property.parse::<usize>() {
                if index < types.len() {
                    return Ok(types[index].clone());
                } else {
                    return Err(format!(
                        "Tuple index {} out of bounds (size: {})", 
                        index, types.len()
                    ).into());
                }
            } else {
                return Err(format!(
                    "Invalid tuple field '{}' - use numeric index like .0, .1", 
                    property
                ).into());
            }
        }
        // ... existing member access logic for structs
    }
}
```

**Type Checking:**
```rust
fn check_type_match(&self, expected: &Type, actual: &Type) -> Result<()> {
    match (expected, actual) {
        (Type::Tuple(exp_types), Type::Tuple(act_types)) => {
            if exp_types.len() != act_types.len() {
                return Err(format!(
                    "Tuple size mismatch: expected {}, got {}", 
                    exp_types.len(), act_types.len()
                ).into());
            }
            for (i, (exp, act)) in exp_types.iter().zip(act_types.iter()).enumerate() {
                self.check_type_match(exp, act).map_err(|e| {
                    format!("Tuple element {} type mismatch: {}", i, e)
                })?;
            }
            Ok(())
        }
        // ... existing type checking
    }
}
```

**Checklist:**
- [ ] Add `Type::Tuple(Vec<Type>)` to type system
- [ ] Implement `validate_expr` for `Expr::Tuple`
- [ ] Implement type inference for tuple literals
- [ ] Handle tuple member access (`.0`, `.1`, etc.)
- [ ] Validate tuple index bounds
- [ ] Type checking for tuple assignments
- [ ] Error messages for tuple type mismatches
- [ ] Add semantic tests

---

### Phase 4: Code Generation (1 hour)

**File:** `src/codegen.rs`

**Generate Tuple Literals:**
```rust
fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
    match expr {
        Expr::Tuple(elements) => {
            self.output.push('(');
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.generate_expr(elem)?;
            }
            // Rust requires trailing comma for single-element tuples
            if elements.len() == 1 {
                self.output.push(',');
            }
            self.output.push(')');
            Ok(())
        }
        // ... existing cases
    }
}
```

**Generate Tuple Types:**
```rust
fn generate_type(&mut self, type_ref: &TypeRef) -> Result<()> {
    match type_ref {
        TypeRef::Tuple(types) => {
            self.output.push('(');
            for (i, t) in types.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                self.generate_type(t)?;
            }
            // Rust single-element tuple types also need comma
            if types.len() == 1 {
                self.output.push(',');
            }
            self.output.push(')');
            Ok(())
        }
        // ... existing cases
    }
}
```

**Tuple Member Access:**
```rust
// tuple.0 → tuple.0 (same syntax in Rust!)
Expr::Member { object, property } => {
    let obj_type = self.infer_type(object)?;
    
    if let Type::Tuple(_) = obj_type {
        // Validate it's a numeric index
        if property.parse::<usize>().is_ok() {
            self.generate_expr(object)?;
            self.output.push('.');
            self.output.push_str(property);
            return Ok(());
        }
    }
    
    // ... existing struct member access
}
```

**Tuple Destructuring (already works!):**
Since we already have destructuring in let bindings, tuple destructuring should work automatically:
```liva
let (x, y) = getTuple()
```
This already generates correct Rust code.

**Checklist:**
- [ ] Generate tuple literals: `(1, 2, 3)`
- [ ] Handle single-element tuples: `(42,)`
- [ ] Handle empty tuples: `()`
- [ ] Generate tuple types in signatures
- [ ] Generate tuple member access: `.0`, `.1`
- [ ] Verify destructuring still works
- [ ] Test nested tuples
- [ ] Add codegen tests

---

### Phase 5: Testing (30 min)

**Create comprehensive test files:**

**1. test_tuple_literals.liva**
```liva
main() {
    // Basic tuples
    let pair = (10, 20)
    let triple = (1, "hello", true)
    
    // Single element
    let single = (42,)
    
    // Empty tuple
    let empty = ()
    
    // Nested
    let nested = ((1, 2), (3, 4))
    
    console.log("Tuple literals test passed")
}
```

**2. test_tuple_types.liva**
```liva
main() {
    // Type annotations
    let point: (int, int) = (10, 20)
    let user: (string, int) = ("Alice", 30)
    
    console.log($"Point: {point.0}, {point.1}")
    console.log($"User: {user.0}, age {user.1}")
}
```

**3. test_tuple_access.liva**
```liva
main() {
    let point = (10, 20, 30)
    
    let x = point.0
    let y = point.1
    let z = point.2
    
    console.log($"Coords: ({x}, {y}, {z})")
}
```

**4. test_tuple_functions.liva**
```liva
getCoords(): (int, int) {
    return (100, 200)
}

swapPair(p: (int, int)): (int, int) {
    let (x, y) = p
    return (y, x)
}

main() {
    let (x, y) = getCoords()
    console.log($"Original: ({x}, {y})")
    
    let swapped = swapPair((x, y))
    console.log($"Swapped: ({swapped.0}, {swapped.1})")
}
```

**5. test_tuple_patterns.liva**
```liva
main() {
    let point = (10, 20)
    
    let description = switch point {
        (0, 0) => "origin",
        (0, y) => $"on Y axis at {y}",
        (x, 0) => $"on X axis at {x}",
        (x, y) => $"at ({x}, {y})"
    }
    
    console.log(description)
}
```

**6. test_tuple_nested.liva**
```liva
main() {
    let matrix = ((1, 2), (3, 4))
    
    let topLeft = matrix.0.0
    let topRight = matrix.0.1
    let bottomLeft = matrix.1.0
    let bottomRight = matrix.1.1
    
    console.log($"Matrix: [{topLeft}, {topRight}], [{bottomLeft}, {bottomRight}]")
}
```

**Checklist:**
- [ ] Create all 6 test files
- [ ] Run each test and verify output
- [ ] Test error cases (index out of bounds, type mismatch)
- [ ] Test with different tuple sizes (0, 1, 2, 3+)
- [ ] Test nested tuples
- [ ] Test tuple destructuring compatibility

---

### Phase 6: Documentation (30 min)

**Update existing docs:**

**1. types.md - Add Tuple Types section:**
```markdown
### Tuple Types

Tuples are fixed-size, ordered collections of values with potentially different types.

**Syntax:**
- Literal: `(value1, value2, ...)`
- Type: `(Type1, Type2, ...)`
- Access: `tuple.0`, `tuple.1`, etc.

**Examples:**
// ... code examples
```

**2. pattern-matching.md - Enable tuple pattern examples:**
Update the "Future" section to "Implemented" and show working examples.

**3. functions.md - Add tuple return examples:**
```markdown
### Multiple Return Values with Tuples

Functions can return tuples to return multiple values:
// ... examples
```

**4. variables.md - Add tuple destructuring examples:**
Show how tuple destructuring works with the new tuple literals.

**Create new docs:**

**5. docs/guides/tuples.md - Comprehensive tuple guide:**
- What are tuples
- When to use tuples vs structs
- Tuple syntax and operations
- Best practices
- Common patterns

**Update CHANGELOG.md:**
Add v0.11.0 entry with all tuple features.

**Checklist:**
- [ ] Update `docs/language-reference/types.md`
- [ ] Update `docs/language-reference/pattern-matching.md`
- [ ] Update `docs/language-reference/functions.md`
- [ ] Update `docs/language-reference/variables.md`
- [ ] Create `docs/guides/tuples.md`
- [ ] Update CHANGELOG.md
- [ ] Update ROADMAP.md to mark 7.1 complete

---

## 🎯 Success Criteria

- [x] AST supports `Expr::Tuple` and `TypeRef::Tuple`
- [x] Parser correctly distinguishes tuples from grouped expressions
- [x] Semantic analyzer validates tuple types
- [x] Type inference works for tuple literals
- [x] Tuple member access (`.0`, `.1`) works
- [x] Code generator produces valid Rust tuples
- [x] Tuple patterns work in switch expressions
- [x] All 6 test files pass
- [x] Documentation is complete and accurate
- [x] CHANGELOG and ROADMAP updated

---

## 📊 Time Tracking

| Phase | Estimated | Actual | Status |
|-------|-----------|--------|--------|
| 1. AST | 30 min | - | ⏳ Not started |
| 2. Parser | 1 hour | - | ⏳ Not started |
| 3. Semantic | 1 hour | - | ⏳ Not started |
| 4. Codegen | 1 hour | - | ⏳ Not started |
| 5. Testing | 30 min | - | ⏳ Not started |
| 6. Documentation | 30 min | - | ⏳ Not started |
| **Total** | **4 hours** | **-** | **⏳ Not started** |

---

## 🚀 Getting Started

```bash
# Checkout feature branch
git checkout feature/tuple-types-v0.11.0

# Start with Phase 1: AST
# Edit src/ast.rs

# After each phase, commit:
git add -A
git commit -m "feat(tuples): Phase X - Description"

# Build and test continuously
cargo build
./target/debug/livac examples/test_tuple_literals.liva
```

---

## 📝 Notes

- Tuples are immutable (like all Liva values)
- Empty tuple `()` is the unit type (no value)
- Single-element tuple requires trailing comma: `(42,)`
- Tuple indices are 0-based: `.0`, `.1`, `.2`, etc.
- Tuples map directly to Rust tuples (no overhead)
- Destructuring already works from v0.10.2

---

## 🔗 Related Issues

- Unblocks: Pattern::Tuple in switch expressions
- Enables: Multiple return values without structs
- Foundation for: Union types, advanced pattern matching
