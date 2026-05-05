---
name: liva-lang
description: >
  Write code in Liva, a modern language that compiles to Rust. Use this skill
  whenever a user asks to create, edit, debug, or explain Liva (.liva) code.
  Covers syntax, types, error handling, concurrency, classes, enums, pattern
  matching, collections (arrays, maps, sets), modules, standard library, and
  testing. Read the references/ directory for deep-dive details on each topic.
---

# Liva Language — Quick Reference

Liva compiles to Rust. It has Python/TypeScript-like syntax with Rust performance. The compiler (`livac`) generates idiomatic Rust code compiled to native binaries. No `fn`/`def`/`class` keywords. No semicolons. No `++`.

**`main()` is auto-detected** — just define `main() { ... }` at top level. No need to call it. The compiler finds it and uses it as the entry point.

## CLI

```bash
livac build file.liva             # Compile to native binary
livac run file.liva               # Compile and run
livac run --release file.liva     # Release mode
livac check file.liva             # Syntax check only
livac fmt file.liva               # Format in place
livac test                        # Run *.test.liva files
livac test --filter "name"        # Filter tests
livac lint file.liva              # Linter warnings (W001-W004)
livac init my-project             # Scaffold new project
livac init .                      # Init in current directory
livac update                      # Self-update to latest version
livac build --verbose file.liva   # Show generated Rust
```

## Variables & Types

```liva
let x = 10                       // Mutable (type inferred)
let name: string = "Alice"       // Explicit type
const MAX: number = 100          // Immutable constant
let maybe: number? = null        // Optional
let nums: [number] = [1, 2, 3]  // Array
```

Primitives: `number` (i32), `float` (f64), `bool`, `string`, `char`, `bytes` (Vec<u8>). Aliases: `int` = `number`, `void` = `()`. Rust types available: `i8`–`i128`, `u8`–`u128`, `f32`, `f64`.

> **Note:** `number` = integer (i32). For decimal/float values, use `float` (f64). Do NOT use `number` for floating-point math — it will truncate. There is no generic "number" type that covers both.
>
> **Liva types vs Rust types — keep them straight.** In Liva source code always use lowercase types: `string`, `number`, `float`, `bool`, `bytes`. The capitalised `String`, `i32`, `f64`, `Vec<u8>` are Rust types that only appear in generated code or inside `rust { }` blocks. Do not write `let x: String = ...` or `let n: i32 = ...` in Liva.

### Destructuring

```liva
let [a, b] = [10, 20]
let [head, ...tail] = [1, 2, 3]
let {id, name} = user
```

## Functions

```liva
// Arrow (one-liner, implicit return)
add(a, b) => a + b
greet(name: string): string => $"Hello, {name}!"

// Block
calculate(items) {
    let total = 0
    for item in items { total = total + item.price }
    return total
}

// Default params
greet(name: string = "World") => $"Hello, {name}!"

// Tuple return (explicit type required)
getPoint(): (number, number) => (10, 20)

// Point-free references
items.forEach(print)
names.map(fmt::format)
```

Functions automatically become async if they contain `async` calls. Functions using `fail` are fallible — callers must handle errors.

## Control Flow

```liva
if age >= 18 { print("Adult") } else { print("Minor") }
if age >= 18 => print("Adult")          // One-liner
for i in 0..5 { print(i) }             // 0,1,2,3,4
for i in 1..=10 { print(i) }           // Inclusive
for name in names { print(name) }
for i, name in names { print($"{i}: {name}") }  // Enumerate
while running { tick() }
break / continue                         // Loop control
```

> **⚠️ `=>` on `if`/`for`/`while` does NOT imply return.** Unlike function arrow (`add(a, b) => a + b`), the one-liner `=>` on a control-flow statement just replaces `{ }`. You still need an explicit `return`, `fail`, etc.
>
> ```liva
> // ✅ explicit return required
> clamp(x: number, lo: number, hi: number): number {
>     if x < lo => return lo
>     if x > hi => return hi
>     return x
> }
>
> // ❌ this returns nothing — "lo" is just an expression statement
> if x < lo => lo
> ```

### Pattern Matching (switch expressions)

```liva
let grade = switch score {
    90..=100 => "A",
    80..=89 => "B",
    _ => "F"
}

// Enum destructuring
let msg = switch shape {
    Shape.Circle(r) => $"radius={r}",
    Shape.Rectangle(w, h) => $"{w}x{h}",
    Shape.Rectangle(_) => "some rectangle",  // _ wildcard ignores field
    Shape.Point => "point"
}

// Guards
let label = switch x {
    n if n > 100 => "big",
    n if n > 50 => "medium",
    _ => "small"
}

// Enum exhaustive switch (v2.0+) — omit _ when all variants covered
let name = switch color {
    Color.Red => "red"       // All 3 variants covered →
    Color.Green => "green"   // no _ needed
    Color.Blue => "blue"     // E0904 if any variant missing
}
```

Switch statements use `case X:` with colon. Switch expressions use `X => val` with arrow.

## Classes & Data Classes

```liva
// Class with constructor
Person {
    name: string
    age: number
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    greet() => $"Hi, I'm {this.name}"
}

// Data class (no constructor → auto-generated positional constructor + Display + PartialEq)
Point { x: number; y: number }
let p = Point(10, 20)               // Auto constructor
print(p == Point(10, 20))           // true (auto PartialEq)
```

## Enums

```liva
enum Color { Red, Green, Blue }
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}
let s = Shape.Circle(5)             // Dot syntax, NOT ::

// Enums get == / != automatically (PartialEq is derived). Reach for them
// before reaching for switch:
if color == Color.Red { ... }       // ✅ idiomatic
sameColor(a, b) => a == b           // ✅ idiomatic
isDone(s: Status): bool => s != Status.Done

// ⚠️ Enums do NOT auto-derive PartialOrd. `priority1 > priority2` will
// fail to compile. For ordering, define a weight function:
//     priorityWeight(p: Priority): number => switch p { ... }
//     if priorityWeight(a) > priorityWeight(b) { ... }

// Use switch only when you destructure fields, map every variant to a
// different value, or need guards/ranges. The arrow form keeps it compact:
statusLabel(s: Status): string => switch s {
    Status.Open => "open"
    Status.InProgress => "in-progress"
    Status.Done => "done"
}
```

## Interfaces & Visibility

```liva
// Interface = body with only method signatures (auto-detected)
Printable { display(): string }
Dog : Printable { name: string; constructor(name: string) { this.name = name }; display() => this.name }
```

`_` prefix = private. No `public`/`private` keywords.

## Error Handling

```liva
// fail makes a function fallible
divide(a: number, b: number): number {
    if b == 0 { fail "Division by zero" }
    return a / b
}

// Error binding (REQUIRED for fallible calls)
let result, err = divide(10, 0)
if err { print($"Error: {err}") }    // Always use `if err {`, NOT `if err != ""`

// IMPORTANT: `err` is internally an Option<Error> with fields:
//   err.message → plain string (just the message text)
//   print(err) → full trace with function names, file locations, and chained causes
// Always check with `if err {` (truthy when error exists), NEVER `if err != ""`

// Shorthand: or fail (propagate)
let data = File.read("f.txt") or fail "Cannot read"

// Shorthand: or <default> (fallback)
let port = parseInt("abc") or 3000

// Unwrap operator (!) — force-unwrap, panics if null
let user = find_user("admin")   // string?
print(user!)                    // "Admin" — panics if null

// Optional chaining (?.) — safe field access on nullable values
let name = user?.name            // string? — null if user is null
let safe = user?.name or "Guest" // string — with fallback
```

## Defer

```liva
// defer registers cleanup that runs when scope exits (LIFO order)
defer DB.close(db)           // Single expression
defer print("goodbye")       // Runs at end of scope, no matter what

// Block form
defer {
    print("cleaning up")
    File.close(handle)
}

// Multiple defers: last registered = first to run (stack/LIFO)
defer print("3rd")
defer print("2nd")
defer print("1st")
// Output: 1st, 2nd, 3rd
```

## Concurrency

```liva
let user = async fetchUser(1)       // Spawns tokio task NOW (doesn't block)
let result = par heavyCalc(1000)    // Spawns thread NOW (doesn't block)

doOtherWork()                       // Runs concurrently with both

print(user)                         // Auto-awaits here on first use of variable
print(result)                       // Auto-joins here on first use of variable

// Explicit task handles
let t1 = task async fetchUser(1)
let u1 = await t1

// Fire-and-forget (not assigned to variable)
async logEvent("login")

// Parallel arrays
let doubled = numbers.par().map(x => x * 2)
```

> **Picking a concurrency primitive:**
>
> | Goal | Use | Example |
> |------|-----|---------|
> | Multiple concurrent I/O calls | `async` (one per call) | `let a = async HTTP.get(u1); let b = async HTTP.get(u2); print(a); print(b)` |
> | Overlap I/O with other work | `async` + work in between | `let r = async HTTP.get(url); let local = readCache(); print(r)` |
> | CPU-bound work in background | `par` | `let r = par heavy_calc(input)` |
> | Map/filter a collection in parallel | `.par().map()` adapter | `nums.par().map(f)` |
> | Tuned parallel loop (chunks, thread count) | `for par … with …` | `for par x in xs with threads 4 { … }` |
> | Fire-and-forget side effect | bare `async`, no binding | `async logEvent("login")` |
> | Single I/O call, result used immediately | plain call (no keyword) | `let resp, err = HTTP.get(url)` |
> | Default sequential | plain `.map()` / `for` | `nums.map(f)` |
>
> **Don't reach for `async` on every I/O call.** A single `let r = async HTTP.get(url); print(r)` is no faster than the plain call — the task spawn just adds overhead. `async` only pays off when something else can run while the task is in flight (another `async` call, sync work between spawn and use, or fire-and-forget).
>
> **Auto-async propagation:** any function that calls an async function (like `HTTP.get`) automatically becomes async itself, so you can call it without the `async` keyword and it will be awaited at the use site.

## Collections

### Arrays

```liva
let nums = [1, 2, 3]
nums.push(4)                         // Mutates in place
nums = nums + [4]                    // Or concatenation (new array)
nums.map(x => x * 2)                // [2, 4, 6]
nums.filter(x => x > 1)             // [2, 3]
nums.reduce(0, (acc, x) => acc + x) // Initial value FIRST
nums.forEach(x => print(x))
nums.find(x => x > 2)               // Some(3)
nums.some(x => x > 2)               // true
nums.every(x => x > 0)              // true
nums.includes(3)                     // true
nums.indexOf(2)                      // 1
["a", "b"].join(", ")               // "a, b"

// v1.4 — Access & slicing
nums.first() / nums.last() / nums.isEmpty()
nums.slice(1, 3) / nums.take(2) / nums.drop(1)

// v1.4 — Transform
nums.sort() / nums.reversed() / nums.distinct()
[[1,2],[3]].flat() / nums.chunks(2) / nums.zip([4,5,6])

// v1.4 — Aggregate
nums.sum() / nums.min() / nums.max()

// v1.4 — Callback
nums.findIndex(x => x > 2) / nums.flatMap(x => [x, x*10]) / nums.count(x => x > 1)
```

### Maps (v1.3.0)

```liva
let ages = Map { "Alice": 30, "Bob": 25 }
let age = ages.get("Alice") or 0     // 30
ages.set("Carol", 28)
ages.has("Bob")                      // true
ages.delete("Bob")
for key, value in ages { print($"{key}: {value}") }
ages.keys() / ages.values() / ages.entries()
```

### Sets (v1.3.0)

```liva
let colors = Set { "red", "green", "blue" }
colors.add("yellow")
colors.has("red")                    // true
colors.delete("green")
let u = a.union(b)
let i = a.intersection(b)
let d = a.difference(b)
for color in colors { print(color) }
```

## Strings

> The blocks below show the most common methods. Full signatures, edge cases, and the complete catalogue (28 string methods, 31 array methods, 14 math functions, etc.) live in `references/stdlib/`. Treat the lists here as a starting point, not a complete inventory.

```liva
let msg = $"Hello, {name}! Sum: {a + b}"    // String templates
text.split(", ") / text.trim() / text.toUpperCase() / text.toLowerCase()
text.replace("a", "b") / text.replaceAll("a", "b") / text.contains("x")
text.substring(0, 5) / text.slice(0, 5) / text.charAt(0)
text.startsWith("H") / text.endsWith("!") / text.indexOf("W") / text.lastIndexOf("W")
text.padStart(5, "0") / text.padEnd(5, ".") / text.repeat(3)
text.capitalize() / text.reverse() / text.truncate(10)
text.isBlank() / text.isEmpty() / text.countMatches("x")
text.removePrefix("pre_") / text.removeSuffix(".txt") / text.chars()
// Escape braces: $"\{\"key\": \"{val}\"\}"
```

## Type Aliases

```liva
type TokenList = [TokenWithSpan]
type Result<T> = (T, error)
type Handler = (Request): Response
```

## Modules & Imports

```liva
import { add, subtract } from "./math.liva"
import { add, subtract } from "./math"       // Extension optional (v2.0+)
import * as math from "./math"
// Paths relative to importing file
// _prefix = private (not exported)
```

## Standard Library

```liva
// Console
print("Hello") / console.log(data) / console.error("err") / console.warn("warn")
let input = console.input("Name: ")

// Math
Math.PI / Math.E / Math.sqrt(16.0) / Math.pow(2.0, 3.0) / Math.abs(-10.5)
Math.floor(3.7) / Math.ceil(3.2) / Math.round(3.5) / Math.random()
Math.min(a, b) / Math.max(a, b) / Math.clamp(val, 0, 10)
Math.sign(-42) / Math.log(2.718)

// Type conversion (fallible)
let num, err = parseInt("42")
let val, err = parseFloat("3.14")
let str = toString(42)

// File I/O (error binding except File.exists and File.extension)
let content, err = File.read("file.txt")
File.write("out.txt", "data") / File.append("log.txt", "line\n") / File.delete("tmp")
File.exists("file.txt")                // bool, no error binding
File.copy("src", "dst") / File.move("old", "new")   // (bool, error)
let bytes, err = File.size("f.txt")    // (int, error)
let ext = File.extension("f.jpg")      // "jpg" — string, no error binding
let lines, err = File.readLines("f.txt")   // ([string], error)
File.writeLines("f.txt", ["a", "b"])       // (bool, error)

// Directory
let entries, err = Dir.list("/path")   // [string] sorted
Dir.isDir("/path")                     // bool, no error binding
Dir.exists("/path")                    // bool — true only if dir
let ok, err = Dir.create("./a/b/c")   // mkdir -p (recursive)
let ok, err = Dir.delete("./tmp")     // rm -rf (recursive)
let files, err = Dir.listRecursive("./src")  // All files, relative paths
let files, err = Dir.walk("./docs")          // Alias for listRecursive

// Regex (crate `regex` auto-injected)
Regex.test("\\d+", text)                   // bool
let found, err = Regex.match("\\d+", text) // (string, error) — first match
Regex.findAll("\\d+", "a1b22")             // ["1", "22"] — all matches
Regex.replace("\\s+", text, " ")           // string — replace all
Regex.split("[,;]", "a,b;c")              // ["a", "b", "c"]

// Date (crate `chrono` auto-injected)
let now = Date.now()                               // Current date/time
let birthday = Date.new(1990, 6, 15)               // Specific date
let parsed, err = Date.parse("2026-03-11", "YYYY-MM-DD")
let ts = Date.timestamp()                          // Unix epoch ms (int)
now.year / now.month / now.day / now.hour          // Properties → int
now.format("DD/MM/YYYY")                           // → string
let nextWeek = now.add(7, "days")                  // → Date
let age = now.diff(birthday, "years")              // → int
now.toString()                                     // → "2026-03-23T14:30:00"
if nextWeek > now { print($"Future: {nextWeek}") } // Comparisons + interpolation

// CSV (pure Rust std, no external crates) — Table = [Map<string, string>]
let rows, err = CSV.read("data.csv")               // [[string]] — raw rows
let table, err = CSV.readTable("data.csv")          // [Map<string, string>] — first row as headers
CSV.write("out.csv", rows)                          // Write [[string]] to file
CSV.writeTable("out.csv", table)                    // Write table with headers
let parsed = CSV.parse(csvText)                     // String → [[string]]
let text = CSV.stringify(rows)                      // [[string]] → CSV string
let hdrs = CSV.headers(table)                       // → [string] header names
let col = CSV.column(table, "name")                 // → [string] column values

// Random (crates rand + uuid auto-injected)
let n = Random.nextInt(1, 100)                      // int in [min, max]
let f = Random.nextFloat(0.0, 1.0)                  // float in [min, max] (args optional)
let pick = Random.choice(["a", "b", "c"])           // Random element
let mixed = Random.shuffle([1, 2, 3])               // Shuffled copy
let id = Random.uuid()                              // UUID v4 string

// Crypto (crates sha2, md-5, base64 auto-injected)
let hash = Crypto.sha256("hello")                    // Hex SHA-256
let md = Crypto.md5("hello")                         // Hex MD5
let enc = Crypto.base64Encode("hello")               // Base64 encode
let dec, err = Crypto.base64Decode(enc)              // Fallible decode

// Process (std::process, no external crates)
let output, err = Process.exec("ls -la")             // Run cmd, capture stdout
let pid, err = Process.spawn("sleep 10")             // Background process
let myPid = Process.pid()                            // Current PID
Process.exit(0)                                      // Exit with code

// System
Sys.args()                               // [string] — args[0] = program name, args[1..] = user args
Sys.env("HOME")                           // Get env variable
Sys.exit(1)                               // Exit with code

// Logging (stderr, timestamped)
Log.info("msg", arg1, arg2)            // Variadic args, concatenated with spaces
Log.warn("warning") / Log.error("err") / Log.debug("detail")  // debug only with --verbose
Log.setLevel("debug")                  // debug/info/warn/error
// Map 4+ keys → Key/Value table, ≤3 keys → inline {k: v}
// Array<Map> → columnar table (console.table style)
// JSON.parse results → runtime auto-detection for table rendering

// JSON
let data: User, err = JSON.parse(jsonStr)
let json = JSON.stringify(obj)

// HTTP (HTTP.get / .post / .put / .delete are async functions —
// the caller becomes async automatically, no `async` keyword needed
// for a single call. Use `async` only to overlap requests; see § Concurrency.)
let resp, err = HTTP.get(url)
let resp, err = HTTP.post(url, body)              // Also: .put(), .delete()
resp.status / resp.body / resp.json()

// DB — SQLite (crate rusqlite bundled, auto-injected)
let db, err = DB.open("myapp.db")                    // Open/create database
let _, err2 = DB.exec(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
let _, err3 = DB.exec(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])  // Parameterized
let rows, err4 = DB.query(db, "SELECT * FROM users")  // → [Map<string, string>]
let results, err5 = DB.query(db, "SELECT * FROM users WHERE name = ?", ["Alice"])
for row in rows {
    print("Name: " + row.get("name"))                // Auto-unwraps in string context
    let name = row.get("name") or "unknown"          // Explicit default
}
DB.close(db)                                          // Close connection
```

## HTTP Server *(axum)*

```liva
let app = Server.create()

app.get("/hello", (req) => {
    Response.text("Hello, World!")
})

app.post("/users", (req) => {
    let body = req.body
    Response.json(body)
})

app.put("/users/:id", (req) => {
    let id = req.params.get("id")
    Response.text("Updated " + id)
})

app.delete("/users/:id", (req) => {
    let id = req.params.get("id")
    Response.status(204)
})

app.listen(3000)
// Routes: app.get/post/put/delete(path, handler)
// Request: req.params.get("name"), req.body
// Response: Response.text(str), Response.json(data), Response.json(data, 201), Response.status(code)
```

## Config *(v1.5.0)*

```liva
let config, err = Config.load(".env")
let host, err = Config.get(config, "HOST")
let port, err = Config.getInt(config, "PORT")
let debug, err = Config.getBool(config, "DEBUG")
let all = Config.getAll(config)   // Map<string, string> sorted
```

## Rust Interop *(v1.5.0)*

```liva
// Inline Rust code as expression
let result = rust {
    let x: i32 = 42;
    x * 2
}

// Crate dependencies (top-level)
use rust "chrono" version "0.4"
use rust "uuid" version "1.0" features ["v4", "serde"]
use rust "tokio" features ["net"]   // Merges with built-in features

// use statements inside rust { } are hoisted to file top
let hash = rust {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert("key", "value");
    map.len()
}
// Internal crates (always available): tokio, serde, serde_json, reqwest, rayon, rand
// E9002: Cannot override internal crate version — only add features
// Liva names are snake_case in generated Rust: myValue → my_value
// No semantic validation of rust block content — errors come from rustc
```

### Rust Interop Details

- **Snake_case transform**: Liva identifiers like `myValue` become `my_value` in Rust. Use `my_value` inside `rust { }` blocks to reference Liva variables.
- **Result types**: Fallible Liva functions generate `Result<T, String>`. Inside `rust { }`, return `Ok(value)` or `Err("message".to_string())`.
- **Liva vars in Rust blocks**: Variables defined in Liva are accessible in `rust { }` blocks by their snake_case name. String vars are `String` type, numbers are `i32`, floats are `f64`.
- **Hyphenated crate names**: `use rust "my-crate"` automatically converts to `my_crate` in Rust imports.

## Testing

```liva
import { describe, test, expect } from "liva/test"

describe("Math", () => {
    test("add", () => {
        expect(add(2, 3)).toBe(5)
        expect(add(-1, 1)).not.toBe(2)
    })
})
// Matchers: toBe, toEqual, toBeTruthy, toBeFalsy, toBeGreaterThan, toBeLessThan,
//           toContain, toBeNull, toThrow, .not.*
// Hooks: beforeAll, afterAll, beforeEach, afterEach
// Run: livac test / livac test --filter "name"
```

## Critical Rules

1. **No `fn`/`def`/`class` keyword** — just write `add(a, b) => a + b`
2. **No semicolons** — newline terminates statements
3. **No `++`** — use `x += 1` or `x = x + 1` (compound assignment `+=  -=  *=  /=  %=` supported)
4. **Error binding required** — `let val, err = riskyCall()` (E0701)
5. **`if err {`** — NOT `if err != ""`
6. **Enum dot syntax** — `Color.Red`, NOT `Color::Red`
7. **`reduce` initial value FIRST** — `.reduce(0, (acc, x) => acc + x)`
8. **Array growth** — `arr.push(x)` (mutates) or `arr = arr + [x]` (new array)
9. **String templates** — `$"text {expr}"` (NOT backticks)
10. **`_` prefix = private** — `_helper()`, `_password: string`
11. **Multi-file projects** — split by responsibility for >50 lines (see references/)
12. **`or fail`** — prefer over verbose error binding for propagation
13. **`defer` for cleanup** — `defer DB.close(db)` right after opening; never forget cleanup
14. **`=>` on if/for/while ≠ implicit return** — only the *function* arrow returns; control-flow `=>` just replaces `{ }`. Keep `return`, `fail`, etc.
15. **`%` is remainder, not modulo** — `-5 % 3 == -2`. For true modulo: `((a % b) + b) % b`.
16. **Enums auto-derive `PartialEq`, NOT `PartialOrd`** — `==`/`!=` work; `<`/`>` don't.
17. **Liva types are lowercase** — `string`, `number`, `float`, `bool`, `bytes`. `String`/`i32`/`f64` are Rust-only.

### Reserved Keywords (cannot use as identifiers)

These are reserved in Liva and will cause parse errors if used as variable/function names:

```
let const import from as if else while for in switch case default return
break continue fail throw try catch async par parallel task await move seq defer
vec parvec with ordered chunk threads enum type use rust test true false null
and or not safe fast static dynamic auto detect schedule reduction prefetch simdWidth
number float bool char string bytes
```

Additionally, avoid Rust reserved words as field/method names: `type`, `match`, `mod`, `self`, `super`, `crate`, `impl`, `trait`, `pub`, `fn`, `struct`, `where`, `loop`, `ref`, `mut`, `dyn`, `abstract`, `yield`. The compiler escapes some of these (e.g., `type` → `r#type`), but it's best to use alternatives (e.g., `kind` instead of `type`).

## References

For detailed docs, read files in `references/`. Key files:

- `references/pattern-matching.md` — Range/or/guard/tuple/enum patterns, exhaustiveness
- `references/enums.md` — Enums with data, recursive enums, exhaustive switch
- `references/collections.md` — Maps, Sets, parallel execution policies
- `references/concurrency.md` — async, par, task, fire-and-forget, parallel arrays
- `references/rust-interop.md` — `rust { }` blocks, crate deps, snake_case rules
- `references/modules.md` — Import paths, wildcard imports, visibility
- `references/file-io.md` — File and Dir methods with signatures
- `references/linter.md` — W001-W004 warning codes
- `references/style-guide.md` — Idiomatic patterns, `=>` vs `{}`, naming
- `references/stdlib/` — Per-module API: arrays, strings, math, date, regex, csv, db, server, etc.
