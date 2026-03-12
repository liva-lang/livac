# Liva Standard Library

> **Status:** ✅ Complete (v1.4.0) - 73+ functions implemented! 🎉  
> **Completion:** Arrays ✅ (31) | Strings ✅ (28) | Math ✅ (14) | Conversions ✅ | I/O ✅ | System ✅

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
if parseErr != "" {
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
