# Control Flow — Additional Reference

> SKILL.md covers all basic if/else, for, while, switch, break/continue, fail syntax,
> AND the canonical `=>` gotcha (control-flow `=>` does not imply return).
> This file covers the **other** edge cases only.

## Point-free `=>` in loops

```liva
for item in items => print          // calls print(item)
for item in items => process        // calls process(item)
```

## Single-Statement If (No Braces)

When the body is a single statement, braces are optional:

```liva
if age < 18 fail "Minor not allowed"
if !valid return "Invalid"
if debug => print("trace")
```

## Switch

Liva's `switch` supports two surface syntaxes:

### Modern arrow form (recommended)

Patterns separated by `=>`, `_` as the wildcard:

```liva
switch status {
    "active"   => print("Active")
    "inactive" => print("Inactive")
    _          => print("Unknown")
}
```

It works in two positions:

- **Statement** — side-effect-only arms. Each arm's body may be any single
  statement (expression, assignment, `return`, `break`, `continue`, …) or a
  `{ ... }` block. Arms do **not** need to return the same value; the
  `match` evaluates to `()` and you do **not** need a `let _ = ...` wrapper
  or a `0` filler in each arm.

  ```liva
  switch op {
      Op.Add(x, y) => print("add = " + (x + y).to_string())
      Op.Sub(x, y) => result = x - y           // assignment OK
      _            => return                   // control flow OK
  }
  ```

- **Expression** — arms produce a value of the same type:

  ```liva
  let label = switch n {
      0          => "zero"
      1 | 2 | 3  => "small"
      n if n < 0 => "negative"
      _          => "large"
  }
  ```

### Legacy `case` / `default:` form

Still supported for backward compatibility:

```liva
switch status {
    case "active": print("Active")
    case "inactive": print("Inactive")
    default: print("Unknown")
}
```

Switches have **no fall-through** — each arm is independent (unlike C/Java).

## Switch Guards

Bind the matched value and add a condition with `if`:

```liva
switch temperature {
    t if t < 0  => print("Freezing")
    t if t < 20 => print("Cold")
    t if t < 30 => print("Warm")
    _           => print("Hot")
}
```

## Try-Catch

```liva
try {
    let result = riskyOperation()
    print($"Success: {result}")
} catch (err) {
    print($"Error: {err}")
}
```

Nested try-catch is supported:

```liva
try {
    let data = fetchData()
    try {
        saveToDatabase(data)
    } catch (err) {
        print($"DB error: {err}")
    }
} catch (err) {
    print($"Fetch error: {err}")
}
```

> **Prefer error binding** over try-catch for idiomatic Liva:
> ```liva
> let result, err = divide(a, b)
> if err { print($"Error: {err}") }
> ```

## Fail Patterns

```liva
// In single-statement if (no braces)
if user.role != "admin" fail "Unauthorized"

// With string interpolation
if age < 0 fail $"Invalid age: {age}"

// In ternary expression
getDiscount(age: number): float => age >= 65 ? 0.2 : age < 18 ? fail "No discount" : 0.0
```

## Data-Parallel For Policies

```liva
for seq item in items { }                                // Explicit sequential (default)
for par item in items { }                                // Parallel (CPU-bound)
for par item in items with chunk 2 threads 4 { }        // With options
for vec value in values { }                              // Vectorized (SIMD)
for vec value in values with simdWidth 4 { }             // With SIMD width
for parvec value in values with simdWidth 4 ordered { }  // Parallel + vectorized
```

See `references/concurrency.md` for details on data-parallel policies.
