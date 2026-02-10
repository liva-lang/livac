# Phase 6.4: Enhanced Pattern Matching - Design Document

**Version**: v0.9.5  
**Started**: 2025-01-21  
**Status**: In Progress  
**Estimated Time**: 3 hours

---

## Overview

Enhance Liva's `switch` statement to support:
1. **Switch as expression** - Return values from switch
2. **Exhaustiveness checking** - Ensure all cases are covered
3. **Pattern guards** - Add conditions to patterns
4. **Range patterns** - Match numeric ranges (1..10)
5. **Array/tuple patterns** - Destructure arrays and tuples

---

## Current State

Liva currently has a **basic switch statement**:

```liva
switch value {
    case 1: print("one")
    case 2: print("two")
    default: print("other")
}
```

**Limitations:**
- Switch is a statement (doesn't return values)
- No exhaustiveness checking
- No pattern guards
- No range patterns
- No destructuring

---

## Proposed Syntax

### 1. Switch as Expression

**Return values from switch:**

```liva
let result = switch value {
    case 1: "one"
    case 2: "two"
    default: "other"
}
```

**Block expressions:**

```liva
let category = switch age {
    case 0..17: {
        print("Minor")
        "child"
    }
    case 18..64: "adult"
    case 65..: "senior"
}
```

### 2. Exhaustiveness Checking

**Compile-time validation that all cases are covered:**

```liva
// ✅ OK - all enum values covered
switch status {
    case "pending": handlePending()
    case "approved": handleApproved()
    case "rejected": handleRejected()
}

// ❌ ERROR - missing "rejected" case
switch status {
    case "pending": handlePending()
    case "approved": handleApproved()
}  // E6001: Non-exhaustive patterns
```

**For int/string/bool:**
- Require `default` case (infinite possible values)
- Or explicit exhaustive list for bool

```liva
// ✅ OK - bool exhaustive without default
switch flag {
    case true: doTrue()
    case false: doFalse()
}

// ❌ ERROR - int requires default
switch count {
    case 0: zero()
    case 1: one()
}  // E6001: Missing default case for int
```

### 3. Pattern Guards

**Add conditions to patterns with `if`:**

```liva
switch value {
    case x if x > 100: "large"
    case x if x > 50: "medium"
    case x if x > 0: "small"
    default: "invalid"
}
```

**Multiple conditions:**

```liva
switch point {
    case (x, y) if x > 0 && y > 0: "quadrant 1"
    case (x, y) if x < 0 && y > 0: "quadrant 2"
    case (x, y) if x < 0 && y < 0: "quadrant 3"
    case (x, y) if x > 0 && y < 0: "quadrant 4"
    default: "on axis"
}
```

### 4. Range Patterns

**Match numeric ranges:**

```liva
switch grade {
    case 90..100: "A"
    case 80..89: "B"
    case 70..79: "C"
    case 60..69: "D"
    default: "F"
}
```

**Inclusive/exclusive ranges:**

```liva
case 1..10:    // 1 to 9 (exclusive end)
case 1..=10:   // 1 to 10 (inclusive end)
case 10..:     // 10 and above (open end)
case ..10:     // Below 10 (open start)
```

**Float ranges:**

```liva
switch temperature {
    case ..-10.0: "freezing"
    case -10.0..10.0: "cold"
    case 10.0..25.0: "mild"
    case 25.0..: "hot"
}
```

### 5. Array/Tuple Patterns

**Destructure arrays:**

```liva
switch arr {
    case []: "empty"
    case [x]: "single: " + x
    case [x, y]: "pair: " + x + ", " + y
    case [first, ...rest]: "first: " + first + ", rest: " + rest.length
}
```

**Destructure tuples:**

```liva
switch point {
    case (0, 0): "origin"
    case (x, 0): "on x-axis at " + x
    case (0, y): "on y-axis at " + y
    case (x, y): "point at (" + x + ", " + y + ")"
}
```

**Nested patterns:**

```liva
switch data {
    case ["user", name, age] if age >= 18: "Adult: " + name
    case ["user", name, age]: "Minor: " + name
    case ["admin", name]: "Admin: " + name
    default: "Unknown"
}
```

---

## AST Changes

### Current AST

```rust
pub struct SwitchStmt {
    pub expr: Box<Expr>,
    pub cases: Vec<SwitchCase>,
    pub default: Option<Vec<Stmt>>,
}

pub struct SwitchCase {
    pub value: Literal,  // Only literals!
    pub body: Vec<Stmt>,
}
```

### New AST

```rust
pub struct SwitchExpr {
    pub expr: Box<Expr>,
    pub arms: Vec<SwitchArm>,
    pub is_exhaustive: bool,  // Computed during semantic analysis
}

pub struct SwitchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,  // Pattern guard (if condition)
    pub body: SwitchBody,
}

pub enum SwitchBody {
    Expr(Box<Expr>),       // Single expression
    Block(Vec<Stmt>),      // Block of statements
}

pub enum Pattern {
    Literal(Literal),                          // case 42
    Range(RangePattern),                       // case 1..10
    Wildcard,                                  // default or _
    Binding(String),                           // case x
    Tuple(Vec<Pattern>),                       // case (x, y)
    Array(Vec<Pattern>, Option<String>),       // case [x, y] or [first, ...rest]
}

pub struct RangePattern {
    pub start: Option<Box<Expr>>,  // None for ..10
    pub end: Option<Box<Expr>>,    // None for 10..
    pub inclusive: bool,            // true for ..=
}
```

---

## Parser Changes

### 1. Parse Switch as Expression

**Detection:** Check if switch is in expression position

```rust
fn parse_primary_expr(&mut self) -> Result<Expr> {
    match self.current_token() {
        Token::Switch => self.parse_switch_expr(),
        // ...
    }
}

fn parse_switch_expr(&mut self) -> Result<Expr> {
    // Parse switch { ... }
    // Return Expr::Switch(SwitchExpr { ... })
}
```

### 2. Parse Pattern Guards

```rust
fn parse_switch_arm(&mut self) -> Result<SwitchArm> {
    let pattern = self.parse_pattern()?;
    
    let guard = if self.match_token(Token::If) {
        Some(Box::new(self.parse_expr()?))
    } else {
        None
    };
    
    self.expect(Token::Colon)?;
    let body = self.parse_switch_body()?;
    
    Ok(SwitchArm { pattern, guard, body })
}
```

### 3. Parse Range Patterns

```rust
fn parse_pattern(&mut self) -> Result<Pattern> {
    match self.current_token() {
        Token::Int(n) => {
            let start = Expr::Literal(Literal::Int(n));
            
            if self.match_token(Token::Range) {  // ..
                let inclusive = self.match_token(Token::Eq);  // ..=
                let end = if self.peek() == Token::Colon {
                    None  // Open range: 10..
                } else {
                    Some(Box::new(self.parse_expr()?))
                };
                
                Ok(Pattern::Range(RangePattern {
                    start: Some(Box::new(start)),
                    end,
                    inclusive,
                }))
            } else {
                Ok(Pattern::Literal(Literal::Int(n)))
            }
        }
        // ...
    }
}
```

### 4. Parse Array/Tuple Patterns

```rust
fn parse_pattern(&mut self) -> Result<Pattern> {
    match self.current_token() {
        Token::LBracket => self.parse_array_pattern(),
        Token::LParen => self.parse_tuple_pattern(),
        // ...
    }
}

fn parse_array_pattern(&mut self) -> Result<Pattern> {
    self.expect(Token::LBracket)?;
    
    let mut patterns = Vec::new();
    let mut rest = None;
    
    while !self.check(Token::RBracket) {
        if self.match_token(Token::DotDotDot) {
            rest = Some(self.expect_identifier()?);
            break;
        }
        
        patterns.push(self.parse_pattern()?);
        
        if !self.check(Token::RBracket) {
            self.expect(Token::Comma)?;
        }
    }
    
    self.expect(Token::RBracket)?;
    Ok(Pattern::Array(patterns, rest))
}
```

---

## Semantic Analysis

### 1. Exhaustiveness Checking

**Algorithm:**

```rust
fn check_exhaustiveness(&mut self, switch: &SwitchExpr) -> Result<()> {
    let expr_type = self.infer_type(&switch.expr)?;
    
    match expr_type {
        Type::Bool => self.check_bool_exhaustive(&switch.arms)?,
        Type::Int | Type::Float | Type::String => {
            // Require default case
            if !self.has_wildcard(&switch.arms) {
                return Err(SemanticError::NonExhaustive {
                    type_name: expr_type.to_string(),
                    hint: "Add a 'default' case",
                });
            }
        }
        Type::Enum(name) => self.check_enum_exhaustive(&switch.arms, &name)?,
        _ => {}
    }
    
    Ok(())
}

fn check_bool_exhaustive(&self, arms: &[SwitchArm]) -> Result<()> {
    let mut has_true = false;
    let mut has_false = false;
    let mut has_wildcard = false;
    
    for arm in arms {
        match &arm.pattern {
            Pattern::Literal(Literal::Bool(true)) => has_true = true,
            Pattern::Literal(Literal::Bool(false)) => has_false = true,
            Pattern::Wildcard | Pattern::Binding(_) => has_wildcard = true,
            _ => {}
        }
    }
    
    if !((has_true && has_false) || has_wildcard) {
        return Err(SemanticError::NonExhaustive {
            type_name: "bool".to_string(),
            hint: "Cover both 'true' and 'false' cases, or add 'default'",
        });
    }
    
    Ok(())
}
```

### 2. Type Checking Switch Expression

**All arms must return same type:**

```rust
fn check_switch_expr_type(&mut self, switch: &SwitchExpr) -> Result<Type> {
    let mut arm_type: Option<Type> = None;
    
    for arm in &switch.arms {
        let body_type = match &arm.body {
            SwitchBody::Expr(e) => self.infer_type(e)?,
            SwitchBody::Block(stmts) => self.infer_block_type(stmts)?,
        };
        
        if let Some(ref expected) = arm_type {
            if !self.types_match(expected, &body_type) {
                return Err(SemanticError::TypeMismatch {
                    expected: expected.clone(),
                    found: body_type,
                    context: "All switch arms must return same type",
                });
            }
        } else {
            arm_type = Some(body_type);
        }
    }
    
    arm_type.ok_or_else(|| SemanticError::EmptySwitch)
}
```

### 3. Pattern Guard Validation

**Guard must be boolean:**

```rust
fn check_pattern_guard(&mut self, guard: &Option<Box<Expr>>) -> Result<()> {
    if let Some(guard_expr) = guard {
        let guard_type = self.infer_type(guard_expr)?;
        if guard_type != Type::Bool {
            return Err(SemanticError::TypeMismatch {
                expected: Type::Bool,
                found: guard_type,
                context: "Pattern guard must be boolean",
            });
        }
    }
    Ok(())
}
```

### 4. Range Pattern Validation

**Start must be less than end:**

```rust
fn check_range_pattern(&mut self, range: &RangePattern) -> Result<()> {
    // Both start and end must be comparable
    if let (Some(start), Some(end)) = (&range.start, &range.end) {
        let start_type = self.infer_type(start)?;
        let end_type = self.infer_type(end)?;
        
        if !self.is_numeric(&start_type) || !self.is_numeric(&end_type) {
            return Err(SemanticError::InvalidRange {
                hint: "Range patterns require numeric types",
            });
        }
        
        if start_type != end_type {
            return Err(SemanticError::TypeMismatch {
                expected: start_type,
                found: end_type,
                context: "Range start and end must have same type",
            });
        }
    }
    
    Ok(())
}
```

---

## Code Generation

### 1. Switch Expression → Rust match

**Basic switch expression:**

```liva
let result = switch x {
    case 1: "one"
    case 2: "two"
    default: "other"
}
```

**Generates:**

```rust
let result = match x {
    1 => "one",
    2 => "two",
    _ => "other",
};
```

### 2. Pattern Guards → Rust if guards

```liva
switch x {
    case y if y > 100: "large"
    case y if y > 0: "small"
    default: "invalid"
}
```

**Generates:**

```rust
match x {
    y if y > 100 => "large",
    y if y > 0 => "small",
    _ => "invalid",
}
```

### 3. Range Patterns → Rust range patterns

```liva
switch grade {
    case 90..100: "A"
    case 80..89: "B"
    default: "F"
}
```

**Generates:**

```rust
match grade {
    90..100 => "A",
    80..89 => "B",
    _ => "F",
}
```

### 4. Array Patterns → Rust slice patterns

```liva
switch arr {
    case []: "empty"
    case [x]: "single"
    case [x, y]: "pair"
    case [first, ...rest]: "many"
}
```

**Generates:**

```rust
match arr.as_slice() {
    [] => "empty",
    [x] => "single",
    [x, y] => "pair",
    [first, rest @ ..] => "many",
    _ => unreachable!(),
}
```

---

## Error Codes

### E6001: Non-Exhaustive Patterns

```
● E6001: Non-exhaustive patterns [Pattern Matching]
────────────────────────────────────────────────────────────
  → example.liva:5:1

     5 │ switch status {
       │ ^^^^^^
     6 │     case "pending": handlePending()
     7 │     case "approved": handleApproved()
       │

  ⓘ Missing case: "rejected"

  💡 Add the missing case or use 'default' to cover all remaining values

  📚 Learn more: https://liva-lang.org/docs/errors/patterns#e6001
────────────────────────────────────────────────────────────
```

### E6002: Type Mismatch in Switch Arms

```
● E6002: Type mismatch in switch arms [Type System]
────────────────────────────────────────────────────────────
  → example.liva:8:14

     6 │ let result = switch x {
     7 │     case 1: "one"
     8 │     case 2: 2
       │              ^
     9 │     default: "other"
       │

  ⓘ Expected 'string', found 'int'

  💡 All switch arms must return the same type

  📚 Learn more: https://liva-lang.org/docs/errors/types#e6002
────────────────────────────────────────────────────────────
```

### E6003: Invalid Range Pattern

```
● E6003: Invalid range pattern [Pattern Matching]
────────────────────────────────────────────────────────────
  → example.liva:3:10

     3 │     case "a".."z": handleLetter()
       │          ^^^^^^^^
       │

  ⓘ Range patterns require numeric types (int or float)

  💡 Use string comparison in a pattern guard instead:
      case s if s >= "a" && s <= "z": handleLetter()

  📚 Learn more: https://liva-lang.org/docs/errors/patterns#e6003
────────────────────────────────────────────────────────────
```

---

## Implementation Plan

### Iteration 1: Switch as Expression (1 hour)
1. Add `Expr::Switch` variant
2. Parse switch in expression position
3. Generate Rust `match` expression
4. Type check all arms return same type
5. Tests: basic switch expressions

### Iteration 2: Exhaustiveness Checking (30 min)
1. Add exhaustiveness check in semantic analyzer
2. Check bool, int, string, enum types
3. Error E6001 for non-exhaustive patterns
4. Tests: exhaustiveness errors

### Iteration 3: Pattern Guards (30 min)
1. Add `guard` field to `SwitchArm`
2. Parse `if` condition after pattern
3. Generate Rust `if` guards
4. Type check guard is boolean
5. Tests: pattern guards

### Iteration 4: Range Patterns (45 min)
1. Add `Pattern::Range` variant
2. Parse range syntax (`.., ..=`)
3. Generate Rust range patterns
4. Validate numeric types
5. Tests: range patterns

### Iteration 5: Array/Tuple Patterns (30 min - DEFERRED)
1. Add `Pattern::Array` and `Pattern::Tuple`
2. Parse destructuring syntax
3. Generate Rust slice patterns
4. Tests: array/tuple patterns
5. **Status:** Deferred to v0.9.6 (complex feature)

### Iteration 6: Documentation (15 min)
1. Update language-reference/pattern-matching.md
2. Add examples and best practices
3. Update CHANGELOG.md
4. Update ROADMAP.md

---

## Success Criteria

- ✅ Switch works as expression returning values
- ✅ Exhaustiveness checking for bool types
- ✅ Exhaustiveness checking requires default for int/string
- ✅ Pattern guards with `if` conditions work
- ✅ Range patterns (1..10, 10.., ..10, 1..=10) work
- ✅ All arms type-checked for consistency
- ✅ Clear error messages with hints
- ✅ Comprehensive tests (15+ test cases)
- ✅ Complete documentation

---

## Examples

### Example 1: HTTP Status Codes

```liva
fn getStatusMessage(code: int): string {
    return switch code {
        case 200..299: "Success"
        case 300..399: "Redirect"
        case 400..499: "Client Error"
        case 500..599: "Server Error"
        default: "Unknown"
    }
}
```

### Example 2: Categorize Age

```liva
fn categorizeAge(age: int): string {
    switch age {
        case x if x < 0: "Invalid age"
        case 0..12: "Child"
        case 13..17: "Teenager"
        case 18..64: "Adult"
        case 65..: "Senior"
    }
}
```

### Example 3: Boolean Exhaustive

```liva
fn boolToInt(flag: bool): int {
    switch flag {
        case true: 1
        case false: 0
    }  // ✓ Exhaustive without default
}
```

### Example 4: Grade Calculator

```liva
fn letterGrade(score: float): string {
    switch score {
        case 90.0..=100.0: "A"
        case 80.0..90.0: "B"
        case 70.0..80.0: "C"
        case 60.0..70.0: "D"
        case 0.0..60.0: "F"
        default: "Invalid score"
    }
}
```

---

## Notes

- **Array/Tuple patterns deferred** to v0.9.6 (adds significant complexity)
- **Enum patterns** require enum type system (future phase)
- **Rust's match** is a perfect compilation target
- **Exhaustiveness** improves code safety significantly
- **Pattern guards** enable complex conditional logic

---

## References

- Rust Pattern Matching: https://doc.rust-lang.org/book/ch18-00-patterns.html
- Scala Pattern Matching: https://docs.scala-lang.org/tour/pattern-matching.html
- Swift Pattern Matching: https://docs.swift.org/swift-book/LanguageGuide/Patterns.html
