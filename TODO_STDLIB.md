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

#### 1.1 Design Array API ✅ COMPLETED
- [x] Review Rust Vec/Rayon methods for inspiration
- [x] Design Liva-style API (syntax, error handling)
- [x] **Design execution policy syntax for methods: ADAPTER STYLE (Rust-like)**
  - Sequential (default): `arr.map(x => x * 2)`
  - Parallel adapter: `arr.par().map(x => x * 2)`
  - Vectorized adapter: `arr.vec().map(x => x * 2)`
  - Parallel+Vec adapter: `arr.parvec().map(x => x * 2)`
  - With options: `arr.par({threads: 4, chunk: 2}).map(x => x * 2)`
- [x] **Define which methods support which policies:**
  - `seq` (sequential) - ALL methods support this (default)
  - `.par()` adapter - `map`, `filter`, `reduce`, `forEach`, `some`, `every`, `includes`
  - `.vec()` adapter - `map`, `filter` (numeric operations only)
  - `.parvec()` adapter - `map`, `filter` (numeric operations only)
- [x] **Design adapter options:**
  - `par({threads: N, chunk: M})` - Parallel with N threads, chunk size M
  - `vec({simdWidth: N})` - SIMD with vector width N
  - `parvec({threads: N, simdWidth: M, ordered: true})` - Combined options
- [x] Document API design in `docs/language-reference/stdlib/arrays.md`

#### 1.2 Implement Core Methods ✅ COMPLETED (3/3)
- [x] `map(fn)` - Transform each element ✅ **WORKING!**
  - Sequential: `[1,2,3].map(x => x * 2)` → `[2,4,6]` ✅
  - Parallel: `[1,2,3].par().map(x => x * 2)` (parser ready, codegen TODO)
  - With options: `[1,2,3].par({threads: 4, chunk: 2}).map(x => heavy(x))` (TODO)
  - Vectorized: `[1,2,3].vec().map(x => x * 2)` (TODO)
  - Par+Vec: `[1,2,3].parvec().map(x => x * 2)` (TODO)
- [x] `filter(fn)` - Keep elements matching predicate ✅ **WORKING!**
  - Sequential: `[1,2,3].filter(x => x > 1)` → `[2,3]` ✅
  - Parallel: `[1,2,3].par().filter(x => x > 1)` (parser ready, codegen TODO)
  - Vectorized: `[1,2,3].vec().filter(x => x > 1)` (TODO)
- [x] `reduce(fn, initial)` - Reduce to single value ✅ **WORKING!**
  - Sequential: `[1,2,3,4,5].reduce((acc, x) => acc + x, 0)` → `15` ✅
  - Uses Rust's `.iter().fold(initial, |acc, &x| expr)`
  - Tested: Sum(15), Product(120), Max(5), Count(5) ✅
  - Parallel: `[1,2,3].par().reduce((acc, x) => acc + x, 0)` (TODO)

#### 1.3 Implement Utility Methods ✅ COMPLETED (6/6)
- [x] `forEach(fn)` - Iterate with side effects ✅ **WORKING!**
  - Sequential: `[1,2,3].forEach(x => print(x))` ✅
  - Uses `.iter().for_each(|&x| { ... })`
  - Tested: prints, squares, sum accumulation ✅
  - Parallel: `arr.par().forEach(x => print(x))` (TODO)
- [x] `find(fn)` - Find first element matching predicate ✅ **WORKING!**
  - Sequential: `[1,5,10,15].find(x => x > 10)` → `Some(15)` ✅
  - Uses `.iter().find(|&&x| pred).copied()`
  - Returns Option<T> (Some/None)
  - Tested: Some(15), None, Some(10), Some(1) ✅
- [x] `some(fn)` - Check if any element matches ✅ **WORKING!**
  - Sequential: `[2,4,6].some(x => x % 2 == 0)` → `true` ✅
  - Uses `.iter().any(|&x| pred)`
  - Returns bool
  - Tested: all boolean checks passing ✅
  - Parallel: `arr.par().some(x => x > 5)` (TODO)
- [x] `every(fn)` - Check if all elements match ✅ **WORKING!**
  - Sequential: `[2,4,6].every(x => x % 2 == 0)` → `true` ✅
  - Uses `.iter().all(|&x| pred)`
  - Returns bool
  - Tested: all boolean checks passing ✅
  - Parallel: `arr.par().every(x => x > 0)` (TODO)
- [x] `indexOf(value)` - Find index of value ✅ **WORKING!**
  - Sequential: `[10,20,30].indexOf(30)` → `2` ✅
  - Uses `.iter().position(|&x| x == value)`
  - Returns i32 (-1 if not found)
  - Tested: 2, 0, 4, -1 (not found) ✅
- [x] `includes(value)` - Check if array contains value ✅ **WORKING!**
  - Sequential: `[10,20,30].includes(20)` → `true` ✅
  - Uses `.iter().any(|&x| x == value)`
  - Returns bool
  - Tested: true/false with numbers and strings ✅
  - Parallel: `arr.par().includes(42)` (TODO)

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

### Task 2: String Methods ✅ COMPLETED (~2 hours)

#### 2.1 Implement String Manipulation ✅ COMPLETED (3/3)
- [x] `split(delimiter)` - Split string into array ✅ **WORKING!**
  - Example: `"apple,banana,orange".split(",")` → `["apple","banana","orange"]` ✅
  - Uses `.split(delim).map(|s| s.to_string()).collect::<Vec<String>>()`
  - Returns Vec<String> for Liva array compatibility
  - Tested: comma delimiter working correctly ✅
- [ ] `join(separator)` - Join array into string (on arrays of strings)
  - Example: `["a","b"].join(",")` → `"a,b"`
  - **NOTE:** Not implemented yet - future enhancement
- [x] `replace(old, new)` - Replace substring ✅ **WORKING!**
  - Example: `"hello world".replace("world", "Liva")` → `"hello Liva"` ✅
  - Uses `.replace(old, new)`
  - Tested: replace working correctly ✅

#### 2.2 Implement String Transformation ✅ COMPLETED (5/5)
- [x] `toUpperCase()` - Convert to uppercase ✅ **WORKING!**
  - Example: `"hello".toUpperCase()` → `"HELLO"` ✅
  - Uses `.to_uppercase()`
  - Tested: uppercase conversion working ✅
- [x] `toLowerCase()` - Convert to lowercase ✅ **WORKING!**
  - Example: `"HELLO WORLD".toLowerCase()` → `"hello world"` ✅
  - Uses `.to_lowercase()`
  - Tested: lowercase conversion working ✅
- [x] `trim()` - Remove leading/trailing whitespace ✅ **WORKING!**
  - Example: `"  hello  ".trim()` → `"hello"` ✅
  - Uses `.trim()`
  - Tested: trim working correctly ✅
- [x] `trimStart()` - Remove leading whitespace ✅ **WORKING!**
  - Example: `"  hello".trimStart()` → `"hello"` ✅
  - Uses `.trim_start()`
  - Tested: trimStart working correctly ✅
- [x] `trimEnd()` - Remove trailing whitespace ✅ **WORKING!**
  - Example: `"hello  ".trimEnd()` → `"hello"` ✅
  - Uses `.trim_end()`
  - Tested: trimEnd working correctly ✅

#### 2.3 Implement String Queries ✅ COMPLETED (5/5)
- [x] `startsWith(prefix)` - Check if starts with prefix ✅ **WORKING!**
  - Example: `"hello.liva".startsWith("hello")` → `true` ✅
  - Uses `.starts_with(prefix)`
  - Returns bool
  - Tested: true/false checks working ✅
- [x] `endsWith(suffix)` - Check if ends with suffix ✅ **WORKING!**
  - Example: `"file.pdf".endsWith(".pdf")` → `true` ✅
  - Uses `.ends_with(suffix)`
  - Returns bool
  - Tested: true/false checks working ✅
- [x] `substring(start, end)` - Extract substring ✅ **WORKING!**
  - Example: `"Hello World".substring(0, 5)` → `"Hello"` ✅
  - Uses slice syntax `[start as usize..end as usize].to_string()`
  - Tested: "Hello" and "World" extraction working ✅
- [x] `charAt(index)` - Get character at index ✅ **WORKING!**
  - Example: `"Hello".charAt(0)` → `'H'` ✅
  - Uses `.chars().nth(index as usize).unwrap_or(' ')`
  - UTF-8 safe character access
  - Returns space for out-of-bounds
  - Tested: 'H' and 'W' extraction working ✅
- [x] `indexOf(substring)` - Find index of substring ✅ **WORKING!**
  - Example: `"The quick brown fox".indexOf("quick")` → `4` ✅
  - Uses `.find(substring).map(|i| i as i32).unwrap_or(-1)`
  - Returns i32 (-1 if not found)
  - Disambiguated from array indexOf by argument type (string literal)
  - Tested: 4, 16, 31, -1 (not found) ✅

#### 2.4 Testing & Documentation ✅ TESTS COMPLETE
- [x] Create comprehensive test files:
  - `test_string_methods.liva` - split, replace, case conversion ✅
  - `test_string_trim.liva` - trim variants, startsWith, endsWith ✅
  - `test_string_access.liva` - substring, charAt ✅
  - `test_string_indexof.liva` - substring search ✅
- [ ] Add unit tests for each method in `tests/stdlib_tests.rs`
- [ ] Add integration tests
- [ ] Create `docs/language-reference/stdlib/strings.md` with:
  - API reference for all string methods
  - Examples for each method
  - Edge cases and error handling
- [ ] Add code snippets to VSCode extension

**Success Criteria:** ✅ All 11 string methods implemented and verified! 🎉

**Implementation Details:**
- Added `generate_string_method_call()` in `src/codegen.rs`
- Reuses existing `MethodCall` AST node (no parser changes needed)
- Direct mapping to Rust string methods (no iterators)
- String method detection based on method name + Seq adapter
- indexOf disambiguation: string literal argument = string indexOf, numeric = array indexOf

**Test Results:**
- ✅ split: ["apple", "banana", "orange"]
- ✅ replace: "hello Liva"
- ✅ toUpperCase: "HELLO"
- ✅ toLowerCase: "hello world"
- ✅ trim, trimStart, trimEnd: whitespace removal working
- ✅ startsWith, endsWith: boolean checks working
- ✅ substring: "Hello", "World" extraction working
- ✅ charAt: 'H', 'W' character access working
- ✅ indexOf: 4, 16, 31, -1 (not found) all correct

---

### Task 3: Math Functions ✅ COMPLETED (~2 hours)

#### 3.1 Design Math Namespace ✅ COMPLETED
- [x] Decide: `Math.sqrt(x)` vs `sqrt(x)` (namespace vs global) - **DECIDED: Namespace style `Math.*`**
- [x] Plan integration with existing number types - **Uses f64 for all operations**
- [x] Document design in `docs/stdlib/math.md` - **Placeholder created**

#### 3.2 Implement Basic Math ✅ COMPLETED (3/4)
- [x] `Math.sqrt(x)` - Square root ✅ **WORKING!**
  - Example: `Math.sqrt(16.0)` → `4.0` ✅
  - Uses `x.sqrt()` method on f64
  - Tested and verified ✅
- [x] `Math.pow(base, exp)` - Power ✅ **WORKING!**
  - Example: `Math.pow(5.0, 2.0)` → `25.0` ✅
  - Uses `base.powf(exp)` method
  - Tested and verified ✅
- [x] `Math.abs(x)` - Absolute value ✅ **WORKING!**
  - Example: `Math.abs(-10.5)` → `10.5` ✅
  - Uses `x.abs()` method with parentheses for unary expressions
  - Fixed precedence issue with negative numbers
  - Tested and verified ✅
- [ ] `Math.sign(x)` - Sign of number (-1, 0, 1) - **Not implemented (future enhancement)**

#### 3.3 Implement Rounding ✅ COMPLETED (3/3)
- [x] `Math.floor(x)` - Round down ✅ **WORKING!**
  - Example: `Math.floor(3.7)` → `3` ✅
  - Uses `x.floor() as i32`
  - Returns i32 (integer)
  - Tested and verified ✅
- [x] `Math.ceil(x)` - Round up ✅ **WORKING!**
  - Example: `Math.ceil(3.2)` → `4` ✅
  - Uses `x.ceil() as i32`
  - Returns i32 (integer)
  - Tested and verified ✅
- [x] `Math.round(x)` - Round to nearest ✅ **WORKING!**
  - Example: `Math.round(3.5)` → `4`, `Math.round(3.4)` → `3` ✅
  - Uses `x.round() as i32`
  - Returns i32 (integer)
  - Tested and verified ✅

#### 3.4 Implement Min/Max/Random ✅ COMPLETED (3/3)
- [x] `Math.min(a, b)` - Minimum value ✅ **WORKING!**
  - Example: `Math.min(10.5, 20.3)` → `10.5` ✅
  - Uses `a.min(b)` method
  - Currently supports 2 arguments only
  - Tested and verified ✅
- [x] `Math.max(a, b)` - Maximum value ✅ **WORKING!**
  - Example: `Math.max(10.5, 20.3)` → `20.3` ✅
  - Uses `a.max(b)` method
  - Currently supports 2 arguments only
  - Tested and verified ✅
- [x] `Math.random()` - Random float [0, 1) ✅ **WORKING!**
  - Example: `Math.random()` → `0.8025414370953201` ✅
  - Uses `rand::random::<f64>()`
  - Auto-detects usage and adds `rand` dependency to Cargo.toml
  - Tested and verified ✅

#### 3.5 Add Constants
- [ ] `Math.PI` - π (3.14159...) - **Not implemented (future enhancement)**
- [ ] `Math.E` - Euler's number (2.71828...) - **Not implemented (future enhancement)**

#### 3.6 Testing & Documentation ✅ TESTS COMPLETE
- [x] Create comprehensive test files:
  - `test_math.liva` - Basic sqrt test ✅
  - `test_math_complete.liva` - All 9 functions tested ✅
- [ ] Add unit tests in `tests/stdlib_tests.rs` - **TODO**
- [ ] Add integration tests - **TODO**
- [x] Create `docs/language-reference/stdlib/math.md` - **Placeholder created** ✅
- [ ] Add code snippets to VSCode extension - **TODO**

**Success Criteria:** ✅ All 9 math functions implemented and verified! 🎉

**Implementation Details:**
- Added `generate_math_function_call()` in `src/codegen.rs`
- Math functions detected by checking if object is "Math" identifier
- Direct mapping to Rust f64 methods (sqrt, powf, abs, floor, ceil, round, min, max)
- Special handling for `Math.random()` using `rand` crate
- Fixed float literal generation: added `_f64` suffix for type clarity
- Fixed abs() precedence issue by wrapping unary expressions in parentheses
- Auto-detection of `Math.random()` usage in `src/desugaring.rs`
- Auto-adds `rand = "0.8"` to generated Cargo.toml when Math.random() is used

**Test Results:**
- ✅ sqrt(16.0) = 4.0
- ✅ pow(5.0, 2.0) = 25.0
- ✅ abs(-10.5) = 10.5 (fixed precedence issue)
- ✅ floor(3.7) = 3
- ✅ ceil(3.2) = 4
- ✅ round(3.5) = 4, round(3.4) = 3
- ✅ min(10.5, 20.3) = 10.5
- ✅ max(10.5, 20.3) = 20.3
- ✅ random() = 0.8025414370953201 (varies each run)

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
