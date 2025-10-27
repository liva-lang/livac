# Phase 7.1: Tuple Types & Literals - Design Document

**Version:** v0.11.0  
**Status:** 📋 Planned  
**Estimated Time:** 4 hours  
**Prerequisites:** Pattern matching infrastructure (v0.10.5) ✅

---

## 🎯 Goal

Add tuple types and literals to Liva, enabling:
- Multiple return values without defining structs
- Tuple pattern matching in switch expressions
- Type-safe heterogeneous fixed-size collections

---

## 📝 Overview

Tuples are fixed-size, ordered collections of potentially different types:

```liva
// Tuple literal
let point = (10, 20)

// Tuple type annotation
let coord: (int, int) = (5, 10)

// Function returning tuple
getUser(): (string, int) {
    return ("Alice", 30)
}

// Destructuring
let (name, age) = getUser()

// Pattern matching
let result = switch point {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}
```

---

## 🎨 Syntax Design

### Tuple Literals

```liva
// Basic tuples
let pair = (10, 20)
let triple = (1, "hello", true)

// Nested tuples
let nested = ((1, 2), (3, 4))

// Single element (requires trailing comma to disambiguate)
let single = (42,)  // Tuple with one element
let notTuple = (42)  // Just a grouped expression
```

### Tuple Types

```liva
// Type annotations
let point: (int, int) = (10, 20)
let user: (string, int, bool) = ("Alice", 30, true)

// Function signatures
getCoords(): (int, int) { ... }
processPoint(p: (int, int)) { ... }

// Nested tuple types
let matrix: ((int, int), (int, int)) = ((1, 2), (3, 4))
```

### Tuple Access

```liva
let point = (10, 20)

// Index access
let x = point.0  // 10
let y = point.1  // 20

// Destructuring (already implemented)
let (x, y) = point
```

---

## 🏗️ Implementation Plan

### Phase 1: AST & Type System (1.5 hours)

**AST Changes:**
```rust
// Add tuple literal expression
pub enum Expr {
    // ... existing variants
    Tuple(Vec<Expr>),  // NEW: (expr1, expr2, ...)
}

// Add tuple type
pub enum TypeRef {
    // ... existing variants
    Tuple(Vec<TypeRef>),  // NEW: (Type1, Type2, ...)
}
```

**Type System:**
- Add `Type::Tuple(Vec<Type>)` to semantic analyzer
- Implement tuple type checking
- Handle tuple type inference from literals

### Phase 2: Parser (1 hour)

**Lexer:** No new tokens needed (`(`, `)`, `,` already exist)

**Parser:**
```rust
fn parse_primary(&mut self) -> Result<Expr> {
    match self.peek() {
        Some(Token::LParen) => {
            self.advance(); // consume (
            
            // Empty tuple: ()
            if self.match_token(&Token::RParen) {
                return Ok(Expr::Tuple(vec![]));
            }
            
            let first = self.parse_expression()?;
            
            // Check for comma (tuple) or closing paren (grouped expr)
            if self.match_token(&Token::Comma) {
                // It's a tuple
                let mut elements = vec![first];
                
                // Parse remaining elements
                if !self.check(&Token::RParen) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        // Allow trailing comma
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
                return Ok(first);
            }
        }
        // ... other cases
    }
}

fn parse_type(&mut self) -> Result<TypeRef> {
    match self.peek() {
        Some(Token::LParen) => {
            self.advance();
            
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
                // Just a grouped type (or error)
                self.expect(Token::RParen)?;
                return Ok(first);
            }
        }
        // ... other cases
    }
}
```

**Key Decision:** Use comma to distinguish tuples from grouped expressions:
- `(42)` → grouped expression
- `(42,)` → single-element tuple
- `(1, 2)` → two-element tuple

### Phase 3: Semantic Analysis (1 hour)

**Type Checking:**
```rust
impl SemanticAnalyzer {
    fn validate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Tuple(elements) => {
                // Validate each element
                for elem in elements {
                    self.validate_expr(elem)?;
                }
                Ok(())
            }
            // ... other cases
        }
    }
    
    fn infer_type(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Tuple(elements) => {
                let mut types = Vec::new();
                for elem in elements {
                    types.push(self.infer_type(elem)?);
                }
                Ok(Type::Tuple(types))
            }
            // ... other cases
        }
    }
}
```

**Tuple Access:**
```rust
// Member access on tuples: tuple.0, tuple.1, etc.
fn validate_member_access(&mut self, object: &Expr, property: &str) -> Result<()> {
    let obj_type = self.infer_type(object)?;
    
    if let Type::Tuple(types) = obj_type {
        // Check if property is a valid tuple index
        if let Ok(index) = property.parse::<usize>() {
            if index < types.len() {
                return Ok(());
            } else {
                return Err(format!("Tuple index {} out of bounds (size: {})", 
                                 index, types.len()).into());
            }
        }
    }
    
    // ... existing member access logic
}
```

### Phase 4: Code Generation (0.5 hours)

**Generate Rust Tuples:**
```rust
impl CodeGenerator {
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
                // Single-element tuple needs trailing comma in Rust
                if elements.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
                Ok(())
            }
            // ... other cases
        }
    }
    
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
                // Single-element tuple type in Rust
                if types.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
                Ok(())
            }
            // ... other cases
        }
    }
}
```

**Member Access:**
```rust
// tuple.0 → tuple.0 (Rust has same syntax)
Expr::Member { object, property } => {
    let obj_type = self.infer_type(object)?;
    
    if let Type::Tuple(_) = obj_type {
        // Tuple field access
        self.generate_expr(object)?;
        self.output.push('.');
        self.output.push_str(property);
    } else {
        // Regular struct field access
        // ... existing logic
    }
}
```

---

## 🧪 Testing Strategy

### Parser Tests

```liva
// test_tuple_literals.liva
main() {
    // Basic tuples
    let pair = (10, 20)
    let triple = (1, "hello", true)
    
    // Single element
    let single = (42,)
    
    // Nested
    let nested = ((1, 2), (3, 4))
    
    // Empty tuple
    let empty = ()
}
```

### Type Checking Tests

```liva
// test_tuple_types.liva
main() {
    // Type annotations
    let point: (int, int) = (10, 20)
    let user: (string, int) = ("Alice", 30)
    
    // Type mismatch should fail
    // let bad: (int, int) = (10, "20")  // Error!
}
```

### Tuple Access Tests

```liva
// test_tuple_access.liva
main() {
    let point = (10, 20)
    
    let x = point.0
    let y = point.1
    
    console.log($"Point: ({x}, {y})")
}
```

### Function Return Tests

```liva
// test_tuple_functions.liva
getCoords(): (int, int) {
    return (10, 20)
}

main() {
    let (x, y) = getCoords()
    console.log($"Coords: {x}, {y}")
}
```

### Pattern Matching Tests

```liva
// test_tuple_patterns.liva
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

---

## 📊 Success Criteria

- [x] AST nodes: `Expr::Tuple` and `TypeRef::Tuple`
- [x] Parser: Tuple literals and types
- [x] Semantic: Tuple type checking and inference
- [x] Codegen: Rust tuple generation
- [x] Tuple access: `tuple.0`, `tuple.1`
- [x] Function returns: `(): (T1, T2)`
- [x] Pattern matching: Tuple patterns in switch
- [x] Tests: 5+ comprehensive tests
- [x] Documentation: Tuple guide

---

## 🚀 Impact

**Enables:**
- ✅ Multiple return values without boilerplate structs
- ✅ Tuple pattern matching in switch
- ✅ More expressive type system
- ✅ Better interop with Rust (tuples map directly)

**Unblocks:**
- Pattern::Tuple in switch expressions (v0.10.5 AST ready)
- More complex pattern matching scenarios
- Cleaner error handling patterns

**Example Before/After:**

```liva
// Before: Need a struct
Point {
    x: int
    y: int
}

getCoords(): Point {
    return Point(10, 20)
}

let p = getCoords()
let x = p.x
let y = p.y

// After: Use tuple
getCoords(): (int, int) {
    return (10, 20)
}

let (x, y) = getCoords()
```

---

## 🔗 Related Features

**Depends On:**
- ✅ Pattern matching infrastructure (v0.10.5)
- ✅ Destructuring in let bindings (v0.10.2)

**Enables:**
- Switch pattern destructuring (Pattern::Tuple usage)
- Advanced error handling: `(): (T, Error?)`
- Cleaner APIs and return types

---

## 📚 Documentation Plan

Update:
- `docs/language-reference/types.md` - Add tuple types section
- `docs/language-reference/pattern-matching.md` - Enable tuple pattern examples
- `docs/language-reference/functions.md` - Add tuple return examples

New:
- `docs/guides/tuples.md` - Comprehensive tuple guide

---

## ⏱️ Time Breakdown

| Task | Estimated | Details |
|------|-----------|---------|
| AST & Type System | 1.5h | Add Expr::Tuple, TypeRef::Tuple, Type::Tuple |
| Parser | 1h | Parse tuple literals and types |
| Semantic Analysis | 1h | Type checking, inference, validation |
| Code Generation | 0.5h | Generate Rust tuples |
| **Total** | **4 hours** | |

---

## 🎯 Next Steps After Completion

1. Document tuple patterns in pattern-matching.md
2. Add tuple examples to all relevant docs
3. Consider tuple unpacking in for loops: `for (x, y) in pairs { ... }`
4. Consider union types: `int | string`
