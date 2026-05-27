# Liva Style Guide

Idiomatic conventions for clean, consistent Liva code.

---

## 1. One-Liner `=>` vs Block `{}`

> **One statement → `=>`**. Two or more → `{}`.

Applies to **functions, if/else, for, while, defer** — all bodies:

```liva
// ✅ Single expression → =>
add(a, b) => a + b
greet(name: string): string => $"Hello, {name}!"
if age >= 18 => print("Adult")
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
```

Use `{}` when: body has **2+ statements**, or line would exceed **~90 characters**.

---

## 2. Naming Conventions

| Element | Style | Example |
|---------|-------|---------|
| Variables, functions | camelCase | `let userName`, `getUser()` |
| Classes, Enums, Interfaces | PascalCase | `Person`, `Color`, `Printable` |
| Constants | SCREAMING_SNAKE_CASE | `const MAX_RETRIES = 3` |
| Private fields/methods | `_` prefix | `_password`, `_validate()` |
| Files, modules | snake_case or kebab-case | `user_service.liva` |

**Booleans:** prefix with `is`, `has`, `can`, `should`:

```liva
let isActive = true
let hasPermission = checkAccess(user)
canDelete(user) => user.role == "admin"
```

---

## 3. Error Handling

### `or fail` for propagation

```liva
let content = File.read(path) or fail "Cannot read config"
let config: Config = JSON.parse(content) or fail "Invalid JSON"
```

### `or <default>` for fallbacks

```liva
let port = parseInt(portStr) or 3000
let name = config.get("name") or "Anonymous"
```

### Manual binding when inspecting the error

```liva
let data, err = async fetchUser(id)
if err {
    Log.error($"Failed to fetch user {id}: {err}")
    return null
}
```

### Guard clauses with `fail`

```liva
validateAge(age: number): number {
    if age < 0 => fail "Age cannot be negative"
    if age > 150 => fail "Unrealistic age"
    return age
}
```

### Error check idiom — always `if err` (truthy), not `if err != ""`

```liva
let result, err = divide(10, 0)
if err => print($"Error: {err}")
```

---

## 4. Functions

- **One expression** → `=>`
- **2+ lines** → `{}`
- **Guard + return** → `{}`

```liva
square(n) => n * n
fullName(first, last) => $"{first} {last}"

processOrder(order) {
    let total = order.items.reduce(0, (acc, i) => acc + i.price)
    let taxed = total + (total * TAX_RATE / 100)
    return taxed
}
```

### Point-free

```liva
// ✅
items.forEach(print)
names.map(toUpperCase)

// ❌ Redundant wrapper
items.forEach(x => print(x))
```

### Type annotations

Annotate **public API** functions. Omit types for internal/obvious code:

```liva
fetchUser(id: number): User { ... }       // Public: annotated
_sum(a, b) => a + b                       // Internal: inferred
```

---

## 5. Classes

Data class (fields only) — compiler auto-generates constructor, `==`, `toString`. Add `constructor()` only for **validation**:

```liva
// Data class — no constructor needed
Point { x: number; y: number }
let p = Point(10, 20)

// Validated class
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
```

**Body order:** fields → constructor → methods.

### Splitting a class with `extend`

Use `extend ClassName { ... }` when a class grows large enough to merit its
own set of files. File naming convention: owner is `name.liva`, extensions
are `name_<concern>.liva`.

```liva
// emitter.liva           — owner: fields + constructor + tiny helpers
// emitter_expr.liva      — extend Emitter { _emitExpr*() ... }
// emitter_stmt.liva      — extend Emitter { _emitStmt*() ... }
```

Extensions add **methods only** (no fields, no constructor). Diagnostics
E0910–E0913 guard misuse. See
[class-extensions.md](../language-reference/class-extensions.md).

---

## 6. Collections

### Prefer functional methods

```liva
let total = prices.reduce(0, (acc, p) => acc + p)
let adults = users.filter(u => u.age >= 18)
let names = users.map(u => u.name)
```

### Map/Set

```liva
let config = Map { "host": "localhost", "port": "8080" }
let tags = Set { "urgent", "new" }
let name = users.get(id) or "Unknown"
```

---

## 7. Pattern Matching

```liva
// Switch expression — returns a value
let label = switch status {
    Status.Active => "Active",
    Status.Inactive => "Inactive",
    _ => "Unknown"
}

// Switch statement — performs actions
switch command {
    case "start": startServer()
    case "stop": stopServer()
    default: showHelp()
}
```

Always handle all enum variants or include `_` wildcard.

---

## 8. Concurrency

| Need | Use | Example |
|------|-----|---------|
| I/O-bound | `async` | `let data = async fetch(url)` |
| CPU-bound | `par` | `let result = par heavyCalc(n)` |
| Multiple independent | `task` | `let t = task async fetch(url)` |
| Fire-and-forget | unassigned `async` | `async logEvent("click")` |
| Parallel collection | `.par()` / `for par` | `nums.par().map(x => x * 2)` |

Auto-await: `async`/`par` calls launch immediately; result is awaited on first **use** of the variable. Don't use `async` if the result is used on the very next line (no concurrency gained).

```liva
// ✅ Real concurrency — independent work between launch and use
let users = async fetchUsers(url)
let config = loadDisplayConfig()       // Runs while fetch is in flight
let report = buildReport(users, config) // Awaits users here
```

---

## 9. Code Organization

- **Single responsibility** per function
- **~20–30 lines** max per function
- **`main()` as orchestrator** — delegate, don't implement

```liva
main() {
    let config = loadConfig() or fail "Bad config"
    let users = fetchUsers(config.apiUrl) or fail "Fetch failed"
    let report = buildReport(users)
    displayReport(report)
}
```

**Dependency direction:** higher-level → lower-level, never reverse.

---

## 10. Imports

Group: stdlib first, then project modules:

```liva
import { describe, test, expect } from "liva/test"

import { User, Role } from "./models.liva"
import { fetchUser } from "./api/client.liva"
```

Prefer named imports over `import *`.

---

## 11. Comments

- **Why**, not **what**
- Section headers for logical blocks
- Brief doc comments on public API

---

## 12. Formatting

- **4 spaces** indent (no tabs) — `livac fmt` handles this
- **≤90 characters** per line
- One blank line between function definitions

---

## 13. Testing

```liva
import { describe, test, expect } from "liva/test"

describe("Calculator", () => {
    test("addition", () => {
        expect(add(2, 3)).toBe(5)
    })
    test("division by zero fails", () => {
        expect(() => divide(10, 0)).toThrow()
    })
})
```

Files: `*.test.liva`. One concept per test.

| Assertion | Matcher |
|-----------|--------|
| Equality | `expect(x).toBe(5)` |
| Negation | `expect(x).not.toBe(0)` |
| Truthiness | `expect(x).toBeTruthy()` |
| Contains | `expect(list).toContain("a")` |
| Throws | `expect(() => f()).toThrow()` |

---

## 14. String Templates

Always `$"..."` for interpolation, never `+` concatenation:

```liva
let msg = $"User {user.name} (ID: {user.id}) logged in"
```

---

## 15. Defer

Acquire → defer release → use. `defer` guarantees cleanup on scope exit.

```liva
let db = DB.open("app.db") or fail "Cannot open database"
defer DB.close(db)
return DB.query(db, "SELECT * FROM users") or fail "Query failed"
```

Single action → `defer stmt`. Multiple → `defer { ... }`. LIFO order.

> ⚠️ **Known limitation (B112):** `defer` captures variables by mutable
> reference for the lifetime of the scope. If you `defer` an operation that
> mutates a binding **and then read or further mutate that same binding** in
> the rest of the scope, the borrow checker rejects it (`E0499` / `E0502`).
> Workaround: defer cleanup of *external* resources (DB connections, files,
> locks) — not in-scope `let` bindings you still need to touch. Test scope
> mutations using a fresh binding instead:
>
> ```liva
> defer items.push(99)   // ❌ items.push(4) below fails to borrow-check
> // …
> defer DB.close(db)     // ✅ db is consumed at scope exit only
> ```
>
> Resolution is tracked for a future `defer` redesign.

---

## 16. Rust Interop

Use `rust { }` as a **last resort** — prefer Liva stdlib. Keep blocks small. Declare `use rust` at file top:

```liva
use rust "uuid" version "1.0" features ["v4"]

let id = rust {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}
```

---

## Anti-Patterns

| ❌ Don't | ✅ Do | Why |
|----------|-------|-----|
| `if x { doThing() }` | `if x => doThing()` | `=>` for single statements |
| `items.forEach(x => print(x))` | `items.forEach(print)` | Point-free |
| `if err != "" { ... }` | `if err { ... }` | Truthy check |
| `!isValid` | `not isValid` | `not` reads better, matches `and`/`or` |
| `Color::Red` | `Color.Red` | Dot syntax |
| `fn add(a, b)` | `add(a, b) => a + b` | No `fn` keyword |
| `"Hi " + name` | `$"Hi {name}"` | String templates |
| `print("ERROR: ...")` | `Log.error(...)` | Structured logging |
| Manual `close()` at end | `defer close()` after open | Guaranteed cleanup |
| `rust { }` for stdlib features | Use Liva stdlib | Only for missing functionality |
| `async f()` used immediately | `f()` (no async) | No gain if awaited next line |
