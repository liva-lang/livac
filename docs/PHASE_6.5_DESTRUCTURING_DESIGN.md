# Phase 6.5: Destructuring Syntax - Design Document

**Version:** v0.10.2  
**Status:** 🚧 In Progress  
**Estimated Time:** 3 hours  
**Started:** 2025-01-26

---

## 🎯 Goal

Add destructuring syntax for objects and arrays to improve code ergonomics and reduce boilerplate.

---

## 📝 Overview

Destructuring allows extracting multiple values from objects/arrays in a single statement:

```liva
// Object destructuring
User {
    name: string
    age: int
    email: string
}

let user = User("Alice", 30, "alice@example.com")
let {name, age} = user
print($"Name: {name}, Age: {age}")

// Array destructuring
let numbers = [1, 2, 3, 4, 5]
let [first, second, ...rest] = numbers
print($"First: {first}, Second: {second}, Rest: {rest}")
```

---

## 🎨 Syntax Design

### Object Destructuring

**Basic:**
```liva
let {field1, field2} = object
```

**With rename:**
```liva
let {name: userName, age: userAge} = user
```

**With defaults (future):**
```liva
let {name, age = 18} = user
```

**Nested (future):**
```liva
let {address: {city, country}} = user
```

### Array Destructuring

**Basic:**
```liva
let [a, b, c] = array
```

**Skip elements:**
```liva
let [first, , third] = array
```

**Rest pattern:**
```liva
let [head, ...tail] = array
```

**Nested (future):**
```liva
let [[a, b], [c, d]] = matrix
```

---

## 🏗️ Implementation Plan

### Phase 1: AST & Parser (1 hour)

**AST Nodes:**
```rust
// New binding patterns
pub enum BindingPattern {
    Identifier(String),                          // x
    ObjectPattern(ObjectPattern),                // {x, y}
    ArrayPattern(ArrayPattern),                  // [x, y]
}

pub struct ObjectPattern {
    pub fields: Vec<ObjectPatternField>,
}

pub struct ObjectPatternField {
    pub key: String,                             // Original field name
    pub binding: String,                         // Variable name (may differ)
}

pub struct ArrayPattern {
    pub elements: Vec<Option<BindingPattern>>,  // None = skip element
    pub rest: Option<String>,                    // ...rest binding
}
```

**Parser Changes:**
```rust
// In parse_let_statement()
fn parse_binding_pattern(&mut self) -> Result<BindingPattern> {
    match self.current_token {
        Token::LBrace => self.parse_object_pattern(),
        Token::LBracket => self.parse_array_pattern(),
        Token::Identifier(_) => Ok(BindingPattern::Identifier(self.parse_identifier())),
        _ => Err("Expected identifier or destructuring pattern")
    }
}

fn parse_object_pattern(&mut self) -> Result<ObjectPattern> {
    // { name, age: userAge, email }
    self.expect(Token::LBrace)?;
    
    let mut fields = Vec::new();
    while self.current_token != Token::RBrace {
        let key = self.parse_identifier()?;
        let binding = if self.current_token == Token::Colon {
            self.next_token();
            self.parse_identifier()?
        } else {
            key.clone()
        };
        fields.push(ObjectPatternField { key, binding });
        
        if self.current_token == Token::Comma {
            self.next_token();
        }
    }
    self.expect(Token::RBrace)?;
    
    Ok(ObjectPattern { fields })
}

fn parse_array_pattern(&mut self) -> Result<ArrayPattern> {
    // [first, , third, ...rest]
    self.expect(Token::LBracket)?;
    
    let mut elements = Vec::new();
    let mut rest = None;
    
    while self.current_token != Token::RBracket {
        if self.current_token == Token::Comma {
            elements.push(None);  // Skip element
            self.next_token();
        } else if self.current_token == Token::DotDotDot {
            self.next_token();
            rest = Some(self.parse_identifier()?);
            break;
        } else {
            let pattern = self.parse_binding_pattern()?;
            elements.push(Some(pattern));
            if self.current_token == Token::Comma {
                self.next_token();
            }
        }
    }
    self.expect(Token::RBracket)?;
    
    Ok(ArrayPattern { elements, rest })
}
```

### Phase 2: Semantic Analysis (0.5 hours)

**Validation:**
1. Verify object pattern fields exist on the type
2. Verify array has enough elements
3. Detect duplicate bindings
4. Type checking for each binding

```rust
fn validate_object_pattern(&mut self, pattern: &ObjectPattern, object_type: &Type) -> Result<()> {
    // Check each field exists in the class
    for field in &pattern.fields {
        if !self.class_has_field(object_type, &field.key) {
            return Err(format!("Field '{}' does not exist on type '{}'", 
                field.key, object_type));
        }
        
        // Register binding in scope
        self.register_variable(&field.binding, field_type);
    }
    Ok(())
}

fn validate_array_pattern(&mut self, pattern: &ArrayPattern, array_type: &Type) -> Result<()> {
    // Extract element type from array
    let element_type = self.get_array_element_type(array_type)?;
    
    // Register each binding
    for element in &pattern.elements {
        if let Some(binding) = element {
            self.register_binding(binding, &element_type);
        }
    }
    
    // Register rest binding as array
    if let Some(rest) = &pattern.rest {
        self.register_variable(rest, array_type);
    }
    
    Ok(())
}
```

### Phase 3: Code Generation (1 hour)

**Object Destructuring:**
```rust
// Liva:
let {name, age: userAge} = user

// Generated Rust:
let name = user.name.clone();
let user_age = user.age;
```

**Array Destructuring:**
```rust
// Liva:
let [first, second, ...rest] = numbers

// Generated Rust:
let first = numbers[0];
let second = numbers[1];
let rest = numbers[2..].to_vec();
```

**Implementation:**
```rust
fn generate_destructuring(&mut self, pattern: &BindingPattern, init_expr: &Expr) -> Result<()> {
    match pattern {
        BindingPattern::ObjectPattern(obj_pattern) => {
            // Generate temporary for init expression
            let temp_var = self.generate_temp_var();
            write!(self.output, "let {} = ", temp_var)?;
            self.generate_expr(init_expr)?;
            writeln!(self.output, ";")?;
            
            // Extract each field
            for field in &obj_pattern.fields {
                writeln!(self.output, "let {} = {}.{}.clone();", 
                    self.to_snake_case(&field.binding),
                    temp_var,
                    self.to_snake_case(&field.key))?;
            }
        }
        
        BindingPattern::ArrayPattern(arr_pattern) => {
            // Generate temporary for array
            let temp_var = self.generate_temp_var();
            write!(self.output, "let {} = ", temp_var)?;
            self.generate_expr(init_expr)?;
            writeln!(self.output, ";")?;
            
            // Extract each element
            for (i, element) in arr_pattern.elements.iter().enumerate() {
                if let Some(binding) = element {
                    writeln!(self.output, "let {} = {}[{}];", 
                        self.to_snake_case(&binding.to_string()),
                        temp_var,
                        i)?;
                }
            }
            
            // Handle rest pattern
            if let Some(rest) = &arr_pattern.rest {
                let start_index = arr_pattern.elements.len();
                writeln!(self.output, "let {} = {}[{}..].to_vec();",
                    self.to_snake_case(rest),
                    temp_var,
                    start_index)?;
            }
        }
        
        BindingPattern::Identifier(name) => {
            // Normal binding (no destructuring)
            write!(self.output, "let {} = ", self.to_snake_case(name))?;
            self.generate_expr(init_expr)?;
            writeln!(self.output, ";")?;
        }
    }
    Ok(())
}
```

### Phase 4: Function Parameters (0.5 hours)

**Syntax:**
```liva
// Object destructuring in parameters
greet({name, age}: User) {
    print($"Hello {name}, you are {age}")
}

// Array destructuring in parameters
sum([a, b, c]: [int]) {
    return a + b + c
}
```

**Implementation:**
```rust
fn parse_function_parameter(&mut self) -> Result<Parameter> {
    let pattern = self.parse_binding_pattern()?;
    self.expect(Token::Colon)?;
    let type_ref = self.parse_type()?;
    
    Ok(Parameter { pattern, type_ref })
}
```

---

## 📊 Test Plan

### Test Files

**1. test_object_destructuring.liva**
```liva
User {
    name: string
    age: int
    email: string
}

main() {
    let user = User("Alice", 30, "alice@example.com")
    
    // Basic destructuring
    let {name, age} = user
    print($"Name: {name}, Age: {age}")
    
    // With rename
    let {name: userName, email: userEmail} = user
    print($"User: {userName} ({userEmail})")
}
```

**2. test_array_destructuring.liva**
```liva
main() {
    let numbers = [1, 2, 3, 4, 5]
    
    // Basic destructuring
    let [first, second] = numbers
    print($"First: {first}, Second: {second}")
    
    // Skip elements
    let [a, , c] = numbers
    print($"First: {a}, Third: {c}")
    
    // Rest pattern
    let [head, ...tail] = numbers
    print($"Head: {head}, Tail: {tail}")
}
```

**3. test_destructuring_parameters.liva**
```liva
Point {
    x: float
    y: float
}

distance({x: x1, y: y1}: Point, {x: x2, y: y2}: Point): float {
    let dx = x2 - x1
    let dy = y2 - y1
    return Math.sqrt(dx * dx + dy * dy)
}

main() {
    let p1 = Point(0.0, 0.0)
    let p2 = Point(3.0, 4.0)
    let dist = distance(p1, p2)
    print($"Distance: {dist}")  // 5.0
}
```

**4. test_destructuring_nested.liva** (future)
```liva
Address {
    street: string
    city: string
    country: string
}

User {
    name: string
    address: Address
}

main() {
    let user = User("Alice", Address("Main St", "NYC", "USA"))
    
    // Nested destructuring
    let {name, address: {city, country}} = user
    print($"User: {name} from {city}, {country}")
}
```

---

## 🎯 Success Criteria

- ✅ Object destructuring works in let statements
- ✅ Array destructuring works in let statements
- ✅ Rest pattern (`...rest`) works for arrays
- ✅ Destructuring works in function parameters
- ✅ Skip elements works (`[a, , c]`)
- ✅ Rename syntax works (`{name: userName}`)
- ✅ All tests pass
- ✅ Documentation complete

---

## 📚 Documentation

### User Guide

**Location:** `docs/language-reference/destructuring.md`

**Sections:**
1. Introduction
2. Object Destructuring
3. Array Destructuring
4. Function Parameters
5. Best Practices
6. Common Patterns
7. Limitations

### Examples

**Location:** `examples/destructuring/`

Files:
- `example_object.liva` - Object destructuring examples
- `example_array.liva` - Array destructuring examples
- `example_functions.liva` - Function parameter destructuring
- `example_rest.liva` - Rest patterns

---

## 🚀 Future Enhancements (v0.11.x)

1. **Default values:** `let {name, age = 18} = user`
2. **Nested destructuring:** `let {address: {city}} = user`
3. **Tuple destructuring:** `let (x, y) = point`
4. **In for loops:** `for {name, age} in users { ... }`
5. **In switch patterns:** `switch value { {x, y} => ... }`

---

## 📝 Notes

- Keep implementation simple for v0.10.2
- Focus on most common use cases
- Advanced features can wait for v0.11.x
- Prioritize clear error messages
- Generate efficient Rust code (avoid unnecessary clones)

---

**Status:** Ready to implement  
**Next Step:** Update AST with BindingPattern types
