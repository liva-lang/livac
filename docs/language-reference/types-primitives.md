# Types: Primitives & Basics

> SKILL.md covers: `number`/`int`, `float`, `string`, `bool`, `char`, `bytes`, `void`, type inference.
> This file: complete type table with Rust mappings, collection types, type annotations guidance.

## Type Table

### Number Types

| Liva Type | Rust Type | Description |
|-----------|-----------|-------------|
| `number` / `int` | `i32` | Default integer (aliases) |
| `float` | `f64` | Default floating-point |
| `string` | `String` | Heap-allocated, growable |
| `bool` | `bool` | `true` / `false` |
| `char` | `char` | Unicode scalar value |
| `bytes` | `Vec<u8>` | Byte array |
| `void` | `()` | No value (return type) |

### Rust Types (also valid in Liva)

All Rust integer/float types can be used directly:

| Signed | Unsigned | Float |
|--------|----------|-------|
| `i8` | `u8` | `f32` |
| `i16` | `u16` | `f64` |
| `i32` | `u32` | |
| `i64` | `u64` | |
| `i128` | `u128` | |
| `isize` | `usize` | |

```liva
let count = 42              // inferred as i32 (number)
let pi = 3.14159            // inferred as f64 (float)
let tiny: i8 = 127          // explicit Rust type
let big: u64 = 1000000
let precise: f32 = 3.14
let name = "Alice"          // inferred as String
let active = true           // inferred as bool
let letter: char = 'A'
```

## Type Inference

Liva infers types automatically in most cases:

```liva
let count = 42              // number (i32)
let pi = 3.14               // float (f64)
let name = "Alice"          // string
let active = true           // bool

sum(a: number, b: number): number => a + b
let result = sum(10, 20)    // result: number
```

### When to Use Type Annotations

1. **API boundaries** — public functions and class fields
2. **Ambiguity** — when the type isn't obvious
3. **Specific precision** — when you need `u32` instead of default `i32`

```liva
// ✅ Good: clear API
calculateArea(width: float, height: float): float {
    return width * height
}

// ✅ Good: disambiguate
let count: u32 = 100

// ❌ Unnecessary: type is obvious
let x: number = 42    // Just use: let x = 42
```

## Collection Types

### Arrays — `[T]`

One array type in Liva: `[T]` → compiles to `Vec<T>`.

```liva
let numbers = [1, 2, 3, 4, 5]          // [number]
let scores: [number] = [85, 90, 78]    // explicit type
let first = numbers[0]
print($"Length: {numbers.length}")
```

### Maps — `Map<K, V>`

```liva
let ages = Map<string, number>{}
ages.set("Alice", 30)
let age = ages.get("Alice") or 0
```

### Sets — `Set<T>`

```liva
let tags = Set<string>{}
tags.add("rust")
let has = tags.contains("rust")
```

## Optional Types — `T?`

Nullable types — can be a value or `none`:

```liva
let maybeValue: number? = none
let name: string? = "Alice"

if maybeValue != none {
    print($"Value: {maybeValue}")
}
```

## Union Types — `T | U`

```liva
let value: number | string = "hello"
```

> **Note:** Type narrowing (`is` checks) is not yet supported. Use `switch` for tagged enums instead.

## Type Aliases

```liva
type UserId = number
type Handler = (string): void

let id: UserId = 123
```

## See Also

- **[Types: Advanced](types-advanced.md)** — Tuple types, type conversions, type checking
- **[Generics](generics-basics.md)** — Generic type parameters `<T>`
- **[Collections](collections.md)** — Full Map/Set/Array API
- **[Enums](enums.md)** — Tagged union types with data
