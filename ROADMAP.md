# 🗺️ Liva Language Roadmap

> **Current Version:** v0.6.0  
> **Status:** Alpha - Feature-complete for core language  
> **Last Updated:** 2025-10-19

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

## 🔥 Phase 1: Consolidation & Quality (v0.6.1)

**Goal:** Production-ready v0.6 with zero warnings and 100% test coverage

**Status:** 🚧 In Progress  
**Branch:** `fix/consolidation-v0.6.1`  
**ETA:** 2-3 hours

### Tasks

#### 1.1 Fix Compiler Warnings (~30 min)
- [ ] Run `cargo fix --lib -p livac --allow-dirty`
- [ ] Remove unused imports in semantic.rs
- [ ] Fix unreachable code in codegen.rs (line 4610)
- [ ] Remove unused variables flagged by compiler
- [ ] Verify: `cargo build` produces 0 warnings

#### 1.2 Fix Failing Test (~15 min)
- [ ] Investigate `ir_codegen_string_templates` failure
- [ ] Review snapshot diff
- [ ] Update snapshot if correct, or fix code generation
- [ ] Verify: `cargo test` passes 100%

#### 1.3 Restore Semantic Unit Tests (~1 hour)
- [ ] Update `SemanticAnalyzer::new()` calls with new signature
- [ ] Fix test_expr_contains_async_variants
- [ ] Fix test_is_builtin_type_matches
- [ ] Uncomment all unit tests in semantic.rs
- [ ] Verify: All semantic unit tests pass

#### 1.4 Audit Inheritance Usage (~30 min)
- [ ] Search codebase for `Class : BaseClass` patterns
- [ ] Distinguish inheritance (illegal) from interfaces (legal)
- [ ] Update any remaining examples with inheritance
- [ ] Verify: No inheritance examples remain

#### 1.5 Update CHANGELOG (~15 min)
- [ ] Document v0.6.1 changes
- [ ] List breaking changes from v0.6.0 (protected removal)
- [ ] Add migration guide for visibility changes
- [ ] Update version numbers

#### 1.6 Final Verification
- [ ] `cargo test` - All tests pass ✅
- [ ] `cargo clippy` - No warnings ✅
- [ ] `cargo fmt --check` - Code formatted ✅
- [ ] Documentation builds correctly ✅
- [ ] VSCode extension works ✅

**Deliverable:** Liva v0.6.1 - Production-ready, zero warnings, 100% tests passing

---

## 🚀 Phase 2: Standard Library (v0.7.0)

**Goal:** Built-in functions and methods for common operations

**Status:** 📋 Planned  
**Branch:** `feature/stdlib-v0.7`  
**ETA:** 6-10 hours

### 2.1 Array Methods (~3 hours)
- [ ] Design API for array methods
- [ ] Implement `map(fn)` - Transform elements
- [ ] Implement `filter(fn)` - Filter elements
- [ ] Implement `reduce(fn, initial)` - Reduce to single value
- [ ] Implement `forEach(fn)` - Iterate with side effects
- [ ] Implement `find(fn)` - Find first match
- [ ] Implement `some(fn)` / `every(fn)` - Boolean checks
- [ ] Add tests for all array methods
- [ ] Update documentation

### 2.2 String Methods (~2 hours)
- [ ] Implement `split(delimiter)` - Split into array
- [ ] Implement `join(separator)` - Join array to string
- [ ] Implement `toUpperCase()` / `toLowerCase()`
- [ ] Implement `trim()` / `trimStart()` / `trimEnd()`
- [ ] Implement `replace(old, new)`
- [ ] Implement `startsWith(prefix)` / `endsWith(suffix)`
- [ ] Implement `substring(start, end)`
- [ ] Add tests for all string methods
- [ ] Update documentation

### 2.3 Math Functions (~2 hours)
- [ ] Design Math namespace/module
- [ ] Implement `Math.sqrt(x)` - Square root
- [ ] Implement `Math.pow(base, exp)` - Power
- [ ] Implement `Math.abs(x)` - Absolute value
- [ ] Implement `Math.floor()` / `Math.ceil()` / `Math.round()`
- [ ] Implement `Math.min(...)` / `Math.max(...)`
- [ ] Implement `Math.random()` - Random number
- [ ] Add constants: `Math.PI`, `Math.E`
- [ ] Add tests for all math functions
- [ ] Update documentation

### 2.4 Type Conversion (~1 hour)
- [ ] Implement `parseInt(str)` - String to int
- [ ] Implement `parseFloat(str)` - String to float
- [ ] Implement `toString(value)` - Any to string
- [ ] Implement `toNumber(str)` - String to number
- [ ] Handle errors in parsing (return error binding)
- [ ] Add tests
- [ ] Update documentation

### 2.5 Console/IO (~1 hour)
- [ ] Implement `console.log(...)` - Enhanced print
- [ ] Implement `console.error(...)` - Error output
- [ ] Implement `console.warn(...)` - Warning output
- [ ] Implement `readLine()` - Read user input
- [ ] Add tests
- [ ] Update documentation

### 2.6 Examples & Documentation (~1 hour)
- [ ] Create comprehensive examples using stdlib
- [ ] Update getting-started guide with stdlib
- [ ] Add stdlib reference documentation
- [ ] Update README with stdlib examples

**Deliverable:** Liva v0.7.0 - Usable standard library

---

## 📦 Phase 3: Module System (v0.8.0)

**Goal:** Organize code across multiple files

**Status:** 📋 Planned  
**Branch:** `feature/modules-v0.8`  
**ETA:** 8-12 hours

### 3.1 Design (~2 hours)
- [ ] Define module syntax (import/export)
- [ ] Design module resolution algorithm
- [ ] Decide on relative vs absolute imports
- [ ] Plan namespace handling
- [ ] Write module system spec

### 3.2 Parser & AST (~2 hours)
- [ ] Add `ImportDecl` to AST
- [ ] Add `ExportDecl` to AST
- [ ] Parse `import { name } from "path"`
- [ ] Parse `export { name }`
- [ ] Parse `export default`
- [ ] Handle multiple imports/exports
- [ ] Add tests

### 3.3 Module Resolver (~3 hours)
- [ ] Implement file resolution (relative paths)
- [ ] Implement module cache (avoid re-parsing)
- [ ] Handle circular dependencies
- [ ] Resolve exported symbols
- [ ] Build dependency graph
- [ ] Add tests

### 3.4 Semantic Analysis (~2 hours)
- [ ] Validate import paths exist
- [ ] Validate imported symbols exist
- [ ] Check for naming conflicts
- [ ] Ensure all exports are defined
- [ ] Add module-aware scope checking
- [ ] Add tests

### 3.5 Code Generation (~2 hours)
- [ ] Generate Rust module structure
- [ ] Handle imports as `use` statements
- [ ] Handle exports as `pub` modifiers
- [ ] Generate Cargo.toml with dependencies
- [ ] Add tests

### 3.6 Documentation & Examples (~1 hour)
- [ ] Write module system documentation
- [ ] Create multi-file example project
- [ ] Update getting-started guide
- [ ] Add best practices guide

**Deliverable:** Liva v0.8.0 - Multi-file projects supported

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

## 🎯 Phase 5: Enhanced Error Messages (v0.9.5)

**Goal:** Developer-friendly error messages

**Status:** 📋 Planned  
**Branch:** `feature/better-errors-v0.9.5`  
**ETA:** 5-8 hours

### 5.1 "Did You Mean?" Suggestions (~2 hours)
- [ ] Implement Levenshtein distance algorithm
- [ ] Suggest similar variable names
- [ ] Suggest similar function names
- [ ] Suggest similar type names
- [ ] Add tests

### 5.2 Enhanced Error Context (~2 hours)
- [ ] Show more context lines in errors
- [ ] Highlight specific tokens in red
- [ ] Show related code locations
- [ ] Add caret (^) under error position
- [ ] Add tests

### 5.3 Error Categories & Codes (~1 hour)
- [ ] Organize errors by category
- [ ] Assign unique error codes (E1001, etc.)
- [ ] Create error code documentation
- [ ] Link errors to docs

### 5.4 Hints & Help (~2 hours)
- [ ] Add helpful hints to common errors
- [ ] Suggest fixes for common mistakes
- [ ] Link to relevant documentation
- [ ] Add examples of correct usage
- [ ] Add tests

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
| **v0.6.1** | Consolidation & Quality | 🚧 In Progress | 2-3 hours |
| **v0.7.0** | Standard Library | 📋 Planned | 6-10 hours |
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
