# Union Types

Union types allow a variable to hold one of several types. The compiler generates Rust enums for them.

---

## Declaration

### Inline Type Annotation

```liva
let x: int | string = 42
let y: string | bool = "hello"
let multi: int | string | bool = 100
```

### Type Alias

```liva
type StringOrInt = string | int
type Value = int | float | string | bool
```

> Generic type aliases (`type Result<T> = T | Error`) are parsed but codegen support is limited. For error handling, prefer Liva's built-in `fail`/`or fail` pattern.

---

## Semantics

- A value of type `A | B` can be any value of type `A` or `B`.
- Unions with arrays: `let items: [int] | string = "text"`
- Unions with tuples: `let coord: (int, int) | string = (10, 20)`
- Unions with optionals: `let maybe: int? | string = 42`
- Duplicates are removed: `int | int | string` → `int | string`
- Unions are flattened: nested `A | (B | C)` → `A | B | C`
- Order does not affect type identity.

---

## Codegen

Union types generate Rust enums named `Union_<type1>_<type2>`:

```liva
let x: int | string = 42
```

→ Generates:

```rust
#[derive(Debug, Clone)]
enum Union_i32_String {
    I32(i32),
    String(String),
}
```

Assignment wraps in the appropriate variant: `Union_i32_String::I32(42)`.

---

## Current Limitations

- **No pattern matching on unions**: `switch` on union-typed values with `case var: type =>` syntax is NOT yet implemented. Use enum types for exhaustive switch.
- **No type narrowing**: `if x is int` is NOT supported. Use enums with explicit variants instead.
- **No member access without narrowing**: Cannot access `.name` on `User | null` directly.
- **Circular union definitions** are not allowed.
- **Prefer enums** for discriminated unions — they have full switch/destructuring support.

---

## When to Use

| Use case | Recommendation |
|----------|---------------|
| Variable holds int or string | `let x: int | string` — works |
| Error handling | Use `fail`/`or fail` — NOT union types |
| Optional values | Use `type?` (nullable) — NOT `T \| null` |
| Discriminated data | Use `enum` with variants — full switch support |
| Type alias for clarity | `type Id = int \| string` — works for annotation |
