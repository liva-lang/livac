# Tuple Types Guide

> **Version:** v0.11.0  
> **Status:** Production Ready  
> **Last Updated:** 2025-01-28

## Overview

Tuples are fixed-size collections of values with potentially different types. They provide a lightweight way to group related data without defining a full struct.

**Key Features:**
- Fixed size (determined at compile time)
- Heterogeneous (mixed types)
- Type-safe access
- Direct mapping to Rust tuples (zero overhead)
- Work with pattern matching

---

## Basic Usage

### Creating Tuples

```liva
// Type inference
let point = (10, 20)              // (int, int)
let user = ("Alice", 30, true)    // (string, int, bool)

// Explicit types
let coords: (int, int) = (0, 0)
let info: (string, int) = ("Bob", 25)
```

### Single-Element Tuples

Require trailing comma to distinguish from parenthesized expressions:

```liva
let single = (42,)     // Tuple with one element
let grouped = (42)      // Just the number 42 (not a tuple)
```

### Empty Tuples

```liva
let unit = ()  // Empty tuple (unit type)
```

---

## Accessing Elements

### By Index

Use dot notation with numeric indices (0-based):

```liva
let point = (10, 20, 30)

let x = point.0  // 10
let y = point.1  // 20
let z = point.2  // 30

print($"Point: ({x}, {y}, {z})")
```

### Chained Access (Nested Tuples)

Requires parentheses due to lexer limitation:

```liva
let matrix = ((1, 2), (3, 4))

// ✅ Correct: Use parentheses
let first_row = matrix.0       // (1, 2)
let elem = (matrix.0).0        // 1

// ❌ Won't work: Lexer parses .0.0 as .0 followed by float 0.0
let elem = matrix.0.0          // Parse error
```

---

## Functions with Tuples

### Returning Tuples

Return multiple values without defining structs:

```liva
// Coordinates
getPosition(): (int, int) {
    return (10, 20)
}

// User info
getUserInfo(): (string, int, bool) {
    return ("Alice", 30, true)
}

// HTTP status
getStatus(): (int, string) {
    return (200, "OK")
}

main() {
    let pos = getPosition()
    print($"Position: ({pos.0}, {pos.1})")
    
    let info = getUserInfo()
    print($"User: {info.0}, Age: {info.1}, Active: {info.2}")
}
```

**Important:** Always specify explicit return types for tuple-returning functions. Type inference may default to `f64` without the annotation.

```liva
// ✅ Good: Explicit type
getCoords(): (int, int) {
    return (10, 20)
}

// ❌ May fail: Inference issue
getCoords() {
    return (10, 20)  // Might infer as f64, not tuple
}
```

### Tuples as Parameters

Pass tuples as function arguments:

```liva
// Function taking tuple
printPoint(point: (int, int)) {
    print($"Point: ({point.0}, {point.1})")
}

main() {
    let coords = (15, 25)
    printPoint(coords)
}
```

---

## Pattern Matching

Tuples work seamlessly with switch expressions:

### Basic Pattern Matching

```liva
let point = (10, 20)

let location = switch point {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}

print(location)  // "at (10, 20)"
```

### Pattern Matching with Guards

```liva
let status = (200, "OK")

let message = switch status {
    (200, text) => $"Success: {text}",
    (201, text) => $"Created: {text}",
    (code, _) if code >= 400 && code < 500 => "Client Error",
    (code, _) if code >= 500 => "Server Error",
    (code, text) => $"Status {code}: {text}"
}
```

### Nested Tuple Patterns

```liva
let matrix = ((1, 2), (3, 4))

let description = switch matrix {
    ((0, 0), (0, 0)) => "zero matrix",
    ((a, b), (c, d)) if a == d && b == c => "symmetric",
    ((a, _), (_, d)) => $"diagonal elements: {a}, {d}",
    _ => "general matrix"
}
```

---

## Nested Tuples

Tuples can contain other tuples:

```liva
// 2D point with color
let coloredPoint = ((10, 20), (255, 0, 0))

// Access nested elements
let coords = coloredPoint.0       // (10, 20)
let x = (coloredPoint.0).0       // 10 (requires parentheses!)
let y = (coloredPoint.0).1       // 20

let rgb = coloredPoint.1          // (255, 0, 0)
let red = (coloredPoint.1).0     // 255
```

---

## When to Use Tuples

### ✅ Use Tuples When:

1. **Returning Multiple Values:**
   ```liva
   // Returns (quotient, remainder)
   divmod(a: int, b: int): (int, int) {
       return (a / b, a % b)
   }
   ```

2. **Small Fixed-Size Groups:**
   ```liva
   // Coordinates
   let point = (10, 20)
   
   // RGB color
   let color = (255, 128, 0)
   ```

3. **Temporary Grouping:**
   ```liva
   // Quick pairing
   let pair = (key, value)
   ```

4. **Order is Obvious:**
   ```liva
   // Date: (year, month, day)
   let date = (2025, 1, 28)
   ```

### ❌ Use Structs When:

1. **Many Fields (>3-4):**
   ```liva
   // ❌ Bad: Hard to remember order
   let user = ("Alice", 30, "alice@example.com", "Engineer", true)
   
   // ✅ Good: Clear field names
   User {
       name: string
       age: int
       email: string
       role: string
       active: bool
   }
   ```

2. **Field Names Add Clarity:**
   ```liva
   // ❌ Unclear: What's what?
   let rect = (10, 20, 30, 40)
   
   // ✅ Clear: Named fields
   Rectangle {
       x: int
       y: int
       width: int
       height: int
   }
   ```

3. **Part of Domain Model:**
   ```liva
   // ❌ Not maintainable
   let product = ("Laptop", 999.99, 10)
   
   // ✅ Domain entity
   Product {
       name: string
       price: float
       stock: int
       
       isAvailable(): bool {
           return this.stock > 0
       }
   }
   ```

---

## Common Patterns

### Pair Pattern

```liva
// Key-value pairs
let entries = [
    ("name", "Alice"),
    ("age", "30"),
    ("city", "NYC")
]

entries.forEach(pair => {
    print($"{pair.0}: {pair.1}")
})
```

### Triple Pattern

```liva
// RGB colors
let colors = [
    (255, 0, 0),      // Red
    (0, 255, 0),      // Green
    (0, 0, 255)       // Blue
]

colors.forEach(color => {
    print($"RGB({color.0}, {color.1}, {color.2})")
})
```

### Result Pattern

```liva
// (value, error) tuple
parseNumber(str: string): (int, string) {
    // ... parsing logic ...
    if valid {
        return (result, "")
    }
    return (0, "Invalid number")
}

main() {
    let result = parseNumber("42")
    
    if result.1 == "" {
        print($"Parsed: {result.0}")
    } else {
        print($"Error: {result.1}")
    }
}
```

---

## Known Limitations (v0.11.0)

### 1. Chained Access Requires Parentheses

**Problem:** Lexer limitation - `.0.0` tokenizes as Dot + FloatLiteral(0.0)

```liva
let matrix = ((1, 2), (3, 4))

// ❌ Won't work
let elem = matrix.0.0

// ✅ Use parentheses
let elem = (matrix.0).0
```

**Workaround:** Always use parentheses for chained tuple access.

### 2. No Destructuring in Let Bindings

**Problem:** Parser doesn't recognize tuple patterns after `let`

```liva
let coords = (10, 20)

// ❌ Not yet supported
let (x, y) = coords

// ✅ Use direct access
let x = coords.0
let y = coords.1
```

**Workaround:** Access elements by index.

### 3. Return Type Inference

**Problem:** Functions without explicit return types may infer as `f64`

```liva
// ❌ May fail to compile
getPoint() {
    return (10, 20)  // Type inference issue
}

// ✅ Always use explicit types
getPoint(): (int, int) {
    return (10, 20)
}
```

**Workaround:** Always specify explicit return types for tuple-returning functions.

### 4. String Type Annotations

**Problem:** `(string, ...)` generates `(String, ...)` but literals are `&str`

```liva
// ❌ Type mismatch
getUserInfo(): (string, int) {
    return ("Alice", 30)  // &str vs String
}

// ✅ Use type inference
getUserInfo() {
    return ("Alice", 30)
}
// Or convert explicitly: "Alice".to_string()
```

**Workaround:** Use type inference or avoid explicit `string` in tuple types.

---

## Best Practices

### 1. Keep Tuples Small

```liva
// ✅ Good: 2-3 elements
let point = (10, 20)
let color = (255, 128, 0)

// ⚠️ Acceptable: 4 elements if obvious
let rect = (x, y, width, height)

// ❌ Bad: Too many elements
let data = (a, b, c, d, e, f, g)  // Use a struct instead!
```

### 2. Use Descriptive Variable Names

```liva
// ❌ Unclear
let t = (10, 20)

// ✅ Clear intent
let coordinates = (10, 20)
let point = (10, 20)
```

### 3. Document Tuple Meanings

```liva
// ✅ Good: Comment what each element means
// Returns (success_count, error_count, total_time)
processItems(): (int, int, float) {
    // ...
}
```

### 4. Use Pattern Matching

```liva
// ❌ Repetitive
let result = getStatus()
if result.0 == 200 {
    print($"Success: {result.1}")
} else if result.0 == 404 {
    print("Not Found")
}

// ✅ Cleaner with switch
let result = getStatus()
let message = switch result {
    (200, text) => $"Success: {text}",
    (404, _) => "Not Found",
    (code, text) => $"{code}: {text}"
}
```

### 5. Prefer Explicit Return Types

```liva
// ❌ Risky: Type inference may fail
calculate() {
    return (result, error)
}

// ✅ Safe: Explicit type
calculate(): (int, string) {
    return (result, error)
}
```

---

## Real-World Examples

### Example 1: HTTP Response

```liva
// Fetch data with status
fetchData(url: string): (int, string) {
    // ... HTTP request ...
    return (statusCode, responseBody)
}

main() {
    let response = fetchData("https://api.example.com/data")
    
    let result = switch response {
        (200, body) => $"Success: {body}",
        (404, _) => "Resource not found",
        (500, _) => "Server error",
        (code, _) => $"Status {code}"
    }
    
    print(result)
}
```

### Example 2: Min/Max

```liva
// Find min and max in one pass
minMax(numbers: [int]): (int, int) {
    let mut min_val = numbers[0]
    let mut max_val = numbers[0]
    
    for num in numbers {
        if num < min_val {
            min_val = num
        }
        if num > max_val {
            max_val = num
        }
    }
    
    return (min_val, max_val)
}

main() {
    let nums = [5, 2, 9, 1, 7]
    let result = minMax(nums)
    
    print($"Min: {result.0}, Max: {result.1}")
}
```

### Example 3: 2D Grid Processing

```liva
// Process 2D coordinates
processGrid() {
    let points = [
        (0, 0), (1, 0), (2, 0),
        (0, 1), (1, 1), (2, 1)
    ]
    
    points.forEach(point => {
        let x = point.0
        let y = point.1
        
        let type = switch point {
            (0, 0) => "origin",
            (x, 0) => $"x-axis at {x}",
            (0, y) => $"y-axis at {y}",
            (x, y) => $"point ({x}, {y})"
        }
        
        print(type)
    })
}
```

---

## Comparison: Tuples vs Structs vs Arrays

| Feature | Tuples | Structs | Arrays |
|---------|--------|---------|--------|
| **Size** | Fixed | Fixed | Variable* |
| **Types** | Mixed | Mixed | Same |
| **Named fields** | No (indexed) | Yes | No (indexed) |
| **Methods** | No | Yes | Built-in |
| **Pattern matching** | Yes | Planned | Planned |
| **Use case** | Temporary grouping | Domain entities | Collections |

*Arrays have fixed size in Liva, but use `Vec<T>` in Rust which is growable.

---

## Migration from Other Languages

### From Python

```python
# Python
point = (10, 20)
x, y = point
```

```liva
// Liva
let point = (10, 20)
// No destructuring yet
let x = point.0
let y = point.1
```

### From TypeScript

```typescript
// TypeScript
type Point = [number, number];
const point: Point = [10, 20];
const [x, y] = point;
```

```liva
// Liva
let point: (int, int) = (10, 20)
// No destructuring yet
let x = point.0
let y = point.1
```

### From Rust

```rust
// Rust
let point = (10, 20);
let (x, y) = point;
let nested = ((1, 2), 3);
let elem = nested.0.0;  // Chained access works
```

```liva
// Liva
let point = (10, 20)
// No destructuring yet
let x = point.0
let y = point.1

let nested = ((1, 2), 3)
let elem = (nested.0).0  // Need parentheses!
```

---

## Future Enhancements

Planned for v0.11.1+:

1. **Tuple Destructuring in Let:**
   ```liva
   let (x, y, z) = point
   ```

2. **Chained Access Without Parentheses:**
   ```liva
   let elem = matrix.0.0  // No parentheses needed
   ```

3. **Tuple Type Aliases:**
   ```liva
   type Point2D = (int, int)
   type Color = (u8, u8, u8)
   ```

4. **Tuple Spreading:**
   ```liva
   let a = (1, 2)
   let b = (...a, 3, 4)  // (1, 2, 3, 4)
   ```

---

## See Also

- [Types](../language-reference/types.md#tuple-types) - Type system reference
- [Pattern Matching](../language-reference/pattern-matching.md) - Switch expressions
- [Functions](../language-reference/functions.md#tuple-returns) - Returning tuples
- [Generics](../language-reference/generics.md) - Generic tuple types

---

**Next:** [Destructuring Guide](destructuring.md)  
**Previous:** [Generics Quick Start](generics-quick-start.md)
