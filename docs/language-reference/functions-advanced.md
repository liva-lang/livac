# Functions: Advanced

> Basic function syntax, error binding, and `or fail`/`or <default>` are in SKILL.md. This file covers async inference rules, fallibility inference, function references, visibility, and closures.

## Async Inference

The compiler auto-detects async functions. **No `async` keyword in declarations.**

### Rules

1. Function contains `async` calls → function is async
2. Function contains `await` expressions → function is async
3. Function calls another async function → **transitive** — also becomes async

```liva
// Auto-async: contains async call
fetchUser(id: number): string {
    let data = async getFromDatabase(id)
    return data.name
}

// Transitive: calls fetchUser (which is async)
processData(url: string): string {
    let data = fetchUser(url)
    return data.toUpperCase()
}
```

### Manual Await

```liva
let userTask = task async fetchUser(1)
// ... do other work ...
let user = await userTask
```

## Fallibility Inference

Functions using `fail` are **fallible** — callers must use error binding.

### Rules

1. Function contains `fail` statement → fallible (returns `Result<T, Error>`)
2. Ternary with `fail` works: `age >= 18 ? "Adult" : fail "Minor"`
3. Non-fallible functions: `let val, err = add(10, 20)` — `err` is always `""`
4. Multiple `fail` points allowed in one function

```liva
validateUser(username: string, password: string): string {
    if username == "" { fail "Username empty" }
    if password.length < 8 { fail "Password too short" }
    if password == "12345678" { fail "Password too weak" }
    return $"User {username} validated"
}
```

## Function References

### Point-Free (Free Functions)

Pass function name directly as single-argument callback:

```liva
nums.forEach(print)              // instead of: x => print(x)
let doubled = nums.map(double)   // instead of: x => double(x)
let pos = nums.filter(isPositive)
let strs = nums.map(toString)
```

**Supported methods:** `forEach`, `map`, `filter`, `find`, `some`, `every`

Also works with `for =>` one-liners:

```liva
for item in items => print       // instead of: for item in items => print(item)
```

### Method References with `::`

Bind an instance method as a callback using `object::method`:

```liva
Formatter {
    prefix: string
    constructor(prefix: string) { this.prefix = prefix }
    format(s: string) => $"{this.prefix}: {s}"
}

main() {
    let fmt = Formatter("Hello")
    let greetings = names.map(fmt::format)
    // ["Hello: Alice", "Hello: Bob", "Hello: Charlie"]

    let validator = Validator(3)
    let valid = names.filter(validator::isValid)
}
```

### When to Use

| Scenario | Syntax | Example |
|----------|--------|---------|
| Built-in function | bare name | `items.forEach(print)` |
| User function | bare name | `nums.map(double)` |
| Instance method | `object::method` | `names.map(fmt::format)` |
| Complex expression | lambda | `nums.map(x => x * 2 + 1)` |
| Multi-argument | lambda | `nums.reduce((a, b) => a + b, 0)` |

> Function references work for **single-argument callbacks only**.

## Visibility

Identifier-based — no `public`/`private` keywords.

| Prefix | Visibility | Exported? |
|--------|-----------|-----------|
| `letter` | Public | Yes |
| `_` | Private | No |

```liva
calculatePrice(q, p) => q * p        // Public
_validateInput(data) => data != null  // Private (same module only)
```
