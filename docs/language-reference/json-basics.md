# JSON: Type-Safe Parsing

> Basic `JSON.parse`/`JSON.stringify` calls are in SKILL.md. This file covers type-safe parsing with type hints, custom classes, optional fields, nested parsing, and error patterns.

## Type-Safe Parsing (v0.10.0+)

Type hint on the variable declaration drives deserialization:

```liva
// Primitives — use Liva's lowercase types in your code
let num: number, err = JSON.parse("42")
let text: string, err = JSON.parse("\"hello\"")
let flag: bool, err = JSON.parse("true")

// You can also use Rust's exact types when you need a specific width
let big: i64, err = JSON.parse("123456789012")
let pi: f64, err = JSON.parse("3.14")

// Arrays
let numbers: [number], err = JSON.parse("[1, 2, 3]")
let floats: [float], err = JSON.parse("[1.5, 2.7]")
let texts: [string], err = JSON.parse("[\"a\", \"b\"]")
```

### Supported Primitive Types

- Liva primitives: `string`, `number` (= `i32`), `float` (= `f64`), `bool`, `int` (alias of `number`).
- Rust integer/float types when you need a specific width: `i8`–`i128`, `u8`–`u128`, `isize`, `usize`, `f32`, `f64`.

Prefer the Liva types in your own code; reach for the Rust ones only when the JSON contains values outside `i32` / `f64` range.

## Parsing into Custom Classes

```liva
User { id: number; name: string; age: number }

let user: User, err = JSON.parse("{\"id\": 1, \"name\": \"Alice\", \"age\": 30}")
let users: [User], err = JSON.parse("[{\"id\": 1, ...}, {\"id\": 2, ...}]")
```

Classes used with `JSON.parse()` auto-get `serde::Deserialize` derives.

### Nested Classes

```liva
Post { id: number; title: string; body: string; userId: number }

let post: Post, err = JSON.parse(postJson)
if !err {
    print($"Post: {post.title} by user {post.userId}")
}
```

## Optional Fields (`?` syntax, v0.10.4+)

Handle nullable/missing JSON fields without parse failure:

```liva
User {
    id: number          // Required — must be present and non-null
    name: string        // Required
    email?: string      // Optional — can be absent, null, or present
    age?: number        // Optional
}
```

### Behavior

| JSON state | Required (`field: Type`) | Optional (`field?: Type`) |
|------------|-------------------------|--------------------------|
| Missing | Parse error | Success (None) |
| Null | Parse error | Success (None) |
| Present | Success | Success (Some) |

### Generated Rust

Optional fields → `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`

```liva
// All parse successfully:
let u1: User, _ = JSON.parse("{\"id\": 1, \"name\": \"Alice\", \"email\": \"a@b.com\"}")  // email = Some(...)
let u2: User, _ = JSON.parse("{\"id\": 2, \"name\": \"Bob\"}")                             // email = None
let u3: User, _ = JSON.parse("{\"id\": 3, \"name\": \"Carol\", \"email\": null}")          // email = None
```

## Error Handling Patterns

### With Error Binding (recommended)

```liva
let value: Type, err = JSON.parse(json)
if !err {
    // Success — use value
} else {
    print($"Parse failed: {err}")
}
```

### Without Error Binding (panics on error)

```liva
let value: Type = JSON.parse(json)
// Panics with "JSON parse failed" if parsing fails
```

### Default Values on Error

| Type | Default |
|------|---------|
| Integers | `0` |
| Floats | `0.0` |
| Booleans | `false` |
| Strings | `""` |
| Arrays | `[]` |
| Classes | All fields default |

## JSON.stringify

```liva
JSON.stringify(value: any): (string?, Error?)
```

Supported: `none`→null, `bool`, `int`, `float`, `string`, arrays, objects.
Unsupported (will error): functions, closures, tasks, circular references.

```liva
let json, err = JSON.stringify({name: "Bob", age: 25})
```

## Implementation Notes

- Uses `serde_json::from_str::<T>()` for deserialization
- Type validation at compile-time (semantic analysis)
- Classes auto-derive `Serialize`/`Deserialize` when used with JSON
- Error messages include serde_json details
