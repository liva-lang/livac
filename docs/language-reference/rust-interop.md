# Rust Interop *(v1.5.0)*

Liva compiles to Rust, so interop is natural. You can embed raw Rust code, declare crate dependencies, and use Rust primitive types — all from Liva source files.

## Table of Contents

1. [Inline Rust Blocks](#inline-rust-blocks)
2. [Crate Dependencies](#crate-dependencies)
3. [Use Hoisting](#use-hoisting)
4. [Internal Crates](#internal-crates)
5. [Rust Types in Liva](#rust-types-in-liva)
6. [Error Codes](#error-codes)
7. [Best Practices](#best-practices)
8. [Limitations & Known Caveats](#limitations--known-caveats)

---

## Inline Rust Blocks

Use `rust { ... }` to embed raw Rust code as an expression anywhere Liva expects a value.

### Basic Usage

```liva
main() {
    let result = rust {
        let x: i32 = 42;
        x * 2
    }
    print(result)  // 84
}
```

The block acts as a **Rust block expression** — the last expression is the value. The generated Rust wraps it in `{ ... }`.

### As a Statement

`rust { }` blocks can also be standalone statements (not assigned to a variable):

```liva
main() {
    rust {
        println!("Direct Rust output!");
    }
    print("Back in Liva")
}
```

### In Any Function

Rust blocks work in any function, not just `main()`:

```liva
computeHash(data: string): number {
    let hash = rust {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        "hello".hash(&mut hasher);
        hasher.finish() as i32
    }
    return hash
}

main() {
    let h = computeHash("hello")
    print($"Hash: {h}")
}
```

### Multiple Blocks

You can use multiple `rust { }` blocks in the same function:

```liva
main() {
    let a = rust {
        let x: i32 = 10;
        x + 5
    }
    let b = rust {
        let y: i32 = 20;
        y + 5
    }
    print(a + b)  // 40
}
```

### Nested Braces

The lexer correctly handles nested braces inside Rust blocks — closures, `if/else`, `match`, iterators, etc.:

```liva
main() {
    let val = rust {
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        let sum: i32 = v.iter()
            .filter(|x| { **x > 2 })
            .map(|x| {
                if *x > 4 { x * 3 } else { x * 2 }
            })
            .sum();
        sum
    }
    print(val)
}
```

### String Literals, Char Literals, and Comments

Braces inside Rust strings, chars, and comments are handled correctly and don't break block detection:

```liva
main() {
    // Strings with braces
    let s = rust {
        let msg = "Hello {world} with {{ braces }}";
        msg.to_string()
    }

    // Char literals (including brace chars)
    let c = rust {
        let brace: char = '{';
        let close: char = '}';
        if brace == '{' { 1_i32 } else { 0_i32 }
    }

    // Comments with braces
    let x = rust {
        // This { brace } in a comment is fine
        /* Block comment { with } braces */
        42_i32
    }
}
```

### Mixed with Liva Code

Rust blocks integrate naturally with regular Liva code:

```liva
square(x: number): number = x * x

main() {
    let liva_val = square(5)
    let rust_val = rust {
        let x: i32 = 25;
        x + 1
    }
    print($"Liva: {liva_val}, Rust: {rust_val}")
}
```

---

## Crate Dependencies

Use `use rust "crate"` at the top of your file to declare Cargo dependencies.

### Basic Declaration

```liva
use rust "serde"
use rust "tokio"
use rust "reqwest"

main() {
    print("Using Rust crates")
}
```

This generates `[dependencies]` entries in `Cargo.toml`.

### Version Pinning

```liva
use rust "chrono" version "0.4"
use rust "uuid" version "1.0"
```

Generates:
```toml
[dependencies]
chrono = "0.4"
uuid = "1.0"
```

### Features

```liva
use rust "uuid" version "1.0" features ["v4", "serde"]
use rust "tokio" features ["net", "io-util"]
```

Generates:
```toml
[dependencies]
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util"] }
```

Note: `tokio` is an **internal crate** — Liva already depends on it. Declaring features for internal crates **merges** them with the built-in features.

### Aliases

```liva
use rust "serde_json" as json
```

This allows using `json` as the module name in your code.

### Combined: Crates + Inline Blocks

```liva
use rust "chrono" version "0.4"

main() {
    let timestamp = rust {
        use std::time::SystemTime;
        let now = SystemTime::now();
        42_i32
    }
    print($"Result: {timestamp}")
}
```

---

## Use Hoisting

When you write `use std::...;` inside a `rust { }` block, the compiler **hoists** those statements to the top of the generated Rust file (after `#![allow(...)]`).

```liva
main() {
    let size = rust {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("key", "value");
        map.len()
    }
}
```

Generated Rust:
```rust
#![allow(unused)]
use std::collections::HashMap;

fn main() {
    let size = {
        let mut map = HashMap::new();
        map.insert("key", "value");
        map.len()
    };
}
```

### Deduplication

If multiple blocks use the same `use` statement, it's emitted only once:

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

---

## Internal Crates

Liva uses certain Rust crates internally. These are always available:

| Crate | Version | Used For |
|-------|---------|----------|
| `tokio` | 1 | Async runtime |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON parsing |
| `reqwest` | 0.11 | HTTP client |
| `rayon` | 1.11 | Parallel execution |
| `rand` | 0.8 | Random numbers |

### Adding Features to Internal Crates

You can add features to internal crates without specifying a version:

```liva
use rust "tokio" features ["net", "io-util"]
```

This merges your features with the built-in ones (e.g., `rt-multi-thread`, `macros`).

### Version Override Protection (E9002)

You **cannot** override the version of an internal crate:

```liva
// ❌ Error E9002
use rust "tokio" version "2.0"
```

```
E9002: Internal crate version override
Cannot override internal crate "tokio" (v1)
  Help: Liva uses tokio v1 internally. The crate is already available
        inside rust { } blocks. To add features: use rust "tokio" features ["..."]
```

---

## Rust Types in Liva

All Rust primitive types work directly in Liva type annotations:

### Size Types

```liva
let byte: u8 = 255
let small: i16 = -1000
let big: u64 = 1_000_000
let huge: u128 = 340282366920938463463374607431768211455
```

### Pointer-Sized Types

```liva
let ptr: usize = 0x1000
let signed: isize = -42
```

### Mapping

| Liva Type | Rust Equivalent |
|-----------|----------------|
| `number` | `i32` |
| `float` | `f64` |
| `bool` | `bool` |
| `string` | `String` |
| `char` | `char` |
| `u8`, `u16`, ... | Same |
| `i8`, `i16`, ... | Same |
| `f32`, `f64` | Same |
| `usize`, `isize` | Same |

---

## Error Codes

| Code | Error | Cause |
|------|-------|-------|
| **E9002** | Internal crate version override | `use rust "tokio" version "2.0"` — can't change version of built-in crates |

---

## Best Practices

### 1. Keep Rust Blocks Small

```liva
// ✅ Good: Small, focused block
let hash = rust {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    h.finish() as i32
}

// ❌ Bad: Entire function in a rust block
let everything = rust {
    // 50 lines of Rust...
}
```

### 2. Use `use rust` for Crates, `rust { }` for Code

```liva
// ✅ Crate dependency declared at top level
use rust "chrono" version "0.4"

// ✅ Inline Rust for specific operations
let now = rust {
    chrono::Utc::now().timestamp() as i32
}
```

### 3. Prefer Liva Syntax When Possible

Use `rust { }` only when you need Rust-specific features not available in Liva (e.g., unsafe code, specific trait implementations, low-level APIs).

### 4. Document Your Rust Blocks

```liva
// SHA-256 hash using the sha2 crate
let digest = rust {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"hello");
    format!("{:x}", hasher.finalize())
}
```

---

## Limitations & Known Caveats

### No Semantic Validation of Rust Code

Liva does **not** analyze the content of `rust { }` blocks. Any type errors, borrow checker violations, or invalid syntax inside the block will only surface when `rustc` compiles the generated Rust code. The compiler reports those errors with `rustc`'s original messages.

```liva
// This compiles through Liva with no errors...
let x = rust {
    let s: String = 42;  // ← Rust type error — caught only by rustc
    s
}
```

### No Type Checking Across Boundaries

The type of a `rust { }` expression is **not validated** by Liva's type checker. If you assign a Rust block that evaluates to `i64` to a Liva variable typed as `string`, Liva won't flag it — `rustc` will.

```liva
// Liva accepts this, but rustc will emit a type mismatch error
let name: string = rust { 42_i64 }
```

### Variable Naming: snake_case Translation

Liva variables are available inside `rust { }` blocks because the generated Rust code lives in the same function scope. However, Liva's codegen converts camelCase names to snake_case:

```liva
let myValue = 10
let doubled = rust {
    // Use snake_case: my_value, not myValue
    my_value * 2
}
```

### Formatter Preserves Block Content

The Liva formatter (`livac --format`) does **not** reformat the interior of `rust { }` blocks. The content is emitted exactly as written. Use `rustfmt` conventions manually inside the block.

### Raw Strings Not Handled in Brace Balancing

The lexer's brace balancer correctly handles regular strings (`"..."`), char literals (`'...'`), line comments (`//`), and block comments (`/* */`). However, **Rust raw strings** (`r"..."`, `r#"..."#`) are not specifically handled. If a raw string contains unbalanced braces, it may confuse the block detection:

```liva
// ⚠️ May fail if raw string contains unbalanced braces
let x = rust {
    let pattern = r#"{ unclosed"#;  // Could confuse brace balancing
    42
}
```

**Workaround:** Avoid unbalanced braces inside raw strings, or construct the string via a Rust function call instead.

### No `rustc` Error Line Mapping

When `rustc` reports errors from generated Rust code, line numbers refer to the **generated `.rs` file**, not the original `.liva` source. Check the generated output with `livac file.liva --verbose` to correlate errors.

### Expression Context Only

`rust { }` blocks can only appear where Liva expects an **expression** — inside functions, assigned to variables, or as standalone statements. They cannot be used at the module top level outside functions.

```liva
// ✅ Inside a function
main() {
    let x = rust { 42 }
}

// ❌ At module top level — not supported
rust { static MY_CONST: i32 = 42; }
```

---

## See Also

- **[Types — Advanced](types-advanced.md)** — Rust primitive types in Liva
- **[Modules](modules.md)** — Module system and imports
- **[Concurrency](concurrency.md)** — async/par with tokio/rayon
- **[Error Codes](../ERROR_CODES.md)** — Full error reference
