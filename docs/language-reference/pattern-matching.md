# Pattern Matching — Extended Reference

See SKILL.md for: switch expression/statement basics, enum destructuring, basic guards, wildcards, exhaustive enum switch.

This file covers additional details NOT in SKILL.md.

---

## Range Patterns — All 5 Forms

| Syntax | Meaning | Example | Matches |
|--------|---------|---------|--------|
| `a..=b` | Inclusive range | `1..=10` | 1, 2, ..., 10 |
| `a..b` | Exclusive end | `1..10` | 1, 2, ..., 9 |
| `a..` | Open end | `18..` | 18, 19, 20, ... |
| `..=b` | Up to inclusive | `..=100` | ..., 99, 100 |
| `..b` | Up to exclusive | `..18` | ..., 16, 17 |

```liva
let category = switch age {
    ..13 => "child"
    13..20 => "teenager"
    20..65 => "adult"
    65.. => "senior"
}
```

---

## Or-Patterns

Multiple patterns with same action using `|`:

```liva
let category = switch num {
    1 | 2 | 3 => "small"
    4 | 5 | 6 => "medium"
    _ => "other"
}

let statusType = switch httpCode {
    200 | 201 | 204 => "success"
    400 | 401 | 403 | 404 => "client error"
    500 | 502 | 503 => "server error"
    _ => "other"
}
```

**Limitation:** Or-patterns with bindings must bind the same variables in all alternatives.

---

## Tuple Patterns

```liva
let location = switch (10, 20) {
    (0, 0) => "origin"
    (0, y) => $"on Y axis at {y}"
    (x, 0) => $"on X axis at {x}"
    (x, y) => $"at ({x}, {y})"
}
```

> Tuple destructuring in `let` bindings not yet supported — use `let x = tuple.0`. Chained access needs parentheses: `(matrix.0).0`.

---

## Guard with Binding

Guards access bound variables. First match wins:

```liva
let description = switch age {
    x if x < 13 => $"Child: {x} years old"
    x if x < 20 => $"Teenager: {x} years old"
    x => $"Adult: {x} years old"
}
```

Nested switch inside guard arm:

```liva
let result = switch x {
    0 => "x is zero"
    n if n > 0 => switch y {
        0 => "x positive, y zero"
        m if m > n => "y greater than x"
        _ => "x >= y"
    }
    _ => "x is negative"
}
```

---

## Exhaustiveness Details

| Type | Requirement |
|------|-------------|
| `bool` | Cover `true` + `false`, or use `_` |
| `int`, `i8`–`i128`, `u8`–`u128` | Requires wildcard/binding |
| `string` | Requires wildcard/binding |
| Enums | Cover all variants or use `_` → **E0904** if missing |
| `float`, `char` | Not yet checked |

### Enum E0904 Example

```liva
enum Color { Red, Green, Blue }

// ❌ E0904: Missing Color.Blue
let label = switch color {
    Color.Red => "red"
    Color.Green => "green"
}
// Error: Pattern matching on enum `Color` is not exhaustive — missing variant(s): Color.Blue
```

---

## Error Codes

| Code | Error |
|------|-------|
| **E6001** | Non-exhaustive pattern match |
| **E6002** | Type mismatch in switch arms (all arms must return same type) |
| **E6003** | Invalid range (start > end) |
| **E0904** | Enum switch missing variant(s) |

---

## Limitations

- No array destructuring patterns: `[x, y, z] => ...`
- No tuple destructuring in `let` bindings
- Float/char exhaustiveness not checked
- Planned: array patterns, as-patterns

---

## Idioms — When NOT To Use `switch`

`switch` is for cases where you actually pattern-match (destructure, branch by range/guard, or map every variant to a distinct value). For simple identity checks on enums or scalars, prefer the basic operators:

```liva
// ✅ Idiomatic — direct equality / inequality
if status == Status.Done => archive(task)
isOpen(s: Status): bool => s != Status.Done
sameStatus(a, b) => a == b

// ✅ Idiomatic — chained ifs when conditions don't share a single subject
clamp(x, lo, hi): number {
    if x < lo => return lo
    if x > hi => return hi
    return x
}

// ❌ Switch as a glorified `if` adds noise
isOpen(s: Status): bool {
    return switch s {
        Status.Done => false
        _ => true
    }
}
```

When the function body is a single switch expression, drop the block:

```liva
statusLabel(s: Status): string => switch s {
    Status.Open => "open"
    Status.InProgress => "in-progress"
    Status.Done => "done"
}
```

