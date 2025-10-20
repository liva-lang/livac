# Array Methods - Liva Standard Library

## Overview

Liva provides a rich set of array methods inspired by functional programming languages and modern JavaScript/TypeScript. All array methods support **execution policies** through adapter methods, enabling sequential, parallel, and vectorized execution.

## Execution Policies

Liva uses **adapter-style syntax** (inspired by Rust's Rayon) for execution policies:

### Available Adapters

```liva
// Sequential (default) - no adapter needed
let doubled = [1,2,3,4].map(x => x * 2)

// Parallel adapter - multi-threading
let doubled = [1,2,3,4].par().map(x => x * 2)

// Parallel with options
let doubled = [1,2,3,4].par({threads: 4, chunk: 2}).map(x => heavy(x))

// Vectorized adapter - SIMD
let doubled = [1,2,3,4].vec().map(x => x * 2)

// Vectorized with options
let doubled = [1,2,3,4].vec({simdWidth: 4}).map(x => x * 2)

// Parallel + Vectorized adapter
let doubled = [1,2,3,4].parvec().map(x => x * 2)

// Combined with all options
let doubled = [1,2,3,4].parvec({
  threads: 4,
  simdWidth: 4,
  ordered: true
}).map(x => x * 2)
```

### Adapter Options

#### `.par()` - Parallel Execution
- **`threads: number`** - Number of threads to use (default: auto)
- **`chunk: number`** - Chunk size for work distribution (default: auto)
- **`ordered: bool`** - Preserve order in results (default: true)

#### `.vec()` - Vectorized Execution (SIMD)
- **`simdWidth: number`** - SIMD vector width (default: auto)

#### `.parvec()` - Combined Parallel + Vectorized
- **`threads: number`** - Number of threads
- **`simdWidth: number`** - SIMD vector width
- **`ordered: bool`** - Preserve order in results (default: true)

### Policy Support Matrix

| Method | Sequential | `.par()` | `.vec()` | `.parvec()` | Notes |
|--------|-----------|----------|----------|-------------|-------|
| `map` | ✅ | ✅ | ✅ | ✅ | Full support |
| `filter` | ✅ | ✅ | ✅ | ✅ | Full support |
| `reduce` | ✅ | ✅ | ❌ | ❌ | Parallel requires associative op |
| `forEach` | ✅ | ✅ | ❌ | ❌ | Side effects only |
| `find` | ✅ | ❌ | ❌ | ❌ | Early exit, order-dependent |
| `some` | ✅ | ✅ | ❌ | ❌ | Short-circuit possible |
| `every` | ✅ | ✅ | ❌ | ❌ | Short-circuit possible |
| `indexOf` | ✅ | ❌ | ❌ | ❌ | Order-dependent |
| `includes` | ✅ | ✅ | ❌ | ❌ | Order-independent |

---

## Core Methods

### `map(fn)`

Transforms each element in the array by applying a function.

**Signature:**
```liva
map<T, R>(fn: (T) => R): Array<R>
```

**Examples:**
```liva
// Sequential (default)
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(x => x * 2)
// Result: [2, 4, 6, 8, 10]

// With arrow function body
let squared = numbers.map(x => {
  let result = x * x
  return result
})
// Result: [1, 4, 9, 16, 25]

// Parallel execution for expensive operations
let results = numbers.par().map(x => heavyComputation(x))

// Parallel with custom options
let results = numbers.par({threads: 4, chunk: 2}).map(x => heavy(x))

// Vectorized for numeric operations
let doubled = numbers.vec().map(x => x * 2)

// Combined parallel + vectorized
let processed = numbers.parvec().map(x => x * 2 + 1)
```

**Thread Safety:**
- Closures must be **immutable captures** for parallel execution
- Mutable captures result in compile-time error

```liva
// ✅ Safe: immutable capture
let multiplier = 10
let scaled = numbers.par().map(x => x * multiplier)

// ❌ Compile error: mutable capture in parallel context
let mut counter = 0
let result = numbers.par().map(x => {
  counter = counter + 1  // ERROR
  return x * 2
})
```

---

### `filter(fn)`

Keeps only elements that match a predicate function.

**Signature:**
```liva
filter<T>(fn: (T) => bool): Array<T>
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5, 6]

// Sequential
let evens = numbers.filter(x => x % 2 == 0)
// Result: [2, 4, 6]

// Parallel
let large = numbers.par().filter(x => x > 3)
// Result: [4, 5, 6]

// Vectorized
let positives = numbers.vec().filter(x => x > 0)

// With complex predicate
let filtered = numbers.filter(x => {
  let isEven = x % 2 == 0
  let isLarge = x > 2
  return isEven and isLarge
})
```

---

### `reduce(fn, initial)`

Reduces the array to a single value by applying a binary function.

**Signature:**
```liva
reduce<T, R>(fn: (R, T) => R, initial: R): R
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]

// Sequential sum
let sum = numbers.reduce((acc, x) => acc + x, 0)
// Result: 15

// Sequential product
let product = numbers.reduce((acc, x) => acc * x, 1)
// Result: 120

// Parallel reduction (requires associative operation)
let parallelSum = numbers.par().reduce((acc, x) => acc + x, 0)

// Complex accumulator
let stats = numbers.reduce((acc, x) => {
  return {
    sum: acc.sum + x,
    count: acc.count + 1
  }
}, {sum: 0, count: 0})
```

**Parallel Constraints:**
- Operation must be **associative**: `(a op b) op c == a op (b op c)`
- Examples: addition, multiplication, min, max
- Non-associative: subtraction, division

---

## Utility Methods

### `forEach(fn)`

Executes a function for each element (side effects).

**Signature:**
```liva
forEach<T>(fn: (T) => void): void
```

**Examples:**
```liva
let numbers = [1, 2, 3]

// Sequential
numbers.forEach(x => print(x))

// Parallel (unordered execution)
numbers.par().forEach(x => {
  print($"Processing {x}")
})

// With side effects
let mut total = 0
numbers.forEach(x => {
  total = total + x  // OK for sequential
})
```

---

### `find(fn)`

Returns the first element matching the predicate (sequential only).

**Signature:**
```liva
find<T>(fn: (T) => bool): T | null
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]

let firstEven = numbers.find(x => x % 2 == 0)
// Result: 2

let firstLarge = numbers.find(x => x > 10)
// Result: null
```

---

### `some(fn)`

Returns `true` if any element matches the predicate.

**Signature:**
```liva
some<T>(fn: (T) => bool): bool
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]

let hasEven = numbers.some(x => x % 2 == 0)
// Result: true

// Parallel check
let hasLarge = numbers.par().some(x => x > 100)
// Result: false
```

---

### `every(fn)`

Returns `true` if all elements match the predicate.

**Signature:**
```liva
every<T>(fn: (T) => bool): bool
```

**Examples:**
```liva
let numbers = [2, 4, 6, 8]

let allEven = numbers.every(x => x % 2 == 0)
// Result: true

let allPositive = numbers.every(x => x > 0)
// Result: true

// Parallel check
let allLarge = numbers.par().every(x => x > 100)
// Result: false
```

---

### `indexOf(value)`

Returns the index of the first occurrence of a value (sequential only).

**Signature:**
```liva
indexOf<T>(value: T): number
```

**Examples:**
```liva
let numbers = [10, 20, 30, 40]

let index = numbers.indexOf(30)
// Result: 2

let notFound = numbers.indexOf(100)
// Result: -1
```

---

### `includes(value)`

Returns `true` if the array contains the value.

**Signature:**
```liva
includes<T>(value: T): bool
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]

let hasThree = numbers.includes(3)
// Result: true

let hasTen = numbers.includes(10)
// Result: false

// Parallel search
let found = numbers.par().includes(4)
// Result: true
```

---

## Method Chaining

Array methods can be chained together for complex transformations:

```liva
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Sequential chain
let result = numbers
  .filter(x => x > 3)
  .map(x => x * 2)
  .reduce((a, b) => a + b, 0)
// Result: 84

// Parallel chain
let parallelResult = numbers
  .par()
  .filter(x => x > 3)
  .map(x => x * 2)
  .reduce((a, b) => a + b, 0)

// Mixed: parallel for expensive ops, then sequential
let mixed = numbers
  .par().map(x => heavyComputation(x))
  .filter(x => x > threshold)
  .reduce((a, b) => a + b, 0)
```

---

## Performance Guidelines

### When to Use Sequential
- Small arrays (< 1000 elements)
- Simple operations (cheap computations)
- Order-dependent operations (find, indexOf)

### When to Use `.par()`
- Large arrays (> 10,000 elements)
- Expensive per-element operations
- CPU-bound workloads
- Operations are thread-safe

### When to Use `.vec()`
- Numeric operations on arrays
- Simple arithmetic transformations
- Data fits in SIMD registers
- Want single-threaded SIMD acceleration

### When to Use `.parvec()`
- Very large arrays with numeric data
- Embarrassingly parallel numeric workloads
- Maximum performance needed
- Both multi-core and SIMD benefits

---

## Error Handling

Array methods with fallible operations (future feature):

```liva
// Future: Methods that can fail
let results, err = numbers.map(x => divide(100, x))
if err != null {
  print($"Error in map: {err}")
}
```

---

## Implementation Notes

### Backend
- Sequential: Direct Rust `Vec` operations
- `.par()`: Rayon parallel iterators
- `.vec()`: SIMD intrinsics (e.g., `packed_simd` crate)
- `.parvec()`: Combined Rayon + SIMD

### Compilation
```liva
// Liva code
let doubled = [1,2,3].map(x => x * 2)

// Generated Rust code (simplified)
let doubled: Vec<i32> = vec![1, 2, 3]
  .into_iter()
  .map(|x| x * 2)
  .collect();
```

```liva
// Liva code with parallel
let doubled = [1,2,3].par().map(x => x * 2)

// Generated Rust code (simplified)
use rayon::prelude::*;
let doubled: Vec<i32> = vec![1, 2, 3]
  .par_iter()
  .map(|x| x * 2)
  .collect();
```

---

## Examples

See `examples/stdlib/arrays_demo.liva` for comprehensive examples.

---

## See Also

- [String Methods](./strings.md)
- [Math Functions](./math.md)
- [Type Conversions](./conversions.md)
- [Control Flow](../control-flow.md)
