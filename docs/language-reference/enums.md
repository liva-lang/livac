# Enums

Complete reference for enum types (algebraic data types) in Liva.

## Table of Contents
- [Declaration](#declaration)
- [Simple Enums](#simple-enums)
- [Enums with Data](#enums-with-data)
- [Construction](#construction)
- [Pattern Matching](#pattern-matching)
- [As Function Parameters and Return Types](#as-function-parameters-and-return-types)
- [Recursive Enums](#recursive-enums)
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

## Recursive Enums

*(v2.0+)*

Enum variants can reference their own enum type. The compiler automatically wraps recursive fields in `Box<T>` (auto-boxing) — no manual annotation needed.

### Tree / AST Pattern

```liva
enum Expr {
    Num(value: number),
    Add(left: Expr, right: Expr),
    Mul(left: Expr, right: Expr)
}

// Construction — Box::new() is auto-generated
let expr = Expr.Add(Expr.Num(1), Expr.Mul(Expr.Num(2), Expr.Num(3)))
```

### Linked List Pattern

```liva
enum List {
    Cons(head: number, tail: List),
    Nil
}

let list = List.Cons(1, List.Cons(2, List.Cons(3, List.Nil)))
```

Only the recursive field (`tail: List`) is boxed. Non-recursive fields (`head: number`) remain unboxed.

### Pattern Matching

Pattern matching works transparently — the compiler auto-dereferences boxed bindings:

```liva
eval(e: Expr): number {
    return switch e {
        Expr.Num(v) => v
        Expr.Add(l, r) => eval(l) + eval(r)
        Expr.Mul(l, r) => eval(l) * eval(r)
    }
}
```

### Array Fields

Array fields like `children: [Tree]` do **not** need boxing — `Vec<T>` already provides heap indirection:

```liva
enum Tree {
    Leaf(value: number),
    Node(children: [Tree])    // Vec<Tree> — no Box needed
}
```

### How It Works

| Liva | Generated Rust |
|------|---------------|
| `left: Expr` (in `enum Expr`) | `left: Box<Expr>` |
| `Expr.Add(a, b)` | `Expr::Add { left: Box::new(a), right: Box::new(b) }` |
| `Expr.Add(l, r) => eval(l)` | `Expr::Add { left: l, right: r } => { let l = *l; let r = *r; eval(l) }` |
| `children: [Expr]` | `children: Vec<Expr>` (no boxing) |

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
