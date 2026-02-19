# Types: Advanced

Advanced type system features in Liva including tuples, conversions, and type checking rules.

## Table of Contents

1. [Tuple Types](#tuple-types)
2. [Type Conversions](#type-conversions)
3. [Type Checking Rules](#type-checking-rules)
4. [Type Inference in Different Contexts](#type-inference-in-different-contexts)
5. [Nullability and Error Types](#nullability-and-error-types)
6. [Platform-Specific Types](#platform-specific-types)
7. [Type System Roadmap](#type-system-roadmap)
8. [Rust Interop](#rust-interop)
9. [Best Practices](#best-practices)

---

## Tuple Types

⭐ **New in v0.11.0**

Tuples are fixed-size collections of values with different types.

### Basic Tuple Types

```liva
// Type annotation
let point: (int, int) = (10, 20)
let user: (string, int, bool) = ("Alice", 30, true)

// Type inference
let coords = (0, 0)           // (int, int)
let mixed = (42, "hello")     // (int, string)
```

### Single-Element Tuples

Require trailing comma to distinguish from grouped expressions:

```liva
let single = (42,)     // Tuple with one element
let grouped = (42)      // Just 42 (not a tuple)
```

### Empty Tuples (Unit Type)

```liva
let unit = ()  // Empty tuple (unit type)
```

### Tuple Member Access

Access elements by index using dot notation:

```liva
let point = (10, 20, 30)
let x = point.0  // 10
let y = point.1  // 20
let z = point.2  // 30
```

**Chained Access:**
Requires parentheses for nested tuples:

```liva
let matrix = ((1, 2), (3, 4))

// ✅ Correct (use parentheses)
let first_row_first = (matrix.0).0  // 1

// ❌ Won't work (lexer issue)
let first_row_first = matrix.0.0    // Parsed as matrix.(0.0)
```

### Tuple Functions

Return multiple values without structs:

```liva
// Function returning tuple
getCoordinates(): (int, int) {
    return (10, 20)
}

main() {
    let coords = getCoordinates()
    print($"x: {coords.0}, y: {coords.1}")
}
```

### Pattern Matching

Tuples work with switch expressions:

```liva
let point = (10, 20)

let location = switch point {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}
```

### Nested Tuples

```liva
// 2x2 matrix as nested tuples
let matrix = ((1, 2), (3, 4))

// Access with parentheses
let first_row = matrix.0      // (1, 2)
let elem = (matrix.0).0       // 1
```

### When to Use Tuples vs Structs

**Use Tuples When:**
- Small, fixed-size collection of values
- Temporary grouping (return values, intermediate results)
- Order is obvious (coordinates, RGB colors)

**Use Structs When:**
- Many fields (>3-4 elements)
- Field names add clarity
- Need methods or behavior
- Part of your domain model

**Example:**

```liva
// ✅ Good: Tuple for simple coordinate
getPosition(): (int, int) => (10, 20)

// ✅ Better: Struct for complex data
User {
    id: u32
    name: string
    email: string
}
```

### Limitations (v0.11.0)

1. **Chained Access Requires Parentheses:**
   - Lexer limitation: `.0.0` tokenizes as Dot + FloatLiteral(0.0)
   - Solution: Use `(tuple.0).0` instead of `tuple.0.0`

2. **No Destructuring in Let Bindings (Yet):**
   - `let (x, y) = tuple` not yet supported
   - Use direct access: `let x = tuple.0`

3. **Return Type Inference:**
   - Explicit return types needed for tuple-returning functions
   - Inference defaults to `f64` without type annotation

## Type Conversions

### Explicit Conversions

```liva
// String to number
let str = "123"
let num = parseInt(str)
print($"Number: {num}")

// Number to string
let n = 42
let s = toString(n)
print($"String: {s}")

// Float conversions
let f = parseFloat("3.14")
print($"Float: {f}")
```

### Type Casting (Rust-style)

```liva
// Explicit cast
let x: i32 = 42
let y: i64 = x as i64
let z: f64 = x as f64
```

## Type Checking Rules

### Assignment Compatibility

```liva
// ✅ Compatible
let x: number = 42
let y: i32 = 42

// ❌ Incompatible (future: will error)
let a: number = 3.14  // float can't go into number without cast
```

### Function Argument Matching

```liva
sum(a: number, b: number): number => a + b

// ✅ Correct
let result = sum(10, 20)

// ❌ Wrong type (future: will error)
let bad = sum(10, 3.14)  // f64 doesn't match i32
```

### Return Type Checking

```liva
// Function must return declared type
calculate(): number {
  let result = 10 + 20
  return result  // ✅ i32 matches number
}

// ❌ Wrong return type (future: will error)
getAge(): number {
  return "25"  // string doesn't match number
}
```

## Type Inference in Different Contexts

### Variable Declarations

```liva
// From literal
let x = 42        // i32
let y = 3.14      // f64
let s = "hello"   // String
let b = true      // bool

// From expression
let sum = 10 + 20           // i32
let product = 3.5 * 2.0     // f64
let message = $"Hi {name}"  // String
```

### Function Returns

```liva
// Inferred from return expression
double(x) {
  return x * 2  // If x is i32, return is i32
}

// Explicit return type (better for APIs)
triple(x: number): number {
  return x * 3
}
```

### Array Elements

```liva
// All elements must have same type
let nums = [1, 2, 3]        // [i32; 3]
let strs = ["a", "b", "c"]  // [String; 3]

// ❌ Mixed types not allowed (future: will error)
let mixed = [1, "two", 3.0]
```

## Nullability and Error Types

Liva doesn't have `null` or `undefined`. Instead:

### Error Binding

```liva
// Functions can fail
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

// Error binding captures both result and error
let result, err = divide(10, 2)

if err != "" {
  print($"Error: {err}")
} else {
  print($"Result: {result}")
}
```

Error values are **strings** (`""`= no error).

## Platform-Specific Types

### Size Types

```liva
// Platform-dependent size (32-bit or 64-bit)
let index: usize = 0
let offset: isize = -10

// Use for array indexing
let items = [1, 2, 3, 4, 5]
let i: usize = 2
let item = items[i]
```

### Pointer-Sized Types

- `usize`: Unsigned, pointer-sized (for array sizes, indices)
- `isize`: Signed, pointer-sized (for offsets)

## Type System Roadmap

Current and planned features:

| Feature | Status |
|---------|--------|
| Basic primitives | ✅ v0.6 |
| Arrays | ✅ v0.6 |
| Objects/Classes | ✅ v0.6 |
| Type inference | ✅ v0.6 (basic) |
| Rust type compatibility | ✅ v0.6 |
| Explicit type checking | ✅ v0.9 |
| Generics (`<T>`) | ✅ v0.9 |
| **Tuple types** | ✅ v0.11.0 |
| Optional types (`?`) | ✅ v0.10.4 |
| Union types (`\|`) | 📋 v0.12+ |
| Type aliases | 📋 v0.12+ |
| Traits/Interfaces | ✅ v0.6 (basic) |

## Rust Interop

All Rust primitive types work in Liva:

```liva
// Use any Rust type
let byte: u8 = 255
let huge: u128 = 340282366920938463463374607431768211455
let ptr: usize = 0x1000

// Rust standard types (future)
use rust "std::collections::HashMap" as HashMap

let map: HashMap<string, number> = HashMap.new()
```

## Best Practices

### 1. Use Type Inference

```liva
// ✅ Good: Let the compiler infer
let count = 42
let name = "Alice"

// ❌ Unnecessary: Type is obvious
let count: number = 42
let name: string = "Alice"
```

### 2. Annotate Public APIs

```liva
// ✅ Good: Clear API contract
calculateTotal(items: [Item], tax: float): float {
  // ...
}

// ❌ Bad: Unclear API
calculateTotal(items, tax) {
  // What types?
}
```

### 3. Choose Appropriate Precision

```liva
// ✅ Good: Use smallest type that fits
let age: u8 = 25        // 0-255 is enough
let count: u32 = 1000   // Reasonable range

// ❌ Bad: Overkill
let age: u128 = 25      // Way too large
```

### 4. Be Explicit with Conversions

```liva
// ✅ Good: Clear intent
let x: i32 = 42
let y: f64 = x as f64

// ❌ Bad: Implicit conversion (doesn't work in Liva)
let y: f64 = x
```

---

## See Also

- **[Types: Primitives & Basics](types-primitives.md)** - Basic types, type inference, collections, and function types
- **[Variables & Constants](variables.md)** - Variable declarations
- **[Functions](functions-basics.md)** - Function type signatures
- **[Classes](classes-basics.md)** - Custom types
- **[Collections](collections.md)** - Arrays and data structures
- **[Type Conversions API](../api/conversions.md)** - Conversion functions
