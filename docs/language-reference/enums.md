# Enums

Complete reference for enum types (algebraic data types) in Liva.

## Table of Contents
- [Declaration](#declaration)
- [Simple Enums](#simple-enums)
- [Enums with Data](#enums-with-data)
- [Construction](#construction)
- [Pattern Matching](#pattern-matching)
- [As Function Parameters and Return Types](#as-function-parameters-and-return-types)
- [Generated Rust Code](#generated-rust-code)
- [Best Practices](#best-practices)

---

## Declaration

### Basic Syntax

Enums define a type with a fixed set of **variants**. Each variant can optionally carry **named fields**.

```liva
enum EnumName {
    Variant1,
    Variant2(field1: type, field2: type),
    Variant3(field: type)
}
```

- `enum` is a **hard keyword** (cannot be used as a variable name)
- Variants are comma-separated
- Fields use **named syntax**: `field: type`
- Trailing comma after the last variant is optional

---

## Simple Enums

Enums with no associated data (unit variants):

```liva
enum Color {
    Red,
    Green,
    Blue
}

enum Direction {
    North,
    South,
    East,
    West
}
```

Simple enums automatically get a `Display` implementation, so they can be printed:

```liva
let c = Color.Red
print(c)  // "Red"
```

---

## Enums with Data

Variants can carry named fields:

```liva
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}
```

You can mix variants with and without data in the same enum. Each field has a name and a type, using the same types available elsewhere in Liva (`number`, `string`, `bool`, etc.).

---

## Construction

### Unit Variants

Access via dot syntax on the enum name:

```liva
let color = Color.Red
let dir = Direction.North
```

### Variants with Data

Call the variant like a function with positional arguments (matched to named fields in order):

```liva
let circle = Shape.Circle(5)
let rect = Shape.Rectangle(10, 20)
let point = Shape.Point
```

---

## Pattern Matching

Enums are designed to work with `switch` expressions for exhaustive pattern matching.

### Basic Matching

```liva
directionName(d: Direction): string {
    return switch d {
        Direction.North => "north"
        Direction.South => "south"
        Direction.East => "east"
        Direction.West => "west"
    }
}
```

### Destructuring Fields

When a variant carries data, you can bind its fields to variables in the pattern:

```liva
area(shape: Shape): number {
    return switch shape {
        Shape.Circle(r) => 3 * r * r
        Shape.Rectangle(w, h) => w * h
        Shape.Point => 0
    }
}
```

The bindings (`r`, `w`, `h`) are matched **positionally** to the variant's named fields. In the example above:
- `Shape.Circle(r)` binds `r` to the `radius` field
- `Shape.Rectangle(w, h)` binds `w` to `width` and `h` to `height`

---

## As Function Parameters and Return Types

Enums can be used as parameter types and return types:

```liva
enum SearchResult {
    Found(value: number),
    NotFound
}

findItem(id: number): SearchResult {
    if id > 0 {
        return SearchResult.Found(id * 10)
    }
    return SearchResult.NotFound
}

main() {
    let result = findItem(5)
    let message = switch result {
        SearchResult.Found(v) => $"Found: {v}"
        SearchResult.NotFound => "Not found"
    }
    print(message)
}
```

---

## Generated Rust Code

Liva enums compile to Rust enums with `#[derive(Debug, Clone, PartialEq)]`:

```liva
enum Color { Red, Green, Blue }
```

Generates:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
```

Variants with data use **named fields** in Rust:

```liva
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}
```

Generates:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Circle { radius: i32 },
    Rectangle { width: i32, height: i32 },
    Point,
}
```

---

## Best Practices

1. **Use PascalCase** for enum names and variant names
2. **Prefer enums over string constants** for fixed sets of values
3. **Always handle all variants** in switch expressions for exhaustiveness
4. **Use meaningful field names** — they document the data each variant carries
5. **Simple enums** (unit variants only) are ideal for state machines and categories
6. **Enums with data** are ideal for representing results, events, or messages with varying payloads
