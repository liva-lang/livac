# Array Methods

> **31 methods** | v1.4.0

---

## Execution Policies

Liva arrays support adapter-style execution policies for parallelism:

```liva
numbers.map(x => x * 2)                           // sequential (default)
numbers.par().map(x => x * 2)                      // parallel (multi-threaded)
numbers.par({threads: 4, chunk: 2}).map(x => heavy(x))  // parallel with options
numbers.vec().map(x => x * 2)                      // vectorized (SIMD)
numbers.parvec().map(x => x * 2)                   // parallel + vectorized
```

**Adapters:** `.par()` (threads, chunk, ordered) | `.vec()` (simdWidth) | `.parvec()` (all options)

All core methods (map, filter, reduce, forEach, find, some, every, indexOf, includes) support all policies.
Closures must be **immutable captures** for parallel execution.

---

## Core Methods

### map(fn: (T) => R) => [R]
  [1, 2, 3].map(x => x * 2)                // [2, 4, 6]
  numbers.map(toString)                      // point-free ref (v1.1.0)
  numbers.par().map(x => heavyComputation(x))  // parallel

### filter(fn: (T) => bool) => [T]
  [1, 2, 3, 4, 5, 6].filter(x => x % 2 == 0)  // [2, 4, 6]
  names.filter(isValid)                          // point-free ref
  numbers.par().filter(x => x > 3)               // parallel

### reduce(initial: R, fn: (R, T) => R) => R
  [1, 2, 3, 4, 5].reduce(0, (acc, x) => acc + x)  // 15
  [1, 2, 3, 4, 5].reduce(1, (acc, x) => acc * x)   // 120
  numbers.par().reduce(0, (a, x) => a + x)          // parallel (requires associative op)

### forEach(fn: (T) => void) => void
  [1, 2, 3].forEach(x => print(x))          // prints each element
  numbers.forEach(print)                      // point-free ref
  numbers.par().forEach(x => process(x))      // parallel (unordered)

### find(fn: (T) => bool) => T?
  [1, 2, 3, 4, 5].find(x => x % 2 == 0)    // 2
  [1, 2, 3].find(x => x > 10)               // null
  — Returns first matching element, or null

### some(fn: (T) => bool) => bool
  [1, 2, 3, 4, 5].some(x => x % 2 == 0)    // true
  [1, 3, 5].some(x => x % 2 == 0)           // false
  — True if any element matches

### every(fn: (T) => bool) => bool
  [2, 4, 6, 8].every(x => x % 2 == 0)      // true
  [2, 4, 5, 8].every(x => x % 2 == 0)      // false
  — True if all elements match

### indexOf(value: T) => int
  [10, 20, 30, 40].indexOf(30)              // 2
  [10, 20, 30].indexOf(100)                 // -1
  — Returns -1 if not found

### includes(value: T) => bool
  [1, 2, 3, 4, 5].includes(3)               // true
  [1, 2, 3].includes(10)                    // false

---

## Callback Methods *(v1.4.0)*

### findIndex(fn: (T) => bool) => int
  [10, 20, 30, 40].findIndex(x => x > 25)   // 2
  [10, 20].findIndex(x => x > 100)          // -1
  — Returns index of first match, or -1

### flatMap(fn: (T) => [R]) => [R]
  [1, 2, 3].flatMap(n => [n, n * 10])       // [1, 10, 2, 20, 3, 30]
  — Maps each element then flattens one level

### count(fn: (T) => bool) => int
  [1, 2, 3, 4, 5].count(x => x % 2 == 0)   // 2
  [1, 3, 5].count(x => x % 2 == 0)          // 0

---

## Access & Slicing *(v1.4.0)*

### first() => T
  [10, 20, 30].first()                      // 10

### last() => T
  [10, 20, 30].last()                       // 30

### slice(start: int, end?: int) => [T]
  [1, 2, 3, 4, 5].slice(1, 3)               // [2, 3]
  [1, 2, 3, 4, 5].slice(2)                  // [3, 4, 5]  (no end → to end)

### take(n: int) => [T]
  [1, 2, 3, 4, 5].take(3)                   // [1, 2, 3]

### drop(n: int) => [T]
  [1, 2, 3, 4, 5].drop(2)                   // [3, 4, 5]

### chunks(size: int) => [[T]]
  [1, 2, 3, 4, 5].chunks(2)                 // [[1, 2], [3, 4], [5]]
  — Named `chunks` (not `chunk`) — `chunk` is a reserved keyword for parallel adapter options

---

## Transformation *(v1.4.0)*

### sort() => [T]
  [3, 1, 4, 1, 5].sort()                    // [1, 1, 3, 4, 5]
  — Returns new sorted array (original unchanged)

### reversed() => [T]
  [1, 2, 3].reversed()                      // [3, 2, 1]

### distinct() => [T]
  [1, 2, 2, 3, 3, 3].distinct()             // [1, 2, 3]
  — Preserves first-occurrence order

### flat() => [T]
  [[1, 2], [3, 4], [5]].flat()              // [1, 2, 3, 4, 5]
  — Flattens one level

### zip(other: [U]) => [(T, U)]
  ["Alice", "Bob"].zip([30, 25])             // [("Alice", 30), ("Bob", 25)]
  — Stops at shorter array

---

## Aggregation *(v1.4.0)*

### sum() => number
  [1, 2, 3, 4, 5].sum()                     // 15

### min() => number
  [3, 1, 4, 1, 5, 9].min()                  // 1

### max() => number
  [3, 1, 4, 1, 5, 9].max()                  // 9

### isEmpty() => bool
  [].isEmpty()                               // true
  [1, 2, 3].isEmpty()                        // false

---

## Method Chaining

```liva
// Sequential chain
[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  .filter(x => x > 3)
  .map(x => x * 2)
  .reduce(0, (a, b) => a + b)               // 84

// Parallel chain
numbers.par()
  .filter(x => x > 3)
  .map(x => x * 2)
  .reduce(0, (a, b) => a + b)
```
