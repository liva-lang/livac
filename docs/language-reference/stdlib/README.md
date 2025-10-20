# Liva Standard Library

> **Status:** 🚧 In Development (v0.7.0)  
> **Completion:** 52.6% (20/38 methods)

The Liva Standard Library provides built-in functions and methods for common programming tasks.

---

## 📚 Modules

### ✅ [Array Methods](./arrays.md)
Methods for working with arrays and collections.

**Status:** Complete (9/9 methods)

- `map(fn)` - Transform elements
- `filter(fn)` - Filter by predicate
- `reduce(fn, initial)` - Reduce to single value
- `forEach(fn)` - Iterate with side effects
- `find(fn)` - Find first match
- `some(fn)` - Check if any match
- `every(fn)` - Check if all match
- `indexOf(value)` - Find index of value
- `includes(value)` - Check if contains value

### ✅ [String Methods](./strings.md)
Methods for string manipulation and queries.

**Status:** Complete (11/11 methods)

- `split(delimiter)` - Split into array
- `replace(old, new)` - Replace substring
- `toUpperCase()` - Convert to uppercase
- `toLowerCase()` - Convert to lowercase
- `trim()` - Remove whitespace
- `trimStart()` - Remove leading whitespace
- `trimEnd()` - Remove trailing whitespace
- `startsWith(prefix)` - Check prefix
- `endsWith(suffix)` - Check suffix
- `substring(start, end)` - Extract substring
- `charAt(index)` - Get character
- `indexOf(substring)` - Find substring position

### 📋 [Math Functions](./math.md)
Mathematical operations and constants.

**Status:** Planned (0/9 functions)

- `Math.sqrt(x)` - Square root
- `Math.pow(base, exp)` - Power
- `Math.abs(x)` - Absolute value
- `Math.floor(x)`, `ceil(x)`, `round(x)` - Rounding
- `Math.min(...)`, `max(...)` - Min/max values
- `Math.random()` - Random number
- `Math.PI`, `Math.E` - Constants

### 📋 [Type Conversion](./conversions.md)
Functions for converting between types.

**Status:** Planned (0/4 functions)

- `parseInt(str)` - Parse integer
- `parseFloat(str)` - Parse float
- `toString(value)` - Convert to string
- `toNumber(str)` - Convert to number

### 📋 [Console/IO](./io.md)
Input/output and console functions.

**Status:** Planned (0/5 functions)

- `console.log(...)` - Print to stdout
- `console.error(...)` - Print to stderr
- `console.warn(...)` - Print warning
- `readLine()` - Read input
- `prompt(message)` - Display prompt and read

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

### Type Conversion (Planned)
Conversion functions will return error bindings:

```liva
let num, err = parseInt("123")
if err != null {
  print($"Parse error: {err}")
}
```

---

## 🔍 See Also

- [Language Reference Index](../README.md)
- [Getting Started Guide](../../getting-started/quick-start.md)
- [Examples](../../../examples/stdlib/)
