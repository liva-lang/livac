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
- [ ] Review Rust Vec methods for inspiration
- [ ] Design Liva-style API (syntax, error handling)
- [ ] Decide: methods vs functions (e.g., `arr.map(fn)` vs `map(arr, fn)`)
- [ ] Document API design in `docs/stdlib/arrays.md`

#### 1.2 Implement Core Methods
- [ ] `map(fn)` - Transform each element
  - Example: `[1,2,3].map(x => x * 2)` → `[2,4,6]`
- [ ] `filter(fn)` - Keep elements matching predicate
  - Example: `[1,2,3].filter(x => x > 1)` → `[2,3]`
- [ ] `reduce(fn, initial)` - Reduce to single value
  - Example: `[1,2,3].reduce((acc, x) => acc + x, 0)` → `6`

#### 1.3 Implement Utility Methods
- [ ] `forEach(fn)` - Iterate with side effects
- [ ] `find(fn)` - Find first element matching predicate
- [ ] `some(fn)` - Check if any element matches
- [ ] `every(fn)` - Check if all elements match
- [ ] `indexOf(value)` - Find index of value
- [ ] `includes(value)` - Check if array contains value

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
