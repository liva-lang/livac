# Logging — `Log` Module

The `Log` module provides structured logging with timestamps, log levels, and smart rendering of data structures (maps, arrays, JSON).

All output goes to **stderr** (`eprintln!`) so it doesn't interfere with `print()` stdout.

## Table of Contents
- [Basic Usage](#basic-usage)
- [Log Levels](#log-levels)
- [Variadic Arguments](#variadic-arguments)
- [Table Rendering](#table-rendering)
- [JSON Support](#json-support)
- [Setting Log Level](#setting-log-level)
- [Debug Mode](#debug-mode)
- [API Reference](#api-reference)

---

## Basic Usage

```liva
Log.info("Server started")
Log.warn("Disk space low")
Log.error("Connection failed")
Log.debug("Request payload received")
```

**Output:**
```
2026-03-12T10:30:00 [INFO ] Server started
2026-03-12T10:30:00 [WARN ] Disk space low
2026-03-12T10:30:00 [ERROR] Connection failed
2026-03-12T10:30:00 [DEBUG] Request payload received
```

Each log line includes:
- **ISO 8601 timestamp** (via `chrono`)
- **Log level** label (color-coded in terminal)
- **Message** content

---

## Log Levels

| Level | Method | Description | Output |
|-------|--------|-------------|--------|
| 0 | `Log.debug()` | Detailed debugging info | Only with `--verbose` |
| 1 | `Log.info()` | General informational messages | Always (default) |
| 2 | `Log.warn()` | Warning conditions | Always |
| 3 | `Log.error()` | Error conditions | Always |

Default level is `info` (1). `Log.debug()` messages are only visible when running with `livac --run --verbose` or when `LIVA_VERBOSE=1` environment variable is set.

---

## Variadic Arguments

All log methods accept multiple arguments. They are concatenated with spaces:

```liva
Log.info("User", username, "logged in from", ip)
// 2026-03-12T10:30:00 [INFO ] User alice logged in from 192.168.1.1

Log.info("Processing", count, "items in", elapsed, "ms")
// 2026-03-12T10:30:00 [INFO ] Processing 42 items in 156 ms
```

You can mix strings, numbers, and other types:

```liva
Log.warn("Retry attempt", attempt, "of", maxRetries)
Log.error("Failed after", duration, "seconds with code", statusCode)
```

---

## Table Rendering

### Map with 4+ keys → Key/Value Table

When a `Map` literal with 4 or more keys is passed to a log method, it renders as a formatted table with Unicode box-drawing borders:

```liva
Log.info("Config:", { host: "localhost", port: 8080, db: "mydb", pool: 10, timeout: 30 })
```

**Output:**
```
2026-03-12T10:30:00 [INFO ] Config:
   ┌─────────┬───────────┐
   │ Key     │ Value     │
   ├─────────┼───────────┤
   │ db      │ mydb      │
   │ host    │ localhost │
   │ pool    │ 10        │
   │ port    │ 8080      │
   │ timeout │ 30        │
   └─────────┴───────────┘
```

### Map with ≤3 keys → Inline

Small maps are rendered inline for brevity:

```liva
Log.info("Status:", { code: 200, ok: true })
// 2026-03-12T10:30:00 [INFO ] Status: {code: 200, ok: true}
```

### Array of Maps → Columnar Table

An array of map literals renders as a columnar table (similar to `console.table` in Node.js):

```liva
Log.info("Users:", [
    { name: "Alice", age: 30 },
    { name: "Bob", age: 25 },
    { name: "Charlie", age: 35 }
])
```

**Output:**
```
2026-03-12T10:30:00 [INFO ] Users:
   ┌─────┬─────────┐
   │ age │ name    │
   ├─────┼─────────┤
   │ 30  │ Alice   │
   │ 25  │ Bob     │
   │ 35  │ Charlie │
   └─────┴─────────┘
```

### Mixed Variadic + Tables

You can combine scalar arguments with tables:

```liva
Log.info("Found", 3, "results:", [
    { id: 1, name: "Alice" },
    { id: 2, name: "Bob" },
    { id: 3, name: "Charlie" }
])
// Prints: "Found 3 results:" followed by the table
```

---

## JSON Support

JSON values from `JSON.parse()` are automatically detected at compile time and rendered with smart formatting at runtime:

### JSON Object → Table or Inline

```liva
let json_str = "{\"name\": \"Alice\", \"age\": 30, \"city\": \"Madrid\", \"role\": \"Engineer\", \"level\": \"Senior\"}"
let config, _err = JSON.parse(json_str)
Log.info("Config:", config)
```

**Output (5 keys → table):**
```
2026-03-12T10:30:00 [INFO ] Config:
   ┌───────┬──────────┐
   │ Key   │ Value    │
   ├───────┼──────────┤
   │ age   │ 30       │
   │ city  │ Madrid   │
   │ level │ Senior   │
   │ name  │ Alice    │
   │ role  │ Engineer │
   └───────┴──────────┘
```

Small JSON objects (≤3 keys) render inline:

```liva
let small, _err = JSON.parse("{\"host\": \"localhost\", \"port\": 8080}")
Log.info("Config:", small)
// 2026-03-12T10:30:00 [INFO ] Config:
//    {host: localhost, port: 8080}
```

### JSON Array of Objects → Columnar Table

```liva
let arr_json = "[{\"name\": \"Alice\", \"age\": 30}, {\"name\": \"Bob\", \"age\": 25}]"
let users, _err = JSON.parse(arr_json)
Log.info("Users:", users)
```

**Output:**
```
2026-03-12T10:30:00 [INFO ] Users:
   ┌─────┬───────┐
   │ age │ name  │
   ├─────┼───────┤
   │ 30  │ Alice │
   │ 25  │ Bob   │
   └─────┴───────┘
```

**Note:** JSON table rendering uses runtime detection (via `serde_json::Value` inspection), so the threshold and format are determined when the program runs.

---

## Setting Log Level

Change the minimum log level at runtime:

```liva
Log.setLevel("debug")    // Show all messages
Log.setLevel("info")     // Show info, warn, error (default)
Log.setLevel("warn")     // Show only warn and error
Log.setLevel("error")    // Show only errors
```

```liva
main() {
    Log.setLevel("debug")
    Log.debug("This will now be visible")
    Log.info("Normal info")
    
    Log.setLevel("error")
    Log.info("This will NOT be visible")
    Log.error("This will be visible")
}
```

---

## Debug Mode

`Log.debug()` is only visible when:
1. `Log.setLevel("debug")` has been called, **or**
2. The program is run with `livac --run --verbose`, which sets `LIVA_VERBOSE=1`

This allows adding debug logging that is silent in production:

```liva
main() {
    Log.debug("Initializing...")           // Only visible with --verbose
    Log.info("Server started on port 8080") // Always visible
}
```

```bash
livac app.liva --run              # Only shows INFO and above
livac app.liva --run --verbose    # Also shows DEBUG messages
```

---

## API Reference

### `Log.info(args...)`
Log an informational message. Level: 1.

### `Log.warn(args...)`
Log a warning message. Level: 2.

### `Log.error(args...)`
Log an error message. Level: 3.

### `Log.debug(args...)`
Log a debug message. Level: 0. Only visible with `--verbose` or `Log.setLevel("debug")`.

### `Log.setLevel(level: string)`
Set the minimum log level. Valid values: `"debug"`, `"info"`, `"warn"`, `"error"`.

### Arguments

All log methods accept variadic arguments:
- **Strings** — printed as-is
- **Numbers** — converted to string
- **Maps (≤3 keys)** — rendered inline: `{key: value, ...}`
- **Maps (4+ keys)** — rendered as Key/Value table
- **Array of Maps** — rendered as columnar table
- **JSON values** — runtime-detected: objects/arrays get table treatment

### Output

- All output goes to **stderr** via `eprintln!`
- Timestamp format: ISO 8601 (`YYYY-MM-DDTHH:MM:SS`)
- Tables use Unicode box-drawing characters: `┌─┬┐│├┼┤└┴┘`

### Dependencies

The Log module automatically adds `chrono = "0.4"` to the generated `Cargo.toml` for timestamp formatting.
