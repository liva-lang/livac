---
name: liva-lang
description: Complete language reference for Liva — a modern programming language that compiles to Rust. Covers syntax, types, error handling, concurrency, classes, enums, pattern matching, and standard library.
---

# Liva Language — Complete Reference

Liva is a modern programming language that compiles to Rust, combining Python's simplicity, TypeScript's clarity, and Rust's performance. The compiler (`livac`) generates idiomatic Rust code that is then compiled to native binaries.

> ⚠️ **IMPORTANT INSTRUCTION FOR AI AGENTS** ⚠️
>
> **This file is 100% self-contained.** Everything you need to write Liva code is here.
>
> **DO NOT:**
> - Search the filesystem for Liva docs
> - Read files from any `docs/` directory
> - Open or fetch any external files or URLs
> - Look for additional `.md` files about Liva
>
> **DO:** Use ONLY the content below. It contains the complete language reference,
> all syntax, all standard library APIs, and all examples you need.

---

## Table of Contents

1. [CLI Commands](#cli-commands)
2. [Variables & Constants](#variables--constants)
3. [Types](#types)
4. [Operators](#operators)
5. [Functions](#functions)
6. [Control Flow](#control-flow)
7. [Pattern Matching](#pattern-matching)
8. [Classes](#classes)
9. [Data Classes](#data-classes)
10. [Enums](#enums)
11. [Interfaces](#interfaces)
12. [Visibility](#visibility)
13. [Error Handling](#error-handling)
14. [Concurrency](#concurrency)
15. [Collections & Arrays](#collections--arrays)
16. [Strings & Templates](#strings--templates)
17. [Modules & Imports](#modules--imports)
18. [Standard Library](#standard-library)
19. [Testing](#testing)
20. [Common Pitfalls & Known Limitations](#common-pitfalls--known-limitations)
21. [Project Structure Best Practices](#project-structure-best-practices)
22. [Practical Patterns](#practical-patterns)

---

## CLI Commands

```bash
livac file.liva               # Compile to Rust project (debug mode)
livac file.liva --run         # Compile and run
livac file.liva --release     # Compile with optimizations (release mode)
livac file.liva --release --run  # Compile optimized and run
livac file.liva --check       # Syntax check only
livac file.liva --fmt         # Format file in place
livac file.liva --fmt-check   # Check if file needs formatting
livac --test                  # Run all *.test.liva files
livac --test file.test.liva   # Run specific test file
livac --test --filter "name"  # Filter tests by name
livac file.liva --verbose     # Show generated Rust code
livac file.liva --output dir  # Set output directory
livac file.liva --json        # Output errors as JSON (IDE integration)
livac --help                  # Show help
livac --version               # Show version
```

---

## Reserved Words

> ⚠️ **CRITICAL:** These words are **hard keywords** in Liva. They CANNOT be used as
> variable names, function names, parameter names, or class/field names.
> Using them will cause a compile error.

### Hard Keywords (NEVER use as identifiers)

```
let       const     import    from      as        if        else
while     for       in        switch    case      default   return
break     continue  fail      throw     try       catch     async
par       parallel  task      fire      await     move      seq
vec       parvec    with      ordered   chunk     threads   enum
type      use       rust      test      true      false     null
and       or        not       safe      fast      static    dynamic
auto      detect    schedule  reduction prefetch  simdWidth
```

### Type Keywords (also reserved)

```
number    float     bool      string    char      bytes
i8        i16       i32       i64       i128
u8        u16       u32       u64       u128
f32       f64       isize     usize
```

### Contextual Keywords (reserved in specific positions)

```
(none — `data` keyword was removed in v1.3.0; data classes are auto-detected)
```

### Common Mistakes

```
❌ let threads = 4              → use: let threadCount = 4
❌ let type = "admin"           → use: let userType = "admin"
❌ let default = 0              → use: let defaultValue = 0
❌ let task = "cleanup"         → use: let taskName = "cleanup"
❌ let auto = true              → use: let autoMode = true
❌ let from = "server"          → use: let source = "server"
❌ let as = "alias"             → use: let alias = "alias"
❌ let static = false           → use: let isStatic = false
❌ let fast = true              → use: let fastMode = true
❌ let schedule = "daily"       → use: let scheduleType = "daily"
❌ fn move(x, y)                → use: fn moveTo(x, y)
❌ fn detect(input)             → use: fn detectInput(input)
```

---

## Variables & Constants

```liva
// Mutable variables (let is mutable)
let x = 10
let name: string = "Alice"
let height: float = 1.75

// Constants (immutable, SCREAMING_SNAKE_CASE)
const PI = 3.14159
const MAX_USERS: number = 100

// All variables must be initialized
let y: number     // ❌ Compile error
```

### Type Annotations (Optional)

```liva
let age: number = 25        // 32-bit integer
let height: float = 1.75    // 64-bit float
let name: string = "Alice"  // UTF-8 string
let active: bool = true     // Boolean
let initial: char = 'A'     // Unicode character
let nums: [number] = [1,2]  // Array
let maybe: number? = null   // Optional
```

### Destructuring

```liva
// Array destructuring
let [a, b] = [10, 20]
let [first, , third] = [1, 2, 3]        // Skip elements
let [head, ...tail] = [1, 2, 3, 4, 5]   // Rest pattern

// Object destructuring
let {id, name} = user
let {name: userName, email: userEmail} = user  // Renaming
let {address: {city, country}} = user          // Nested
```

### Error Binding

```liva
// Fallible functions return (value, error_string)
let result, err = divide(10, 0)
if err {
    print($"Error: {err}")
} else {
    print($"Result: {result}")
}

// Ignore error
let value, _ = divide(10, 2)

// With async/par
let data, err = async fetchData(url)
let result, err = par processData(50)
```

### Scoping

Variables are **block-scoped**. Shadowing is allowed in inner scopes.

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
let count = 42          // number (i32)
let pi = 3.14159        // float (f64)
let name = "Alice"      // string
let active = true       // bool
```

### Collection & Special Types

```liva
let nums: [number] = [1, 2, 3]         // Array
let maybeAge: number? = null            // Optional
let result: number! = divide(10, 2)     // Fallible
let data = null                         // Null
```

### Tuple Types

```liva
// Functions can return tuples (explicit type required)
getPoint(): (int, int) {
    return (10, 20)
}

let p = getPoint()
let x = p.0    // Access first element
let y = p.1    // Access second element
```

---

## Operators

### Arithmetic

| Op | Description | Example |
|----|-------------|---------|
| `+` | Addition | `5 + 3` → `8` |
| `-` | Subtraction | `5 - 3` → `2` |
| `*` | Multiplication | `5 * 3` → `15` |
| `/` | Division | `10 / 3` → `3` |
| `%` | Modulo | `10 % 3` → `1` |

**No compound assignment** — use `x = x + 5` instead of `x += 5`.

### Comparison

`==` `!=` `<` `>` `<=` `>=`

### Logical

| Liva (preferred) | Symbol alternative |
|------------------|--------------------|
| `and` | `&&` |
| `or` | `\|\|` |
| `not` | `!` |

```liva
let canVote = age >= 18 and isRegistered
let showBanner = isPremium or isTrial
let isInvalid = not isValid
```

### Range

```liva
1..10      // Exclusive: 1,2,3,...,9
1..=10     // Inclusive: 1,2,3,...,10
```

### Ternary

```liva
let status = age >= 18 ? "Adult" : "Minor"
```

### Member & Index Access

```liva
user.name           // Dot access
items[0]            // Index access
fmt::format         // Method reference (v1.1.0)
```

### Precedence (highest to lowest)

`() [] . ::` → `-` `!` `not` → `* / %` → `+ -` → `..` → `< <= > >=` → `== !=` → `and &&` → `or ||` → `? :` → `=`

---

## Functions

### Arrow Functions (one-liners)

```liva
add(a, b) => a + b
greet(name: string): string => $"Hello, {name}!"
square(x: number): number => x * x
```

### Block Functions

```liva
calculateTotal(items) {
    let total = 0.0
    for item in items {
        total = total + item.price
    }
    return total
}
```

### Default Parameters

```liva
greet(name: string = "World") => $"Hello, {name}!"
```

### Parameter Destructuring

```liva
// Array destructuring in params
printPair([first, second]: [int]): int {
    print("First:", first)
    return first + second
}

// Object destructuring in params
printUser({id, name}: User) {
    print($"User #{id}: {name}")
}

// In lambdas
pairs.forEach(([x, y]) => print($"{x},{y}"))
users.map(({id, name}) => $"User {id}")

// Rest pattern
processList([head, ...tail]: [int]) {
    print("First:", head)
    print("Rest:", tail)
}
```

### Tuple Returns

```liva
// Explicit return type required for tuples
getCoordinates(): (int, int) {
    return (10, 20)
}

let coords = getCoordinates()
print(coords.0)  // 10
print(coords.1)  // 20
```

### Void Functions

```liva
logMessage(msg: string) {
    print($"[LOG] {msg}")
    // No return = void
}
```

### Async Inference

Functions automatically become async if they contain `async` calls — **no `async` keyword needed in declarations**.

### Fallibility

Functions that use `fail` are **fallible** — callers must use error binding.

```liva
divide(a: number, b: number): number {
    if b == 0 { fail "Division by zero" }
    return a / b
}

// Ternary with fail
checkAge(age: number) => age >= 18 ? "Adult" : fail "Minor"
```

### Function References (v1.1.0)

```liva
// Point-free: pass function name directly
items.forEach(print)           // instead of: items.forEach(x => print(x))
nums.map(toString)
names.filter(isValid)

// Method reference with ::
let fmt = Formatter("Hello")
let greetings = names.map(fmt::format)  // binds fmt.format as callback
```

**Supported methods:** `forEach`, `map`, `filter`, `find`, `some`, `every`

---

## Control Flow

### If / Else

```liva
if age >= 18 {
    print("Adult")
} else if age >= 13 {
    print("Teen")
} else {
    print("Child")
}
```

### One-liner `=>` Syntax (v1.1.0)

```liva
if age >= 18 => print("Adult")
if age >= 18 => print("Adult") else => print("Minor")
for item in items => print(item)
while running => tick()
```

### While Loop

```liva
let i = 0
while i < 5 {
    print(i)
    i = i + 1
}
```

### For Loop (Range)

```liva
for i in 0..5 { print(i) }       // 0,1,2,3,4
for i in 1..=10 { print(i) }     // 1 to 10 inclusive
```

### For Loop (Array)

```liva
let names = ["Alice", "Bob"]
for name in names {
    print($"Hello, {name}")
}
```

### Break / Continue

```liva
let i = 0
while i < 10 {
    i = i + 1
    if i == 5 { continue }
    if i == 8 { break }
    print(i)
}
```

### Data-Parallel For Loops

```liva
// Parallel (CPU-bound threads)
for par item in items with chunk 2 threads 4 {
    heavyComputation(item)
}

// Vectorized (SIMD)
for vec value in values with simdWidth 4 {
    compute(value)
}

// Parallel + Vectorized
for parvec value in values with simdWidth 4 ordered {
    process(value)
}
```

### Switch Statement

```liva
switch userType {
    case "admin": print("Admin")
    case "user": print("Regular")
    default: print("Unknown")
}
```

### Try-Catch

```liva
try {
    let result = riskyOperation()
} catch (err) {
    print($"Error: {err}")
}
```

> **Prefer error binding** (`let result, err = ...`) over try-catch for Liva-style error handling.

---

## Pattern Matching

### Switch Expression

```liva
let result = switch value {
    0 => "zero",
    1 => "one",
    _ => "other"     // Wildcard (default)
}
```

### Range Patterns

```liva
let grade = switch score {
    90..=100 => "A",
    80..=89 => "B",
    70..=79 => "C",
    _ => "F"
}
```

### Or-Patterns

```liva
let result = switch day {
    "Saturday" | "Sunday" => "Weekend",
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
    n => n * 2    // 'n' binds to the value
}
```

### Tuple Patterns

```liva
let message = switch status {
    (200, text) => $"Success: {text}",
    (404, _) => "Not Found",
    (code, text) => $"Status {code}: {text}"
}
```

### Enum Destructuring

```liva
let msg = switch shape {
    Shape.Circle(r) => $"Circle r={r}",
    Shape.Rectangle(w, h) => $"Rect {w}x{h}",
    Shape.Point => "Point"
}
```

---

## Classes

### Declaration

```liva
Person {
    name: string
    age: number

    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }

    greet() => $"Hi, I'm {this.name}"
    isAdult(): bool => this.age >= 18
}
```

### Instantiation

```liva
let person = Person("Alice", 25)
person.greet()               // "Hi, I'm Alice"
print(person.isAdult())      // true
```

### Field Defaults (v0.10.4)

```liva
User {
    name: string
    age: int = 18
    role: string = "user"
    active: bool = true

    constructor(name: string) {
        this.name = name
    }
}

let user = User("Alice")   // age=18, role="user", active=true
```

### Default Constructor (no explicit constructor)

```liva
Config {
    host: string = "localhost"
    port: int = 8080
    debug: bool = false
}

let config = Config()   // Uses all defaults
```

### Constructor with Validation

```liva
User {
    username: string
    password: string

    constructor(username: string, password: string) {
        if username == "" { fail "Username required" }
        if password.length < 8 { fail "Password too short" }
        this.username = username
        this.password = password
    }
}
```

### Factory Functions (instead of multiple constructors)

```liva
createSquare(size: number): Rectangle {
    return Rectangle(size, size)
}
```

---

## Data Classes

Classes with fields but **no explicit constructor** are automatically data classes.
Auto-generate: positional constructor, `PartialEq`, and `Display`:

```liva
Point {
    x: number
    y: number
}

let p = Point(10, 20)
print(p)                    // "Point { x: 10, y: 20 }" (auto Display)
print(p == Point(10, 20))   // true (auto PartialEq)
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

### Data Classes in Arrays

```liva
Item {
    name: string
    price: number
}

let items: [Item] = []

// Add items using array concatenation
items = items + [Item("Apple", 150)]
items = items + [Item("Banana", 80)]

// Iterate and access fields with dot notation
for item in items {
    print($"{item.name}: ${item.price}")
}

// Index access also works
let first = items[0]
print(first.name)   // "Apple"
```

---

## Enums

Enums define types with a fixed set of variants (algebraic data types).

### Simple Enums

```liva
enum Color {
    Red,
    Green,
    Blue
}

let c = Color.Red
print(c)              // "Red" (auto Display)
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

### As Parameters and Return Types

```liva
enum SearchResult {
    Found(value: number),
    NotFound
}

findItem(id: number): SearchResult {
    if id > 0 { return SearchResult.Found(id * 10) }
    return SearchResult.NotFound
}
```

> Construction uses dot syntax: `Color.Red`, `Shape.Circle(5)` — NOT `Color::Red`.

---

## Interfaces

```liva
// Interface (only method signatures, no fields or constructor)
Animal {
    makeSound(): string
    getName(): string
}

// Implementation
Dog : Animal {
    name: string

    constructor(name: string) {
        this.name = name
    }

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
    name: string

    constructor(name: string) {
        this.name = name
    }

    makeSound() => "Meow!"
    getName() => this.name
    draw() => print($"Drawing {this.name}")
}
```

### Interface vs Class Detection

The compiler automatically detects interfaces (bodies with only method signatures, no fields, no constructor) vs classes. No special keyword needed.

---

## Visibility

Liva uses **identifier-based visibility** — no `public`/`private` keywords:

| Naming | Visibility | Example |
|--------|-----------|---------|
| Regular name | **Public** | `calculate()`, `name: string` |
| `_` prefix | **Private** | `_helper()`, `_password: string` |

```liva
User {
    name: string        // Public
    _password: string   // Private

    constructor(name: string, password: string) {
        this.name = name
        this._password = password
    }

    validatePassword(input: string): bool {
        return this._password == input
    }
}
```

Applies to: functions, fields, methods, classes, constants.

---

## Error Handling

Liva uses **explicit error handling** instead of exceptions. Functions can `fail`, and callers use **error binding** to capture errors.

### The `fail` Keyword

```liva
divide(a: number, b: number): number {
    if b == 0 { fail "Division by zero" }
    return a / b
}

// Inline fail (ternary)
checkAge(age: number) => age >= 18 ? "Adult" : fail "Minor"
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

- Error is always a **string** — `""` means no error
- Compiler enforces error binding for fallible calls (**E0701**)
- For non-fallible functions, `err` is always `""`

> **IMPORTANT:** Always check errors with `if err {` (truthy check). The compiler transforms this to `!err.is_empty()` in Rust. Do NOT use `if err != ""` — while it works, `if err {` is the idiomatic Liva pattern.

### `or fail` — Error Propagation Shorthand

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

### `or default` — Default Value on Error

```liva
let config = loadConfig("app.toml") or default defaultConfig()
let port = parsePort(input) or default 8080
```

### Common Patterns

```liva
// Early return pattern
processUser(id: number): string {
    let user, err = fetchUser(id)
    if err { fail $"Failed to fetch: {err}" }

    let processed, err2 = transformUser(user)
    if err2 { fail $"Failed to transform: {err2}" }

    return processed
}

// Pipeline with or fail
pipeline(data: string): string {
    let step1 = validate(data) or fail "Validation failed"
    let step2 = transform(step1) or fail "Transform failed"
    let step3 = save(step2) or fail "Save failed"
    return "Pipeline success"
}

// Retry pattern
fetchWithRetry(url: string, maxRetries: number): string {
    for i in 0..maxRetries {
        let data, err = async fetchData(url)
        if !err { return data }
        print($"Attempt {i + 1} failed: {err}")
    }
    fail "Max retries exceeded"
}

// Fallback pattern
getData(): string {
    let data, err = fetchFromPrimary()
    if !err { return data }
    let backup, err2 = fetchFromBackup()
    if !err2 { return backup }
    fail "All data sources failed"
}
```

### Error Binding with Concurrency

```liva
let data, err = async fetchData(url)
let result, err = par processData(50)

let task1 = task async fetchUser(1)
let user, err = await task1
```

---

## Concurrency

Liva provides a **hybrid concurrency model**: async (I/O-bound) + parallel (CPU-bound).

| Keyword | Type | Use Case | Blocks |
|---------|------|----------|--------|
| `async` | Asynchronous | I/O-bound (network, file) | No (lazy await on use) |
| `par` | Parallel | CPU-bound (compute) | No (lazy join on use) |
| `task` | Handle | Explicit control | No (explicit `await`) |
| `fire` | Fire-and-forget | Background work | No |
| `await` | Wait | Wait for task handle | Yes |

### Async (I/O-bound)

```liva
main() {
    let user = async fetchUser(123)
    print($"Got: {user}")   // Auto-awaited on first use
}

// Multiple concurrent calls
main() {
    let u1 = async fetchUser(1)
    let u2 = async fetchUser(2)
    let u3 = async fetchUser(3)
    // All run concurrently, await on use
    print($"{u1}, {u2}, {u3}")
}
```

### Par (CPU-bound)

```liva
main() {
    let result = par heavyComputation(1000)
    print($"Result: {result}")   // Auto-joined on first use
}
```

### Task (Explicit Handles)

```liva
main() {
    let t1 = task async fetchUser(1)
    let t2 = task async fetchUser(2)
    let t3 = task par heavyComputation(100)

    print("Tasks started, doing other work...")

    let user1 = await t1
    let user2 = await t2
    let result = await t3
}
```

### Fire (Fire-and-Forget)

```liva
fire async logEvent("user_login")     // Don't wait for result
fire par backgroundCleanup()
```

### Hybrid Concurrency

```liva
main() {
    let rawData = async fetchFromAPI("/users")     // I/O
    let processed = par processData(rawData)        // CPU
    print($"Final: {processed}")
}
```

### Auto-Async Inference

Functions **automatically become async** if they contain async calls — no `async` keyword needed in declaration.

### Array Execution Policies

```liva
// Sequential (default)
let doubled = numbers.map(x => x * 2)

// Parallel (multi-threading via Rayon)
let doubled = numbers.par().map(x => x * 2)
let result = numbers.par({threads: 4, chunk: 2}).map(x => heavy(x))

// Vectorized (SIMD, currently sequential fallback)
let doubled = numbers.vec().map(x => x * 2)

// Combined parallel + vectorized
let doubled = numbers.parvec().map(x => x * 2)
```

### Data-Parallel For Loops

```liva
for par item in workloads with chunk 2 threads 4 {
    process(item)
}

for parvec lane in data with simdWidth 4 ordered {
    process(lane)
}
```

### Best Practices

- **async** for I/O (network, file, database)
- **par** for CPU (computation, data processing)
- **task** for explicit orchestration
- **fire** for logs, analytics, background cleanup
- Don't use `par` for I/O (wastes threads)
- Don't use `async` for CPU (doesn't utilize cores)

---

## Collections & Arrays

### Creation

```liva
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob"]
let empty: [number] = []
```

### Access

```liva
let first = numbers[0]
let second = names[1]
```

### Adding Elements to Arrays

Arrays in Liva are immutable-size. To add elements, use concatenation:

```liva
let items: [string] = []
items = items + ["apple"]
items = items + ["banana"]
// items is now ["apple", "banana"]
```

### Functional Methods

```liva
let numbers = [1, 2, 3, 4, 5]

// Transform
let doubled = numbers.map(x => x * 2)            // [2, 4, 6, 8, 10]

// Filter
let evens = numbers.filter(x => x % 2 == 0)      // [2, 4]

// Reduce (initial value FIRST, then lambda)
let sum = numbers.reduce(0, (acc, x) => acc + x)  // 15

// Iterate
numbers.forEach(x => print(x))

// Find
let found = numbers.find(x => x > 3)              // Some(4)

// Check
let hasEven = numbers.some(x => x % 2 == 0)       // true
let allPos = numbers.every(x => x > 0)            // true

// Search
let idx = numbers.indexOf(3)                       // 2
let exists = numbers.includes(5)                   // true

// Join
let words: [string] = ["hello", "world"]
let sentence = words.join(" ")                     // "hello world"
```

### Chaining

```liva
let result = numbers
    .filter(x => x > 2)
    .map(x => x * 2)
    .reduce(0, (acc, x) => acc + x)   // 24
```

### Point-Free Function References (v1.1.0)

```liva
items.forEach(print)              // instead of: items.forEach(x => print(x))
nums.map(toString)
names.filter(isValid)

// Method reference with ::
let fmt = Formatter("Hello")
let greetings = names.map(fmt::format)
```

### Destructuring in Lambdas

```liva
let pairs = [[1, 2], [3, 4]]
pairs.forEach(([x, y]) => print($"{x},{y}"))

let points = [[1, 2], [3, 4], [5, 6]]
let sums = points.map(([a, b]) => a + b)     // [3, 7, 11]
let filtered = points.filter(([x, y]) => x > 2)
```

### Parallel Execution

```liva
let doubled = numbers.par().map(x => x * 2)
let results = numbers.par({threads: 4}).map(x => heavy(x))
let large = numbers.par().filter(x => x > 3)
let sum = numbers.par().reduce(0, (acc, x) => acc + x)
```

All array methods support: `.par()`, `.vec()`, `.parvec()` adapters.

---

## Strings & Templates

### String Templates

```liva
let name = "Alice"
let age = 25
let greeting = $"Hello, {name}! You are {age} years old."

// Expression interpolation
let msg = $"Sum: {a + b}"
let info = $"Name: {user.getName().toUpperCase()}"
```

### Escaping Braces in Templates

To include literal `{` or `}` in string templates, use `\{` and `\}`:

```liva
let json = $"\{\"key\": \"{value}\"\}"
// Output: {"key": "some_value"}
```

### String Methods

```liva
let text = "Hello, World!"

text.split(", ")                   // ["Hello", "World!"]
text.toUpperCase()                 // "HELLO, WORLD!"
text.toLowerCase()                 // "hello, world!"
"  hello  ".trim()                 // "hello"
"  hello  ".trimStart()            // "hello  "
"  hello  ".trimEnd()              // "  hello"
text.replace("World", "Liva")      // "Hello, Liva!"
text.startsWith("Hello")           // true
text.endsWith("!")                 // true
text.contains("World")             // true
text.substring(0, 5)               // "Hello"
text.charAt(0)                     // 'H'
text.indexOf("World")              // 7
```

---

## Modules & Imports

Any `.liva` file is a module. Functions/classes/constants are exported by default (unless `_` prefix).

### Named Imports

```liva
import { add, subtract } from "./math.liva"
```

### Wildcard Import

```liva
import * as math from "./math.liva"
math.add(10, 5)
```

### Path Resolution

```liva
import { helper } from "./utils/helper.liva"    // Subdirectory
import { config } from "../config.liva"          // Parent directory
```

- Paths must end with `.liva`
- Paths are resolved relative to the importing file
- Use `/` as path separator (cross-platform)

### Visibility Rules

```liva
// math.liva
add(a, b) => a + b           // Public (exported)
_helper(x) => x * 2          // Private (not exported)
```

---

## Standard Library

### Console I/O

```liva
print("Hello!")                          // Simple output
console.log(data)                        // Debug output (shows structure)
console.error("Error message")           // Red, to stderr
console.warn("Warning")                  // Yellow, to stderr
console.success("Done!")                 // Green, to stdout
let input = console.input("Name: ")     // Read user input
```

### Math

```liva
Math.PI                    // 3.141592653589793
Math.E                     // 2.718281828459045
Math.sqrt(16.0)            // 4.0
Math.pow(2.0, 3.0)         // 8.0
Math.abs(-10.5)            // 10.5
Math.floor(3.7)            // 3
Math.ceil(3.2)             // 4
Math.round(3.5)            // 4
Math.min(10.5, 20.3)       // 10.5
Math.max(10.5, 20.3)       // 20.3
Math.random()              // 0.0 to 1.0
```

### Type Conversions

```liva
let num, err = parseInt("42")
let val, err2 = parseFloat("3.14")
let str = toString(42)
```

> `parseInt` and `parseFloat` use error binding. Check with `if err {`.

### File I/O

```liva
let content, err = File.read("file.txt")
let ok, err2 = File.write("out.txt", "Hello")
let ok2, err3 = File.append("log.txt", "Line\n")
let exists = File.exists("file.txt")          // bool (no error binding needed)
let ok3, err4 = File.delete("temp.txt")
```

All File operations (except `File.exists`) use error binding.

### Directory Operations

```liva
let entries, err = Dir.list("/path/to/dir")    // [string] - file/dir names sorted
let isDir = Dir.isDir("/some/path")            // bool (no error binding needed)
```

`Dir.list` returns sorted file/directory names (not full paths). `Dir.isDir` is non-fallible like `File.exists`.

### System

```liva
let args = Sys.args()              // [string] - command line arguments (index 0 = program)
let home = Sys.env("HOME")        // string - environment variable (empty if not set)
Sys.exit(1)                        // exit program with code
```

### JSON

```liva
// Type-safe parsing (v0.10.0+)
let data: [number], err = JSON.parse("[1, 2, 3]")
let user: User, err = JSON.parse(jsonString)

// Basic parsing (returns JsonValue)
let data, err = JSON.parse("[1, 2, 3]")

// Stringify
let json = JSON.stringify(data)
```

### JSON — Type-Safe Parsing

```liva
// Primitives
let num: i32, err = JSON.parse("42")
let text: String, err = JSON.parse("\"hello\"")
let flag: bool, err = JSON.parse("true")

// Arrays
let nums: [i32], err = JSON.parse("[1, 2, 3]")

// Custom classes
User {
    id: u64
    name: String
    age: i32
}

let user: User, err = JSON.parse("{\"id\": 1, \"name\": \"Alice\", \"age\": 30}")

// Optional fields (v0.10.4+)
Settings {
    theme: string = "dark"
    fontSize: int = 14
    autoSave?: bool = true     // Optional with default
}
```

### HTTP Client

```liva
// GET
let response, err = async HTTP.get("https://api.example.com/users")
if !err {
    print($"Status: {response.status}")
    let data, jsonErr = response.json()
}

// POST
let body = "{\"name\": \"Alice\"}"
let resp, err = async HTTP.post("https://api.example.com/users", body)

// PUT
let resp, err = async HTTP.put("https://api.example.com/users/1", body)

// DELETE
let resp, err = async HTTP.delete("https://api.example.com/users/1")

// Typed JSON response
let data: User, jsonErr = response.json()
```

All HTTP methods are **async by default** with error binding. Response object has: `status` (int), `body` (string), `json()` method.

---

## Testing

Liva includes a built-in test library with a Jest-like API.

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
expect(x).toBe(y)
expect(x).toEqual(y)                    // alias

// Truthiness
expect(x).toBeTruthy()
expect(x).toBeFalsy()

// Comparison
expect(x).toBeGreaterThan(y)
expect(x).toBeLessThan(y)
expect(x).toBeGreaterThanOrEqual(y)
expect(x).toBeLessThanOrEqual(y)

// Collections
expect(x).toContain(y)

// Null
expect(x).toBeNull()

// Errors
expect(x).toThrow()

// Negation
expect(x).not.toBe(y)
expect(x).not.toBeTruthy()
expect(x).not.toContain(y)
```

### Lifecycle Hooks

```liva
describe("Suite", () => {
    beforeAll(() => { /* once before all tests */ })
    afterAll(() => { /* once after all tests */ })
    beforeEach(() => { /* before each test */ })
    afterEach(() => { /* after each test */ })

    test("example", () => {
        expect(true).toBeTruthy()
    })
})
```

### Running Tests

```bash
livac --test                          # All *.test.liva files
livac --test file.test.liva           # Specific file
livac --test --filter "math"          # Filter by name
livac --test --verbose                # Detailed output
```

---

## Common Pitfalls & Known Limitations

### Syntax Pitfalls

1. **No `fn`/`def`/`class` keyword** — write `add(a, b) => a + b` not `fn add(...)` or `def add(...)`
2. **No semicolons** — newline terminates statements
3. **`and`/`or`/`not`** — preferred over `&&`/`||`/`!` (both work)
4. **Switch expressions vs statements** — expressions: `X => val`, statements: `case X:` with colon
5. **Error binding required** — `let value, err = riskyCall()` for fallible functions (E0701)
6. **`or fail`** — shorthand propagation: `let x = riskyCall() or fail "message"`
7. **String templates** — use `$"text {expr}"` not backticks
8. **Private members** — prefix with `_` (e.g., `_count: number`)
9. **`describe`** is reserved for the test framework
10. **Enum construction** — `Color.Red`, `Shape.Circle(5)` (dot syntax, not `::`)
11. **No `+=` or `++`** — use `x = x + 1`
12. **Tuples need explicit types** — `getPoint(): (int, int)` in return type
13. **Ranges** — `1..5` excludes 5, `1..=5` includes 5
14. **reduce argument order** — initial value FIRST: `.reduce(0, (acc, x) => acc + x)`

### Known Limitations

- **No generics** — Generic types (`<T>`) are not yet implemented. Use concrete types.
- **No `push()`/`pop()` array methods** — Use `arr = arr + [element]` to grow arrays.
- **No `Map`/`Set`/`Dictionary` type** — Use arrays of data classes or JSON objects.
- **No inheritance** — Use interfaces and composition.
- **Generating JSON** — Use `JSON.stringify(obj)` for serialization and `JSON.parse(str)` for deserialization. Both work with data classes automatically.

---

## Project Structure & Code Quality

> ⚠️ **CRITICAL FOR AI AGENTS** ⚠️
>
> **ALWAYS generate multi-file projects.** A single monolithic file is NEVER acceptable
> for anything beyond a trivial 50-line script. Split code into focused modules.
> This is mandatory, not optional.

### Mandatory Rules

1. **Split by responsibility** — one file per concern (models, logic, output, utils)
2. **Short functions** — max ~20 lines per function. If longer, decompose it
3. **No god functions** — `main()` should only orchestrate; never contain business logic
4. **Use imports** — Liva has a full module system. Use it
5. **Use `JSON.stringify()`** — never build JSON strings manually
6. **Use data classes** — for struct-like data, declare fields without a constructor
7. **Use `or fail`** — instead of verbose error binding when you just want to propagate
8. **Prefer `if err {`** — not `if err != ""`

### Project Structure

**Small utility (<50 lines):**
```
my-tool/
└── src/
    └── main.liva
```

**Any real project (>50 lines) — ALWAYS use this pattern:**
```
my-project/
└── src/
    ├── main.liva           # Entry point only — orchestration, CLI parsing
    ├── models.liva         # Data classes (types, structs)
    ├── services.liva       # Core business logic
    ├── output.liva         # Formatting / display / JSON output
    └── utils.liva          # Small internal helpers (_private prefix)
```

For larger projects, use subdirectories:
```
larger-project/
└── src/
    ├── main.liva
    ├── models/
    │   ├── user.liva
    │   └── config.liva
    ├── services/
    │   ├── auth.liva
    │   └── data.liva
    └── utils/
        └── strings.liva
```

### Example: Correct Multi-file Project

```liva
// src/models.liva — Data definitions ONLY
data CpuInfo {
    model: string
    cores: number
}

data MemoryInfo {
    totalMB: number
    usedMB: number
    freeMB: number
}

data SystemReport {
    hostname: string
    cpu: CpuInfo
    memory: MemoryInfo
}
```

```liva
// src/services.liva — Business logic
import { CpuInfo, MemoryInfo, SystemReport } from "./models.liva"

readHostname(): string {
    let content = File.read("/etc/hostname") or fail "Cannot read hostname"
    return content.trim()
}

readCpuInfo(): CpuInfo {
    let content = File.read("/proc/cpuinfo") or fail "Cannot read cpuinfo"
    let model = _extractValue(content, "model name")
    let cores = _countOccurrences(content, "processor")
    return CpuInfo(model, cores)
}

readMemoryInfo(): MemoryInfo {
    let content = File.read("/proc/meminfo") or fail "Cannot read meminfo"
    let total = _extractNumber(content, "MemTotal")
    let free = _extractNumber(content, "MemFree")
    return MemoryInfo(total / 1024, (total - free) / 1024, free / 1024)
}

buildReport(): SystemReport {
    let hostname = readHostname() or fail "hostname failed"
    let cpu = readCpuInfo() or fail "cpu info failed"
    let memory = readMemoryInfo() or fail "memory info failed"
    return SystemReport(hostname, cpu, memory)
}

// Private helpers — not exported
_extractValue(content: string, key: string): string {
    let lines = content.split("\n")
    for line in lines {
        if line.contains(key) {
            let idx = line.indexOf(":")
            if idx >= 0 {
                return line.substring(idx + 1, line.length).trim()
            }
        }
    }
    return "unknown"
}

_extractNumber(content: string, key: string): number {
    let val = _extractValue(content, key)
    let parts = val.split(" ")
    let num, err = parseInt(parts[0])
    if err { return 0 }
    return num
}

_countOccurrences(content: string, key: string): number {
    let lines = content.split("\n")
    let count = 0
    for line in lines {
        if line.startsWith(key) { count = count + 1 }
    }
    return count
}
```

```liva
// src/output.liva — Display / serialization
import { SystemReport } from "./models.liva"

printJson(report: SystemReport) {
    let json = JSON.stringify(report) or fail "JSON serialization failed"
    print(json)
}

printPretty(report: SystemReport) {
    print($"Hostname: {report.hostname}")
    print($"CPU: {report.cpu.model} ({report.cpu.cores} cores)")
    print($"Memory: {report.memory.usedMB}MB / {report.memory.totalMB}MB")
}
```

```liva
// src/main.liva — Entry point ONLY
import { buildReport } from "./services.liva"
import { printJson, printPretty } from "./output.liva"

main() {
    let args = Sys.args()
    let jsonMode = args.includes("--json")

    let report, err = buildReport()
    if err {
        console.error($"Error: {err}")
        Sys.exit(1)
    }

    if jsonMode {
        printJson(report)
    } else {
        printPretty(report)
    }
}
```

### Anti-Patterns — NEVER Do This

```
❌ Everything in one file (600+ lines)
❌ main() with 100+ lines of logic
❌ Building JSON with string templates: print($"\"key\": \"{value}\"")
❌ Copy-pasting similar parsing code instead of helper functions
❌ Functions longer than 20-25 lines
❌ Not using imports when project has multiple files
❌ Using `class` when `data` suffices (no methods needed)
❌ Verbose `if err != ""` instead of `if err {`
❌ Verbose error binding when `or fail` suffices
```

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Files | `kebab-case.liva` or `snake_case.liva` | `file-utils.liva` |
| Functions | `camelCase` | `readFile()` |
| Classes/Data | `PascalCase` | `UserProfile` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES` |
| Private | `_` prefix | `_helper()` |
| Fields | `camelCase` | `firstName: string` |

---

## Practical Patterns

### Reading and Parsing Text Files

This is the most common pattern for system tools:

```liva
// Read a file, split into lines, parse key-value pairs
readCpuModel(): string {
    let content, err = File.read("/proc/cpuinfo")
    if err { return "unknown" }

    let lines = content.split("\n")
    for line in lines {
        if line.contains("model name") {
            let idx = line.indexOf(":")
            if idx >= 0 {
                return line.substring(idx + 1, line.length).trim()
            }
        }
    }
    return "unknown"
}
```

### Parsing Numbers from Strings

```liva
// parseInt/parseFloat are fallible — always check err
readTemperature(): number {
    let content, err = File.read("/sys/class/thermal/thermal_zone0/temp")
    if err { return -1 }

    let val, perr = parseInt(content.trim())
    if perr { return -1 }

    return val / 1000
}
```

### Collecting Items with Array Concatenation

```liva
// Build a filtered list from parsed data
readCoreIds(): [string] {
    let content, err = File.read("/proc/cpuinfo")
    if err { return [] }

    let lines = content.split("\n")
    let coreIds: [string] = []

    for line in lines {
        if line.contains("core id") {
            let idx = line.indexOf(":")
            if idx >= 0 {
                let id = line.substring(idx + 1, line.length).trim()
                if not coreIds.includes(id) {
                    coreIds = coreIds + [id]
                }
            }
        }
    }
    return coreIds
}
```

### Tokenizing Whitespace-Separated Data

```liva
// Split by spaces, filter empty strings to extract tokens
_tokenize(text: string): [string] {
    let parts = text.split(" ")
    let tokens: [string] = []
    for p in parts {
        if p != "" {
            tokens = tokens + [p]
        }
    }
    return tokens
}
```

### Generating JSON Output with JSON.stringify

Use `JSON.stringify()` to serialize data classes to JSON. **Never build JSON strings manually.**

```liva
// Data classes auto-derive Serialize — JSON.stringify works out of the box
SystemReport {
    hostname: string
    cpu: CpuInfo
    memory: MemoryInfo
    disks: [DiskInfo]
}

printJsonReport(r: SystemReport) {
    let json, err = JSON.stringify(r)
    if err {
        console.error($"JSON error: {err}")
        return
    }
    print(json)
}
```

> **NEVER** build JSON manually with string templates. Always use `JSON.stringify(obj)`.
> It handles escaping, nested objects, arrays, and special characters correctly.

### Environment Variables and CLI Args

```liva
main() {
    let user = Sys.env("USER")
    let home = Sys.env("HOME")
    let args = Sys.args()

    let verbose = false
    for arg in args {
        if arg == "--verbose" {
            verbose = true
        }
    }

    if verbose {
        print($"User: {user}, Home: {home}")
    }
}
```

### Complete Example: File Processing Tool

> In a real project, split this into `models.liva`, `services.liva`, and `main.liva`.
> Shown here in one block for brevity only.

```liva
// data class — no constructor needed, auto-detected
Package {
    name: string
    version: string
}

// Load packages from file
loadPackages(path: string): [Package] {
    let content = File.read(path) or fail "Cannot read file"
    let packages: [Package] = JSON.parse(content) or fail "Invalid JSON"
    return packages
}

// Filter packages by prefix
filterByPrefix(packages: [Package], prefix: string): [Package] {
    return packages.filter(p => p.name.startsWith(prefix))
}

main() {
    let packages, err = loadPackages("packages.json")

    if err {
        console.error($"Error: {err}")
        return
    }

    let devPackages = filterByPrefix(packages, "dev-")

    for pkg in devPackages {
        print($"{pkg.name}@{pkg.version}")
    }

    print($"Found {devPackages.length} dev packages")

    // JSON output
    let json = JSON.stringify(devPackages) or fail "JSON error"
    print(json)
}
```

---

## Quick Reference Card

### All Reserved Words

```
let const import from as if else while for in break continue switch
case default return fail throw try catch async par parallel task fire
await move seq vec parvec with ordered chunk threads enum type use
rust test true false null and or not safe fast static dynamic auto
detect schedule reduction prefetch simdWidth data

Type keywords: number float bool string char bytes
              i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 isize usize
```

### Operators

```
+ - * / %                    Arithmetic
== != < > <= >=              Comparison
and or not (or && || !)      Logical
= =>                         Assignment / Arrow
? :                          Ternary
.. ..=                       Ranges (exclusive / inclusive)
```

### Delimiters

```
( )    Function calls, grouping
{ }    Blocks
[ ]    Arrays, indexing
,      Separators
:      Type annotations
;      Optional statement terminator
.      Field/method access
::     Method reference
```

---

## Online Documentation (for humans only — AI agents: ignore this section)

The following URLs are web links for **human developers** who want to browse extended documentation in a browser. AI agents must NOT attempt to fetch, read, or navigate to these URLs or any local paths derived from them.

- Full docs: `https://github.com/liva-lang/livac/tree/main/docs`
- Quick ref: `https://github.com/liva-lang/livac/blob/main/docs/QUICK_REFERENCE.md`
- Language ref: `https://github.com/liva-lang/livac/tree/main/docs/language-reference`
- Stdlib ref: `https://github.com/liva-lang/livac/tree/main/docs/language-reference/stdlib`
