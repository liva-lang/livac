# Liva Language Quick Reference

> **Version:** 1.1.0-dev  
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
- [Interfaces](#interfaces)
- [Visibility](#visibility)
- [Error Handling](#error-handling)
- [Concurrency](#concurrency)
- [Collections (Arrays)](#collections-arrays)
- [Strings](#strings)
- [Modules](#modules)
- [Standard Library](#standard-library)
- [Complete Example](#complete-example)

---

## CLI Commands

```bash
livac file.liva               # Compile to Rust project
livac file.liva --run         # Compile and run
livac file.liva --check       # Syntax check only
livac file.liva --fmt         # Format file in place (v1.0.2+)
livac file.liva --fmt-check   # Check if file needs formatting
livac file.liva --verbose     # Show generated Rust code
livac file.liva --output dir  # Set output directory
livac file.liva --json        # Output errors as JSON (IDE integration)
livac --help                  # Show help
livac --version               # Show version
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

For single-expression bodies, use `=>` instead of `{}`:

```liva
if age >= 18 => print("Adult")
if age >= 18 => print("Adult") else => print("Minor")
for item in items => print(item)
while running => tick()
```

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

### Data Class (No Methods)

```liva
Point {
    constructor(x: number, y: number) {
        this.x = x
        this.y = y
    }
    x: number
    y: number
}

let p = Point(10, 20)
print($"({p.x}, {p.y})")
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

### Fire and Forget

```liva
fire async logEvent("user_login")  // Don't wait for result
```

### Concurrency Summary

| Keyword | Type | Use Case | Blocks |
|---------|------|----------|--------|
| `async` | Asynchronous | I/O-bound | No (lazy) |
| `par` | Parallel | CPU-bound | No (lazy) |
| `task` | Handle | Explicit control | No |
| `fire` | Fire-and-forget | Background work | No |
| `await` | Wait | Wait for task | Yes |

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

text.split(", ")              // ["Hello", "World!"]
text.toUpperCase()            // "HELLO, WORLD!"
text.toLowerCase()            // "hello, world!"
"  hello  ".trim()            // "hello"
text.replace("World", "Liva") // "Hello, Liva!"
text.startsWith("Hello")      // true
text.endsWith("!")            // true
text.substring(0, 5)          // "Hello"
text.charAt(0)                // 'H'
text.indexOf("World")         // 7
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

### Math

```liva
Math.sqrt(16.0)      // 4.0
Math.pow(2.0, 3.0)   // 8.0
Math.abs(-10.5)      // 10.5
Math.floor(3.7)      // 3
Math.ceil(3.2)       // 4
Math.round(3.5)      // 4
Math.min(10.5, 20.3) // 10.5
Math.max(10.5, 20.3) // 20.3
Math.random()        // 0.0 to 1.0
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
fire await true false null and or not
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
= =>                         // Assignment / Arrow
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
