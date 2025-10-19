# 📋 Phase 2: Standard Library (v0.7.0) - TODO

> **Branch:** `feature/stdlib-v0.7.0`  
> **Status:** 🚧 In Progress  
> **Started:** 2025-10-20  
> **Goal:** Built-in functions and methods for common operations

---

## 🎯 Overview

Implement a comprehensive standard library with:
- Array methods (map, filter, reduce, etc.)
- String methods (split, trim, replace, etc.)
- Math functions (sqrt, pow, abs, etc.)
- Type conversion utilities
- Console/IO functions

**Expected Outcome:**
- Usable standard library in Liva
- 100% test coverage for all stdlib functions
- Complete documentation
- Examples demonstrating usage

---

## 📝 Tasks

### Task 1: Array Methods (~3 hours)

#### 1.1 Design Array API
- [ ] Review Rust Vec/Rayon methods for inspiration
- [ ] Design Liva-style API (syntax, error handling)
- [ ] **Design execution policy syntax for methods: ADAPTER STYLE (Rust-like)**
  - Sequential (default): `arr.map(x => x * 2)`
  - Parallel adapter: `arr.par().map(x => x * 2)`
  - Vectorized adapter: `arr.vec().map(x => x * 2)`
  - Parallel+Vec adapter: `arr.parvec().map(x => x * 2)`
  - With options: `arr.par({threads: 4, chunk: 2}).map(x => x * 2)`
- [ ] **Define which methods support which policies:**
  - `seq` (sequential) - ALL methods support this (default)
  - `.par()` adapter - `map`, `filter`, `reduce`, `forEach`, `some`, `every`, `includes`
  - `.vec()` adapter - `map`, `filter` (numeric operations only)
  - `.parvec()` adapter - `map`, `filter` (numeric operations only)
- [ ] **Design adapter options:**
  - `par({threads: N, chunk: M})` - Parallel with N threads, chunk size M
  - `vec({simdWidth: N})` - SIMD with vector width N
  - `parvec({threads: N, simdWidth: M, ordered: true})` - Combined options
- [ ] Document API design in `docs/language-reference/stdlib/arrays.md`

#### 1.2 Implement Core Methods
- [ ] `map(fn)` - Transform each element
  - Sequential: `[1,2,3].map(x => x * 2)` → `[2,4,6]`
  - Parallel: `[1,2,3].par().map(x => x * 2)`
  - With options: `[1,2,3].par({threads: 4, chunk: 2}).map(x => heavy(x))`
  - Vectorized: `[1,2,3].vec().map(x => x * 2)`
  - Par+Vec: `[1,2,3].parvec().map(x => x * 2)`
- [ ] `filter(fn)` - Keep elements matching predicate
  - Sequential: `[1,2,3].filter(x => x > 1)` → `[2,3]`
  - Parallel: `[1,2,3].par().filter(x => x > 1)`
  - Vectorized: `[1,2,3].vec().filter(x => x > 1)`
- [ ] `reduce(fn, initial)` - Reduce to single value
  - Sequential: `[1,2,3].reduce((acc, x) => acc + x, 0)` → `6`
  - Parallel: `[1,2,3].par().reduce((acc, x) => acc + x, 0)`

#### 1.3 Implement Utility Methods
- [ ] `forEach(fn)` - Iterate with side effects
  - Sequential: `arr.forEach(x => print(x))`
  - Parallel: `arr.par().forEach(x => print(x))`
- [ ] `find(fn)` - Find first element matching predicate
  - Sequential only: `arr.find(x => x > 5)`
- [ ] `some(fn)` - Check if any element matches
  - Sequential: `arr.some(x => x > 5)`
  - Parallel: `arr.par().some(x => x > 5)`
- [ ] `every(fn)` - Check if all elements match
  - Sequential: `arr.every(x => x > 0)`
  - Parallel: `arr.par().every(x => x > 0)`
- [ ] `indexOf(value)` - Find index of value
  - Sequential only: `arr.indexOf(42)`
- [ ] `includes(value)` - Check if array contains value
  - Sequential: `arr.includes(42)`
  - Parallel: `arr.par().includes(42)`

#### 1.4 Testing & Documentation
- [ ] Add unit tests for each method in `tests/stdlib_tests.rs`
- [ ] Add integration tests with real examples
- [ ] Create `docs/language-reference/stdlib/arrays.md` with:
  - API reference for all array methods
  - Examples for each method
  - Error handling examples
  - Performance notes
- [ ] Add code snippets to VSCode extension

**Success Criteria:** All array methods work correctly with 100% test coverage

---

### Task 2: String Methods (~2 hours)

#### 2.1 Implement String Manipulation
- [ ] `split(delimiter)` - Split string into array
  - Example: `"a,b,c".split(",")` → `["a","b","c"]`
- [ ] `join(separator)` - Join array into string (on arrays of strings)
  - Example: `["a","b"].join(",")` → `"a,b"`
- [ ] `replace(old, new)` - Replace substring
  - Example: `"hello".replace("l", "x")` → `"hexxo"`

#### 2.2 Implement String Transformation
- [ ] `toUpperCase()` - Convert to uppercase
- [ ] `toLowerCase()` - Convert to lowercase
- [ ] `trim()` - Remove leading/trailing whitespace
- [ ] `trimStart()` - Remove leading whitespace
- [ ] `trimEnd()` - Remove trailing whitespace

#### 2.3 Implement String Queries
- [ ] `startsWith(prefix)` - Check if starts with prefix
- [ ] `endsWith(suffix)` - Check if ends with suffix
- [ ] `substring(start, end)` - Extract substring
- [ ] `charAt(index)` - Get character at index
- [ ] `indexOf(substring)` - Find index of substring

#### 2.4 Testing & Documentation
- [ ] Add unit tests for each method in `tests/stdlib_tests.rs`
- [ ] Add integration tests
- [ ] Create `docs/language-reference/stdlib/strings.md` with:
  - API reference for all string methods
  - Examples for each method
  - Edge cases and error handling
- [ ] Add code snippets to VSCode extension

**Success Criteria:** All string methods work correctly with 100% test coverage

---

### Task 3: Math Functions (~2 hours)

#### 3.1 Design Math Namespace
- [ ] Decide: `Math.sqrt(x)` vs `sqrt(x)` (namespace vs global)
- [ ] Plan integration with existing number types
- [ ] Document design in `docs/stdlib/math.md`

#### 3.2 Implement Basic Math
- [ ] `Math.sqrt(x)` - Square root
- [ ] `Math.pow(base, exp)` - Power
- [ ] `Math.abs(x)` - Absolute value
- [ ] `Math.sign(x)` - Sign of number (-1, 0, 1)

#### 3.3 Implement Rounding
- [ ] `Math.floor(x)` - Round down
- [ ] `Math.ceil(x)` - Round up
- [ ] `Math.round(x)` - Round to nearest

#### 3.4 Implement Min/Max/Random
- [ ] `Math.min(a, b, ...)` - Minimum value
- [ ] `Math.max(a, b, ...)` - Maximum value
- [ ] `Math.random()` - Random float [0, 1)

#### 3.5 Add Constants
- [ ] `Math.PI` - π (3.14159...)
- [ ] `Math.E` - Euler's number (2.71828...)

#### 3.6 Testing & Documentation
- [ ] Add unit tests in `tests/stdlib_tests.rs`
- [ ] Add integration tests
- [ ] Create `docs/language-reference/stdlib/math.md` with:
  - API reference for all math functions
  - Mathematical definitions
  - Examples and use cases
  - Constants documentation
- [ ] Add code snippets to VSCode extension

**Success Criteria:** All math functions work correctly with 100% test coverage

---

### Task 4: Type Conversion (~1 hour)

#### 4.1 Implement Parsing Functions
- [ ] `parseInt(str)` - Parse string to integer
  - Example: `parseInt("42")` → `42`
  - Handle errors: `let num, err = parseInt("abc")`
- [ ] `parseFloat(str)` - Parse string to float
  - Example: `parseFloat("3.14")` → `3.14`

#### 4.2 Implement Conversion Functions
- [ ] `toString(value)` - Convert any value to string
- [ ] `toNumber(str)` - Convert string to number (int or float)
- [ ] `toInt(value)` - Convert to integer
- [ ] `toFloat(value)` - Convert to float

#### 4.3 Testing & Documentation
- [ ] Add unit tests with error cases in `tests/stdlib_tests.rs`
- [ ] Add integration tests
- [ ] Create `docs/language-reference/stdlib/conversions.md` with:
  - API reference for conversion functions
  - Error handling examples
  - Type compatibility matrix
- [ ] Add code snippets to VSCode extension

**Success Criteria:** All conversion functions handle errors gracefully

---

### Task 5: Console/IO (~1 hour)

#### 5.1 Implement Console Functions
- [ ] `console.log(...)` - Print to stdout (enhanced print)
- [ ] `console.error(...)` - Print to stderr
- [ ] `console.warn(...)` - Print warning to stderr
- [ ] `console.debug(...)` - Print debug info (only in debug mode)

#### 5.2 Implement Input Functions
- [ ] `readLine()` - Read line from stdin
  - Example: `let input = readLine()`
- [ ] `prompt(message)` - Display message and read input
  - Example: `let name = prompt("Enter name: ")`

#### 5.3 Testing & Documentation
- [ ] Add unit tests in `tests/stdlib_tests.rs`
- [ ] Add integration tests
- [ ] Create `docs/language-reference/stdlib/io.md` with:
  - API reference for console/IO functions
  - Examples for input/output
  - Error handling
- [ ] Add code snippets to VSCode extension

**Success Criteria:** All I/O functions work correctly

---

### Task 6: Examples & Documentation (~1 hour)

#### 6.1 Create Comprehensive Examples
- [ ] `examples/stdlib/arrays_demo.liva` - Array methods showcase
- [ ] `examples/stdlib/strings_demo.liva` - String methods showcase
- [ ] `examples/stdlib/math_demo.liva` - Math functions showcase
- [ ] `examples/stdlib/conversions_demo.liva` - Type conversion examples
- [ ] `examples/stdlib/io_demo.liva` - Console/IO examples

#### 6.2 Create Documentation Structure
- [ ] Create `docs/language-reference/stdlib/` directory
- [ ] Create `docs/language-reference/stdlib/README.md` - Overview
- [ ] Create `docs/language-reference/stdlib/arrays.md` - Array methods reference
- [ ] Create `docs/language-reference/stdlib/strings.md` - String methods reference
- [ ] Create `docs/language-reference/stdlib/math.md` - Math functions reference
- [ ] Create `docs/language-reference/stdlib/conversions.md` - Type conversion reference
- [ ] Create `docs/language-reference/stdlib/io.md` - Console/IO reference

#### 6.3 Update Existing Documentation
- [ ] Update `docs/getting-started/basic-usage.md` with stdlib examples
- [ ] Update `docs/README.md` to include stdlib section
- [ ] Update root `README.md` with stdlib showcase
- [ ] Add stdlib to language reference index

#### 6.3 Update VSCode Extension
- [ ] Add stdlib functions to IntelliSense
- [ ] Add stdlib snippets
- [ ] Update syntax highlighting if needed

**Success Criteria:** Comprehensive documentation and examples for all stdlib features

---

## 🎯 Completion Checklist

- [ ] Task 1: Array Methods
- [ ] Task 2: String Methods
- [ ] Task 3: Math Functions
- [ ] Task 4: Type Conversion
- [ ] Task 5: Console/IO
- [ ] Task 6: Examples & Documentation
- [ ] All tests passing (100% coverage)
- [ ] Documentation complete
- [ ] CHANGELOG updated
- [ ] Ready for v0.7.0 release

---

## 📝 Implementation Notes

### Design Decisions

**Method Syntax:**
```liva
// Option 1: Methods on values (preferred)
let doubled = [1,2,3].map(x => x * 2)
let upper = "hello".toUpperCase()

// Option 2: Standalone functions
let doubled = map([1,2,3], x => x * 2)
let upper = toUpperCase("hello")
```

**Decision:** Use method syntax (Option 1) for better ergonomics and consistency with modern languages.

**Execution Policies for Array Methods:**

Liva uses **adapter methods** (inspired by Rust's Rayon) for execution policies:

```liva
// Sequential (default) - no adapter needed
let doubled = [1,2,3,4].map(x => x * 2)

// Parallel adapter - multi-threading
let doubled = [1,2,3,4].par().map(x => heavyComputation(x))

// Parallel with options
let doubled = [1,2,3,4].par({threads: 4, chunk: 2}).map(x => heavy(x))

// Vectorized adapter - SIMD
let doubled = [1,2,3,4].vec().map(x => x * 2)
let doubled = [1,2,3,4].vec({simdWidth: 4}).map(x => x * 2)

// Parallel + Vectorized
let doubled = [1,2,3,4].parvec().map(x => x * 2)
let doubled = [1,2,3,4].parvec({threads: 4, simdWidth: 4, ordered: true}).map(x => x * 2)

// Chaining multiple operations
let result = [1,2,3,4,5,6,7,8]
  .par()
  .filter(x => x > 3)
  .map(x => x * 2)
  .reduce((a, b) => a + b, 0)
```

**Adapter Methods:**
- `.par()` - Returns parallel iterator adapter
- `.par({threads: N, chunk: M, ordered: bool})` - Parallel with options
- `.vec()` - Returns vectorized iterator adapter
- `.vec({simdWidth: N})` - Vectorized with SIMD width
- `.parvec()` - Returns parallel+vectorized adapter
- `.parvec({threads: N, simdWidth: M, ordered: bool})` - Combined with options

**For Loops (different syntax - kept as is):**
```liva
// For loops use prefix syntax (existing, unchanged)
for par item in numbers {
  heavy(item)
}

for vec value in values {
  compute(value)
}

for parvec item in data with simdWidth 4 ordered {
  process(item)
}
```

**Why Different Syntax?**
- **For loops**: Prefix syntax (`for par`) - matches language keywords, clear control flow
- **Array methods**: Adapter syntax (`.par()`) - chainable, composable, familiar to Rust/Java developers

**Policy Support Matrix:**

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

**Arrow Functions and Closures:**

```liva
// Arrow functions for array methods
let numbers = [1, 2, 3, 4, 5]

// Simple arrow
let doubled = numbers.map(x => x * 2)

// Block arrow
let processed = numbers.map(x => {
  let result = x * 2
  return result + 1
})

// Closure capture
let multiplier = 10
let scaled = numbers.map(x => x * multiplier)  // Captures 'multiplier'

// Parallel with closure (immutable capture is safe)
let scaled = numbers.par().map(x => x * multiplier)

// Complex parallel pipeline
let result = numbers
  .par({threads: 4})
  .filter(x => x > 5)
  .map(x => x * multiplier)
  .reduce((a, b) => a + b, 0)
```

**Thread Safety:**
- Closures must be `Send + Sync` for `.par()`/`.parvec()`
- Mutable captures not allowed in parallel contexts
- Compiler enforces safety at compile time

```liva
// ✅ Safe: immutable capture
let multiplier = 10
let scaled = numbers.par().map(x => x * multiplier)

// ❌ Compile error: mutable capture in parallel context
let mut counter = 0
let result = numbers.par().map(x => {
  counter = counter + 1  // ERROR: cannot mutate in parallel
  return x * 2
})

// ✅ Safe: no shared state
let result = numbers.parvec().map(x => x * 2)
```

**Error Handling:**
```liva
// Parsing returns error binding
let num, err = parseInt("abc")
if err != null {
  print($"Parse error: {err}")
}

// Math functions panic on invalid input (like Rust)
let result = Math.sqrt(-1)  // Runtime error
```

**Namespaces:**
- `Math.*` - Math functions (e.g., `Math.sqrt()`)
- `console.*` - Console functions (e.g., `console.log()`)
- Methods attached to types (e.g., `arr.map()`, `str.trim()`)

---

## 🚀 After Completion

1. **Merge to main:**
   ```bash
   git checkout main
   git merge feature/stdlib-v0.7.0
   git push origin main
   ```

2. **Tag release:**
   ```bash
   git tag -a v0.7.0 -m "Release v0.7.0: Standard Library"
   git push origin v0.7.0
   ```

3. **Update ROADMAP:**
   - Mark Phase 2 as completed
   - Document any design decisions or changes

---

## ⏰ Time Tracking

| Task | Estimated | Actual | Status |
|------|-----------|--------|--------|
| Array Methods | 3h | - | 📋 Not Started |
| String Methods | 2h | - | 📋 Not Started |
| Math Functions | 2h | - | 📋 Not Started |
| Type Conversion | 1h | - | 📋 Not Started |
| Console/IO | 1h | - | 📋 Not Started |
| Examples & Docs | 1h | - | 📋 Not Started |
| **Total** | **10h** | **-** | **📋 Not Started** |
