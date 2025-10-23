# 🗺️ Liva Language Roadmap

> **Current Version:** v0.8.0  
> **Status:** Alpha - Module system complete, working on enhanced errors  
> **Last Updated:** 2025-10-23

---

## 🎯 Vision

Build a modern, practical programming language that combines:
- **Simplicity** of Python/TypeScript
- **Performance** of Rust
- **Safety** with explicit error handling
- **Hybrid concurrency** (async + parallel)

---

## 📍 Current Status (v0.6.0)

### ✅ Completed Features

**Core Language:**
- ✅ Variables (`let`, `const`) with type inference
- ✅ Functions (one-liner, block, typed parameters/returns)
- ✅ Classes (constructors, fields, methods)
- ✅ Interfaces (method signatures, multiple implementation)
- ✅ Control flow (`if`, `while`, `for`, `switch`, ternary)
- ✅ Operators (arithmetic, logical, comparison, bitwise)
- ✅ String templates with interpolation
- ✅ Visibility modifiers (public, private)

**Concurrency:**
- ✅ Async/await for I/O-bound operations
- ✅ Parallel execution for CPU-bound operations
- ✅ Task handles (`task`, `fire`, `await`)
- ✅ Hybrid concurrency (mix async + parallel)

**Error Handling:**
- ✅ Explicit `fail` statements
- ✅ Error binding (`let value, err = ...`)
- ✅ Fallibility inference (automatic detection)

**Tooling:**
- ✅ VS Code extension with IntelliSense
- ✅ Real-time interface validation
- ✅ Syntax highlighting and snippets
- ✅ Comprehensive test suite (110+ tests)
- ✅ Complete documentation (23 files)

**Recent Changes (2025-10-19):**
- ✅ Removed `protected` visibility (no inheritance = no need)
- ✅ Simplified to `public` (no prefix) and `private` (`_` prefix)
- ✅ Updated 68 files, 110+ test snapshots

---

## 🔥 Phase 1: Consolidation & Quality (v0.6.1) ✅ COMPLETED

**Goal:** Production-ready v0.6 with zero warnings and 100% test coverage

**Status:** ✅ COMPLETED (2025-10-20)  
**Branch:** `fix/consolidation-v0.6.1` (merged to main)  
**Release:** v0.6.1

### Completed Tasks

#### 1.1 Fix Compiler Warnings ✅
- [x] Run `cargo fix --lib -p livac --allow-dirty`
- [x] Remove unused imports in semantic.rs
- [x] Fix unreachable code in codegen.rs (line 4610)
- [x] Remove unused variables flagged by compiler
- [x] Verify: `cargo build` produces 0 warnings

**Result:** 26 warnings → 0 warnings ✅

#### 1.2 Fix Failing Test ✅
- [x] Investigate `ir_codegen_string_templates` failure
- [x] Implemented variable type tracking for format specifiers
- [x] Use `{}` for Display types, `{:?}` for Debug types
- [x] Update snapshot with correct output
- [x] Verify: `cargo test` passes 100%

**Result:** All 178 tests passing ✅

#### 1.3 Restore Semantic Unit Tests ⏭️ SKIPPED
- [x] Investigation showed tests were deleted, not commented
- [x] Old tests incompatible with current AST structure
- [x] Integration tests provide adequate coverage
- [x] Decision: Skip restoration, rely on integration tests

**Result:** Skipped (documented in TODO.md)

#### 1.4 Audit Inheritance Usage ✅
- [x] Search codebase for `Class : BaseClass` patterns
- [x] Found 1 illegal inheritance: `Empleado : Persona`
- [x] Replaced with composition pattern
- [x] Verified all other `:` usages are valid interfaces
- [x] Verify: No inheritance examples remain

**Result:** 0 class inheritance examples ✅

#### 1.5 Update CHANGELOG ✅
- [x] Created CHANGELOG.md following Keep a Changelog format
- [x] Document v0.6.1 changes (warnings, tests, inheritance)
- [x] List breaking changes from v0.6.0 (protected removal)
- [x] Add migration guide for visibility changes
- [x] Update version numbers

**Result:** CHANGELOG.md complete ✅

#### 1.6 Final Verification ✅
- [x] `cargo test` - All tests pass ✅ (178/178)
- [x] `cargo build` - 0 warnings ✅
- [x] `cargo fmt` - Code formatted ✅
- [x] Merged to main ✅
- [x] Tagged v0.6.1 ✅
- [x] Pushed to remote ✅

**Result:** Phase 1 Complete! 🎉
- [ ] `cargo clippy` - No warnings ✅
- [ ] `cargo fmt --check` - Code formatted ✅
- [ ] Documentation builds correctly ✅
- [ ] VSCode extension works ✅

**Deliverable:** Liva v0.6.1 - Production-ready, zero warnings, 100% tests passing

---

## 🚀 Phase 2: Standard Library (v0.7.0) ✅ COMPLETED

**Goal:** Built-in functions and methods for common operations

**Status:** ✅ COMPLETED - 37 FUNCTIONS IMPLEMENTED! 🎉  
**Branch:** `feature/stdlib-v0.7.0`  
**Started:** 2025-10-20  
**Completed:** 2025-10-20 (same day!)  
**Release:** v0.7.0 (2025-10-20)  
**Progress:** Arrays 9/9 ✅ | Strings 11/11 ✅ | Math 9/9 ✅ | Conversions 3/3 ✅ | I/O 5/5 ✅ | Print 1/1 ✅  
**Achievement:** Full stdlib implementation in one day! 🎉

### 2.1 Array Methods ✅ COMPLETED (9/9 methods) 🎉
- [x] Design API for array methods ✅
  - [x] Adapter syntax (`.par()`, `.vec()`, `.parvec()`)
  - [x] Parser implementation for adapters
  - [x] AST extensions (`MethodCallExpr`, `ArrayAdapter`)
- [x] Implement `map(fn)` - Transform elements ✅ **WORKING!**
  - [x] Sequential: `.map(x => x * 2)` ✅
  - [x] Generates: `.iter().map(|&x| ...).collect()`
  - [x] Tested with simple and block lambdas
- [x] Implement `filter(fn)` - Filter elements ✅ **WORKING!**
  - [x] Sequential: `.filter(x => x > 5)` ✅
  - [x] Generates: `.iter().filter(|&&x| ...).copied().collect()`
  - [x] Tested with simple and complex predicates
- [x] Implement `reduce(fn, initial)` - Reduce to single value ✅ **WORKING!**
  - [x] Uses Rust's `.iter().fold(initial, |acc, &x| expr)`
  - [x] Tested: Sum(15), Product(120), Max(5), Count(5)
- [x] Implement `forEach(fn)` - Iterate with side effects ✅ **WORKING!**
  - [x] Uses `.iter().for_each(|&x| { ... })`
  - [x] Tested: print, squares, sum accumulation
- [x] Implement `find(fn)` - Find first match ✅ **WORKING!**
  - [x] Uses `.iter().find(|&&x| pred).copied()`
  - [x] Returns Option<T> (Some/None)
  - [x] Tested: Some(15), None, Some(10), Some(1)
- [x] Implement `some(fn)` / `every(fn)` - Boolean checks ✅ **WORKING!**
  - [x] some: `.iter().any(|&x| pred)` → bool
  - [x] every: `.iter().all(|&x| pred)` → bool
  - [x] Tested: all boolean checks passing
- [x] Implement `indexOf(value)` / `includes(value)` - Search ✅ **WORKING!**
  - [x] indexOf: `.iter().position(|&x| x == value)` → i32
  - [x] includes: `.iter().any(|&x| x == value)` → bool
  - [x] Tested: indexOf(2, 0, 4, -1), includes(true/false)
- [x] All 9 core array methods complete! 🎉
- [x] Comprehensive tests created (6 test files)
- [ ] Add comprehensive unit tests in tests/stdlib_tests.rs
- [ ] Update documentation with working examples

**Achievement Unlocked:** 🚀 Complete array methods implementation in 1 day!

### 2.2 String Methods ✅ COMPLETED (11/11 methods) 🎉
- [x] Implement `split(delimiter)` - Split into array ✅ **WORKING!**
  - [x] Uses `.split(delim).map(|s| s.to_string()).collect::<Vec<String>>()`
  - [x] Returns Vec<String> for Liva array compatibility
  - [x] Tested: ["apple", "banana", "orange"] ✅
- [x] Implement `replace(old, new)` - Replace substring ✅ **WORKING!**
  - [x] Uses `.replace(old, new)`
  - [x] Tested: "hello Liva" ✅
- [x] Implement `toUpperCase()` / `toLowerCase()` ✅ **WORKING!**
  - [x] Uses `.to_uppercase()` / `.to_lowercase()`
  - [x] Tested: "HELLO" / "hello world" ✅
- [x] Implement `trim()` / `trimStart()` / `trimEnd()` ✅ **WORKING!**
  - [x] Uses `.trim()` / `.trim_start()` / `.trim_end()`
  - [x] Tested: whitespace removal working correctly ✅
- [x] Implement `startsWith(prefix)` / `endsWith(suffix)` ✅ **WORKING!**
  - [x] Uses `.starts_with()` / `.ends_with()`
  - [x] Returns bool
  - [x] Tested: boolean checks working ✅
- [x] Implement `substring(start, end)` ✅ **WORKING!**
  - [x] Uses slice syntax `[start as usize..end as usize].to_string()`
  - [x] Tested: "Hello", "World" extraction ✅
- [x] Implement `charAt(index)` ✅ **WORKING!**
  - [x] Uses `.chars().nth(index as usize).unwrap_or(' ')`
  - [x] UTF-8 safe character access
  - [x] Tested: 'H', 'W' character access ✅
- [x] Implement `indexOf(substring)` ✅ **WORKING!**
  - [x] Uses `.find(substring).map(|i| i as i32).unwrap_or(-1)`
  - [x] Returns i32 (-1 if not found)
  - [x] Disambiguated from array indexOf by argument type
  - [x] Tested: 4, 16, 31, -1 (not found) ✅
- [x] All 11 string methods complete! 🎉
- [x] Comprehensive tests created (4 test files)
- [ ] Implement `join(separator)` on string arrays (future enhancement)
- [ ] Add comprehensive unit tests in tests/stdlib_tests.rs
- [ ] Update documentation with working examples

**Achievement Unlocked:** 🔥 Complete string methods implementation in 1 day!

**Technical Highlights:**
- Reused existing `MethodCall` AST node (no parser changes)
- Added `generate_string_method_call()` in codegen.rs
- Direct mapping to Rust string methods (no iterators)
- indexOf disambiguation by argument type detection

### 2.3 Math Functions ✅ COMPLETED (9/9 functions) 🎉
- [x] Design Math namespace/module - **Namespace style `Math.*`** ✅
- [x] Implement `Math.sqrt(x)` - Square root ✅ **WORKING!**
  - Example: `Math.sqrt(16.0)` → `4.0` ✅
- [x] Implement `Math.pow(base, exp)` - Power ✅ **WORKING!**
  - Example: `Math.pow(5.0, 2.0)` → `25.0` ✅
- [x] Implement `Math.abs(x)` - Absolute value ✅ **WORKING!**
  - Example: `Math.abs(-10.5)` → `10.5` ✅
- [x] Implement `Math.floor()` / `Math.ceil()` / `Math.round()` ✅ **WORKING!**
  - floor: `Math.floor(3.7)` → `3` ✅
  - ceil: `Math.ceil(3.2)` → `4` ✅
  - round: `Math.round(3.5)` → `4` ✅
- [x] Implement `Math.min()` / `Math.max()` ✅ **WORKING!**
  - min: `Math.min(10.5, 20.3)` → `10.5` ✅
  - max: `Math.max(10.5, 20.3)` → `20.3` ✅
- [x] Implement `Math.random()` - Random number ✅ **WORKING!**
  - Example: `Math.random()` → `0.8025414370953201` ✅
  - Auto-adds `rand` crate dependency
- [ ] Add constants: `Math.PI`, `Math.E` - **Future enhancement**
- [x] Add tests for all math functions ✅
  - Created `test_math_complete.liva` with all 9 functions
- [x] Update documentation ✅
  - CHANGELOG.md updated
  - TODO_STDLIB.md updated
  - math.md placeholder created

**Implementation:**
- Added `generate_math_function_call()` in codegen.rs
- Auto-detection of `Math.random()` usage adds `rand` to Cargo.toml
- Float literals now generate with `_f64` suffix for type clarity
- Fixed precedence issue with `abs()` for unary expressions

### 2.4 Type Conversion ✅ COMPLETED (~1 hour)
- [x] Implement `parseInt(str)` - String to int with error binding
- [x] Implement `parseFloat(str)` - String to float with error binding
- [x] Implement `toString(value)` - Any to string
- [ ] Implement `toNumber(str)` - String to number (future enhancement)
- [x] Handle errors in parsing (return error binding tuples)
- [x] Add tests (test_conversions.liva)
- [x] Update documentation (conversions.md, CHANGELOG, TODO, ROADMAP)

**Status:** ✅ Complete (3/3 functions implemented)  
**Branch:** `feature/stdlib-v0.7.0`  
**Completion:** 2025-10-20

**Deliverables:**
- ✅ parseInt/parseFloat with error binding pattern
- ✅ toString for all primitive types
- ✅ Comprehensive test suite
- ✅ Full documentation

**Next:** Console/IO functions

### 2.5 Console/IO ✅ COMPLETED (~1 hour)
- [x] Implement `console.log(...)` - Enhanced print ✅
- [x] Implement `console.error(...)` - Error output ✅
- [x] Implement `console.warn(...)` - Warning output ✅
- [x] Implement `console.readLine()` - Read user input ✅
- [x] Implement `console.prompt(message)` - Prompt and read ✅
- [x] Add tests (test_io.liva) ✅
- [x] Update documentation (io.md, CHANGELOG, TODO, ROADMAP) ✅

**Status:** ✅ Complete (5/5 functions implemented)  
**Branch:** `feature/stdlib-v0.7.0`  
**Completion:** 2025-10-20

**Design Decision: Hybrid I/O Approach**
- **`print()`** - Simple function for beginners, Display format `{}`
  - Use case: Final output, user-facing messages
  - Example: `print("Hello")`, `print($"Name: {name}")`
- **`console.*`** - Professional namespace, Debug format `{:?}`
  - Use case: Debugging, development, structured logging
  - Functions: `console.log()`, `console.error()`, `console.warn()`, `console.readLine()`, `console.prompt()`
  - Familiar to JavaScript/Node.js developers
  - Organized under single namespace for discoverability

**Deliverables:**
- ✅ Hybrid approach: `print()` + `console.*` namespace
- ✅ console.log/error/warn for different output streams
- ✅ console.readLine/prompt for user input
- ✅ Comprehensive test suite
- ✅ Full documentation

**Next:** Phase 2 complete! Ready for v0.7.0 release 🎉

### 2.6 Examples & Documentation ✅ COMPLETED
- [x] Create comprehensive examples using stdlib ✅
  - test_arrays_complete.liva, test_strings_complete.liva
  - test_math_complete.liva, test_conversions.liva
  - test_io.liva, test_print_vs_console.liva
- [x] Update getting-started guide with stdlib ✅
- [x] Add stdlib reference documentation ✅
  - docs/language-reference/stdlib/arrays.md
  - docs/language-reference/stdlib/strings.md
  - docs/language-reference/stdlib/math.md
  - docs/language-reference/stdlib/conversions.md
  - docs/language-reference/stdlib/io.md (667 lines, comprehensive)
- [x] Update README with stdlib examples ✅

**Deliverable:** Liva v0.7.0 - Production-ready standard library ✅ RELEASED!

---

## 📦 Phase 3: Module System (v0.8.0) ✅ COMPLETE

**Goal:** Organize code across multiple files

**Status:** ✅ 100% Complete - RELEASED v0.8.0  
**Branch:** `feature/modules-v0.8.0` → **Merged to main**  
**Started:** 2024-10-20  
**Completed:** 2025-10-21  
**Progress:** 17h actual / 53h estimated  
**Efficiency:** 3.1x faster than estimated  
**Tag:** v0.8.0

**Design Decision:** Hybrid approach
- **Public by default** - Functions, classes, constants without `_` prefix are exported
- **Private with `_` prefix** - Consistent with Liva's existing conventions
- **JavaScript-style imports** - `import { name } from "./path.liva"`
- **Wildcard imports** - `import * as name from "./path.liva"`
- **No new keywords** - Simple and intuitive

### 3.1 Design ✅ COMPLETED (2 hours)
- [x] Define module syntax (import/export)
  - Syntax: `import { name } from "./file.liva"`
  - Wildcard: `import * as name from "./file.liva"`
  - Public: No `_` prefix (auto-exported)
  - Private: `_` prefix (not exported)
- [x] Design module resolution algorithm
  - Relative path resolution
  - Recursive loading with caching
  - Dependency graph with cycle detection
- [x] Decide on relative vs absolute imports
  - Relative paths for now: `./`, `../`
  - Absolute paths from root: future enhancement
- [x] Plan namespace handling
  - Named imports: add to scope directly
  - Wildcard imports: namespace with dot notation
- [x] Write module system spec
  - Complete specification document created
  - Examples and edge cases documented

### 3.2 Parser & AST ✅ COMPLETED (2 hours, Commit: 4e0d8b6)
- [x] Add `ImportDecl` to AST with Display trait
- [x] Parse `import { name } from "path"`
- [x] Parse `import * as name from "path"`
- [x] Handle multiple imports in braces with trailing commas
- [x] Added `from` keyword to lexer
- [x] Verified with DEBUG output - all import variants parse correctly

**Estimated:** 8 hours | **Actual:** 2 hours | **Efficiency:** 4x faster

### 3.3 Module Resolver ✅ COMPLETED (4 hours, Commits: 11abaaf, ad229ef)
- [x] Implement file resolution (relative paths with ./, ../)
- [x] Implement module cache (HashMap with canonical paths)
- [x] Handle circular dependencies (DFS cycle detection)
- [x] Resolve exported symbols (extract non-`_` symbols)
- [x] Build dependency graph with topological sort
- [x] Add unit tests (3 cycle detection tests in module.rs)
- [x] Integrate with compiler pipeline
  - compile_with_modules() function
  - Auto-detection of imports
  - resolve_all() returns modules in compilation order
  - Tested with multi-file example
- [ ] Integration tests (comprehensive test suite pending)

**Estimated:** 15 hours | **Actual:** 4 hours | **Efficiency:** 3.75x faster

### 3.4 Semantic Analysis ✅ COMPLETED (3 hours, Commit: eabe7d8)
- [x] Extend SemanticAnalyzer with import context
  - Added imported_modules and imported_symbols fields
  - New function: analyze_with_modules()
  - Accepts module context map
- [x] Validate imported symbols exist (E4006)
  - Check against module's public_symbols
  - Clear error if symbol not found
  - Path resolution for module lookup
- [x] Validate imported symbols are public (E4007)
  - Error if importing `_` prefixed symbol
  - Clear message about privacy rules
- [x] Detect name collisions
  - E4008: Import vs local definition
  - E4009: Import vs another import
  - Helpful error messages with suggestions
- [x] Path resolution
  - Resolve relative paths (./,  ../)
  - Canonicalize paths for matching
  - Fallback by filename matching
- [x] Symbol registration
  - Add imported symbols to function registry
  - Permissive arity checking for imports
- [x] Integration with compiler
  - Updated compile_with_modules()
  - Builds module context map
  - Passes to semantic analyzer

**Estimated:** 8 hours | **Actual:** 3 hours | **Efficiency:** 2.67x faster

### 3.5 Code Generation ✅ COMPLETED (2 hours, Commits: fae5280, 23c7335)
- [x] Generate multi-file Rust project structure
  - Implemented `generate_multifile_project()` with HashMap<PathBuf, String>
  - Each module → separate .rs file (math.rs, operations.rs, utils.rs)
  - Entry point → main.rs with mod declarations
- [x] Convert imports to Rust `use` statements
  - `import { add } from "./math.liva"` → `use crate::math::add;`
  - `import { a, b } from "./m.liva"` → `use crate::m::{a, b};`
  - Wildcard imports skip use (module available via mod)
- [x] Add `pub` modifiers to exported symbols
  - Functions without `_` prefix → `pub fn name()`
  - Private functions with `_` → `fn name()` (prefix removed)
- [x] Generate module declarations
  - All modules listed in main.rs: `mod math;`, `mod operations;`
- [x] Multi-file output system
  - `write_multifile_output()` writes all files
  - Proper directory structure (src/ folder)
- [x] Integration and testing
  - Tested with examples/modules/test_import_syntax.liva
  - Compiles successfully: `cargo build`
  - Executes correctly: "10 + 20 = 30" ✅

**Estimated:** 13 hours | **Actual:** 2 hours | **Efficiency:** 6.5x faster  
**Documentation:** docs/compiler-internals/multifile-codegen.md (650+ lines)

### 3.6 Integration & Examples ✅ COMPLETED (4 hours, Commits: 0f64234, 959f18e, 0aa99a7)
- [x] Write module system documentation (docs/language-reference/modules.md - 500+ lines) ✅
- [x] Write compiler internals docs (6 documents, ~2,500 lines total) ✅
- [x] Create multi-file example project (calculator - 65 lines, 3 modules) ✅
  * examples/calculator/calculator.liva - Entry point
  * examples/calculator/basic.liva - Basic operations (+, -, *, /)
  * examples/calculator/advanced.liva - Advanced operations
  * Demonstrates: named imports, public/private visibility
  * Tested: compiles and runs successfully
- [x] Update getting-started guide ✅
  * Added "Working with Modules" section to docs/getting-started/quick-start.md
  * Import syntax examples, public/private visibility demo
  * Multi-file compilation workflow
- [x] Add best practices guide ✅
  * Created docs/guides/module-best-practices.md (500+ lines)
  * Project structure patterns, naming conventions
  * Import patterns, visibility guidelines
  * Common patterns and anti-patterns
  * Performance tips and comprehensive examples
- [x] Polish error messages ✅
  * Enhanced E4003-E4009 with helpful hints and suggestions
  * Better context for circular dependencies
  * Specific suggestions (e.g., use aliases for name collisions)
  * Actionable guidance for resolving issues
- [x] Update TODO_MODULES.md (marked Phase 3.5 complete) ✅
- [x] Update CHANGELOG.md with Phase 3.6 ✅
- [x] Update ROADMAP.md with Phase 3.6 ✅
- [x] Run comprehensive test suite ✅ (27/27 lib tests, 3/3 module tests)
- [x] Prepare release notes and merge to main ✅ (Released Oct 21, 2025)
- [x] Update all documentation to v0.8.0 ✅ (README.md, docs/README.md, TODO_MODULES.md)

**Estimated:** 9 hours | **Actual:** ~4 hours | **Efficiency:** 2.25x faster

**Deliverable:** Liva v0.8.0 - Multi-file projects supported ✅ DELIVERED

**Final Status:** ✅ 100% Complete (All 6 phases done, 17h/53h actual - 3.1x faster than estimated!)  
**Released:** October 21, 2025  
**Tag:** v0.8.0

---

## 🔧 Phase 4: Generics (v0.9.0)

**Goal:** Generic types and functions for code reuse

**Status:** 📋 Planned  
**Branch:** `feature/generics-v0.9`  
**ETA:** 10-15 hours

### 4.1 Design (~2 hours)
- [ ] Define generic syntax `<T>`
- [ ] Plan type parameter bounds
- [ ] Design constraint system
- [ ] Handle variance (covariance/contravariance)
- [ ] Write generics spec

### 4.2 Parser & AST (~3 hours)
- [ ] Add type parameters to function declarations
- [ ] Add type parameters to class declarations
- [ ] Parse type arguments `Box<T>`
- [ ] Parse constraints `<T: Comparable>`
- [ ] Handle multiple type parameters
- [ ] Add tests

### 4.3 Type System (~4 hours)
- [ ] Implement type parameter substitution
- [ ] Implement type inference for generics
- [ ] Check type parameter constraints
- [ ] Handle generic method calls
- [ ] Implement monomorphization strategy
- [ ] Add tests

### 4.4 Code Generation (~3 hours)
- [ ] Generate Rust generic code
- [ ] Handle type parameter mapping
- [ ] Generate trait bounds
- [ ] Handle generic instantiation
- [ ] Add tests

### 4.5 Standard Library Updates (~2 hours)
- [ ] Make array generic: `Array<T>`
- [ ] Make Result generic: `Result<T, E>`
- [ ] Make Option generic: `Option<T>`
- [ ] Update existing stdlib with generics
- [ ] Add tests

### 4.6 Documentation & Examples (~1 hour)
- [ ] Write generics documentation
- [ ] Create examples (generic functions, classes)
- [ ] Update language reference
- [ ] Add best practices guide

**Deliverable:** Liva v0.9.0 - Generic programming support

---

## 🎯 Phase 5: Enhanced Error Messages (v0.8.1)

**Goal:** Developer-friendly error messages with suggestions and better context

**Status:** � In Progress  
**Branch:** `feature/better-errors-v0.8.1`  
**Started:** 2025-10-23  
**ETA:** 5-8 hours

### 5.1 "Did You Mean?" Suggestions (~2 hours) ✅ COMPLETE
- [x] Implement Levenshtein distance algorithm
- [x] Suggest similar variable names
- [x] Suggest similar function names
- [x] Suggest similar type names
- [x] Add tests

### 5.2 Enhanced Error Context (~2 hours) ✅ COMPLETE
- [x] Show more context lines in errors (2 before, 2 after)
- [x] Show precise token length in underline
- [x] Add caret (^) under error position with exact length
- [x] Update ErrorLocation structure with context fields
- [x] Implement get_context_lines() in semantic analyzer
- [x] Update parser error formatting
- [x] Add tests (test_parse_context.liva)

### 5.3 Error Categories & Codes (~1 hour) ✅ COMPLETE
- [x] Organize errors by category (E0xxx-E7xxx)
- [x] Create error_codes module with constants
- [x] Implement ErrorCategory enum
- [x] Display category in error messages
- [x] Document all error codes in ERROR_CODES.md
- [x] Add category detection from error code
- [x] Add tests for error categories

### 5.4 Hints & Help (~2 hours) ✅ COMPLETE
- [x] Create hints module with contextual help
- [x] Add automatic hints based on error codes
- [x] Add code examples for common errors
- [x] Add documentation links for each error
- [x] Integrate hints into error display
- [x] Add get_common_fixes() for error categories
- [x] Add get_tip() for improvement suggestions
- [x] Add tests for all hint functions

### 5.5 Documentation (~1 hour)
- [ ] Document error message format
- [ ] Create error code reference
- [ ] Add troubleshooting guide

**Deliverable:** Liva v0.9.5 - Best-in-class error messages

---

## 🚢 Phase 6: Production Release (v1.0.0)

**Goal:** Stable, production-ready language

**Status:** 📋 Planned  
**Branch:** `release/v1.0`  
**ETA:** TBD

### 6.1 Language Server Protocol (LSP)
- [ ] Implement LSP server
- [ ] Auto-completion
- [ ] Go to definition
- [ ] Find references
- [ ] Rename refactoring
- [ ] Hover documentation
- [ ] Signature help

### 6.2 Debugger Support
- [ ] Debug adapter protocol
- [ ] Breakpoint support
- [ ] Step through code
- [ ] Variable inspection
- [ ] Call stack

### 6.3 Performance Optimizations
- [ ] Profile compiler performance
- [ ] Optimize parsing
- [ ] Optimize type checking
- [ ] Optimize code generation
- [ ] Benchmark suite

### 6.4 Stability & Polish
- [ ] Comprehensive test suite (>90% coverage)
- [ ] Stress testing
- [ ] Memory leak detection
- [ ] Security audit
- [ ] Performance benchmarks

### 6.5 Documentation
- [ ] Complete language specification
- [ ] Tutorial series
- [ ] API reference
- [ ] Migration guides
- [ ] Best practices

### 6.6 Package Manager (Optional)
- [ ] Design package format
- [ ] Implement package registry
- [ ] Package discovery
- [ ] Dependency resolution
- [ ] Version management

**Deliverable:** Liva v1.0.0 - Production-ready language

---

## 📊 Milestones Summary

| Version | Focus | Status | ETA |
|---------|-------|--------|-----|
| **v0.6.1** | Consolidation & Quality | ✅ Completed | 2025-10-20 |
| **v0.7.0** | Standard Library | ✅ Completed | 2025-10-20 |
| **v0.8.0** | Module System | 📋 Planned | 8-12 hours |
| **v0.9.0** | Generics | 📋 Planned | 10-15 hours |
| **v0.9.5** | Better Errors | 📋 Planned | 5-8 hours |
| **v1.0.0** | Production Release | 📋 Planned | TBD |

**Total estimated effort to v1.0:** ~40-50 hours of focused development

---

## 🎯 Success Metrics

- **Compile time:** <500ms for 1000 LOC
- **Test coverage:** >90%
- **Zero compiler warnings**
- **Zero failing tests**
- **Documentation:** Complete & up-to-date
- **Community:** >100 GitHub stars
- **Adoption:** >10 real-world projects

---

## 📝 Notes

- Each phase should be completed on a separate branch
- All changes must pass CI (tests + linting)
- Documentation must be updated with each feature
- Breaking changes should be clearly documented
- Follow semantic versioning

---

## 🤝 Contributing

See main [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

For roadmap discussions, open an issue with the `roadmap` label.

---

**Last Updated:** 2025-10-19  
**Maintainer:** Fran Nadal
