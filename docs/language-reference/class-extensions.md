# Class Extensions (`extend`)

> Status: 🆕 introduced in v2.1.
> Purpose: split a single class definition across multiple files so large classes
> remain mantenible without sacrificing encapsulation.

## TL;DR

```liva
// shapes.liva — owner: fields + constructor
Circle {
    radius: number
    constructor(r: number) { this.radius = r }
}

// shapes_area.liva — extension: just methods
import { Circle } from "./shapes"

extend Circle {
    area(): number {
        return 3.14159 * this.radius * this.radius
    }
}
```

`extend Circle { ... }` adds methods to an existing class. Code that imports
`Circle` from `./shapes` automatically sees `area()` too — Liva merges the
extension into the class at compile time.

## Why?

Some classes naturally grow large: the compiler's own `RustEmitter` has ~250
methods. Putting them in one file kills readability. Extensions let you:

- Group related methods together by file (`Circle` + `circle_serde.liva` +
  `circle_geometry.liva`).
- Keep field declarations and invariants in one place (the owner file).
- Avoid the "god class" anti-pattern without inheritance gymnastics.

Liva is opinionated: this is the **only** sanctioned way to split a class.
There is no inheritance, no mixins, no partial keyword soup.

## Rules

### 1. Methods only — no fields

```liva
extend Circle {
    color: string       // ✗ error E0905 — extensions cannot declare fields
    paint(): string { ... }   // ✓ ok
}
```

**Why?** Fields define memory layout. Liva compiles each class to one Rust
`struct`; allowing extensions to add fields would require either:
- a runtime hashmap (slow + boxed values), or
- compile-time field reordering across files (fragile + unpredictable size).

If you need more state, put it in the owner file.

### 2. The base class must be in scope

```liva
extend Circle { ... }   // ✗ E0906 if Circle isn't imported
```

You must `import { Circle } from "./shapes"` (or define `Circle` in the same
file). Liva does **not** do global name resolution — extension targets follow
the same scoping as any other identifier.

### 3. No duplicate methods

```liva
// shapes.liva
Circle {
    area(): number { return 1.0 }
}

// other.liva
extend Circle {
    area(): number { return 2.0 }   // ✗ E0907 — already defined
}
```

Duplicates are a compile-time error, never a silent override.

### 4. Same crate only

You can extend a class only from within the same project / library.
Cross-crate extensions are forbidden (analogous to Rust's orphan rules) to
prevent "spooky action at a distance" when consuming third-party code.

### 5. Method visibility & access

Methods on an extension are full members of the class:
- `this.<field>` reads/writes work, including private fields.
- Method dispatch is identical (`circle.area()`).
- No syntactic distinction at the call site.

## Compilation Model

For each `extend Foo { ... }` Liva emits an additional Rust `impl Foo { ... }`
block. The base class always emits its primary `impl Foo { ... }`. There is
zero runtime overhead — it's just sugar over Rust's natural support for
multiple `impl` blocks per type.

```liva
// shapes.liva
Circle {
    radius: number
    area(): number { return 3.14 * this.radius * this.radius }
}

// shapes_paint.liva
extend Circle {
    describe(): string { return $"Circle r={this.radius}" }
}
```

generates:

```rust
pub struct Circle { pub radius: f64 }

impl Circle {
    pub fn new(radius: f64) -> Self { Self { radius } }
    pub fn area(&self) -> f64 { 3.14 * self.radius * self.radius }
}

impl Circle {
    pub fn describe(&self) -> String {
        format!("Circle r={}", self.radius)
    }
}
```

## Error Codes

| Code  | Trigger                                                          |
| ----- | ---------------------------------------------------------------- |
| E0905 | Extension declares a field                                       |
| E0906 | Extension target class not in scope                              |
| E0907 | Duplicate method (already defined in base or another extension)  |
| E0908 | Extension declares a constructor                                 |

## File Naming Convention

Recommended (not enforced):

```
foo.liva          # owner — fields + constructor + core methods
foo_serde.liva    # extension — serialization methods
foo_io.liva       # extension — I/O methods
```

This makes ownership obvious at a glance and groups related extensions in the
directory listing.

## Comparison with Other Languages

| Language | Mechanism                  | Fields in ext? | Notes                            |
| -------- | -------------------------- | -------------- | -------------------------------- |
| Swift    | `extension X { ... }`      | ✗              | Closest match. Same philosophy.  |
| Rust     | multiple `impl T { ... }`  | ✗              | Liva compiles to this directly.  |
| C#       | `partial class X`          | ✓ (discouraged) | Liva intentionally stricter.    |
| Kotlin   | extension functions        | ✗              | Functions only, no methods.      |
| Go       | methods anywhere in pkg    | ✗              | No explicit syntax needed.       |

## When NOT to Use

- **For inheritance / polymorphism** — Liva has no inheritance. Use composition
  or traits/interfaces.
- **To "monkey-patch" stdlib types** — extensions only work for user-defined
  classes in your own project.
- **As a substitute for free functions** — if the method doesn't need `this`,
  prefer a top-level function.

## See Also

- [Classes: Basics](./classes-basics.md)
- [Modules](./modules.md)
- [Style Guide: Splitting Large Classes](../guides/style-guide.md)
