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

Liva compiles to Rust. It has Python/TypeScript-like syntax with Rust performance. The compiler (`livac`) generates idiomatic Rust code compiled to native binaries. No `fn`/`def`/`class` keywords. No semicolons. No `+=`/`++`.

## CLI

```bash
livac file.liva --run            # Compile and run
livac file.liva --release --run  # Release mode
livac file.liva --check          # Syntax check only
livac file.liva --fmt            # Format in place
livac --test                     # Run *.test.liva files
livac --test --filter "name"     # Filter tests
livac file.liva --verbose        # Show generated Rust
```

## Variables & Types

```liva
let x = 10                       // Mutable (type inferred)
let name: string = "Alice"       // Explicit type
const MAX: number = 100          // Immutable constant
let maybe: number? = null        // Optional
let nums: [number] = [1, 2, 3]  // Array
```

Primitives: `number` (i32), `float` (f64), `bool`, `string`, `char`. Rust types available: `i8`–`i128`, `u8`–`u128`, `f32`, `f64`.

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
getPoint(): (int, int) => (10, 20)

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
while running { tick() }
break / continue                         // Loop control
```

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
    Shape.Point => "point"
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

// Shorthand: or fail (propagate)
let data = File.read("f.txt") or fail "Cannot read"

// Shorthand: or <default> (fallback)
let port = parsePort("abc") or 3000

// Error traces are automatic — print(err) shows full trace with function names and lines
// Use err.message for plain text without trace
```

## Concurrency

```liva
let user = async fetchUser(1)       // I/O-bound (auto-awaited on use)
let result = par heavyCalc(1000)    // CPU-bound (auto-joined on use)

// Explicit task handles
let t1 = task async fetchUser(1)
let u1 = await t1

// Fire-and-forget (not assigned to variable)
async logEvent("login")

// Parallel arrays
let doubled = numbers.par().map(x => x * 2)
```

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
let msg = $"Hello, {name}! Sum: {a + b}"    // String templates
text.split(", ") / text.trim() / text.toUpperCase() / text.toLowerCase()
text.replace("a", "b") / text.contains("x") / text.substring(0, 5)
text.startsWith("H") / text.endsWith("!") / text.indexOf("W")
// Escape braces: $"\{\"key\": \"{val}\"\}"
```

## Modules & Imports

```liva
import { add, subtract } from "./math.liva"
import * as math from "./math.liva"
// Paths relative to importing file, must end with .liva
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
Math.min(a, b) / Math.max(a, b)

// Type conversion (fallible)
let num, err = parseInt("42")
let val, err = parseFloat("3.14")
let str = toString(42)

// File I/O (error binding except File.exists)
let content, err = File.read("file.txt")
File.write("out.txt", "data") / File.append("log.txt", "line\n") / File.delete("tmp")
File.exists("file.txt")                // bool, no error binding

// Directory
let entries, err = Dir.list("/path")   // [string] sorted
Dir.isDir("/path")                     // bool, no error binding

// System
Sys.args() / Sys.env("HOME") / Sys.exit(1)

// JSON
let data: User, err = JSON.parse(jsonStr)
let json = JSON.stringify(obj)

// HTTP (async)
let resp, err = async HTTP.get(url)
let resp, err = async HTTP.post(url, body)        // Also: .put(), .delete()
resp.status / resp.body / resp.json()
```

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
// Run: livac --test / livac --test --filter "name"
```

## Critical Rules

1. **No `fn`/`def`/`class` keyword** — just write `add(a, b) => a + b`
2. **No semicolons** — newline terminates statements
3. **No `+=`/`++`** — use `x = x + 1`
4. **Error binding required** — `let val, err = riskyCall()` (E0701)
5. **`if err {`** — NOT `if err != ""`
6. **Enum dot syntax** — `Color.Red`, NOT `Color::Red`
7. **`reduce` initial value FIRST** — `.reduce(0, (acc, x) => acc + x)`
8. **Array growth** — `arr.push(x)` (mutates) or `arr = arr + [x]` (new array)
9. **String templates** — `$"text {expr}"` (NOT backticks)
10. **`_` prefix = private** — `_helper()`, `_password: string`
11. **Multi-file projects** — split by responsibility for >50 lines (see references/)
12. **`or fail`** — prefer over verbose error binding for propagation

### Reserved Keywords (cannot use as identifiers)

```
let const import from as if else while for in switch case default return
break continue fail throw try catch async par parallel task await move seq
vec parvec with ordered chunk threads enum type use rust test true false null
and or not safe fast static dynamic auto detect schedule reduction prefetch simdWidth
```

## References

For detailed documentation on each topic, read the corresponding file in `references/`:

### Language Reference
- `references/variables.md` — Variables, constants, scoping, destructuring, type annotations
- `references/types-primitives.md` — Primitive types, Rust types, type inference, collections
- `references/operators.md` — Arithmetic, comparison, logical, range, ternary, precedence
- `references/functions-basics.md` — Arrow/block functions, parameters, return types
- `references/functions-advanced.md` — Closures, function references, method references
- `references/control-flow.md` — If/else, while/for loops, break/continue, one-liner `=>`
- `references/pattern-matching.md` — Switch expressions, range/or/guard/tuple/enum patterns
- `references/classes-basics.md` — Class declaration, constructors, field defaults, validation
- `references/classes-data.md` — Auto data classes (no constructor), PartialEq, Display
- `references/classes-interfaces.md` — Interface detection, implementation, multiple interfaces
- `references/enums.md` — Simple enums, enums with data, pattern matching on enums
- `references/visibility.md` — `_` prefix convention, public/private rules
- `references/error-handling.md` — fail, error binding, or fail, or default, error traces
- `references/collections.md` — Arrays, Maps, Sets, functional methods, parallel execution
- `references/concurrency.md` — async, par, task, await, fire-and-forget, array policies
- `references/modules.md` — Imports, exports, path resolution, visibility rules
- `references/string-templates.md` — String interpolation, escaping braces

### Standard Library
- `references/stdlib/arrays.md` — Array methods (map, filter, reduce, forEach, find, etc.)
- `references/stdlib/strings.md` — String methods (split, trim, replace, contains, etc.)
- `references/stdlib/io.md` — File I/O, Directory operations, console API
- `references/stdlib/math.md` — Math constants and functions
- `references/stdlib/conversions.md` — parseInt, parseFloat, toString
- `references/stdlib/system.md` — Sys.args, Sys.env, Sys.exit, HTTP client, JSON

### Guides
- `references/quick-reference.md` — Complete quick reference card (all syntax in one file)
- `references/project-structure.md` — Multi-file patterns, naming conventions, practical examples
- `references/style-guide.md` — Idiomatic style: `=>` vs `{}`, naming, error handling, SOLID, anti-patterns
