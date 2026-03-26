# Classes: Data Classes

> SKILL.md covers: data class syntax (no constructor → auto-generated), `Point { x: number; y: number }`.
> This file: what auto-generation provides, data vs regular class decision, field ordering.

## Auto-Detection Rule

A class with **no explicit `constructor()`** is automatically a data class. No keyword needed.

| Has `constructor()`? | Result |
|----------------------|--------|
| **No** (fields only, or fields + methods) | Data class — auto constructor, PartialEq, Display |
| **Yes** | Regular class — you control initialization |

## What Auto-Generation Gives You

| Feature | Regular class | Data class (no constructor) |
|---------|--------------|----------------------------|
| **Constructor** | Your custom logic | Auto positional `new(field1, field2, ...)` |
| **`PartialEq`** | Not derived | Structural equality (`==` works) |
| **`Display`** | Not derived | `ClassName { field: value, ... }` |
| **Methods** | Supported | Supported |

```liva
Point { x: number; y: number }

let p = Point(10, 20)
print(p)                    // "Point { x: 10, y: 20 }"
print(p == Point(10, 20))   // true
```

## Data Class with Methods

Adding methods doesn't break auto-detection — still a data class if no `constructor()`:

```liva
Color {
    r: number
    g: number
    b: number

    sum() => this.r + this.g + this.b
}

let c = Color(255, 128, 0)
print(c.sum())   // 383
print(c)         // "Color { r: 255, g: 128, b: 0 }"
```

## Constructor Argument Order

The auto-generated constructor takes arguments in **field declaration order**:

```liva
Person { name: string; age: number; email: string }

// Constructor is Person(name, age, email) — matches field order
let p = Person("Alice", 30, "alice@ex.com")
```

## When to Use Data vs Regular Class

**Data class** (no constructor):
- Simple value types (coordinates, colors, config structs)
- DTOs / records where you just store fields
- Types used with `==` comparison
- No validation needed on construction

**Regular class** (with constructor):
- Need validation on creation (`fail` in constructor)
- Complex initialization logic
- Need to transform input before storing
- Fields derived from constructor args

```liva
// Data class — simple value, no validation
Coordinate { lat: float; lon: float }

// Regular class — needs validation
User {
    email: string
    age: number

    constructor(email: string, age: number) {
        if email == "" { fail "Email required" }
        this.email = email
        this.age = age
    }
}
```
