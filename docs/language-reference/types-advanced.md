# Types: Advanced

> SKILL.md covers: primitives, type aliases, Rust interop types, `as` cast.
> This file: tuple access/limitations, type casting details, nullable `?` types.

## Tuple Types

Fixed-size collections of mixed types.

### Declaration & Access

```liva
let point: (int, int) = (10, 20)
let mixed = (42, "hello", true)

// Access by index with dot notation
let x = point.0    // 10
let y = point.1    // 20
```

### Single-Element Tuples

Trailing comma required to distinguish from grouping:

```liva
let single = (42,)    // Tuple with one element
let grouped = (42)     // Just 42 (not a tuple)
```

### Chained Access — Parentheses Required

Nested tuple access requires parentheses (lexer parses `.0.0` as float `0.0`):

```liva
let matrix = ((1, 2), (3, 4))

// ✅ Correct
let val = (matrix.0).0    // 1

// ❌ Won't work — parsed as matrix.(0.0)
let val = matrix.0.0
```

### Tuple Return Functions

Explicit return type required for tuple-returning functions:

```liva
getCoords(): (int, int) => (10, 20)

main() {
    let coords = getCoords()
    print($"x: {coords.0}, y: {coords.1}")
}
```

### Pattern Matching with Tuples

```liva
let point = (10, 20)

let location = switch point {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}
```

### Tuple Limitations

- **No destructuring in `let`**: `let (x, y) = tuple` not supported — use `tuple.0`, `tuple.1`
- **Chained access needs parens**: `(tuple.0).0` not `tuple.0.0`
- **Explicit return types**: Tuple-returning functions need type annotation

### Tuples vs Data Classes

| Use tuples | Use data classes |
|------------|-----------------|
| ≤3 elements | >3 elements |
| Temporary/intermediate values | Domain model types |
| Order is obvious (x,y coords) | Field names add clarity |
| No methods needed | Need methods |

## Type Casting

Explicit `as` keyword for numeric conversions:

```liva
let x: i32 = 42
let y: i64 = x as i64
let z: f64 = x as f64
let b: u8 = 255
let wide: u32 = b as u32
```

No implicit numeric conversions — `as` is always required.

## Nullable Types (`?`)

Suffix `?` marks a type as nullable (compiles to `Option<T>`):

```liva
let maybe: number? = null
let name: string? = "Alice"

// Check before use
if maybe != null {
    print(maybe)
}
```

### Optional Fields in Classes

```liva
User {
    name: string
    email: string?     // Can be null

    constructor(name: string) {
        this.name = name
        // email defaults to null
    }
}
```

## Type Conversion Functions

All type conversions are **fallible** — use error binding:

```liva
let num, err = parseInt("42")
let val, err2 = parseFloat("3.14")
let s = toString(42)              // Infallible
```
