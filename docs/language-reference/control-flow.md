# Control Flow — Additional Reference

> SKILL.md covers all basic if/else, for, while, switch, break/continue, and fail syntax.
> This file covers **gotchas, edge cases, and advanced patterns only**.

## One-liner `=>` Gotcha

For `if`/`for`/`while`, `=>` replaces `{}` but has **no implicit return** (unlike function `=>`):

```liva
// ✅ Function => has implicit return
add(a, b) => a + b

// ⚠️ if/for/while => does NOT — use explicit return
clamp(val: number, lo: number, hi: number): number {
    if val < lo => return lo   // explicit return required
    if val > hi => return hi   // explicit return required
    return val
}
```

Point-free also works with `=>`:

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

## Switch Guards

Bind the matched value and add a condition with `if`:

```liva
switch temperature {
    case t if t < 0: print("Freezing")
    case t if t < 20: print("Cold")
    case t if t < 30: print("Warm")
    default: print("Hot")
}
```

Switches have **no fall-through** — each case is independent (unlike C/Java).

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
