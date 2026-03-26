# Error Handling — Fallibility System

> SKILL.md covers: `fail`, error binding (`let val, err = call()`), `or fail`, `or <default>`, `if err {`.
> This file: error trace chaining mechanics, `err.message` vs `print(err)`, nested propagation, E0701.

## Error Type Internals

`err` is internally `Option<Error>` with a `.message` field:

| Expression | What it does |
|-----------|-------------|
| `if err {` | Truthy when error exists (`err.is_some()`) |
| `err.message` | Plain string — just the message text |
| `print(err)` | Full trace with function names, file locations, chained causes |
| `$"Error: {err}"` | Same as `print(err)` — full trace in string interpolation |

**Never** use `if err != ""` or `if err == ""` — use `if err {` / `if !err {`.

## Error Trace Chaining

When `or fail` adds a message, it creates a **chain** — the original error is preserved as a cause:

```liva
readConfig(path: string): string {
    let content = File.read(path) or fail "Cannot read config"
    return content
}

main() {
    let config, err = readConfig("/missing.toml")
    if err {
        print(err.message)   // "Cannot read config"
        print(err)            // "Cannot read config
                              //   caused by: No such file or directory (os error 2)
                              //   at readConfig() in main.liva:2"
    }
}
```

### `or fail` (bare — no message)

Propagates the original error unchanged — no new chain link:

```liva
let data = File.read("f.txt") or fail   // Propagates original error as-is
```

### `or fail "message"` (with message)

Wraps the original error as a cause:

```liva
let data = File.read("f.txt") or fail "Cannot read file"
// err.message = "Cannot read file"
// print(err) shows the full chain including the original OS error
```

## Nested Error Propagation

Each `or fail` adds a layer to the trace:

```liva
loadDB(): string {
    let content = File.read("db.json") or fail "Cannot read DB file"
    let data = JSON.parse(content) or fail "Invalid DB format"
    return data
}

initApp(): string {
    let db = loadDB() or fail "Database init failed"
    return db
}

main() {
    let app, err = initApp()
    if err {
        // err.message = "Database init failed"
        // print(err) shows full chain:
        //   "Database init failed"
        //     caused by: Cannot read DB file
        //       caused by: No such file or directory
        //     at loadDB() in app.liva:2
        //     at initApp() in app.liva:7
        print(err)
    }
}
```

## E0701: Missing Error Binding

The compiler **requires** error binding for fallible function calls:

```liva
divide(a: number, b: number): number {
    if b == 0 { fail "Division by zero" }
    return a / b
}

main() {
    // ❌ E0701: Missing error binding
    let x = divide(10, 2)

    // ✅ Correct
    let x, err = divide(10, 2)
    if err { print(err) }
}
```

E0701 applies everywhere: assignments, string templates, binary operations, function arguments.

## Ignoring Errors

Use `_` to explicitly discard:

```liva
let result, _ = divide(10, 2)    // Ignore error
let _, err = validateUser("x")   // Ignore result
```

## Error Handling with Async/Par

Error binding works identically with concurrency:

```liva
let data, err = async fetchData(url)
if err { print($"Async error: {err}") }

let result, err2 = par heavyCalc(input)
if err2 { print($"Parallel error: {err2}") }
```

## Common Patterns

### Retry

```liva
fetchWithRetry(url: string, maxRetries: number): string {
    for i in 0..maxRetries {
        let data, err = async HTTP.get(url)
        if !err { return data.body }
        print($"Attempt {i + 1} failed: {err.message}")
    }
    fail "Max retries exceeded"
}
```

### Fallback chain

```liva
getData(): string {
    let data, err = fetchFromPrimary()
    if !err { return data }

    let backup, err2 = fetchFromBackup()
    if !err2 { return backup }

    fail "All sources failed"
}
```
