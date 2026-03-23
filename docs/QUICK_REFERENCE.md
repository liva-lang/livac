# Liva Language Quick Reference

> **Version:** 1.4.0  
> **Liva** — Python's simplicity, TypeScript's clarity, Rust's performance

---

## Table of Contents

- [CLI Commands](#cli-commands)
- [Basics](#basics)
- [Variables](#variables)
- [Types](#types)
- [Operators](#operators)
- [Functions](#functions)
- [Control Flow](#control-flow)
- [Pattern Matching](#pattern-matching)
- [Loops](#loops)
- [Classes](#classes)
- [Enums](#enums)
- [Interfaces](#interfaces)
- [Visibility](#visibility)
- [Error Handling](#error-handling)
- [Concurrency](#concurrency)
- [Collections (Arrays)](#collections-arrays)
- [Strings](#strings)
- [Modules](#modules)
- [Rust Interop](#rust-interop-v150-)
- [Testing](#testing)
- [Standard Library](#standard-library)
- [Complete Example](#complete-example)

---

## CLI Commands

```bash
livac build file.liva          # Compile to Rust project (debug mode)
livac run file.liva            # Compile and run
livac build --release file.liva  # Compile with optimizations (release mode)
livac run --release file.liva  # Compile optimized and run
livac check file.liva          # Syntax check only
livac fmt file.liva            # Format file in place (v1.0.2+)
livac fmt --check file.liva    # Check if file needs formatting
livac test                     # Run all *.test.liva files (v1.2.0+)
livac test file.test.liva      # Run specific test file
livac test --filter "name"     # Filter tests by name
livac build --verbose file.liva  # Show generated Rust code
livac build --output dir file.liva  # Set output directory
livac build --json file.liva   # Output errors as JSON (IDE integration)
livac init my-project          # Create new project (v1.5.0+)
livac init my-app --template cli   # CLI template
livac init my-data --template data # Data processing template
livac --help                   # Show help
livac --version                # Show version
```

---

## Basics

### Hello World

```liva
main() => print("Hello, World!")
```

### Comments

```liva
// Single line comment

/* 
   Multi-line
   comment 
*/
```

---

## Variables

```liva
let x = 10              // Mutable variable
let y: number = 20      // With explicit type
const PI = 3.14159      // Constant (immutable)
```

### Mutability

```liva
let count = 0
count = count + 1      // ✅ OK - let is mutable

const MAX = 100
MAX = 200              // ❌ Error - const is immutable
```

---

## Types

### Primitive Types

| Liva Type | Description | Rust Equivalent |
|-----------|-------------|-----------------|
| `number` | 32-bit signed integer | `i32` |
| `float` | 64-bit floating point | `f64` |
| `bool` | Boolean | `bool` |
| `string` | UTF-8 string | `String` |
| `char` | Unicode character | `char` |

### Rust Types (Direct)

```liva
let tiny: i8 = 127
let small: i16 = 32000
let big: i64 = 9223372036854775807
let huge: u64 = 18446744073709551615
let precise: f32 = 3.14
```

### Type Inference

```liva
let count = 42          // Inferred as number (i32)
let pi = 3.14159        // Inferred as float (f64)
let name = "Alice"      // Inferred as string
let active = true       // Inferred as bool
```

### Null

```liva
let data = null         // Represents absence of value
```

---

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `5 + 3` → `8` |
| `-` | Subtraction | `5 - 3` → `2` |
| `*` | Multiplication | `5 * 3` → `15` |
| `/` | Division | `10 / 3` → `3` |
| `%` | Modulo | `10 % 3` → `1` |

### Comparison

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `5 == 5` → `true` |
| `!=` | Not equal | `5 != 3` → `true` |
| `<` | Less than | `3 < 5` → `true` |
| `>` | Greater than | `5 > 3` → `true` |
| `<=` | Less or equal | `3 <= 3` → `true` |
| `>=` | Greater or equal | `5 >= 5` → `true` |

### Logical

| Operator | Description | Example |
|----------|-------------|---------|
| `and` | Logical AND | `true and false` → `false` |
| `or` | Logical OR | `true or false` → `true` |
| `not` | Logical NOT | `not true` → `false` |

**Alternative symbols:** `&&`, `||`, `!` also work.

---

## Functions

### One-liner (Arrow)

```liva
greet() => print("Hello!")
add(a, b) => a + b
square(x: number): number => x * x
```

### Block Function

```liva
calculate(a: number, b: number): number {
    let result = a + b * 2
    return result
}
```

### With Type Annotations

```liva
multiply(a: number, b: number): number => a * b

createGreeting(name: string, age: number): string {
    return $"Hello, {name}! You are {age} years old."
}
```

### Default Parameters

```liva
greet(name: string = "World") => print($"Hello, {name}!")

main() {
    greet()           // Hello, World!
    greet("Alice")    // Hello, Alice!
}
```

---

## Control Flow

### If / Else

```liva
if condition {
    // code
}

if x > 0 {
    print("positive")
} else if x < 0 {
    print("negative")
} else {
    print("zero")
}
```

### One-liner `=>` Syntax *(v1.1.0)* 🆕

For single-statement bodies, use `=>` instead of `{}`:

```liva
if age >= 18 => print("Adult")
if age >= 18 => print("Adult") else => print("Minor")
for item in items => print(item)
while running => tick()
```

> **Important:** Unlike function `=>` (which has implicit return), `if`/`for`/`while =>` simply replaces `{}` — there is **no implicit return**. Use explicit `return` when needed:
> ```liva
> // Function => = implicit return
> square(x: number): number => x * x
>
> // If => inside block = NO implicit return, needs explicit return
> clamp(val: number, lo: number, hi: number): number {
>     if val < lo => return lo
>     if val > hi => return hi
>     return val
> }
> ```

> **Note:** Block `{}` syntax remains the standard for multi-line bodies. Both forms are valid.

### Ternary Operator

```liva
let status = age >= 18 ? "adult" : "minor"
```

---

## Pattern Matching

### Switch Expression

```liva
let result = switch value {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"  // Wildcard (default)
}
```

### Range Patterns

```liva
let grade = switch score {
    90..=100 => "A",   // Inclusive range
    80..=89 => "B",
    70..=79 => "C",
    _ => "F"
}
```

### Or-Patterns

```liva
let result = switch day {
    "Saturday" | "Sunday" => "Weekend!",
    _ => "Weekday"
}
```

### Pattern Guards

```liva
let category = switch age {
    x if x < 13 => "child",
    x if x < 20 => "teenager",
    x => "adult"
}
```

### Binding Pattern

```liva
let doubled = switch num {
    0 => 0,
    n => n * 2  // 'n' binds to the value
}
```

---

## Loops

### While

```liva
let i = 0
while i < 5 {
    print(i)
    i = i + 1
}
```

### For (Range)

```liva
for i in 0..5 {        // 0, 1, 2, 3, 4
    print(i)
}

for i in 1..=10 {      // 1 to 10 inclusive
    print(i)
}
```

### For (Array)

```liva
let names = ["Alice", "Bob", "Charlie"]
for name in names {
    print(name)
}
```

### Break / Continue

```liva
let i = 0
while i < 10 {
    i = i + 1
    if i == 5 { continue }  // Skip 5
    if i == 8 { break }     // Stop at 8
    print(i)
}
// Output: 1 2 3 4 6 7
```

---

## Classes

### Basic Class

```liva
Person {
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    
    name: string
    age: number
    
    greet() => print($"Hi, I'm {this.name}")
    
    isAdult(): bool => this.age >= 18
}
```

### Instantiation

```liva
main() {
    let person = Person("Alice", 25)
    person.greet()
    print(person.isAdult())  // true
}
```

### Data Class (Auto-detected)

```liva
// Classes with fields but no explicit constructor are automatically data classes
// Auto-generates: constructor, PartialEq, and Display
Point {
    x: number
    y: number
}

let p = Point(10, 20)
print(p)  // "Point { x: 10, y: 20 }" (auto Display)
print(p == Point(10, 20))  // true (auto PartialEq)
```

### Data Class with Methods

```liva
Color {
    r: number
    g: number
    b: number

    sum() => r + g + b
}

let c = Color(255, 128, 0)
print(c.sum())  // 383
```

---

## Enums

Enums define types with a fixed set of variants:

### Simple Enums

```liva
enum Color {
    Red,
    Green,
    Blue
}

let c = Color.Red
print(c)              // "Red"
```

### Enums with Data

```liva
enum Shape {
    Circle(radius: number),
    Rectangle(width: number, height: number),
    Point
}

let s = Shape.Circle(5)
let r = Shape.Rectangle(10, 20)
let p = Shape.Point
```

### Pattern Matching on Enums

```liva
area(shape: Shape): number {
    return switch shape {
        Shape.Circle(r) => 3 * r * r
        Shape.Rectangle(w, h) => w * h
        Shape.Point => 0
    }
}
```

### Enums as Parameters and Return Types

```liva
enum SearchResult {
    Found(value: number),
    NotFound
}

findItem(id: number): SearchResult {
    if id > 0 {
        return SearchResult.Found(id * 10)
    }
    return SearchResult.NotFound
}
```

---

## Interfaces

Interfaces define contracts for classes:

```liva
// Interface (only method signatures)
Animal {
    makeSound(): string
    getName(): string
}

// Class implementing interface
Dog : Animal {
    constructor(name: string) {
        this.name = name
    }
    
    name: string
    
    makeSound() => "Woof!"
    getName() => this.name
}
```

### Multiple Interfaces

```liva
Drawable {
    draw(): void
}

Cat : Animal, Drawable {
    constructor(name: string) {
        this.name = name
    }
    
    name: string
    
    makeSound() => "Meow!"
    getName() => this.name
    draw() => print($"Drawing {this.name}")
}
```

---

## Visibility

```liva
User {
    constructor(name: string, password: string) {
        this.name = name
        this._password = password
    }
    
    name: string        // Public (default)
    _password: string   // Private (underscore prefix)
    
    validatePassword(input: string): bool {
        return this._password == input
    }
}
```

**Rule:** Names starting with `_` are private.

---

## Error Handling

Liva uses **explicit error handling** instead of exceptions.

### Fail Statement

```liva
divide(a: number, b: number): number {
    if b == 0 {
        fail "Cannot divide by zero"
    }
    return a / b
}
```

### Error Binding

```liva
let result, err = divide(10, 0)

if err {
    print($"Error: {err}")
} else {
    print($"Result: {result}")
}
```

### Ternary Fail

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b
```

### Or Fail *(v1.1.0)* 🆕

Shorthand error propagation — fails immediately if the expression returns an error:

```liva
let response = HTTP.get(url) or fail "Connection error"
let content = File.read("config.json") or fail "Cannot read config"
let data = JSON.parse(content) or fail "Invalid JSON"
```

Equivalent to:
```liva
let response, err = HTTP.get(url)
if err { fail "Connection error" }
```

> **Note:** The traditional `let value, err = expr` + `if err` pattern continues to work.

### Error Trace *(v1.3.0)* 🆕

Errors automatically chain with function names and source locations:

```liva
parsePort(s: string): number {
  if s == "" => fail "port is empty"
  return 8080
}

loadConfig(path: string): string {
  let port = parsePort("") or fail "config error"
  return $"server:{port}"
}

main() {
  let cfg, err = loadConfig("app.toml")
  if err {
    print(err)       // Full trace with colors
    print(err.message) // Plain: "config error"
  }
}
```

Output:
```
╭─ Error Trace ─────────────────────────────────────╮
│  ✗ config error
│    → loadConfig()  main.liva:8
│  ⊘ port is empty
│    → parsePort()  main.liva:3
╰───────────────────────────────────────────────────╯
```

- `✗` (red) = top-level error (what stopped your code)
- `⊘` (yellow) = cause errors in the chain
- Chaining is automatic: `or fail`, `if err => fail`, and `if err { fail }` all chain when an error is in scope
- `err.message` returns the plain message string without the trace

### Or Value *(v1.3.0)* 🆕

Provide a default value when a fallible function fails (like JavaScript's `||`):

```liva
let result = divide(10, 0) or 42          // 42 (failed → default)
let result2 = divide(10, 2) or 42         // 5  (succeeded → value)
let port = parsePort("abc") or 3000       // 3000 (failed → default)
```

Equivalent to:
```liva
let result, err = divide(10, 0)
let result = err ? 42 : result
```

> **Note:** Only works when the left side is a function/method call. For logical OR between booleans, use `a or b` as usual.

---

## Concurrency

### Async (I/O-bound)

```liva
fetchData(url: string): string {
    // I/O operation
    return "data"
}

main() {
    let data = async fetchData("https://api.example.com")
    print(data)  // Auto-awaited on use
}
```

### Par (CPU-bound)

```liva
heavyCalc(n: number): number {
    // CPU-intensive work
    return n * n
}

main() {
    let result = par heavyCalc(1000)
    print(result)  // Auto-joined on use
}
```

### Task Handles

```liva
main() {
    let t1 = task async fetchUser(1)
    let t2 = task async fetchUser(2)
    
    // Do other work...
    
    let user1 = await t1
    let user2 = await t2
}
```

### Fire-and-Forget (Auto-inferred)

When an `async` or `par` call appears as a statement (not assigned to a variable), it's automatically fire-and-forget:

```liva
async logEvent("user_login")  // Fire-and-forget (not assigned)
par backgroundCleanup()      // Fire-and-forget (not assigned)
```

### Concurrency Summary

| Keyword | Type | Use Case | Blocks |
|---------|------|----------|--------|
| `async` | Asynchronous | I/O-bound | No (lazy) |
| `par` | Parallel | CPU-bound | No (lazy) |
| `task` | Handle | Explicit control | No |
| `await` | Wait | Wait for task | Yes |

> **Note:** `async`/`par` calls used as statements (not assigned to a variable) are automatically fire-and-forget.

---

## Collections (Arrays)

### Creation

```liva
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]
let empty: [number] = []
```

### Access

```liva
let first = numbers[0]    // 1
let second = names[1]     // "Bob"
```

### Functional Methods

```liva
let numbers = [1, 2, 3, 4, 5]

// Transform
let doubled = numbers.map(x => x * 2)        // [2, 4, 6, 8, 10]

// Filter
let evens = numbers.filter(x => x % 2 == 0)  // [2, 4]

// Reduce
let sum = numbers.reduce((acc, x) => acc + x, 0)  // 15

// Find
let found = numbers.find(x => x > 3)         // Some(4)

// Check
let hasEven = numbers.some(x => x % 2 == 0)  // true
let allPos = numbers.every(x => x > 0)       // true

// Search
let idx = numbers.indexOf(3)                 // 2
let exists = numbers.includes(5)             // true

// Join
let words: [string] = ["hello", "world"]
let sentence = words.join(" ")               // "hello world"
let csv = words.join(",")                    // "hello,world"
```

### v1.4 — New Array Methods

```liva
let numbers = [1, 2, 3, 4, 5]

// Access
let f = numbers.first()                      // 1
let l = numbers.last()                       // 5
let empty = numbers.isEmpty()                // false

// Slicing
let mid = numbers.slice(1, 3)                // [2, 3]
let top3 = numbers.take(3)                   // [1, 2, 3]
let rest = numbers.drop(2)                   // [3, 4, 5]

// Transform
let sorted = numbers.sort()                  // [1, 2, 3, 4, 5]
let rev = numbers.reversed()                 // [5, 4, 3, 2, 1]
let uniq = [1, 2, 2, 3].distinct()           // [1, 2, 3]

// Combine & split
let flat = [[1, 2], [3, 4]].flat()           // [1, 2, 3, 4]
let chunked = numbers.chunks(2)              // [[1,2],[3,4],[5]]
let zipped = [1, 2].zip(["a", "b"])          // [(1,"a"),(2,"b")]

// Aggregate
let total = numbers.sum()                    // 15
let lo = numbers.min()                       // 1
let hi = numbers.max()                       // 5

// Callback-based
let idx = numbers.findIndex(x => x > 3)      // 3
let fm = numbers.flatMap(n => [n, n * 10])    // [1,10,2,20,...]
let cnt = numbers.count(x => x > 2)           // 3
```

### Chaining

```liva
let result = numbers
    .filter(x => x > 2)
    .map(x => x * 2)
    .reduce((acc, x) => acc + x, 0)  // 24
```

### Point-Free Function References *(v1.1.0)*

Pass function names directly where a single-argument callback is expected:

```liva
items.forEach(print)           // instead of: items.forEach(x => print(x))
nums.map(toString)             // instead of: nums.map(n => toString(n))
names.filter(isValid)          // instead of: names.filter(n => isValid(n))
```

**Supported methods:** `forEach`, `map`, `filter`, `find`, `some`, `every`

Also works with `for =>` loops:

```liva
for item in items => print      // instead of: for item in items => print(item)
for item in items => showItem   // instead of: for item in items => showItem(item)
```

### Method References with `::` *(v1.1.0)*

Reference an instance method using the `object::method` syntax:

```liva
Formatter {
    prefix: string
    constructor(prefix: string) { this.prefix = prefix }
    format(s: string) => $"{this.prefix}: {s}"
}

main() {
    let names = ["Alice", "Bob", "Charlie"]
    let fmt = Formatter("Hello")

    // Method reference: pass instance method as callback
    let greetings = names.map(fmt::format)     // ["Hello: Alice", "Hello: Bob", ...]
    greetings.forEach(print)

    // Also works with forEach, filter, find, some, every
    names.forEach(fmt::format)
}
```

**Supported methods:** `forEach`, `map`, `filter`, `find`, `some`, `every`

> **Note:** `object::method` binds the method to the specific instance. The method must accept a single argument matching the array element type.

> **Note:** Lambda syntax `x => expr` continues to work and is required for multi-argument or complex expressions.

---

## Collections (Maps) *(v1.3.0)*

Maps (dictionaries/hashmaps) store key-value pairs with O(1) lookup.

### Creation

```liva
// Empty map with type annotation
let empty: Map<string, int> = Map {}

// Map literal with entries
let ages = Map {
  "Alice": 30,
  "Bob": 25,
  "Carlos": 35
}

// Return type annotation
getDefaults(): Map<string, int> {
  return Map { "timeout": 30, "retries": 3 }
}
```

### Access & Modification

```liva
// Get value (returns Option — use `or` for default)
let age = ages.get("Alice") or 0      // 30
let miss = ages.get("Unknown") or -1   // -1

// Set value (insert or update)
ages.set("Diana", 28)

// Check existence
let hasBob = ages.has("Bob")           // true

// Delete entry
ages.delete("Carlos")

// Get size
let count = ages.length                // 3

// Clear all entries
ages.clear()
```

### Iteration

```liva
// for key, value in map
let scores = Map { "math": 95, "english": 88 }

for key, value in scores {
  print($"{key}: {value}")
}

// forEach with lambda
scores.forEach((key, value) => {
  print($"{key} = {value}")
})
```

### Keys, Values & Entries

```liva
let config = Map { "host": "localhost", "port": "8080" }

let allKeys = config.keys()      // [string]
let allValues = config.values()  // [string]
let pairs = config.entries()     // [(string, string)]
```

### Map Methods Summary

| Method | Description | Returns |
|--------|-------------|---------|
| `map.get(key)` | Get value by key | `value?` (use `or` for default) |
| `map.set(key, value)` | Insert or update | `void` |
| `map.has(key)` | Check key exists | `bool` |
| `map.delete(key)` | Remove entry | `void` |
| `map.keys()` | All keys | `[K]` |
| `map.values()` | All values | `[V]` |
| `map.entries()` | All key-value pairs | `[(K, V)]` |
| `map.clear()` | Remove all entries | `void` |
| `map.forEach(fn)` | Iterate with callback | `void` |
| `map.length` | Number of entries | `int` |

---

## Collections (Sets) *(v1.3.0)*

### Creating Sets

```liva
// Empty set (type annotation required)
let empty: Set<string> = Set {}

// Set with values (type inferred)
let colors = Set { "red", "green", "blue" }
let primes = Set { 2, 3, 5, 7, 11 }
```

### Set Operations

```liva
let colors = Set { "red", "green", "blue" }

// Add element
colors.add("yellow")

// Check membership
let hasRed = colors.has("red")       // true

// Remove element
colors.delete("green")

// Get all values as array
let vals = colors.values()            // [string]

// Clear all elements
colors.clear()
```

### Set Algebra

```liva
let a = Set { 1, 2, 3 }
let b = Set { 3, 4, 5 }

let u = a.union(b)            // Set { 1, 2, 3, 4, 5 }
let i = a.intersection(b)     // Set { 3 }
let d = a.difference(b)       // Set { 1, 2 }
```

### Iterating Sets

```liva
let fruits = Set { "apple", "banana", "cherry" }

// for-in loop
for fruit in fruits {
  print(fruit)
}

// forEach with lambda
fruits.forEach((f) => {
  print(f)
})
```

### Set Methods Summary

| Method | Description | Returns |
|--------|-------------|--------|
| `set.add(value)` | Add element | `void` |
| `set.has(value)` | Check membership | `bool` |
| `set.delete(value)` | Remove element | `void` |
| `set.values()` | All values as array | `[T]` |
| `set.union(other)` | Set union | `Set<T>` |
| `set.intersection(other)` | Set intersection | `Set<T>` |
| `set.difference(other)` | Set difference | `Set<T>` |
| `set.clear()` | Remove all elements | `void` |
| `set.forEach(fn)` | Iterate with callback | `void` |
| `set.length` | Number of elements | `int` |

---

## Strings

### String Templates

```liva
let name = "Alice"
let age = 25
let greeting = $"Hello, {name}! You are {age} years old."
```

### String Methods

```liva
let text = "Hello, World!"

// Core methods
text.split(", ")              // ["Hello", "World!"]
text.toUpperCase()            // "HELLO, WORLD!"
text.toLowerCase()            // "hello, world!"
"  hello  ".trim()            // "hello"
text.replace("World", "Liva") // "Hello, Liva!"
text.startsWith("Hello")      // true
text.endsWith("!")            // true
text.contains("World")        // true
text.substring(0, 5)          // "Hello"
text.charAt(0)                // 'H'
text.indexOf("World")         // 7

// v1.4 — Search & slicing
text.lastIndexOf("l")         // 10
text.slice(0, 5)              // "Hello"
"hello".chars()               // ["h","e","l","l","o"]

// v1.4 — Transform
"hello".capitalize()          // "Hello"
"hello".reverse()             // "olleh"
"hello world".truncate(5)     // "hello"
"ha".repeat(3)                // "hahaha"
text.replaceAll("l", "L")     // "HeLLo, WorLd!"

// v1.4 — Padding
"5".padStart(3, "0")          // "005"
"hi".padEnd(5, ".")           // "hi..."

// v1.4 — Query
"  ".isBlank()                // true
"".isEmpty()                  // true
"banana".countMatches("an")   // 2

// v1.4 — Remove prefix/suffix
"prefix_val".removePrefix("prefix_")   // "val"
"file.txt".removeSuffix(".txt")        // "file"
```

---

## Modules

### Export (Default)

All functions without `_` prefix are public/exported:

```liva
// math.liva
add(a, b) => a + b           // Public (exported)
subtract(a, b) => a - b      // Public (exported)
_helper(x) => x * 2          // Private (not exported)
```

### Import

```liva
// main.liva
import { add, subtract } from "./math.liva"

main() {
    print(add(10, 5))
}
```

### Wildcard Import

```liva
import * as math from "./math.liva"

main() {
    print(math.add(10, 5))
}
```

---

## Rust Interop *(v1.5.0)* 🆕

### Inline Rust Blocks

Embed raw Rust code as an expression with `rust { ... }`:

```liva
main() {
    let result = rust {
        let x: i32 = 42;
        x * 2
    }
    print(result)  // 84
}
```

### Rust Blocks with Use Hoisting

`use` statements inside `rust { }` are automatically hoisted to the top of the generated file:

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

### Crate Dependencies

Declare Cargo dependencies at the top level:

```liva
use rust "chrono" version "0.4"
use rust "uuid" version "1.0" features ["v4", "serde"]
use rust "serde_json" as json
use rust "tokio" features ["net"]   // Merges with built-in features
```

### Rust Block as Statement

```liva
main() {
    rust {
        println!("Direct Rust output!");
    }
}
```

> See [Rust Interop](language-reference/rust-interop.md) for full documentation.

---

## Testing

### Test Library (`liva/test`) *(v1.2.0+)*

Liva includes a built-in test library with a Jest-like API.

```liva
import { describe, test, expect } from "liva/test"
```

### Writing Tests

```liva
import { describe, test, expect } from "liva/test"

add(a: int, b: int): int => a + b

describe("Math operations", () => {
    test("addition works", () => {
        expect(add(2, 3)).toBe(5)
        expect(add(-1, 1)).toBe(0)
    })

    test("negative results", () => {
        expect(add(-5, 2)).toBe(-3)
    })
})
```

### Matchers

```liva
// Equality
expect(x).toBe(y)                    // assert_eq!
expect(x).toEqual(y)                 // assert_eq! (alias)

// Truthiness
expect(x).toBeTruthy()               // assert!(x)
expect(x).toBeFalsy()                // assert!(!(x))

// Comparison
expect(x).toBeGreaterThan(y)         // assert!(x > y)
expect(x).toBeLessThan(y)            // assert!(x < y)
expect(x).toBeGreaterThanOrEqual(y)  // assert!(x >= y)
expect(x).toBeLessThanOrEqual(y)     // assert!(x <= y)

// Collections
expect(x).toContain(y)               // assert!(x.contains(&y))

// Null
expect(x).toBeNull()                 // assert!(x.is_none())

// Errors
expect(x).toThrow()                  // assert!(catch_unwind(x).is_err())
```

### Negation (`.not`)

```liva
expect(x).not.toBe(y)               // assert_ne!
expect(x).not.toBeTruthy()           // assert!(!(x))
expect(x).not.toContain(y)           // assert!(!x.contains(&y))
```

### Lifecycle Hooks

```liva
describe("Suite", () => {
    beforeAll(() => { /* runs once before all tests */ })
    afterAll(() => { /* runs once after all tests */ })
    beforeEach(() => { /* runs before each test */ })
    afterEach(() => { /* runs after each test */ })

    test("example", () => {
        expect(true).toBeTruthy()
    })
})
```

### Running Tests

```bash
livac test                            # Run all *.test.liva files
livac test file.test.liva             # Run specific test file
livac test --filter "math"            # Filter tests by name
livac test --verbose                  # Show individual test results
```

---

## Standard Library

### Console I/O

```liva
print("Hello!")                      // Simple output
console.log(data)                    // Debug output (shows structure)
console.error("Error message")       // Red, to stderr
console.warn("Warning")              // Yellow, to stderr
console.success("Done!")             // Green, to stdout

let input = console.input("Name: ") // Read user input
```

### Logging

```liva
Log.info("Server started")                    // Timestamped info to stderr
Log.warn("Disk space low")
Log.error("Connection failed")
Log.debug("Payload received")                 // Only with --verbose

// Variadic args
Log.info("User", name, "from", ip)            // Concatenated with spaces

// Map 4+ keys → Key/Value table
Log.info("Config:", { host: "localhost", port: 8080, db: "mydb", pool: 10 })

// Map ≤3 keys → inline
Log.info("Status:", { code: 200, ok: true })  // {code: 200, ok: true}

// Array<Map> → columnar table (console.table)
Log.info("Users:", [{ name: "Alice", age: 30 }, { name: "Bob", age: 25 }])

// JSON runtime tables
let data, _err = JSON.parse(jsonString)
Log.info("Data:", data)                       // Auto table rendering

// Set minimum level
Log.setLevel("debug")                         // debug/info/warn/error
```

### Math

```liva
Math.PI              // 3.141592653589793
Math.E               // 2.718281828459045
Math.sqrt(16.0)      // 4.0
Math.pow(2.0, 3.0)   // 8.0
Math.abs(-10.5)      // 10.5
Math.floor(3.7)      // 3
Math.ceil(3.2)       // 4
Math.round(3.5)      // 4
Math.min(10.5, 20.3) // 10.5
Math.max(10.5, 20.3) // 20.3
Math.random()        // 0.0 to 1.0

// v1.4
Math.clamp(15, 0, 10) // 10
Math.sign(-42)        // -1
Math.log(2.718)       // ~1.0
```

### Type Conversion

```liva
let num, err = parseInt("42")
let val, err2 = parseFloat("3.14")
let str = toString(42)
```

### File I/O

```liva
let content, err = File.read("file.txt")
let ok, err2 = File.write("out.txt", "Hello")
let ok2, err3 = File.append("log.txt", "Line\n")
let exists = File.exists("file.txt")
let ok3, err4 = File.delete("temp.txt")

// v1.6: Extended File operations
let ok4, copyErr = File.copy("src.txt", "dest.txt")
let ok5, moveErr = File.move("old.txt", "new.txt")
let bytes, sizeErr = File.size("data.bin")
let ext = File.extension("photo.jpg")           // "jpg"
let lines, readErr = File.readLines("data.txt")  // [string]
let ok6, writeErr = File.writeLines("out.txt", ["line1", "line2"])
```

### Directory Operations *(v1.3.0+)*

```liva
let entries, err = Dir.list("./src")    // List directory entries
let isDir = Dir.isDir("./src")          // Check if path is directory

// v1.6: Extended Dir operations
let dirExists = Dir.exists("./output")               // true if path is dir
let ok, createErr = Dir.create("./output/subdir")     // mkdir -p
let ok2, delErr = Dir.delete("./temp")                // rm -rf
let files, walkErr = Dir.listRecursive("./src")       // All files recursively
let files2, walkErr2 = Dir.walk("./docs")             // Alias for listRecursive

// Recursive traversal (manual)
for i in 0..entries.length {
    let entry = entries[i]
    let fullPath = dirPath + "/" + entry
    if Dir.isDir(fullPath) {
        // recurse...
    }
}
```

### Regex *(v1.6.0)*

```liva
// Test: returns bool
let isEmail = Regex.test("^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$", email)

// Match: returns (string, error) — first match
let found, err = Regex.match("\\d+", "Order #42")

// FindAll: returns [string]
let numbers = Regex.findAll("\\d+", "a1b22c333")   // ["1", "22", "333"]

// Replace: returns string (replaces all)
let clean = Regex.replace("\\s+", text, " ")

// Split: returns [string]
let parts = Regex.split("[,;]", "a,b;c")            // ["a", "b", "c"]
```

### Date *(v1.6.0)*

```liva
// Constructors
let now = Date.now()                               // Current date/time
let birthday = Date.new(1990, 6, 15)               // Specific date
let parsed, err = Date.parse("2026-03-11", "YYYY-MM-DD")  // Fallible
let ts = Date.timestamp()                          // Unix epoch ms (int)

// Properties: .year, .month, .day, .hour, .minute, .second
print(now.year)    // 2026
print(now.month)   // 3

// Methods
let formatted = now.format("DD/MM/YYYY")           // "11/03/2026"
let nextWeek = now.add(7, "days")                  // Date arithmetic
let age = now.diff(birthday, "years")              // Approximate years
let iso = now.toString()                           // ISO 8601

// Comparisons
if nextWeek > now { print("Future!") }

// Interpolation (auto ISO 8601)
print($"Today is {now}")                           // "Today is 2026-03-11T..."
```

### CSV *(v1.6.0)*

```liva
// Read/write raw CSV
let data, err = CSV.read("data.csv")               // [[string]], error
let ok, err = CSV.write("out.csv", data)            // bool, error

// Custom separator (TSV)
let tsv, err = CSV.read("data.tsv", "\t")

// Parse/stringify (pure, no I/O)
let rows = CSV.parse("a,b\n1,2")                   // [[string]]
let csv = CSV.stringify(rows)                       // string

// Table = [Map<string, string>] (first row as headers)
let table, err = CSV.readTable("ventas.csv")
let headers = CSV.headers(table)                    // ["col1", "col2", ...]
let col = CSV.column(table, "ventas")              // [string]
let ok, err = CSV.writeTable("result.csv", table)

// Table ops via standard array methods
let filtered = table.filter(row => row.get("region") == "Europa")
```

### Random *(v1.7.0)*

```liva
// Random numbers
let n = Random.nextInt(1, 100)                     // int in [min, max]
let f = Random.nextFloat(0.0, 1.0)                 // float in [min, max]
let f2 = Random.nextFloat()                        // float in [0.0, 1.0)

// Random selection
let pick = Random.choice(["a", "b", "c"])          // Random element
let mixed = Random.shuffle([1, 2, 3, 4, 5])        // Shuffled copy

// UUID
let id = Random.uuid()                             // "550e8400-e29b-..."
```

### Crypto *(v1.7.0)*

```liva
// Hashing
let hash = Crypto.sha256("hello world")             // Hex-encoded SHA-256
let md = Crypto.md5("hello world")                  // Hex-encoded MD5

// Base64
let encoded = Crypto.base64Encode("hello")          // "aGVsbG8="
let decoded, err = Crypto.base64Decode(encoded)     // Fallible
```

### Process *(v1.7.0)*

```liva
// Execute command and capture output
let output, err = Process.exec("ls -la")            // string, error

// Spawn background process
let pid, err = Process.spawn("sleep 10")            // int, error

// Process info
let myPid = Process.pid()                          // Current PID
Process.exit(0)                                    // Exit with code
```

### Configuration *(v1.5.0)*

```liva
// Load .env file
let config, err = Config.load(".env")
if err {
    Log.error("Config error:", err)
}

// Get string, int, bool values
let host, _ = Config.get(config, "HOST")        // "localhost"
let port, _ = Config.getInt(config, "PORT")      // 8080
let debug, _ = Config.getBool(config, "DEBUG")   // true

// Get all entries as sorted map
let all = Config.getAll(config)
```

### System *(v1.3.0)*

```liva
let args = Sys.args()              // Command-line arguments
let home = Sys.env("HOME")         // Environment variable
Sys.exit(1)                        // Exit with code
```

### JSON

```liva
// Type-safe parsing
let data: [number], err = JSON.parse("[1, 2, 3]")
let user: User, err2 = JSON.parse(jsonString)

// Stringify
let json = JSON.stringify(data)
```

### HTTP Client

```liva
// GET request
let response, err = async HTTP.get("https://api.example.com/users")
if err == "" {
    print(response.status)
    let data, jsonErr = response.json()
}

// POST request
let body = "{\"name\": \"Alice\"}"
let resp, err = async HTTP.post("https://api.example.com/users", body)

// Also: HTTP.put(), HTTP.delete()
```

---

## Complete Example

```liva
// Package metadata structure
Package {
    constructor(name: string, version: string) {
        this.name = name
        this.version = version
    }
    name: string
    version: string
    
    display(): string {
        return $"{this.name}@{this.version}"
    }
}

// Load packages from file
loadPackages(path: string): [Package] {
    let content, err = File.read(path)
    if err != "" {
        fail $"Cannot read file: {err}"
    }
    
    let packages: [Package], parseErr = JSON.parse(content)
    if parseErr != "" {
        fail $"Invalid JSON: {parseErr}"
    }
    
    return packages
}

// Filter packages by prefix
filterByPrefix(packages: [Package], prefix: string): [Package] {
    return packages.filter(p => p.name.startsWith(prefix))
}

main() {
    // Load and process packages
    let packages, err = loadPackages("packages.json")
    
    if err != "" {
        console.error($"Error: {err}")
        return
    }
    
    // Filter and display
    let devPackages = filterByPrefix(packages, "dev-")
    
    for pkg in devPackages {
        print(pkg.display())
    }
    
    print($"Found {devPackages.length} dev packages")
}
```

---

## Quick Reference Card

### Keywords

```
let const import from as if else while for in
switch case default return fail async par task
await true false null and or not
```

### Type Keywords

```
number float bool string char bytes
i8 i16 i32 i64 i128 u8 u16 u32 u64 u128
f32 f64 isize usize
```

### Operators

```
+ - * / %                    // Arithmetic
== != < > <= >=              // Comparison  
and or not (or && || !)      // Logical
=                            // Assignment
=>                           // Arrow (function one-liner / lambda / switch arm / control flow one-liner)
? :                          // Ternary
.. ..=                       // Ranges
```

### Delimiters

```
( )    // Function calls, grouping
{ }    // Blocks
[ ]    // Arrays, indexing
,      // Separators
:      // Type annotations
;      // Optional statement terminator
.      // Field/method access
```

---

**Happy coding with Liva! 🚀**

```bash
livac --help
```
