# Pattern Matching

> **Version:** v0.10.5  
> **Status:** Production Ready  
> **Last Updated:** 2025-01-24  
> **New in v0.10.5:** Or-patterns, Enhanced exhaustiveness checking

Pattern matching provides a powerful way to inspect and destructure values in Liva.

---

## Table of Contents

- [Switch Expressions](#switch-expressions)
- [Pattern Types](#pattern-types)
- [Pattern Guards](#pattern-guards)
- [Exhaustiveness](#exhaustiveness)
- [Examples](#examples)
- [Best Practices](#best-practices)
- [Limitations](#limitations)
- [Error Codes](#error-codes)

---

## Switch Expressions

### Basic Syntax

```liva
let result = switch value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
};
```

- Switch expressions **must** have at least one arm
- Each arm has a pattern, optional guard, and body
- Bodies can be single expressions or blocks
- Trailing commas are optional

### Expression vs Statement

```liva
// As expression (returns a value)
let grade = switch score {
    90..=100 => "A",
    80..=89 => "B",
    70..=79 => "C",
    _ => "F"
};

// With block bodies
let message = switch status {
    200 => {
        print("Success!");
        "OK"
    },
    404 => {
        print("Not found");
        "Error"
    },
    _ => "Unknown"
};
```

---

## Pattern Types

### 1. Literal Patterns

Match exact values:

```liva
let result = switch x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"
};
```

**Supported Types:** `int`, `float`, `bool`, `string`, `char`

### 2. Wildcard Pattern

```liva
let result = switch value {
    1 => "one",
    2 => "two",
    _ => "something else"  // Matches anything
};
```

### 3. Binding Pattern

Captures the matched value in a variable:

```liva
let doubled = switch num {
    0 => 0,
    n => n * 2  // 'n' binds to the value
};

let description = switch age {
    x if x < 13 => $"Child: {x} years old",
    x if x < 20 => $"Teenager: {x} years old",
    x => $"Adult: {x} years old"
};
```

### 4. Range Patterns

```liva
let category = switch score {
    90..=100 => "A",    // Inclusive range
    80..=89 => "B",
    70..=79 => "C",
    60..=69 => "D",
    0..=59 => "F",
    _ => "Invalid"
};
```

**Range Syntax:**

| Syntax | Meaning | Example | Matches |
|--------|---------|---------|---------|
| `a..=b` | Inclusive range | `1..=10` | 1, 2, ..., 10 |
| `a..b` | Exclusive end | `1..10` | 1, 2, ..., 9 |
| `a..` | Open end | `18..` | 18, 19, 20, ... |
| `..=b` | Up to and including | `..=100` | ..., 99, 100 |
| `..b` | Up to but not including | `..18` | ..., 16, 17 |

```liva
let category = switch age {
    ..13 => "child",
    13..20 => "teenager",
    20..65 => "adult",
    65.. => "senior"
};
```

### 5. Or-Patterns (v0.10.5)

Match multiple patterns with the same action using `|`:

```liva
let category = switch num {
    1 | 2 | 3 => "small",
    4 | 5 | 6 => "medium",
    7 | 8 | 9 => "large",
    _ => "out of range"
};

let isWeekend = switch day {
    "Saturday" | "Sunday" => true,
    _ => false
};

let statusType = switch httpCode {
    200 | 201 | 204 => "success",
    400 | 401 | 403 | 404 => "client error",
    500 | 502 | 503 => "server error",
    _ => "other"
};
```

**Limitations:** Or-patterns with bindings must bind the same variables in all alternatives.

### 6. Tuple Patterns (v0.11.0)

```liva
let location = switch (10, 20) {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}
```

> **Note:** Tuple destructuring in `let` bindings not yet supported — use `let x = tuple.0`. Chained access needs parentheses: `(matrix.0).0`.

---

## Pattern Guards

Add conditional logic to patterns with `if` clauses:

```liva
let status = switch value {
    x if x < 0 => "negative",
    x if x == 0 => "zero",
    x if x > 0 => "positive"
};
```

Guards are evaluated in order; first match wins. Guards can use any boolean expression and have access to bound variables.

---

## Exhaustiveness

> ✅ Implemented for `bool`, `int`, and `string` types (v0.10.5)

The compiler checks that all possible values are covered in pattern matching, preventing runtime errors from unhandled cases.

### Boolean Exhaustiveness

Both values must be covered:

```liva
// ✅ Exhaustive
let result = switch flag {
    true => "yes",
    false => "no"
};

// ✅ Wildcard catches remaining case
let result = switch flag {
    true => "yes",
    _ => "no"
};

// ❌ Non-exhaustive - Compiler error: E0901
let result = switch flag {
    true => "yes"
};
```

### Integer Exhaustiveness

Requires a wildcard or binding pattern (integers have too many values to enumerate):

```liva
// ✅ Exhaustive
let result = switch num {
    0 => "zero",
    1 => "one",
    _ => "other"
};

// ❌ Non-exhaustive - Compiler error: E0902
let result = switch num {
    0 => "zero",
    1 => "one"
};
```

### String Exhaustiveness

A wildcard or binding is **always required**:

```liva
// ✅ Exhaustive
let code = switch status {
    "active" => 1,
    "inactive" => 2,
    _ => 0
};

// ❌ Non-exhaustive - Compiler error: E0903
let code = switch status {
    "active" => 1,
    "inactive" => 2
};
```

### Exhaustiveness Summary

| Type | Requirement | Since |
|------|-------------|-------|
| `bool` | Cover `true` and `false` (or use `_`) | v0.9.5 |
| `int`, `i8`-`i128`, `u8`-`u128` | Requires wildcard/binding | v0.10.5 |
| `string`, `String` | Requires wildcard/binding | v0.10.5 |
| Or-patterns | Supported, still need wildcard | v0.10.5 |
| `float`, `char`, enums | Not yet checked | — |

---

## Examples

### HTTP Status Codes

```liva
main() {
    let statusCode = 404;
    let message = switch statusCode {
        200 => "OK",
        201 => "Created",
        400 | 401 | 403 => "Client Error",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown Status"
    };
    print($"Status: {statusCode} - {message}");
}
```

### Nested Switch

```liva
let result = switch x {
    0 => "x is zero",
    n if n > 0 => switch y {
        0 => "x positive, y zero",
        m if m > n => "y greater than x",
        _ => "x >= y"
    },
    _ => "x is negative"
};
```

---

## Best Practices

### 1. Order Patterns from Specific to General

```liva
// ✅ Good - specific cases first
switch value {
    0 => "zero",
    1 => "one",
    n if n < 10 => "single digit",
    _ => "large number"
}

// ❌ Bad - wildcard too early catches everything
switch value {
    _ => "any number",
    0 => "zero"         // Never reached
}
```

### 2. Use Ranges for Continuous Values

```liva
// ✅ Good
switch score {
    90..=100 => "A",
    80..=89 => "B",
    _ => "C or below"
}
```

### 3. Always Include a Wildcard for Non-Bool Types

```liva
// ❌ May miss cases
switch status {
    "active" => handleActive(),
    "pending" => handlePending()
}

// ✅ Safe
switch status {
    "active" => handleActive(),
    "pending" => handlePending(),
    _ => handleUnknown()
}
```

### 4. Extract Complex Guards to Functions

```liva
// ❌ Hard to read — long guard expression
switch user {
    u if u.age >= 18 and u.hasLicense and !u.isSuspended => "can drive",
    _ => "cannot drive"
}

// ✅ Extract to a function
canDrive(u: User): bool => u.age >= 18 and u.hasLicense and !u.isSuspended

switch user {
    u if canDrive(u) => "can drive",
    _ => "cannot drive"
}
```

---

## Limitations

**Current (v0.11.0):**
- No enum variant patterns yet (coming soon)
- No array destructuring patterns: `[x, y, z] => ...`
- Tuple destructuring in `let` bindings not yet supported
- Float/char exhaustiveness not checked

**Planned:**
- Enum variant patterns, array patterns, as-patterns

---

## Error Codes

### E6001: Non-Exhaustive Pattern Match

```liva
// ❌ Error
let result = switch flag {
    true => "yes"
};

// ✅ Fix
let result = switch flag {
    true => "yes",
    false => "no"
};
```

### E6002: Type Mismatch in Switch Arms

All arms must return the same type.

```liva
// ❌ Error: string vs int
let result = switch x {
    0 => "zero",
    1 => 1,
    _ => "other"
};
```

### E6003: Invalid Range Pattern

```liva
// ❌ Error: start > end
let result = switch x {
    10..5 => "invalid",
    _ => "ok"
};
```

---

## See Also

- [Control Flow](control-flow.md) - Traditional switch statements
- [Error Handling](error-handling.md) - Error patterns
- [Types](types.md) - Type system overview
- [Operators](operators.md) - Comparison and range operators

---

**Next:** [String Templates](string-templates.md)  
**Previous:** [Operators](operators.md)
