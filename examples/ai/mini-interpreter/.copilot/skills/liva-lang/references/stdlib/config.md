# Config — Environment Configuration Module

The `Config` module provides `.env` file loading and typed value retrieval, following the standard `KEY=VALUE` format.

All Config operations return `(value, error)` tuples, consistent with Liva's explicit error handling pattern.

## Table of Contents
- [Basic Usage](#basic-usage)
- [Loading Config](#loading-config)
- [Getting Values](#getting-values)
- [Typed Getters](#typed-getters)
- [Getting All Values](#getting-all-values)
- [Error Handling](#error-handling)
- [.env File Format](#env-file-format)
- [API Reference](#api-reference)

---

## Basic Usage

```liva
// Load a .env file
let config, err = Config.load(".env")
if err {
    Log.error("Failed to load config:", err)
}

// Read values
let host, _ = Config.get(config, "HOST")
let port, _ = Config.getInt(config, "PORT")
let debug, _ = Config.getBool(config, "DEBUG")

print($"Server: {host}:{port} (debug={debug})")
```

---

## Loading Config

### Config.load(path)

Reads and parses a `.env` file, returning a config map.

```liva
let config, err = Config.load(".env")
if err {
    print("Error:", err)
}
```

- Returns `(Map<string, string>, string)` — the config map and an error string
- Error string is empty (`""`) on success
- Skips blank lines and comment lines (starting with `#`)
- Strips surrounding quotes (`"..."` and `'...'`) from values

---

## Getting Values

### Config.get(config, key)

Retrieves a string value from the config map.

```liva
let config, _ = Config.load(".env")
let host, err = Config.get(config, "HOST")
if err {
    print("Missing HOST:", err)
}
print(host)  // "localhost"
```

- Returns `(string, string)` — value and error
- Error: `"Config key not found: KEY"` if the key doesn't exist

---

## Typed Getters

### Config.getInt(config, key)

Retrieves an integer value, parsing the string.

```liva
let config, _ = Config.load(".env")
let port, err = Config.getInt(config, "PORT")
if err {
    print("Invalid PORT:", err)
}
print(port)  // 8080
```

- Returns `(int, string)` — parsed integer and error
- Error if key not found or value can't be parsed as integer

### Config.getBool(config, key)

Retrieves a boolean value.

```liva
let config, _ = Config.load(".env")
let debug, err = Config.getBool(config, "DEBUG")
if err {
    print("Invalid DEBUG:", err)
}
print(debug)  // true
```

- Returns `(bool, string)` — parsed boolean and error
- Truthy values: `"true"`, `"1"`, `"yes"`, `"on"` (case-insensitive)
- All other values resolve to `false`
- Error only if the key doesn't exist

---

## Getting All Values

### Config.getAll(config)

Returns all key-value pairs as a sorted map.

```liva
let config, _ = Config.load(".env")
let all = Config.getAll(config)
Log.info("Config entries:", all.len())
```

- Returns `Map<string, string>` (sorted by key)
- Useful for debugging or iterating all config values

---

## Error Handling

Config follows Liva's `let value, err = ...` error pattern:

```liva
let config, err = Config.load(".env")
if err {
    // File not found or unreadable
    Log.error("Config error:", err)
}

let port, err = Config.getInt(config, "PORT")
if err {
    // Key missing or not a valid integer
    let port = 3000  // fallback value
}
```

Error strings are non-empty on failure, enabling the `if err { ... }` idiom.

---

## .env File Format

```env
# Database settings
DB_HOST=localhost
DB_PORT=5432
DB_NAME=myapp

# App settings
APP_NAME="My Application"
SECRET_KEY='super-secret'
DEBUG=true
PORT=8080
MAX_CONNECTIONS=100
```

### Rules
- **Format:** `KEY=VALUE` (one per line)
- **Comments:** Lines starting with `#` are ignored
- **Blank lines:** Skipped
- **Quotes:** Surrounding `"double"` or `'single'` quotes are stripped
- **Whitespace:** Leading/trailing whitespace in values is preserved (except quotes)
- **No interpolation:** `$VAR` references are NOT expanded

---

## API Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `Config.load(path)` | `(string) → (Map, string)` | Load `.env` file |
| `Config.get(config, key)` | `(Map, string) → (string, string)` | Get string value |
| `Config.getInt(config, key)` | `(Map, string) → (int, string)` | Get integer value |
| `Config.getBool(config, key)` | `(Map, string) → (bool, string)` | Get boolean value |
| `Config.getAll(config)` | `(Map) → Map<string, string>` | Get all entries (sorted) |

---

## Generated Rust

Config uses only the Rust standard library — no external crates required:

- `std::fs::read_to_string` for file reading
- `std::collections::HashMap` for internal storage
- `std::collections::BTreeMap` for sorted output (`getAll`)

---

*Since: v1.5.0*
