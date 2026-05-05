# Enums — Extended Reference

See SKILL.md for: basic enums, data enums, dot syntax (`Color.Red`), switch destructuring.

This file covers additional details NOT in SKILL.md.

---

## Recursive Enums (v2.0+) — CRITICAL

Enum variants can reference their own enum type. The compiler **auto-boxes** recursive fields in `Box<T>` — no manual annotation needed.

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

### Pattern Matching on Recursive Enums

The compiler auto-dereferences boxed bindings — transparent to the user:

```liva
eval(e: Expr): number {
    return switch e {
        Expr.Num(v) => v
        Expr.Add(l, r) => eval(l) + eval(r)
        Expr.Mul(l, r) => eval(l) * eval(r)
    }
}
```

### Array Fields — No Boxing Needed

`Vec<T>` already provides heap indirection:

```liva
enum Tree {
    Leaf(value: number),
    Node(children: [Tree])    // Vec<Tree> — no Box needed
}
```

### Auto-Boxing Codegen Summary

| Liva | Generated Rust |
|------|---------------|
| `left: Expr` (in `enum Expr`) | `left: Box<Expr>` |
| `Expr.Add(a, b)` | `Expr::Add { left: Box::new(a), right: Box::new(b) }` |
| `Expr.Add(l, r) => eval(l)` | `Expr::Add { left: l, right: r } => { let l = *l; let r = *r; eval(l) }` |
| `children: [Expr]` | `children: Vec<Expr>` (no boxing) |

---

## Generated Rust for Each Variant Type

### Simple enum (unit variants)

```liva
enum Color { Red, Green, Blue }
```

→ Rust: `#[derive(Debug, Clone, PartialEq)]` enum + `Display` impl (prints variant name).

### Data enum (named fields)

```liva
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}
```

→ Rust uses **named fields**: `Circle { radius: i32 }`, `Rectangle { width: i32, height: i32 }`, `Point`.

All enums get: `Debug`, `Clone`, `PartialEq`, `Display`.

---

## Exhaustive Switch — E0904

When all variants of an enum are covered, `_` is optional. Missing a variant produces **E0904**:

```liva
enum Direction { North, South, East, West }

// ✅ All variants covered — no _ needed
directionName(d: Direction): string {
    return switch d {
        Direction.North => "north"
        Direction.South => "south"
        Direction.East => "east"
        Direction.West => "west"
    }
}

// ❌ E0904: Missing Direction.West
let name = switch d {
    Direction.North => "north"
    Direction.South => "south"
    Direction.East => "east"
}
```

Using `_` still works for partial matching:

```liva
let label = switch color {
    Color.Red => "red"
    _ => "other"
}
```

---

## Wildcard `_` in Destructuring

Ignore fields you don't need:

```liva
label(shape: Shape): string {
    return switch shape {
        Shape.Circle(_) => "circle"
        Shape.Rectangle(w, _) => $"w={w}"
        Shape.Point => "point"
    }
}
```

Bindings match positionally to named fields: `Shape.Circle(r)` → `r` binds to `radius`.

---

## Idioms — Don't Reach For `switch` First

All enums get `PartialEq` derived, so `==` and `!=` work natively. **Use them instead of `switch` whenever you only care about identity, not destructuring.**

### Equality / inequality

```liva
// ✅ Idiomatic
sameStatus(a: Status, b: Status): bool => a == b
isDone(s: Status): bool => s == Status.Done
isActive(s: Status): bool => s != Status.Done

// ❌ Avoid — switch adds noise without adding value
sameStatus(a: Status, b: Status): bool {
    return switch a {
        Status.Open => switch b { Status.Open => true, _ => false }
        Status.InProgress => switch b { Status.InProgress => true, _ => false }
        Status.Done => switch b { Status.Done => true, _ => false }
    }
}
```

### Single-variant check

```liva
// ✅ Idiomatic
if status == Status.Done => archive(task)

// ❌ Avoid
switch status {
    Status.Done => archive(task)
    _ => {}
}
```

### Use `switch` when…

- You destructure fields: `Shape.Circle(r) => r * r * 3.14`.
- You map every variant to a different value: `statusLabel(s) => switch s { ... }`.
- You match ranges, tuples, or guards.

### Arrow form with switch body

A function that returns a switch expression can drop the block + `return`:

```liva
// ✅ Idiomatic — arrow body is a single switch expression
statusLabel(s: Status): string => switch s {
    Status.Open => "open"
    Status.InProgress => "in-progress"
    Status.Done => "done"
}

// ❌ Verbose
statusLabel(s: Status): string {
    return switch s {
        Status.Open => "open"
        Status.InProgress => "in-progress"
        Status.Done => "done"
    }
}
```

