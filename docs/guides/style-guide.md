# Liva Style Guide

Idiomatic conventions for writing clean, consistent Liva code. Follow these rules to produce code that is readable, maintainable, and aligned with the language's design philosophy.

---

## 1. One-Liner `=>` vs Block `{}`

> **One statement → `=>`**. Two or more statements → `{}`.

This applies to **functions, if/else, for, and while** — all bodies follow the same rule:

```liva
// ✅ Single expression/statement → =>
add(a, b) => a + b
greet(name: string): string => $"Hello, {name}!"
if age >= 18 => print("Adult")
if b == 0 => fail "Division by zero"
if err => return
for item in items => print(item)
while alive => tick()

// ✅ If-else one-liner
if age >= 18 => print("Adult") else => print("Minor")

// ✅ Point-free (even shorter)
items.forEach(print)
names.map(toUpperCase)

// ✅ Multiple statements → {}
calculate(items) {
    let total = 0
    for item in items => total = total + item.price
    return total
}

if err {
    console.error($"Error: {err}")
    return
}

// ❌ Avoid: braces for a single statement
add(a, b) { return a + b }
if b == 0 { fail "Division by zero" }
for item in items { print(item) }
```

Use `{}` without hesitation when:
- The body has **2+ statements**
- The line would exceed **~90 characters** with `=>`

---

## 2. Naming Conventions

| Element | Style | Example |
|---------|-------|---------|
| Variables, functions | camelCase | `let userName`, `getUser()` |
| Classes, Enums, Interfaces | PascalCase | `Person`, `Color`, `Printable` |
| Constants | SCREAMING_SNAKE_CASE | `const MAX_RETRIES = 3` |
| Private fields/methods | `_` prefix + camelCase | `_password`, `_validate()` |
| Files, modules | snake_case or kebab-case | `user_service.liva`, `api-client.liva` |

```liva
// ✅ Good naming
const MAX_ITEMS = 100
let currentUser = getActiveUser()

UserService {
    _cache: Map<string, User>
    
    fetchUser(id: number): User { ... }
    _validateToken(token: string): bool { ... }
}

// ❌ Bad naming
const max_items = 100        // Constants: SCREAMING_SNAKE_CASE
let CurrentUser = ...        // Variables: camelCase
user_service { ... }         // Classes: PascalCase
```

### Booleans

Prefix boolean variables and functions with `is`, `has`, `can`, `should`:

```liva
let isActive = true
let hasPermission = checkAccess(user)
canDelete(user) => user.role == "admin"
```

---

## 3. Error Handling

### Prefer `or fail` for propagation

When you don't need to inspect the error, `or fail` is cleaner than manual error binding:

```liva
// ✅ Concise — or fail
loadConfig(path: string): Config {
    let content = File.read(path) or fail "Cannot read config"
    let config: Config = JSON.parse(content) or fail "Invalid JSON"
    return config
}

// ❌ Verbose — manual binding when not needed
loadConfig(path: string): Config {
    let content, err1 = File.read(path)
    if err1 => fail $"Cannot read config: {err1}"
    let config: Config, err2 = JSON.parse(content)
    if err2 => fail $"Invalid JSON: {err2}"
    return config
}
```

### Prefer `or <default>` for fallbacks

```liva
// ✅ Clean
let port = parseInt(portStr) or 3000
let name = config.get("name") or "Anonymous"

// ❌ Verbose
let port, err = parseInt(portStr)
if err => port = 3000
```

### Use manual binding when you need the error

```liva
// ✅ Correct — when you inspect or log the error
let data, err = async fetchUser(id)
if err {
    console.error($"Failed to fetch user {id}: {err}")
    return null
}
```

### Guard clauses with fail

Place validation at the top of functions. Use `=>` for single-line guards:

```liva
// ✅ Guard clauses at the top
validateAge(age: number): number {
    if age < 0 => fail "Age cannot be negative"
    if age > 150 => fail "Unrealistic age"
    return age
}

// ❌ Nested validation
validateAge(age: number): number {
    if age >= 0 {
        if age <= 150 {
            return age
        } else {
            fail "Unrealistic age"
        }
    } else {
        fail "Age cannot be negative"
    }
}
```

### Error check idiom

Always use `if err {` (truthy check), never `if err != ""`:

```liva
// ✅ Idiomatic
let result, err = divide(10, 0)
if err => print($"Error: {err}")

// ❌ Verbose
let result, err = divide(10, 0)
if err != "" { print($"Error: {err}") }
```

---

## 4. Functions

### Arrow vs block threshold

- **One expression** → always `=>`
- **2–3 lines with clear logic** → `{}` block
- **Guard + return** → `{}` block (even if short)

```liva
// ✅ Pure expression → arrow
square(n) => n * n
fullName(first, last) => $"{first} {last}"
urgencyScore(u: Urgency): number => switch u {
    Urgency.Critical => 4,
    Urgency.High => 3,
    Urgency.Normal => 2,
    Urgency.Low => 1
}

// ✅ Multiple steps → block
processOrder(order) {
    let total = order.items.reduce(0, (acc, i) => acc + i.price)
    let taxed = total + (total * TAX_RATE / 100)
    return taxed
}
```

### Prefer point-free when applicable

```liva
// ✅ Point-free
items.forEach(print)
names.map(toUpperCase)
for item in items => process

// ❌ Redundant wrapper
items.forEach(x => print(x))
names.map(x => toUpperCase(x))
for item in items => process(item)
```

### Type annotations

Annotate **public API** functions. Omit types for internal/obvious code:

```liva
// ✅ Public: annotated
fetchUser(id: number): User { ... }
calculateTax(amount: number, rate: float): float => amount * rate

// ✅ Internal: inferred is fine
_sum(a, b) => a + b
let doubled = nums.map(x => x * 2)
```

---

## 5. Classes

### Data class vs regular class

If a class has only fields (and optionally methods), it's a **data class** — the compiler auto-generates the constructor, `==`, and `toString`. Add an explicit `constructor()` only when you need **validation**:

```liva
// ✅ Data class — no constructor needed
Point {
    x: number
    y: number
}
let p = Point(10, 20)   // Auto-generated constructor

// ✅ Regular class — constructor validates
User {
    email: string
    age: number

    constructor(email: string, age: number) {
        if email == "" => fail "Email required"
        if age < 0 => fail "Age must be positive"
        this.email = email
        this.age = age
    }
}

// ❌ Unnecessary constructor — just repeats the fields
Point {
    x: number
    y: number
    constructor(x: number, y: number) {
        this.x = x
        this.y = y
    }
}
```

### Class body order

Always: **fields → constructor → methods**:

```liva
TodoItem {
    // 1. Fields
    title: string
    done: bool = false

    // 2. Constructor (only if needed)
    constructor(title: string) {
        if title == "" => fail "Title required"
        this.title = title
    }

    // 3. Methods
    toggle() => this.done = !this.done
    display(): string => $"[{if this.done => "x" else => " "}] {this.title}"
}
```

### Prefer composition over deep hierarchies

Use small interfaces. A class implements only what it needs:

```liva
// ✅ Focused interfaces
Readable { read(): string }
Writable { write(data: string) }

LogFile : Writable {
    path: string
    write(data: string) => File.append(this.path, data) or fail "Write error"
}
```

---

## 6. Collections

### Prefer functional methods over manual loops

```liva
// ✅ Functional
let total = prices.reduce(0, (acc, p) => acc + p)
let adults = users.filter(u => u.age >= 18)
let names = users.map(u => u.name)
let found = users.find(u => u.id == targetId)

// ❌ Manual loop for what a method does better
let total = 0
for p in prices { total = total + p }
```

### Array growth

Two options: `push` (mutates in place) or concatenation (creates new array):

```liva
// ✅ push — mutates the array
result.push(newItem)
this.books.push(title)

// ✅ Concatenation — creates a new array
result = result + [newItem]
result = result + items.filter(x => x.active)
```

Prefer `push` for building up arrays in loops. Use concatenation when you want a new array or are chaining with functional methods.

### Map/Set initialization

Use the `Map {}` / `Set {}` literal syntax:

```liva
// ✅ Literal
let config = Map { "host": "localhost", "port": "8080" }
let tags = Set { "urgent", "new" }

// ✅ Empty
let cache = Map {}
let visited = Set {}
```

### Map access with `or`

Always provide a default for `map.get()`:

```liva
// ✅ Safe access
let name = users.get(id) or "Unknown"
let count = counters.get(key) or 0

// ⚠️ Risky — get() on a missing key
let name = users.get(id)
```

---

## 7. Pattern Matching

### Switch expressions for value mapping

Use switch expressions (with `=>`) to map values. Use switch statements (with `case:`) for side effects:

```liva
// ✅ Switch expression — returns a value
let label = switch status {
    Status.Active => "Active",
    Status.Inactive => "Inactive",
    _ => "Unknown"
}

// ✅ Switch statement — performs actions
switch command {
    case "start": startServer()
    case "stop": stopServer()
    default: showHelp()
}
```

### Exhaustive matching

Always handle all enum variants or include `_` as catch-all:

```liva
// ✅ All variants covered
let icon = switch urgency {
    Urgency.Critical => "[!!!]",
    Urgency.High => "[!!]",
    Urgency.Normal => "[!]",
    Urgency.Low => "[-]"
}

// ✅ Catch-all for open-ended values
let category = switch ext {
    ".liva" => "Source",
    ".md" => "Docs",
    _ => "Other"
}
```

---

## 8. Concurrency

### Choose the right primitive

| Need | Use | Example |
|------|-----|---------|
| I/O-bound (HTTP, files) | `async` | `let data = async fetch(url)` |
| CPU-bound (computation) | `par` | `let result = par heavyCalc(n)` |
| Multiple independent tasks | `task` | `let t = task async fetch(url)` |
| Fire-and-forget | unassigned `async` | `async logEvent("click")` |
| Parallel collection | `.par()` or `for par` | `nums.par().map(x => x * 2)` |

### Task grouping

All `async` and `par` calls launch immediately without blocking. The result is auto-awaited/joined on first **use** of the variable:

```liva
// ✅ All three launch concurrently — await happens on first use
let u1 = async fetchUser(1)    // Launches, does NOT block
let u2 = async fetchUser(2)    // Launches, does NOT block
let u3 = async fetchUser(3)    // Launches, does NOT block

// All 3 are running. Awaits happen here when values are accessed:
print($"User 1: {u1.name}")   // Awaits u1 here
print($"User 2: {u2.name}")   // Awaits u2 here
print($"User 3: {u3.name}")   // Awaits u3 here
```

Use `task` + explicit `await` when you need fine-grained control over when to collect results:

```liva
// task + await — explicit control
let t1 = task async fetchUser(1)
let t2 = task async fetchUser(2)
let u1 = await t1
let u2 = await t2
```

### Fire-and-forget

When you don't assign to a variable, the call runs in the background with no await:

```liva
async logEvent("user_login")    // Fire-and-forget, no result needed
```

### Don't use `async` if you need the result immediately

Since `async` auto-awaits on first **use**, calling `async` and using the variable on the very next line gains nothing — it behaves identically to a synchronous call. Only use `async` when there is independent work between the launch and the first use:

```liva
// ❌ Pointless async — used immediately, no concurrency gained
let users = async fetchUsers(url)
let report = buildReport(users)    // Awaits here → same as sync

// ✅ No async needed — just call it directly
let users = fetchUsers(url)
let report = buildReport(users)

// ✅ Useful async — independent work happens while fetch is in flight
let users = async fetchUsers(url)      // Launches
let config = loadDisplayConfig()       // Runs while fetch is in flight
let report = buildReport(users, config) // Awaits users here — real gain
```

---

## 9. Code Organization & Design Principles

### Single Responsibility

Every function should do **one thing**. If you can describe what a function does with "and", split it:

```liva
// ❌ Does two things: validates AND saves
processUser(user: User) {
    if user.name == "" => fail "Name required"
    if user.age < 0 => fail "Invalid age"
    let data = JSON.stringify(user)
    File.write("users.json", data) or fail "Write failed"
}

// ✅ Split into focused functions
validateUser(user: User) {
    if user.name == "" => fail "Name required"
    if user.age < 0 => fail "Invalid age"
}

saveUser(user: User) {
    let data = JSON.stringify(user)
    File.write("users.json", data) or fail "Write failed"
}

processUser(user: User) {
    validateUser(user) or fail "Validation failed"
    saveUser(user) or fail "Save failed"
}
```

### Keep functions short and focused

A function should be **readable at a glance**. If you need to scroll to understand it, it's too long. Aim for functions that:
- Fit in one screen (~20–30 lines max)
- Have one level of abstraction
- Have a name that fully describes what they do

```liva
// ❌ Long function mixing abstraction levels
generateReport(users: [User]) {
    let active = users.filter(u => u.isActive)
    let total = active.reduce(0, (acc, u) => acc + u.orders)
    let avg = total / active.length
    let header = "=== Report ==="
    let separator = "─".repeat(40)
    print(header)
    print(separator)
    for user in active {
        let pct = (user.orders * 100) / total
        print($"  {user.name}: {user.orders} orders ({pct}%)")
    }
    print(separator)
    print($"  Total: {total} | Avg: {avg}")
}

// ✅ Each function is one abstraction level
calcStats(users: [User]): Stats {
    let active = users.filter(u => u.isActive)
    let total = active.reduce(0, (acc, u) => acc + u.orders)
    return Stats(active, total, total / active.length)
}

printReport(stats: Stats) {
    printHeader("Report")
    for user in stats.users => printUserLine(user, stats.total)
    printFooter(stats.total, stats.avg)
}

generateReport(users: [User]) {
    let stats = calcStats(users)
    printReport(stats)
}
```

### Split modules when it adds value

Don't split files just because they're long — split when **separation adds clarity**. Good reasons to split:
- Different responsibilities (models vs logic vs I/O)
- Reusable utilities that other modules could import
- Reducing cognitive load (a reader only needs part of the file)

Bad reasons to split:
- "The file has more than X lines"
- Functions that are tightly coupled and always used together

```
// ✅ Split by responsibility
src/
├── main.liva           // Orchestration
├── models.liva         // Data classes, enums
├── api.liva            // HTTP calls, external I/O
├── validation.liva     // Input validation, guards
└── format.liva         // Display, formatting helpers
```

### Dependency direction

Higher-level modules depend on lower-level ones, never the reverse:

```
main.liva → api.liva → models.liva
main.liva → format.liva → models.liva
                ↑ never imports from main
```

### `main()` as orchestrator

`main()` should read like a table of contents — delegate, don't implement:

```liva
// ✅ main() orchestrates
main() {
    let config = loadConfig() or fail "Bad config"
    let users = fetchUsers(config.apiUrl) or fail "Fetch failed"
    let report = buildReport(users)
    displayReport(report)
}

// ❌ main() does everything
main() {
    let content = File.read("config.json") or fail "No config"
    let config: Config = JSON.parse(content) or fail "Bad JSON"
    let resp = HTTP.get(config.url) or fail "HTTP error"
    let users: [User] = resp.json() or fail "Parse error"
    let active = users.filter(u => u.isActive)
    let total = active.reduce(0, (acc, u) => acc + u.score)
    print("=== Report ===")
    for user in active {
        print($"  {user.name}: {user.score}")
    }
    print($"Total: {total}")
}
```

### Open/Closed: design for extension

Use enums and switch expressions so adding a variant only requires adding a case, not changing existing logic:

```liva
// ✅ Adding a new Shape only requires a new variant + case
enum Shape {
    Circle(radius: float),
    Rectangle(w: float, h: float),
    Triangle(base: float, height: float)    // Easy to add
}

area(s: Shape): float => switch s {
    Shape.Circle(r) => Math.PI * r * r,
    Shape.Rectangle(w, h) => w * h,
    Shape.Triangle(b, h) => 0.5 * b * h    // Just add a case
}
```

---

## 10. Imports

Group imports: standard library first, then project modules, separated by a blank line:

```liva
// ✅ Ordered imports
import { describe, test, expect } from "liva/test"

import { User, Role } from "./models.liva"
import { fetchUser } from "./api/client.liva"
import { formatName } from "./utils/format.liva"
```

Import only what you use. Prefer named imports over `import *`:

```liva
// ✅ Named — clear what's used
import { formatDate, formatNumber } from "./utils/format.liva"

// ⚠️ Wildcard — okay for small modules
import * as math from "./math.liva"
```

---

## 11. Comments

### When to comment

- **Why**, not **what**. The code shows *what*; comments explain *why*.
- Section headers for logical blocks in longer functions.
- Public API functions: brief doc comment.

```liva
// ✅ Explains "why"
// Retry up to 3 times because the API is flaky during deployments
let data = fetchWithRetry(url, 3) or fail "API unavailable"

// ❌ Restates the code
// Add a and b
add(a, b) => a + b
```

### Section separators

For longer files, use a consistent separator:

```liva
// --- Constants ---
const MAX_RETRIES = 3
const TIMEOUT = 5000

// --- Models ---
User { name: string; email: string }

// --- API ---
fetchUser(id: number): User { ... }
```

---

## 12. Formatting

### Indentation

4 spaces. No tabs. (The `livac --fmt` formatter handles this.)

### Line length

Aim for **≤90 characters**. Break long lines logically:

```liva
// ✅ Break at logical points
let result = items
    .filter(x => x.active)
    .map(x => x.name)
    .join(", ")

// ✅ Break long string templates
print($"User {user.name} (id={user.id}) "
    + $"has {user.orders.length} orders")
```

### Blank lines

One blank line between:
- Function definitions
- Logical sections within a function
- Import block and first declaration

```liva
import { User } from "./models.liva"

const MAX_USERS = 100

fetchUser(id: number): User { ... }

deleteUser(id: number) { ... }
```

---

## 13. Testing

### Use the Jest-like API

```liva
import { describe, test, expect } from "liva/test"

describe("Calculator", () => {
    test("addition", () => {
        expect(add(2, 3)).toBe(5)
        expect(add(-1, 1)).toBe(0)
    })

    test("division by zero fails", () => {
        expect(() => divide(10, 0)).toThrow()
    })
})
```

### Test naming

- **Files:** `*.test.liva` (required by the test runner)
- **Descriptions:** natural language, describe behavior: `"returns empty list when no matches"`
- **One concept per test** — if you'd use "and" in the name, split it

```liva
// ✅ Focused tests
test("filters active users", () => { ... })
test("returns empty list when no users", () => { ... })

// ❌ Testing multiple things
test("filters users and sorts by name", () => { ... })
```

### Keep tests independent

Use `beforeEach` for setup, not shared mutable state across tests:

```liva
describe("UserService", () => {
    beforeEach(() => {
        // Fresh state for each test
    })

    test("creates user", () => { ... })
    test("rejects duplicate email", () => { ... })
})
```

### Matchers cheat sheet

| Assertion | Matcher |
|-----------|--------|
| Equality | `expect(x).toBe(5)` |
| Negation | `expect(x).not.toBe(0)` |
| Truthiness | `expect(x).toBeTruthy()` |
| Null | `expect(x).toBeNull()` |
| Comparison | `expect(x).toBeGreaterThan(3)` |
| Contains | `expect(list).toContain("a")` |
| Throws | `expect(() => f()).toThrow()` |

---

## 14. String Templates

Always use `$"..."` for string interpolation. Never concatenate with `+`:

```liva
// ✅ String template
let msg = $"User {user.name} (ID: {user.id}) logged in"

// ❌ Manual concatenation
let msg = "User " + user.name + " (ID: " + user.id + ") logged in"
```

Keep expressions inside `{}` simple. Extract complex logic to a variable:

```liva
// ✅ Clean — extract first
let total = items.reduce(0, (acc, i) => acc + i.price)
print($"Total: {total}")

// ❌ Complex expression buried in template
print($"Total: {items.reduce(0, (acc, i) => acc + i.price)}")
```

---

## Quick Reference: Style Decision Tree

```
Is it a single expression/statement?
├── Yes → use =>
│   ├── Function body?     → add(a, b) => a + b
│   ├── if body?           → if x > 0 => print(x)
│   ├── for body?          → for i in items => process(i)
│   └── while body?        → while alive => tick()
└── No (2+ statements) → use {}
    ├── Function body?     → calculate(x) { ... return y }
    ├── if body?           → if err { log(err); return }
    └── Loop body?         → for i in items { ...; ... }

Need error handling?
├── Propagate error?       → or fail "message"
├── Default on error?      → or <value>
└── Inspect/log error?     → let val, err = call()
                              if err => ...

Mapping values from enum/string?
├── Return a value?        → switch expr (=> arrows)
└── Perform actions?       → switch stmt (case: colons)
```

---

## Anti-Patterns

| ❌ Don't | ✅ Do | Why |
|----------|-------|-----|
| `if x { doThing() }` | `if x => doThing()` | `=>` for single statements |
| `items.forEach(x => print(x))` | `items.forEach(print)` | Point-free is cleaner |
| `if err != "" { ... }` | `if err { ... }` | Truthy check is idiomatic |
| `x = x + 1; y = y + 1` | Semicolons anywhere | Liva has no semicolons (newlines) |
| `let result, _ = f()` silently | Handle or `or fail`/`or default` | Don't swallow errors |
| Everything in `main()` | Extract into functions | Readability, testability |
| `Color::Red` | `Color.Red` | Dot syntax, not Rust's `::` |
| `let x += 1` | `x = x + 1` | No compound assignment |
| `fn add(a, b)` | `add(a, b) => a + b` | No `fn`/`def` keyword |
| `async f()` used immediately | `f()` (no async) | No concurrency if awaited on next line |
| `"Hi " + name` | `$"Hi {name}"` | Use string templates |

---

*This guide reflects Liva v1.3.0. Run `livac --fmt` to auto-format your code.*
