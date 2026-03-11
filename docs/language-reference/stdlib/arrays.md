# Array Methods - Liva Standard Library

## Overview

Liva provides a rich set of array methods inspired by functional programming languages and modern JavaScript/TypeScript. All array methods support **execution policies** through adapter methods, enabling sequential, parallel, and vectorized execution.

> **Status:** ✅ 31 methods  
> **Version:** v1.4.0

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
| `map` | ✅ | ✅ | ✅† | ✅ | Full support |
| `filter` | ✅ | ✅ | ✅† | ✅ | Full support |
| `reduce` | ✅ | ✅ | ✅† | ✅ | Parallel requires associative op |
| `forEach` | ✅ | ✅ | ✅† | ✅ | Parallel: unordered execution |
| `find` | ✅ | ✅ | ✅† | ✅ | Uses `find_first` for ordered parallel |
| `some` | ✅ | ✅ | ✅† | ✅ | Short-circuit possible |
| `every` | ✅ | ✅ | ✅† | ✅ | Short-circuit possible |
| `indexOf` | ✅ | ✅ | ✅† | ✅ | Uses `position_first` for ordered parallel |
| `includes` | ✅ | ✅ | ✅† | ✅ | Order-independent |

> **†** `.vec()` currently uses a sequential fallback. SIMD vectorization is planned for a future release.

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

// Point-free function reference (v1.1.0)
let strs = numbers.map(toString)        // equivalent to: x => toString(x)

// Instance method reference with :: (v1.1.0)
let fmt = Formatter("Item")
let labels = numbers.map(fmt::format)   // equivalent to: x => fmt.format(x)

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

// Point-free function reference (v1.1.0)
let valid = names.filter(isValid)           // equivalent to: x => isValid(x)

// Instance method reference with :: (v1.1.0)
let checker = Validator(3)
let ok = names.filter(checker::isValid)     // equivalent to: x => checker.isValid(x)

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
reduce<T, R>(initial: R, fn: (R, T) => R): R
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]

// Sequential sum
let sum = numbers.reduce(0, (acc, x) => acc + x)
// Result: 15

// Sequential product
let product = numbers.reduce(1, (acc, x) => acc * x)
// Result: 120

// Parallel reduction (requires associative operation)
let parallelSum = numbers.par().reduce(0, (acc, x) => acc + x)
let parallelProduct = numbers.par().reduce(1, (acc, x) => acc * x)

// Complex accumulator
let stats = numbers.reduce({sum: 0, count: 0}, (acc, x) => {
  return {
    sum: acc.sum + x,
    count: acc.count + 1
  }
})
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

// Point-free function reference (v1.1.0)
numbers.forEach(print)          // equivalent to: x => print(x)

// Instance method reference with :: (v1.1.0)
let logger = Logger("APP")
numbers.forEach(logger::log)    // equivalent to: x => logger.log(x)

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

Returns the first element matching the predicate.

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

// Point-free function reference (v1.1.0)
let found = names.find(isAdmin)

// Instance method reference with :: (v1.1.0)
let found = names.find(matcher::matches)

// Parallel find (ordered - finds leftmost match)
let firstBig = numbers.par().find(x => x > 3)
// Result: 4
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

// Point-free (v1.1.0)
let hasAdmin = users.some(isAdmin)

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

// Point-free (v1.1.0)
let allValid = names.every(isValid)

// Instance method reference with :: (v1.1.0)
let allMatch = names.every(validator::check)

// Parallel check
let allLarge = numbers.par().every(x => x > 100)
// Result: false
```

---

### `indexOf(value)`

Returns the index of the first occurrence of a value.

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

// Parallel indexOf (ordered - finds leftmost position)
let idx = numbers.par().indexOf(30)
// Result: 2
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

## New Methods *(v1.4.0)*

### `findIndex(fn)`

Returns the index of the first element matching the predicate.

**Signature:**
```liva
findIndex<T>(fn: (T) => bool): int
```

**Examples:**
```liva
let numbers = [10, 20, 30, 40]
let idx = numbers.findIndex(x => x > 25)
print(idx)  // 2

let notFound = numbers.findIndex(x => x > 100)
print(notFound)  // -1
```

**Rust Codegen:**
```rust
numbers.iter().position(|x| *x > 25).map(|i| i as i32).unwrap_or(-1)
```

---

### `flatMap(fn)`

Maps each element and flattens the result by one level.

**Signature:**
```liva
flatMap<T, R>(fn: (T) => [R]): [R]
```

**Examples:**
```liva
let numbers = [1, 2, 3]
let result = numbers.flatMap(n => [n, n * 10])
print(result)  // [1, 10, 2, 20, 3, 30]
```

**Rust Codegen:**
```rust
numbers.iter().flat_map(|n| vec![*n, *n * 10]).collect::<Vec<_>>()
```

---

### `count(fn)`

Count how many elements match a predicate.

**Signature:**
```liva
count<T>(fn: (T) => bool): int
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]
let evens = numbers.count(x => x % 2 == 0)
print(evens)  // 2
```

**Rust Codegen:**
```rust
numbers.iter().filter(|x| **x % 2 == 0).count() as i32
```

---

### `slice(start, end?)`

Extract a sub-array from start to optional end index.

**Signature:**
```liva
slice<T>(start: int, end?: int): [T]
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]
let mid = numbers.slice(1, 3)
print(mid)  // [2, 3]

// Without end — to end of array
let rest = numbers.slice(2)
print(rest)  // [3, 4, 5]
```

---

### `first()` / `last()`

Get the first or last element.

**Signature:**
```liva
first<T>(): T
last<T>(): T
```

**Examples:**
```liva
let numbers = [10, 20, 30]
print(numbers.first())  // 10
print(numbers.last())   // 30
```

**Rust Codegen:**
```rust
numbers.first().cloned().unwrap()
numbers.last().cloned().unwrap()
```

---

### `take(n)` / `drop(n)`

Take the first n elements, or skip the first n elements.

**Signature:**
```liva
take<T>(n: int): [T]
drop<T>(n: int): [T]
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]
let top3 = numbers.take(3)
print(top3)  // [1, 2, 3]

let rest = numbers.drop(2)
print(rest)  // [3, 4, 5]
```

---

### `sort()`

Sort the array (returns new sorted array).

**Signature:**
```liva
sort<T>(): [T]
```

**Examples:**
```liva
let numbers = [3, 1, 4, 1, 5]
let sorted = numbers.sort()
print(sorted)  // [1, 1, 3, 4, 5]
```

**Rust Codegen:**
```rust
{ let mut v = numbers.clone(); v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v }
```

---

### `reversed()`

Reverse the array (returns new reversed array).

**Signature:**
```liva
reversed<T>(): [T]
```

**Examples:**
```liva
let numbers = [1, 2, 3]
let rev = numbers.reversed()
print(rev)  // [3, 2, 1]
```

---

### `distinct()`

Remove duplicate elements (preserving first occurrence order).

**Signature:**
```liva
distinct<T>(): [T]
```

**Examples:**
```liva
let numbers = [1, 2, 2, 3, 3, 3]
let unique = numbers.distinct()
print(unique)  // [1, 2, 3]
```

**Rust Codegen:**
```rust
{
    let mut seen = std::collections::HashSet::new();
    numbers.iter().filter(|x| seen.insert((*x).clone())).cloned().collect::<Vec<_>>()
}
```

---

### `flat()`

Flatten a nested array by one level.

**Signature:**
```liva
flat<T>(): [T]
```

**Examples:**
```liva
let nested = [[1, 2], [3, 4], [5]]
let flat = nested.flat()
print(flat)  // [1, 2, 3, 4, 5]
```

**Rust Codegen:**
```rust
nested.concat()
```

---

### `chunks(n)`

Divide an array into sub-arrays of size n.

**Signature:**
```liva
chunks<T>(size: int): [[T]]
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]
let chunked = numbers.chunks(2)
print(chunked)  // [[1, 2], [3, 4], [5]]
```

**Rust Codegen:**
```rust
numbers.chunks(2).map(|c| c.to_vec()).collect::<Vec<Vec<_>>>()
```

**Notes:**
- Named `chunks()` (not `chunk()`) — `chunk` is a reserved keyword for parallel adapter options

---

### `zip(other)`

Combine two arrays element-wise into tuples.

**Signature:**
```liva
zip<T, U>(other: [U]): [(T, U)]
```

**Examples:**
```liva
let names = ["Alice", "Bob"]
let ages = [30, 25]
let pairs = names.zip(ages)
print(pairs)  // [("Alice", 30), ("Bob", 25)]
```

**Rust Codegen:**
```rust
names.iter().zip(ages.iter()).map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>()
```

**Notes:**
- If arrays have different lengths, stops at the shorter one

---

### `sum()`

Sum all elements in a numeric array.

**Signature:**
```liva
sum(): number
```

**Examples:**
```liva
let numbers = [1, 2, 3, 4, 5]
let total = numbers.sum()
print(total)  // 15
```

**Rust Codegen:**
```rust
numbers.iter().sum::<i32>()
```

---

### `min()` / `max()`

Find the minimum or maximum element.

**Signature:**
```liva
min(): number
max(): number
```

**Examples:**
```liva
let numbers = [3, 1, 4, 1, 5, 9]
print(numbers.min())  // 1
print(numbers.max())  // 9
```

**Rust Codegen:**
```rust
numbers.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).cloned().unwrap()
numbers.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).cloned().unwrap()
```

---

### `isEmpty()`

Check if the array has no elements.

**Signature:**
```liva
isEmpty(): bool
```

**Examples:**
```liva
let empty: [int] = []
print(empty.isEmpty())  // true

let numbers = [1, 2, 3]
print(numbers.isEmpty())  // false
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
  .reduce(0, (a, b) => a + b)
// Result: 84

// Parallel chain
let parallelResult = numbers
  .par()
  .filter(x => x > 3)
  .map(x => x * 2)
  .reduce(0, (a, b) => a + b)

// Mixed: parallel for expensive ops, then sequential
let mixed = numbers
  .par().map(x => heavyComputation(x))
  .filter(x => x > threshold)
  .reduce(0, (a, b) => a + b)
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
- `.par()`: Rayon parallel iterators (`par_iter()`, `find_first()`, `position_first()`)
- `.vec()`: Sequential fallback (SIMD via `packed_simd` planned)
- `.parvec()`: Rayon parallel iterators (SIMD layer planned)

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
