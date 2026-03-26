# String Templates — Extended Reference

See SKILL.md for: `$"text {expr}"` basics, variable/expression/function interpolation.

This file covers additional details NOT in SKILL.md.

---

## Brace Escaping

Use backslash for literal `{` or `}` inside template strings:

```liva
let code = $"function() \{ return 42; \}"
// Output: function() { return 42; }
```

> **Note:** Do NOT use `{{` for escaping — `{` always starts interpolation. Use `\{` and `\}` for literal braces.

---

## Nested Expressions

Any expression inside `{}` — arithmetic, function calls, member access, methods:

```liva
let total = $"Total: ${price * (1 + tax)}"
let intro = $"Hello! {person.greet()}"
let info = $"Sum: {a + b}, Name: {user.name}"
```

Note: `$` before `{` inside a template is just the dollar character — `${price}` outputs a dollar sign followed by the `price` value only if `price` evaluates. For actual interpolation, `{expr}` suffices.

---

## Type Coercion

All interpolated values are auto-converted to strings:

```liva
let count = 42
let active = true
let price = 99.99
let info = $"Count: {count}, Active: {active}, Price: {price}"
// Output: Count: 42, Active: true, Price: 99.99
```

---

## Escape Sequences

| Sequence | Output |
|----------|--------|
| `\n` | Newline |
| `\"` | Double quote |
| `\\` | Backslash |
| `\{` | Literal `{` |
| `\}` | Literal `}` |

```liva
let path = $"C:\\Users\\Alice"
let msg = $"She said, \"Hello!\""
let multiline = $"Line 1\nLine 2\nLine 3"
```

---

## Multi-line Strings

No triple-quote syntax. Use `\n` for multi-line:

```liva
let letter = $"Dear {name},\n\nThank you.\n\nBest,\nThe Team"
```

---

## Common Patterns

```liva
// Error messages with fail
fail $"Invalid age: {age}. Must be between {minAge} and {maxAge}"

// Logging
print($"[INFO] User {userId} action '{action}' at {timestamp}")

// Enum display
let label = $"Shape: {shape}"   // Uses Display impl
```

---

## Regular vs Template Strings

```liva
"Hello, {name}!"     // Regular string — outputs literal {name}
$"Hello, {name}!"    // Template string — interpolates name variable
```

Always use `$"..."` for interpolation. Regular `"..."` strings do NOT interpolate.
