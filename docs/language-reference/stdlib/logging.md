# Logging — `Log` Module

Structured logging with timestamps, log levels, and smart data rendering. All output goes to **stderr** (`eprintln!`).

## Log Methods

```liva
Log.info("Server started")
Log.warn("Disk space low")
Log.error("Connection failed")
Log.debug("Request payload received")
```

Output format: `2026-03-12T10:30:00 [INFO ] Server started`

## Log Levels

| Level | Method | Description |
|-------|--------|-------------|
| 0 | `Log.debug()` | Only with `--verbose` or `LIVA_VERBOSE=1` |
| 1 | `Log.info()` | General info (default) |
| 2 | `Log.warn()` | Warnings |
| 3 | `Log.error()` | Errors |

## Variadic Arguments

All log methods accept multiple arguments, concatenated with spaces:

```liva
Log.info("User", username, "logged in from", ip)
Log.warn("Retry attempt", attempt, "of", maxRetries)
```

Mix strings, numbers, and other types freely.

## Table Rendering

- **Map with 4+ keys** → rendered as Unicode Key/Value table
- **Map with ≤3 keys** → rendered inline: `{code: 200, ok: true}`
- **Array of Maps** → rendered as columnar table (like `console.table`)

```liva
// Renders as table (5 keys)
Log.info("Config:", { host: "localhost", port: 8080, db: "mydb", pool: 10, timeout: 30 })

// Renders inline (2 keys)
Log.info("Status:", { code: 200, ok: true })

// Renders as columnar table
Log.info("Users:", [
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 }
])
```

Scalar arguments and tables can be mixed in the same call.

## JSON Support

JSON values from `JSON.parse()` get the same table treatment at runtime:
- JSON objects with 4+ keys → Key/Value table
- JSON arrays of objects → columnar table
- Small JSON objects → inline

## Setting Log Level

```liva
Log.setLevel("debug")    // Show all messages
Log.setLevel("info")     // Default — info, warn, error
Log.setLevel("warn")     // Only warn and error
Log.setLevel("error")    // Only errors
```

## Debug Mode

`Log.debug()` is only visible when:
1. `Log.setLevel("debug")` has been called, **or**
2. Running with `livac run --verbose` (sets `LIVA_VERBOSE=1`)

## API Summary

| Method | Level | Args |
|--------|-------|------|
| `Log.info(args...)` | 1 | Variadic: strings, numbers, maps, arrays |
| `Log.warn(args...)` | 2 | Same |
| `Log.error(args...)` | 3 | Same |
| `Log.debug(args...)` | 0 | Same |
| `Log.setLevel(level: string)` | — | `"debug"`, `"info"`, `"warn"`, `"error"` |

### Rendering Rules

- **Strings/Numbers** — printed as-is
- **Maps (≤3 keys)** — inline `{key: value, ...}`
- **Maps (4+ keys)** — Key/Value table with Unicode borders
- **Array of Maps** — columnar table
- **JSON values** — runtime-detected, same table rules
- Output: stderr, ISO 8601 timestamps, `chrono = "0.4"` auto-injected
