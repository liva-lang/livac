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

// Error binding (REQUIRED for fallible calls — E0701)
let result, err = divide(10, 0)
if err { print($"Error: {err}") }    // ✅ always `if err {` — NEVER `if err != ""`

// `err` is Option<Error>:
//   err.message → plain message string
//   print(err)  → full trace (causes + locations)

// Shorthands
let data = File.read("f.txt") or fail "Cannot read"   // propagate
let port = parseInt("abc") or 3000                     // fallback

// Optional value operators (only on T?)
let user = find_user("admin")    // string?
let name = user!                  // unwrap, panics if null
let name = user?.name             // optional chaining → string?
let safe = user?.name or "Guest"  // chain + fallback → string
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

Full catalogue (31 methods) in `references/stdlib/arrays.md`. Most-used:

```liva
let nums = [1, 2, 3]
nums.push(4)                         // Mutates in place
nums = nums + [4]                    // Or concatenation (new array)

// Higher-order
nums.map(x => x * 2) / nums.filter(x => x > 1) / nums.forEach(print)
nums.reduce(0, (acc, x) => acc + x)  // ⚠️ initial value FIRST
nums.find(p) / nums.findIndex(p) / nums.some(p) / nums.every(p)
nums.flatMap(x => [x, x*10]) / nums.count(p)

// Access / slicing / aggregate
nums.first() / nums.last() / nums.isEmpty() / nums.length
nums.slice(i, j) / nums.take(n) / nums.drop(n) / nums.chunks(n)
nums.includes(x) / nums.indexOf(x) / ["a","b"].join(", ")
nums.sort() / nums.reversed() / nums.distinct()
nums.sum() / nums.min() / nums.max()
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

```liva
let msg = $"Hello, {name}! Sum: {a + b}"    // String templates (NOT backticks)
// Escape braces: $"\{\"key\": \"{val}\"\}"
```

Most-used methods (full catalogue — 28 methods — in `references/stdlib/strings.md`):

```liva
text.split(sep) / text.trim() / text.toUpperCase() / text.toLowerCase()
text.replace(a, b) / text.replaceAll(a, b) / text.contains(s)
text.substring(i, j) / text.charAt(i)
text.startsWith(s) / text.endsWith(s) / text.indexOf(s)
text.padStart(n, c) / text.padEnd(n, c) / text.repeat(n)
text.isBlank() / text.isEmpty()
text.removePrefix(p) / text.removeSuffix(s)
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

## Standard Library — Map

The stdlib is large. **Always read the matching `references/stdlib/*.md` file before generating non-trivial code with one of these modules.** This section is just a locator + the most common entry points.

| Module | What it does | Where to look | Most-used calls |
|--------|--------------|---------------|-----------------|
| `print`, `console` | stdout/stderr | (built-in) | `print(x)`, `console.error(msg)`, `console.input("Name: ")` |
| `Math` | Numeric functions | `references/stdlib/math.md` | `Math.PI`, `Math.sqrt(x)`, `Math.pow(b,e)`, `Math.abs(x)`, `Math.floor/ceil/round`, `Math.min/max`, `Math.clamp(v,lo,hi)`, `Math.random()` |
| `parseInt`, `parseFloat`, `toString` | Conversions (fallible except `toString`) | `references/stdlib/conversions.md` | `let n, err = parseInt(s)` |
| `File` | File I/O | `references/stdlib/io.md` | `let c, err = File.read(p)`, `File.write(p, s)`, `File.exists(p)`, `File.append(p, s)`, `File.readLines(p)` |
| `Dir` | Directory I/O | `references/stdlib/io.md` | `Dir.list(p)`, `Dir.create(p)` (mkdir -p), `Dir.delete(p)` (rm -rf), `Dir.listRecursive(p)` |
| `Regex` | Regex (crate `regex`) | `references/stdlib/regex.md` | `Regex.test(re, s)`, `Regex.findAll(re, s)`, `Regex.replace(re, s, repl)`, `Regex.split(re, s)` |
| `Date` | Date/time (crate `chrono`) | `references/stdlib/date.md` | `Date.now()`, `Date.new(y,m,d)`, `now.format("DD/MM/YYYY")`, `now.add(7, "days")`, `now.diff(other, "years")` |
| `CSV` | CSV read/write | `references/stdlib/csv.md` | `CSV.read(p)`, `CSV.readTable(p)` (first row = headers), `CSV.write(p, rows)`, `CSV.writeTable(p, table)` |
| `Random` | Random + UUID (crates `rand`, `uuid`) | `references/stdlib/random.md` | `Random.nextInt(lo, hi)`, `Random.choice(arr)`, `Random.shuffle(arr)`, `Random.uuid()` |
| `Crypto` | Hash + base64 | `references/stdlib/crypto.md` | `Crypto.sha256(s)`, `Crypto.md5(s)`, `Crypto.base64Encode/Decode` |
| `Process` | Subprocess + current PID | `references/stdlib/process.md` | `Process.exec(cmd)` (capture stdout), `Process.spawn(cmd)` (background), `Process.pid()`, `Process.exit(code)` |
| `Sys` | Args + env | `references/stdlib/system.md` | `Sys.args()` (`args[0]` = program), `Sys.env(name)`, `Sys.exit(code)` |
| `Log` | Stderr logger (timestamps + table rendering) | `references/stdlib/logging.md` | `Log.info(msg, ...)`, `Log.warn/error/debug`, `Log.setLevel("debug")` |
| `JSON` | Parse/stringify (typed parsing supported) | `references/json-basics.md` | `let data: User, err = JSON.parse(s)`, `JSON.stringify(obj)` |
| `HTTP` | Async HTTP client | `references/stdlib/io.md`, `references/concurrency.md` | `let resp, err = HTTP.get(url)` — also `.post(url, body)`, `.put`, `.delete`. `resp.status` / `resp.body` / `resp.json()` |
| `DB` | SQLite (crate `rusqlite`, bundled) | `references/stdlib/db.md` | `DB.open(path)`, `DB.exec(db, sql, params)`, `DB.query(db, sql, params)` → `[Map<string,string>]`, `DB.close(db)` |
| `Server` + `Response` | HTTP server (axum) | `references/stdlib/server.md` | `Server.create()`, `app.get/post/put/delete(path, (req) => ...)`, `Response.text(s)` / `.json(data)` / `.status(code)`, `app.listen(port)` |
| `Config` | `.env` loader | `references/stdlib/config.md` | `let cfg, err = Config.load(".env")`, `Config.get/getInt/getBool(cfg, key)`, `Config.getAll(cfg)` |

> **Always use error binding for fallible calls** (most stdlib I/O is fallible): `let val, err = Module.call(...); if err { ... }`. Exceptions like `File.exists` and `File.extension` return their value directly — see the per-module reference.

## Rust Interop *(v1.5+)*

```liva
// Inline Rust as an expression
let result = rust {
    let x: i32 = 42;
    x * 2
}

// Crate dependencies (top-level)
use rust "chrono" version "0.4"
use rust "uuid" version "1.0" features ["v4", "serde"]

// `use std::...;` inside rust { } is hoisted to the file top
```

- **Snake_case transform**: a Liva identifier `myValue` is `my_value` inside `rust { }`.
- **Internal crates** (always available, do not redeclare): `tokio`, `serde`, `serde_json`, `reqwest`, `rayon`, `rand`. Adding `features` is OK; overriding `version` triggers E9002.
- **Hyphenated crate names** convert to underscores: `"my-crate"` → `my_crate` in `use`.
- **Result types in Rust blocks**: Liva-fallible functions compile to `Result<T, String>`. Inside `rust { }` you can `return Ok(v)` or `Err("...".to_string())`. Outside, prefer `fail` from Liva.
- No semantic validation of `rust { }` content — errors surface from `rustc`.

See `references/rust-interop.md` for full details.

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

### Reserved Keywords

Language keywords (cannot be identifiers): `let const import from as if else while for in switch case default return break continue fail throw try catch async par parallel task await move seq defer vec parvec with ordered chunk threads enum type use rust test true false null and or not safe fast static dynamic auto detect schedule reduction prefetch simdWidth number float bool char string bytes`.

Also avoid Rust reserved words as field/method names (`type`, `match`, `mod`, `self`, `super`, `crate`, `impl`, `trait`, `pub`, `fn`, `struct`, `where`, `loop`, `ref`, `mut`, `dyn`, `abstract`, `yield`). Some are escaped automatically (`type` → `r#type`); prefer alternatives like `kind`.

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
