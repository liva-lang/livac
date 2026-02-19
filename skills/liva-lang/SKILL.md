---
name: liva-lang
description: Complete language reference for Liva — a modern programming language that compiles to Rust. Covers syntax, types, error handling, concurrency, classes, enums, pattern matching, and standard library.
---

# Liva Language

Liva is a modern programming language that compiles to Rust, combining Python's simplicity, TypeScript's clarity, and Rust's performance. The compiler (`livac`) generates idiomatic Rust code that is then compiled to native binaries.

## Key Characteristics

- **Compiles to Rust** — generates `.rs` files + `Cargo.toml`, then `cargo build`
- **Type inference** — types are inferred but can be annotated explicitly
- **Explicit error handling** — `fail` keyword, error binding, `or fail` propagation
- **Hybrid concurrency** — `async`/`await` for I/O + `par`/`parallel` for CPU
- **No semicolons** — newline-terminated statements
- **No `fn`/`def` keyword** — functions declared by name directly

## Essential Syntax Summary

```liva
// Variables
let x = 42
let name: string = "Liva"
const MAX = 100

// Functions
add(a: number, b: number): number => a + b
process(items: [string]) {
    for item in items {
        print(item)
    }
}

// Classes
Person {
    name: string
    age: number
    
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    
    greet() => $"Hello, I'm {this.name}"
}

// Enums (algebraic data types)
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}

// Pattern matching
area(s: Shape): number {
    return switch s {
        Shape.Circle(r) => 3 * r * r
        Shape.Rectangle(w, h) => w * h
        Shape.Point => 0
    }
}

// Error handling
readConfig(path: string): string {
    let content, err = File.read(path)
    if err {
        fail "Cannot read config: " + err
    }
    return content
}

// Concurrency
fetchData(url: string): string {
    let response, err = async HTTP.get(url)
    return response or fail
}
```

## Documentation Navigation

For detailed information, consult the following files based on the topic:

### Core Language
- **Variables, constants, scoping** → `../../docs/language-reference/variables.md`
- **Functions (basics, arrow syntax, block functions)** → `../../docs/language-reference/functions-basics.md`
- **Functions (async, fallibility, references, advanced)** → `../../docs/language-reference/functions-advanced.md`
- **Operators (arithmetic, logical, comparison, bitwise)** → `../../docs/language-reference/operators.md`
- **Control flow (if/else, while, for, switch, loops)** → `../../docs/language-reference/control-flow.md`

### Types & Data Structures
- **Primitive types, type inference, annotations** → `../../docs/language-reference/types-primitives.md`
- **Advanced types (tuples, unions, type aliases, Rust native types)** → `../../docs/language-reference/types-advanced.md`
- **Generics (basics, type parameters, constraints)** → `../../docs/language-reference/generics-basics.md`
- **Generics (advanced patterns, multiple constraints)** → `../../docs/language-reference/generics-advanced.md`
- **Enums (algebraic data types, variants, pattern matching)** → `../../docs/language-reference/enums.md`

### Object-Oriented Programming
- **Classes (declaration, constructors, fields, methods)** → `../../docs/language-reference/classes-basics.md`
- **Interfaces (declaration, implementation, multiple)** → `../../docs/language-reference/classes-interfaces.md`
- **Data classes (auto-derive sugar, visibility)** → `../../docs/language-reference/classes-data.md`

### Error Handling & Pattern Matching
- **Error handling (fail, error binding, or fail, try/catch)** → `../../docs/language-reference/error-handling.md`
- **Pattern matching (switch expressions, destructuring, or-patterns)** → `../../docs/language-reference/pattern-matching.md`

### Concurrency
- **Async/await, parallel, tasks, fire** → `../../docs/language-reference/concurrency.md`

### Standard Library
- **Arrays (map, filter, reduce, forEach, find, push, pop, join)** → `../../docs/language-reference/stdlib/arrays.md`
- **Strings (split, replace, trim, toUpper, toLower, includes)** → `../../docs/language-reference/stdlib/strings.md`
- **Math (sqrt, pow, abs, random, PI, E)** → `../../docs/language-reference/stdlib/math.md`
- **Type conversions (parseInt, parseFloat, toString)** → `../../docs/language-reference/stdlib/conversions.md`
- **I/O (print, console.log/error/warn)** → `../../docs/language-reference/stdlib/io.md`

### APIs
- **JSON (parse, stringify, typed parsing)** → `../../docs/language-reference/json-basics.md`
- **JSON (error handling, type mapping, advanced)** → `../../docs/language-reference/json-advanced.md`
- **HTTP (get, post, put, delete, async requests)** → `../../docs/language-reference/http.md`
- **File I/O (read, write, exists, delete)** → `../../docs/language-reference/file-io.md`
- **String templates ($"..." interpolation)** → `../../docs/language-reference/string-templates.md`

### Project Structure
- **Modules and imports (multi-file projects)** → `../../docs/language-reference/modules.md`
- **Visibility (public/private via _ prefix)** → `../../docs/language-reference/visibility.md`
- **Collections (arrays, objects, iteration)** → `../../docs/language-reference/collections.md`

### Quick Reference
- **Complete syntax cheat sheet (all features on one page)** → `../../docs/QUICK_REFERENCE.md`

## Common Pitfalls

1. **No `fn`/`def` keyword** — write `add(a, b) => a + b` not `fn add(a, b)`
2. **No semicolons** — newline terminates statements
3. **`and`/`or`/`not`** — use these instead of `&&`/`||`/`!` (both work, but Liva style prefers words)
4. **Switch expressions vs statements** — expressions use `X => val`, statements use `case X:` with colon
5. **Error binding** — `let value, err = riskyCall()` captures both value and error
6. **`or fail`** — shorthand for propagating errors: `let x = riskyCall() or fail`
7. **String templates** — use `$"text {expr}"` not backticks
8. **Private members** — prefix with `_` (e.g., `_count: number`)
9. **`describe`** is reserved for the test framework — don't use as a function name
10. **Enum construction** — use `Color.Red`, `Shape.Circle(5)` (dot syntax, not `::`)
