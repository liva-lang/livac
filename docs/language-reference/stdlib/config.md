# Config — Environment Configuration Module

The `Config` module loads `.env` files and provides typed value retrieval. All operations return `(value, error)` tuples.

## Basic Usage

```liva
let config, err = Config.load(".env")
if err {
    Log.error("Failed to load config:", err)
}

let host, _ = Config.get(config, "HOST")
let port, _ = Config.getInt(config, "PORT")
let debug, _ = Config.getBool(config, "DEBUG")

print($"Server: {host}:{port} (debug={debug})")
```

## API Reference

| Function | Signature | Description |
|----------|-----------|-------------|
| `Config.load(path)` | `(string) → (Map, string)` | Load `.env` file |
| `Config.get(config, key)` | `(Map, string) → (string, string)` | Get string value |
| `Config.getInt(config, key)` | `(Map, string) → (int, string)` | Get integer value |
| `Config.getBool(config, key)` | `(Map, string) → (bool, string)` | Get boolean value |
| `Config.getAll(config)` | `(Map) → Map<string, string>` | Get all entries (sorted) |

## Config.load(path)

Reads and parses a `.env` file. Skips blank lines and `#` comments. Strips surrounding quotes from values.

## Config.get(config, key)

Returns `(string, string)`. Error: `"Config key not found: KEY"` if missing.

## Config.getInt(config, key)

Returns `(int, string)`. Error if key not found or value not parseable as integer.

## Config.getBool(config, key)

Returns `(bool, string)`. Truthy values: `"true"`, `"1"`, `"yes"`, `"on"` (case-insensitive). Error only if key missing.

## Config.getAll(config)

Returns `Map<string, string>` sorted by key.

## .env File Format

```env
# Comments start with #
DB_HOST=localhost
DB_PORT=5432
APP_NAME="My Application"
SECRET_KEY='super-secret'
DEBUG=true
```

Rules: `KEY=VALUE` per line, `#` comments, blank lines skipped, surrounding quotes stripped, no `$VAR` interpolation.

## Error Handling

```liva
let config, err = Config.load(".env")
if err {
    Log.error("Config error:", err)
}

let port, err = Config.getInt(config, "PORT")
if err {
    let port = 3000  // fallback
}
```

*Since: v1.5.0*
