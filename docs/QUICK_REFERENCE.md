# Quick Reference — Extended

> Addenda to SKILL.md — only gotchas, edge cases, and features not fully covered there.

---

## 1. CLI Flags

| Flag | Command | Effect |
|------|---------|--------|
| `--output <dir>` | `build` | Custom output directory |
| `--json` | `build`, `check` | Errors as JSON (IDE integration) |
| `--check` | `fmt` | Check formatting without modifying |
| `--verbose` | `build`, `test` | Show generated Rust / individual test results |
| `--template <t>` | `init` | Scaffold: `cli` or `data` |
| `--filter "name"` | `test` | Run only matching tests |
| `--release` | `run` | Release-mode binary |

```bash
livac fmt --check file.liva
livac build --output dist --json file.liva
livac init my-app --template cli
livac test --verbose --filter "Math"
```

---

## 2. Gotcha: `=>` Does NOT Imply Return in Blocks

Function `=>` = implicit return. Control-flow `=>` = shorthand for `{}`, **no return**.

```liva
// Function => → implicit return ✅
square(x: number): number => x * x

// if => inside a block → NO implicit return
clamp(val: number, lo: number, hi: number): number {
    if val < lo => return lo
    if val > hi => return hi
    return val
}

// One-liner forms (no return, just execute)
if age >= 18 => print("Adult") else => print("Minor")
for item in items => print(item)
while running => tick()
```

The LAST expression in a block body is NOT auto-returned. Always use `return` in block functions.

---

## 3. Switch: Binding & Or-Patterns

### Variable binding — captures matched value

```liva
let label = switch num {
    0 => "zero",
    n => $"other: {n}"
}
```

### Or-patterns — multiple values in one arm

```liva
let kind = switch day {
    "Saturday" | "Sunday" => "Weekend",
    _ => "Weekday"
}

let tier = switch code {
    200 | 201 | 204 => "Success",
    400 | 404 => "Client error",
    _ => "Other"
}
```

### Guards + ranges (combine freely)

```liva
let label = switch score {
    n if n >= 90 => "A",
    n if n >= 80 => "B",
    0 => "Missing",
    _ => "F"
}
```

---

## 4. Error Trace Format

```liva
main() {
    let cfg, err = loadConfig("app.toml")
    if err {
        print(err)           // Full box trace (below)
        print(err.message)   // Plain: "config error"
    }
}
```

Output:

```
╭─ Error Trace ─────────────────────────────────────╮
│  ✗ config error                                    │
│    → loadConfig()  main.liva:8                     │
│  ⊘ port is empty                                   │
│    → parsePort()  main.liva:3                      │
╰───────────────────────────────────────────────────╯
```

- `✗` (red) = top-level error
- `⊘` (yellow) = chained cause
- Chaining via `or fail`, `if err => fail`, `if err { fail }`

---

## 5. Recursive Enums

Auto-boxed by compiler — no manual `Box<>`.

```liva
enum Expr {
    Num(value: number),
    Add(left: Expr, right: Expr),
    Mul(left: Expr, right: Expr)
}

enum List {
    Cons(head: number, tail: List),
    Nil
}

// Construction — Box::new() auto-generated
let expr = Expr.Add(Expr.Num(1), Expr.Mul(Expr.Num(2), Expr.Num(3)))
let list = List.Cons(1, List.Cons(2, List.Cons(3, List.Nil)))

// Pattern matching — auto-dereferenced
eval(e: Expr): number {
    return switch e {
        Expr.Num(v) => v
        Expr.Add(l, r) => eval(l) + eval(r)
        Expr.Mul(l, r) => eval(l) * eval(r)
    }
}
```

Array fields like `children: [Tree]` don't need boxing — `Vec<T>` is heap-allocated.

---

## 6. Enums as Return Types

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

main() {
    let result = findItem(5)
    let msg = switch result {
        SearchResult.Found(v) => $"Got {v}",
        SearchResult.NotFound => "Nothing"
    }
    print(msg)
}
```

---

## 7. Multiple Interfaces

```liva
Drawable { draw(): void }
Serializable { serialize(): string }

Cat : Animal, Drawable {
    name: string
    constructor(name: string) { this.name = name }
    makeSound() => "Meow!"
    getName() => this.name
    draw() => print($"Drawing {this.name}")
}
```

Comma-separated after `:`. All interface methods must be implemented.

---

## 8. Point-Free References

### Free functions as callbacks

```liva
items.forEach(print)                // same as: items.forEach(x => print(x))
names.map(toUpperCase)              // same as: names.map(x => toUpperCase(x))
```

### Instance method refs with `::`

```liva
Formatter {
    prefix: string
    constructor(prefix: string) { this.prefix = prefix }
    format(s: string) => $"{this.prefix}: {s}"
}

main() {
    let fmt = Formatter("Hello")
    let greetings = ["Alice", "Bob"].map(fmt::format)
    // → ["Hello: Alice", "Hello: Bob"]
}
```

Works with: `forEach`, `map`, `filter`, `find`, `some`, `every`.

### Point-free with `for =>`

```liva
for item in items => print          // same as: for item in items => print(item)
```

---

## 9. `sortBy` / `groupBy`

Not in SKILL.md — additional array methods:

```liva
let byAge = users.sortBy(u => u.age)           // Sort by extracted key
let groups = users.groupBy(u => u.role)         // Map<string, [User]>

// Example
let people = [
    Person("Alice", 30),
    Person("Bob", 25),
    Person("Carol", 30)
]
let sorted = people.sortBy(p => p.age)          // Bob, Alice, Carol
let byAge = people.groupBy(p => toString(p.age))  // {"30": [Alice, Carol], "25": [Bob]}
```

---

## 10. Map & Set Extra Methods

### Map extras (beyond SKILL.md)

```liva
let ages = Map { "Alice": 30, "Bob": 25 }
let count = ages.length               // 2
ages.clear()                          // Remove all entries
ages.forEach((key, value) => {
    print($"{key} = {value}")
})
```

### Set extras (beyond SKILL.md)

```liva
let colors = Set { "red", "green" }
let vals = colors.values()            // [string] — alias for toArray
let count = colors.length             // 2
colors.clear()                        // Remove all elements
colors.forEach((c) => { print(c) })
```

### `console.success`

```liva
console.success("Done!")              // Green output to stdout
```

---

## 11. Type Alias Expansion

Aliases expand inline at codegen — no Rust `type` declaration emitted.

```liva
type Matrix = [[number]]              // → Vec<Vec<i32>>
type Handler = (Request): Response    // → function signature
type Result<T> = (T, error)           // → generic expansion
type TokenList = [TokenWithSpan]      // → Vec<TokenWithSpan>
```

Generic aliases like `Result<T>` substitute `T` at each usage site.

---

## 12. Testing — current status

> **Reality check (v2.0):** Liva ships a **basic test runner** that
> auto-discovers free functions whose name starts with `test_` and
> executes them, reporting pass/fail. The Jest-style `describe`/`expect`
> matcher API documented in older drafts is **not implemented yet** and
> is planned for v2.x as `liva/test` stdlib module.

### What works today

```liva
// any-file.liva
test_addition() {
    let result = 2 + 2
    if result != 4 {
        fail "addition broken"
    }
}

test_string_concat() {
    if "a" + "b" != "ab" { fail "concat broken" }
}
```

Run with:

```bash
livac test any-file.liva
```

The runner prints `[PASS] test_addition`, `[FAIL] test_string_concat`
and exits non-zero on any failure.

### Roadmap (`liva/test`, v2.x)

`describe(name, fn)` / `test(name, fn)` / `expect(x).toBe(y)` plus
matchers (`toContain`, `toBeTruthy`, `.not.*`) and lifecycle hooks
(`beforeAll`, `afterEach`, …) will land as a stdlib module once the
runtime supports closures-as-handlers stably across modules. Tracked
in `BACKLOG § Fase 11 → C8`.

---

## 13. Reduce: Initial Value FIRST

Unlike JavaScript — the accumulator seed comes **before** the lambda:

```liva
// ✅ Liva — initial value first
let sum = nums.reduce(0, (acc, x) => acc + x)
let product = nums.reduce(1, (acc, x) => acc * x)
let csv = names.reduce("", (acc, n) => acc + ", " + n)

// ❌ WRONG — JS style (lambda first) does NOT work
// nums.reduce((acc, x) => acc + x, 0)
```

---

## 14. `or fail` in Ternary / Combined Patterns

### Ternary with `fail`

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b
```

### `or fail` chaining (reminder)

```liva
let content = File.read("data.txt") or fail "Cannot read file"
let port = Config.getInt(cfg, "PORT") or fail "Missing PORT"
let rows = DB.query(db, "SELECT * FROM users") or fail "Query failed"
```

### `or <default>` combined with ternary

```liva
let port = Config.getInt(cfg, "PORT") or 3000
let name = user.get("name") or "anonymous"
let val = parseInt(input) or 0
```

---

## 15. Optional Operators (`!` unwrap, `?.` chaining)

### Unwrap Operator (`!`)

Force-unwraps optional values. Panics at runtime if `null`.

```liva
let user = find_user("admin")   // string?
print(user!)                    // "Admin" — panics if null
```

### Optional Chaining (`?.`)

Safely accesses fields on optional values. Returns `null` if base is `null`.

```liva
User { name: string; age: number }

let user = find_user("admin")   // User?
let name = user?.name            // string? — "Admin" or null
print(name or "Unknown")        // "Admin"

let nobody = find_user("nobody")
print(nobody?.age or 0)          // 0
```

### Combining with `or`

`or` on an optional value provides a safe default (not logical OR):

```liva
let name = user?.name or "Guest"   // string — always has a value
let age = user?.age or 0           // number — fallback to 0
```

---

## CSV: Custom Separator

```liva
let tsv, err = CSV.read("data.tsv", "\t")
```
