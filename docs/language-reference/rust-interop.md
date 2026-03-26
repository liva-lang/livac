# Rust Interop — Extended Reference

See SKILL.md for: `rust { }` basics, `use rust` deps, snake_case rule, E9002, internal crates list.

This file covers additional details NOT in SKILL.md.

---

## Snake_case Transform Examples

Liva codegen converts camelCase → snake_case. Inside `rust { }` blocks, use the snake_case name:

| Liva identifier | In `rust { }` block |
|-----------------|---------------------|
| `myValue` | `my_value` |
| `userName` | `user_name` |
| `isActive` | `is_active` |
| `httpCode` | `http_code` |

```liva
let myValue = 10
let doubled = rust {
    my_value * 2    // NOT myValue
}
```

---

## Result Types in Rust Blocks

Fallible Liva functions generate `Result<T, String>` in Rust. Inside `rust { }`, return `Ok(value)` or `Err("message".to_string())`:

```liva
divide(a: number, b: number): number {
    if b == 0 { fail "Division by zero" }
    return rust {
        Ok(a / b)
    }
}
```

Liva string vars are `String` type, numbers are `i32`, floats are `f64` in generated Rust.

---

## Crate Feature Merging

Internal crates already have built-in features. Declaring features **merges** with existing ones:

```liva
use rust "tokio" features ["net", "io-util"]
// Merges with built-in: rt-multi-thread, macros
// Result: tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util"] }
```

You cannot override the version:

```liva
// ❌ E9002: Cannot override internal crate version
use rust "tokio" version "2.0"
```

---

## `use` Hoisting Rules

`use std::...;` statements inside `rust { }` blocks are **hoisted** to the top of the generated Rust file. Duplicates across multiple blocks are deduplicated:

```liva
main() {
    let a = rust {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert("x", 1);
        m.len()
    }
    let b = rust {
        use std::collections::HashMap;  // Not duplicated in output
        let mut m = HashMap::new();
        m.insert("y", 2);
        m.len()
    }
}
```

Generated Rust has `use std::collections::HashMap;` once at the top.

---

## Internal Crate List

Always available — no `use rust` declaration needed to use inside `rust { }`:

| Crate | Version | Used For |
|-------|---------|----------|
| `tokio` | 1 | Async runtime |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON parsing |
| `reqwest` | 0.11 | HTTP client |
| `rayon` | 1.11 | Parallel execution |
| `rand` | 0.8 | Random numbers |

---

## E9002 Details

```
E9002: Internal crate version override
Cannot override internal crate "tokio" (v1)
  Help: Liva uses tokio v1 internally. The crate is already available
        inside rust { } blocks. To add features: use rust "tokio" features ["..."]
```

---

## String Handling in Blocks

Braces inside Rust strings, chars, and comments are handled correctly by the lexer:

```liva
let s = rust {
    let msg = "Hello {world} with {{ braces }}";
    msg.to_string()
}

let c = rust {
    let brace: char = '{';
    if brace == '{' { 1_i32 } else { 0_i32 }
}
```

**Caveat:** Rust raw strings (`r#"..."#`) with unbalanced braces may confuse brace balancing. Avoid or construct via function call.

---

## Hyphenated Crate Names

```liva
use rust "serde-json"    // Automatically becomes serde_json in Rust
```

---

## Limitations

- **No semantic validation**: Liva does not analyze `rust { }` content. Type errors surface only from `rustc`.
- **No type checking across boundaries**: Assigning `rust { 42_i64 }` to `let name: string` — Liva won't flag it, `rustc` will.
- **Expression context only**: `rust { }` blocks work inside functions only, not at module top level.
- **No `rustc` line mapping**: Error line numbers refer to generated `.rs`, not `.liva`. Use `livac build --verbose` to correlate.
- **Formatter ignores block content**: `livac fmt` preserves `rust { }` interior as-is.
