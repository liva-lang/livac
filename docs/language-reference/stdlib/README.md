# Liva Standard Library

> **Status:** ✅ Complete (v1.8.0-dev) - 115+ functions implemented! 🎉  
> **Completion:** Arrays ✅ (31) | Strings ✅ (28) | Math ✅ (14) | Config ✅ (5) | File ✅ (11) | Dir ✅ (7) | Regex ✅ (5) | Date ✅ (14) | CSV ✅ (8) | Random ✅ (5) | Crypto ✅ (4) | Process ✅ (4) | Server ✅ (3) | Response ✅ (3) | DB ✅ (4) | Conversions ✅ | I/O ✅ | System ✅ | Logging ✅

The Liva Standard Library provides built-in functions and methods for common programming tasks.

---

## 📚 Modules

### ✅ [Array Methods](./arrays.md)
Methods for working with arrays and collections.

**Status:** Complete (31 methods)

**Core (v1.0-v1.3):**
- `map(fn)`, `filter(fn)`, `reduce(fn, initial)`, `forEach(fn)`, `find(fn)`
- `some(fn)`, `every(fn)`, `indexOf(value)`, `includes(value)`, `join(sep)`, `length`

**v1.4 — Access & Slicing:**
- `first()`, `last()`, `isEmpty()`, `slice(start, end?)`, `take(n)`, `drop(n)`

**v1.4 — Transform:**
- `sort()`, `reversed()`, `distinct()`, `flat()`, `chunks(n)`, `zip(other)`

**v1.4 — Aggregate:**
- `sum()`, `min()`, `max()`

**v1.4 — Callback-based:**
- `findIndex(fn)`, `flatMap(fn)`, `count(fn)`

### ✅ [String Methods](./strings.md)
Methods for string manipulation and queries.

**Status:** Complete (28 methods)

**Core (v1.0-v1.3):**
- `split(delimiter)`, `replace(old, new)`, `toUpperCase()`, `toLowerCase()`
- `trim()`, `trimStart()`, `trimEnd()`, `startsWith(prefix)`, `endsWith(suffix)`
- `contains(substring)`, `substring(start, end)`, `charAt(index)`, `indexOf(substring)`

**v1.4 — New:**
- `lastIndexOf(sub)`, `slice(start, end?)`, `padStart(len, char?)`, `padEnd(len, char?)`
- `repeat(n)`, `replaceAll(old, new)`, `chars()`, `capitalize()`, `isBlank()`, `isEmpty()`
- `reverse()`, `truncate(len)`, `countMatches(sub)`, `removePrefix(pre)`, `removeSuffix(suf)`

### ✅ [Math Functions](./math.md)
Mathematical operations and constants.

**Status:** Complete (14 functions/constants)

- `Math.sqrt(x)` - Square root ✅
- `Math.pow(base, exp)` - Power ✅
- `Math.abs(x)` - Absolute value ✅
- `Math.floor(x)`, `ceil(x)`, `round(x)` - Rounding ✅
- `Math.min(a, b)`, `max(a, b)` - Min/max values ✅
- `Math.random()` - Random number ✅
- `Math.PI`, `Math.E` - Constants ✅
- `Math.clamp(val, min, max)` - Clamp to range ✅ *(v1.4)*
- `Math.sign(val)` - Sign (-1, 0, 1) ✅ *(v1.4)*
- `Math.log(x)` - Natural logarithm ✅ *(v1.4)*

### ✅ [Type Conversion](./conversions.md)
Functions for converting between types.

**Status:** Complete (3/3 functions)

- `parseInt(str)` - Parse string to integer with error binding ✅
- `parseFloat(str)` - Parse string to float with error binding ✅
- `toString(value)` - Convert value to string ✅
- `toNumber(str)` - Convert string to number (future enhancement)
- `toInt(value)` - Convert to integer (future enhancement)
- `toFloat(value)` - Convert to float (future enhancement)

### ✅ [Console/IO](./io.md)
Input/output and console functions.

**Status:** Complete (5/5 functions)

- `console.log(...)` - Print to stdout ✅
- `console.error(...)` - Print to stderr ✅
- `console.warn(...)` - Print warning to stderr ✅
- `readLine()` - Read line from stdin ✅
- `prompt(message)` - Display prompt and read input ✅

### ✅ [System Functions](./system.md)
System-level functions for CLI args, environment, and process control.

**Status:** Complete (3/3 functions)

- `Sys.args()` - Get command-line arguments ✅
- `Sys.env(name)` - Read environment variable ✅
- `Sys.exit(code)` - Exit with code ✅

### ✅ [Logging](./logging.md)
Structured logging with timestamps, levels, and smart table rendering.

**Status:** Complete (v1.5) — 5 methods + variadic args + table rendering

- `Log.info(args...)` - Informational messages ✅
- `Log.warn(args...)` - Warning messages ✅
- `Log.error(args...)` - Error messages ✅
- `Log.debug(args...)` - Debug messages (only with `--verbose`) ✅
- `Log.setLevel(level)` - Set minimum log level at runtime ✅
- Variadic arguments (concatenated with spaces) ✅
- Map 4+ keys → Key/Value table, ≤3 keys → inline ✅
- Array of Maps → columnar table (console.table style) ✅
- JSON runtime detection → auto table rendering ✅

### ✅ [Config](./config.md)
Environment configuration loading from `.env` files with typed getters.

**Status:** Complete (v1.5) — 5 functions

- `Config.load(path)` - Load and parse `.env` file ✅
- `Config.get(config, key)` - Get string value ✅
- `Config.getInt(config, key)` - Get integer value ✅
- `Config.getBool(config, key)` - Get boolean value ✅
- `Config.getAll(config)` - Get all entries as sorted map ✅
- No external crates — uses only `std::fs` and `std::collections` ✅

### ✅ [File & Dir I/O](../file-io.md)
File and directory operations with error binding pattern.

**Status:** Complete (v1.6) — 11 File + 7 Dir = 18 functions

**File (v0.9+):**
- `File.read(path)`, `File.write(path, content)`, `File.append(path, content)` ✅
- `File.exists(path)`, `File.delete(path)` ✅

**File (v1.6 — new):**
- `File.copy(src, dest)`, `File.move(src, dest)` ✅
- `File.size(path)`, `File.extension(path)` ✅
- `File.readLines(path)`, `File.writeLines(path, lines)` ✅

**Dir (v1.3+):**
- `Dir.list(path)`, `Dir.isDir(path)` ✅

**Dir (v1.6 — new):**
- `Dir.exists(path)`, `Dir.create(path)`, `Dir.delete(path)` ✅
- `Dir.listRecursive(path)` / `Dir.walk(path)` ✅

### ✅ [Regex](./regex.md)
Regular expression matching, searching, replacing, and splitting.

**Status:** Complete (v1.6) — 5 functions + crate `regex` auto-injected

- `Regex.test(pattern, text)` — Boolean match test ✅
- `Regex.match(pattern, text)` — First match with error binding ✅
- `Regex.findAll(pattern, text)` — All matches as `[string]` ✅
- `Regex.replace(pattern, text, replacement)` — Replace all occurrences ✅
- `Regex.split(pattern, text)` — Split by pattern ✅

### ✅ [Date](./date.md)
First-class date/time type with constructors, properties, methods, and comparisons.

**Status:** Complete (v1.6) — 4 constructors + 6 properties + 4 methods + comparisons + interpolation

**Constructors:** `Date.now()`, `Date.new(y,m,d)`, `Date.parse(str, pattern)`, `Date.timestamp()`
**Properties:** `.year`, `.month`, `.day`, `.hour`, `.minute`, `.second` → `int`
**Methods:** `d.format(pattern)`, `d.add(n, unit)`, `d.diff(other, unit)`, `d.toString()`
**Comparisons:** `>`, `<`, `>=`, `<=`, `==`, `!=`
**Interpolation:** `$"{date}"` auto-formats as ISO 8601

Crate `chrono` auto-injected when `Date.*` is used.

### ✅ [CSV](./csv.md)
Read, write, and manipulate CSV data with Table support.

**Status:** Complete (v1.6) — 8 functions, pure Rust `std` (no external crates)

**I/O (fallible):** `CSV.read(path)`, `CSV.read(path, sep)`, `CSV.write(path, data)`, `CSV.readTable(path)`, `CSV.writeTable(path, table)`
**Pure:** `CSV.parse(text)`, `CSV.stringify(rows)`
**Table ops:** `CSV.headers(table)`, `CSV.column(table, colName)`
**Table type:** `[Map<string, string>]` — use standard array methods for filter/sort/group

### ✅ Random
Random number generation, array shuffling, and UUID creation.

**Status:** Complete (v1.7) — 5 functions, crates `rand` + `uuid` auto-injected

- `Random.nextInt(min, max)` → `int` — random integer in range
- `Random.nextFloat([min, max])` → `float` — random float (args optional)
- `Random.choice(arr)` → `T` — random element from array
- `Random.shuffle(arr)` → `[T]` — shuffled copy of array
- `Random.uuid()` → `string` — UUID v4

### ✅ Crypto
Hashing and encoding utilities.

**Status:** Complete (v1.7) — 4 functions, crates `sha2`, `md-5`, `base64` auto-injected

- `Crypto.sha256(input)` → `string` — hex-encoded SHA-256 hash
- `Crypto.md5(input)` → `string` — hex-encoded MD5 hash
- `Crypto.base64Encode(input)` → `string` — Base64 encode
- `Crypto.base64Decode(input)` → `string, error` — Base64 decode (fallible)

### ✅ Process
Process execution and control.

**Status:** Complete (v1.7) — 4 functions, no external crates (`std::process`)

- `Process.exec(cmd)` → `string, error` — execute command, capture stdout (fallible)
- `Process.spawn(cmd)` → `int, error` — spawn background process, return PID (fallible)
- `Process.pid()` → `int` — current process PID
- `Process.exit(code)` — exit with status code

### ✅ [DB](./db.md)
SQLite database operations for persistent storage.

**Status:** Complete (v1.8) — 4 functions, crate `rusqlite` (bundled) auto-injected

- `DB.open(path)` → `connection, error` — open/create SQLite database (fallible)
- `DB.exec(db, sql[, params])` → `_, error` — execute SQL (CREATE/INSERT/UPDATE/DELETE) (fallible)
- `DB.query(db, sql[, params])` → `rows, error` — query rows as `[Map<string, string>]` (fallible)
- `DB.close(db)` — close the connection

### ✅ [Server](./server.md)
HTTP server creation and routing via axum.

**Status:** Complete (v1.7) — 3 functions, crate `axum` + `tokio` auto-injected

- `Server.create()` → `App` — create a new HTTP server app
- `app.get(path, handler)` / `app.post(...)` / `app.put(...)` / `app.delete(...)` — register route handlers
- `app.listen(port)` — start listening on the given port

### ✅ [Response](./response.md)
HTTP response helpers for route handlers.

**Status:** Complete (v1.7) — 3 functions

- `Response.text(s)` → `Response` — plain text response
- `Response.json(s)` → `Response` — JSON response with `application/json` content type
- `Response.status(code)` → `Response` — set HTTP status code

---

## 🚀 Quick Start

### Array Methods

```liva
let numbers = [1, 2, 3, 4, 5]

// Transform
let doubled = numbers.map(x => x * 2)
print(doubled)  // [2, 4, 6, 8, 10]

// Filter
let evens = numbers.filter(x => x % 2 == 0)
print(evens)  // [2, 4]

// Reduce
let sum = numbers.reduce((acc, x) => acc + x, 0)
print(sum)  // 15

// Search
let hasThree = numbers.includes(3)
print(hasThree)  // true
```

### String Methods

```liva
let text = "Hello, World!"

// Case conversion
print(text.toUpperCase())  // "HELLO, WORLD!"
print(text.toLowerCase())  // "hello, world!"

// Substring operations
let words = text.split(", ")
print(words)  // ["Hello", "World!"]

let greeting = text.substring(0, 5)
print(greeting)  // "Hello"

// Search
let hasWorld = text.indexOf("World")
print(hasWorld)  // 7

let startsWithHello = text.startsWith("Hello")
print(startsWithHello)  // true
```

### Math Functions

```liva
// Basic operations
let root = Math.sqrt(16.0)
print(root)  // 4.0

let power = Math.pow(2.0, 3.0)
print(power)  // 8.0

let absolute = Math.abs(-10.5)
print(absolute)  // 10.5

// Rounding
let floored = Math.floor(3.7)
print(floored)  // 3

let ceiled = Math.ceil(3.2)
print(ceiled)  // 4

// Min/Max
let maximum = Math.max(10.5, 20.3)
print(maximum)  // 20.3

// Random
let random = Math.random()
print(random)  // 0.0 to 1.0 (varies)
```

### Type Conversion

```liva
// Parse strings to numbers with error handling
let num, err = parseInt("42")
if err == "" {
    print($"Parsed: {num}")  // "Parsed: 42"
}

let invalid, parseErr = parseInt("abc")
if parseErr {
    print($"Error: {parseErr}")  // "Error: Invalid integer format"
}

// Parse floats
let pi, _ = parseFloat("3.14")
print(pi)  // 3.14

// Convert to string
let str1 = toString(42)
print(str1)  // "42"

let str2 = toString(true)
print(str2)  // "true"
```

---

## 📖 Design Principles

### 1. Familiar Syntax
Methods follow conventions from JavaScript/TypeScript/Rust for ease of learning.

### 2. Method Chaining
Most operations return values that can be chained:

```liva
let result = [1, 2, 3, 4, 5]
  .filter(x => x > 2)
  .map(x => x * 2)
  .reduce((acc, x) => acc + x, 0)

print(result)  // 24 (3*2 + 4*2 + 5*2)
```

### 3. Iterator-Based (Arrays)
Array methods use Rust's iterator patterns for efficiency:

```liva
// Compiles to: numbers.iter().map(|&x| x * 2).collect()
let doubled = numbers.map(x => x * 2)
```

### 4. Direct Mapping (Strings)
String methods map directly to Rust standard library:

```liva
// Compiles to: text.to_uppercase()
let upper = text.toUpperCase()
```

### 5. Type Safety
- Array methods preserve element types
- String methods return appropriate types (string, bool, char, i32)
- No implicit conversions

---

## 🎯 Execution Policies (Future)

Liva will support parallel and vectorized execution for array methods:

```liva
// Sequential (current)
let doubled = numbers.map(x => x * 2)

// Parallel (planned)
let doubled = numbers.par().map(x => heavyComputation(x))

// Vectorized/SIMD (planned)
let doubled = numbers.vec().map(x => x * 2)

// Combined (planned)
let doubled = numbers.parvec().map(x => x * 2)
```

**Adapter Methods:**
- `.par()` - Parallel execution using threads
- `.vec()` - Vectorized execution using SIMD
- `.parvec()` - Combined parallel + vectorized

---

## 📝 Error Handling

### Array Methods
Most array methods don't return errors. Empty arrays return appropriate default values:

```liva
let empty = []
let found = empty.find(x => x > 0)  // None
let index = empty.indexOf(42)       // -1
let hasValue = empty.includes(42)   // false
```

### String Methods
String methods handle edge cases gracefully:

```liva
let text = "Hello"

// Out of bounds returns defaults
let char = text.charAt(100)  // ' ' (space)

// Not found returns -1
let index = text.indexOf("xyz")  // -1
```

### Type Conversion
Conversion functions use error binding for parse failures:

```liva
// parseInt and parseFloat return error binding tuples
let num, err = parseInt("123")
if err == "" {
  print($"Success: {num}")  // Prints: "Success: 123"
} else {
  print($"Parse error: {err}")
}

// Invalid input returns default value + error
let invalid, parseErr = parseInt("abc")
// invalid = 0, parseErr = "Invalid integer format"

// toString never fails
let str = toString(42)  // Always returns "42"
```

---

## 🔍 See Also

- [Language Reference Index](../README.md)
- [Getting Started Guide](../../getting-started/quick-start.md)
- [Examples](../../../examples/stdlib/)
