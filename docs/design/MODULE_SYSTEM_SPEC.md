# 📦 Module System Specification - Liva v0.8.0

> **Status:** ✅ Approved - Ready for Implementation  
> **Date:** 2025-10-20  
> **Branch:** `feature/modules-v0.8.0`

---

## 🎯 Overview

Liva's module system allows organizing code across multiple files with a simple, intuitive syntax:

- **Public by default** - Easy for beginners
- **`_` prefix for private** - Consistent with existing Liva conventions
- **JavaScript-style imports** - Familiar syntax
- **No new keywords** - Keeps the language simple

---

## 📝 Syntax

### Import Statement

```liva
import { symbol1, symbol2, ... } from "path/to/file.liva"
```

### Wildcard Import (Namespace)

```liva
import * as moduleName from "path/to/file.liva"
```

### Visibility Rules

**Public (Exported):**
- Functions, classes, constants **WITHOUT** `_` prefix
- Automatically exported and available to importers

**Private (Not Exported):**
- Functions, classes, constants **WITH** `_` prefix
- Only accessible within the same file

---

## 📖 Examples

### Example 1: Simple Math Module

**File: `math.liva`**
```liva
// Public functions (exported)
add(a: Int, b: Int) -> Int {
    return a + b
}

multiply(a: Int, b: Int) -> Int {
    return a * b
}

// Private helper (not exported)
_validate(x: Int) -> Bool {
    return x > 0
}

// Public constant
const PI = 3.14159

// Private constant
const _EPSILON = 0.00001
```

**File: `main.liva`**
```liva
import { add, multiply, PI } from "./math.liva"

main() {
    let result = add(5, 3)
    print($"5 + 3 = {result}")
    
    let product = multiply(4, 7)
    print($"4 * 7 = {product}")
    
    print($"PI = {PI}")
}
```

---

### Example 2: Class-Based Module

**File: `models/user.liva`**
```liva
// Public class (exported)
class User {
    name: String
    age: Int
    _id: Int  // Private field (convention, not enforced by modules)
    
    constructor(name: String, age: Int) {
        this.name = name
        this.age = age
        this._id = _generateId()
    }
    
    greet() {
        print($"Hello, I'm {this.name}")
    }
    
    _validate() {
        // Private method
        return this.age >= 0
    }
}

// Private helper function (not exported)
_generateId() -> Int {
    return Math.random() * 1000
}

// Public utility function (exported)
createUser(name: String, age: Int) -> User {
    return User(name, age)
}
```

**File: `main.liva`**
```liva
import { User, createUser } from "./models/user.liva"

main() {
    let user1 = User("Alice", 25)
    user1.greet()
    
    let user2 = createUser("Bob", 30)
    user2.greet()
}
```

---

### Example 3: Wildcard Imports (Namespace)

**File: `utils.liva`**
```liva
greet(name: String) {
    print($"Hello, {name}!")
}

farewell(name: String) {
    print($"Goodbye, {name}!")
}

const MAX_USERS = 100
```

**File: `main.liva`**
```liva
import * as utils from "./utils.liva"

main() {
    utils.greet("Alice")
    utils.farewell("Bob")
    print($"Max users: {utils.MAX_USERS}")
}
```

---

### Example 4: Multi-Level Project Structure

**Project Structure:**
```
calculator/
├── main.liva
├── operations/
│   ├── basic.liva
│   └── advanced.liva
├── models/
│   └── calculator.liva
└── utils/
    └── formatter.liva
```

**File: `operations/basic.liva`**
```liva
add(a: Int, b: Int) -> Int {
    return a + b
}

subtract(a: Int, b: Int) -> Int {
    return a - b
}

multiply(a: Int, b: Int) -> Int {
    return a * b
}

divide(a: Int, b: Int) -> Int {
    if b == 0 {
        fail("Division by zero")
    }
    return a / b
}
```

**File: `operations/advanced.liva`**
```liva
import { multiply } from "./basic.liva"

power(base: Int, exp: Int) -> Int {
    if exp == 0 {
        return 1
    }
    let result = base
    let i = 1
    while i < exp {
        result = multiply(result, base)
        i = i + 1
    }
    return result
}

factorial(n: Int) -> Int {
    if n <= 1 {
        return 1
    }
    return multiply(n, factorial(n - 1))
}
```

**File: `models/calculator.liva`**
```liva
import { add, subtract, multiply, divide } from "../operations/basic.liva"
import { power, factorial } from "../operations/advanced.liva"

class Calculator {
    result: Int
    
    constructor() {
        this.result = 0
    }
    
    add(value: Int) {
        this.result = add(this.result, value)
        return this
    }
    
    subtract(value: Int) {
        this.result = subtract(this.result, value)
        return this
    }
    
    multiply(value: Int) {
        this.result = multiply(this.result, value)
        return this
    }
    
    divide(value: Int) {
        this.result = divide(this.result, value)
        return this
    }
    
    power(exp: Int) {
        this.result = power(this.result, exp)
        return this
    }
    
    getResult() -> Int {
        return this.result
    }
}
```

**File: `utils/formatter.liva`**
```liva
formatNumber(num: Int) -> String {
    return toString(num)
}

formatResult(label: String, value: Int) -> String {
    return $"{label}: {formatNumber(value)}"
}
```

**File: `main.liva`**
```liva
import { Calculator } from "./models/calculator.liva"
import { formatResult } from "./utils/formatter.liva"
import * as basic from "./operations/basic.liva"
import * as advanced from "./operations/advanced.liva"

main() {
    // Using Calculator class
    let calc = Calculator()
    calc.add(10).multiply(5).subtract(3)
    print(formatResult("Calculator result", calc.getResult()))
    
    // Using imported functions directly
    let sum = basic.add(5, 3)
    print(formatResult("5 + 3", sum))
    
    let pow = advanced.power(2, 8)
    print(formatResult("2^8", pow))
    
    let fact = advanced.factorial(5)
    print(formatResult("5!", fact))
}
```

---

## 🔧 Module Resolution Rules

### Relative Paths

```liva
import { X } from "./file.liva"        // Same directory
import { Y } from "./sub/file.liva"    // Subdirectory
import { Z } from "../file.liva"       // Parent directory
import { W } from "../../file.liva"    // Up two levels
```

### Path Resolution Algorithm

1. **Parse import path** from string literal
2. **Resolve relative to current file** using filesystem
3. **Check if file exists** (must end with `.liva`)
4. **Parse the imported file** if not already in cache
5. **Extract public symbols** (non-`_` prefixed)
6. **Validate imported symbols** exist in the file
7. **Add to dependency graph**

### Circular Dependency Detection

```liva
// ❌ ERROR: Circular dependency
// file1.liva
import { funcB } from "./file2.liva"

// file2.liva
import { funcA } from "./file1.liva"
```

**Error Message:**
```
Error: Circular dependency detected:
  file1.liva → file2.liva → file1.liva
```

---

## 📐 AST Changes

### New AST Nodes

```rust
// Import declaration
pub struct ImportDecl {
    pub imports: Vec<ImportSpecifier>,
    pub source: String,           // Path to file
    pub is_wildcard: bool,        // true for `import * as`
    pub alias: Option<String>,    // For wildcard: `import * as name`
}

pub enum ImportSpecifier {
    Named(String),                // import { name }
}

// Top-level item
pub enum TopLevelItem {
    Function(FunctionDecl),
    Class(ClassDecl),
    Interface(InterfaceDecl),
    Const(ConstDecl),
    Import(ImportDecl),           // ← NEW
}
```

---

## 🔍 Semantic Analysis

### Symbol Table Per Module

Each module maintains:
- **Public symbols** - Functions, classes, constants without `_` prefix
- **Private symbols** - Functions, classes, constants with `_` prefix
- **Imported symbols** - From other modules

### Cross-Module Resolution

```rust
pub struct ModuleResolver {
    modules: HashMap<PathBuf, Module>,
    dependency_graph: DependencyGraph,
}

pub struct Module {
    path: PathBuf,
    ast: Vec<TopLevelItem>,
    public_symbols: HashSet<String>,
    private_symbols: HashSet<String>,
    imports: Vec<ImportDecl>,
}
```

### Validation Rules

1. **Import path must exist** on filesystem
2. **Imported symbols must be public** (no `_` prefix)
3. **No circular dependencies** allowed
4. **No name collisions** in import scope
5. **All imports must be used** (warning)

---

## 🎨 Code Generation Strategy

### Multi-File Rust Project Structure

```
target/liva_build/
├── Cargo.toml
└── src/
    ├── main.rs              // Entry point
    ├── math.rs              // Corresponds to math.liva
    ├── operations/
    │   ├── mod.rs           // Module declaration
    │   ├── basic.rs         // basic.liva
    │   └── advanced.rs      // advanced.liva
    └── models/
        ├── mod.rs
        └── calculator.rs
```

### Generated Rust Code

**From: `math.liva`**
```liva
add(a: Int, b: Int) -> Int {
    return a + b
}

_privateHelper() {
    // ...
}

const PI = 3.14
```

**To: `src/math.rs`**
```rust
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn private_helper() {
    // ...
}

pub const PI: f64 = 3.14_f64;
```

**From: `main.liva`**
```liva
import { add, PI } from "./math.liva"

main() {
    let result = add(5, 3)
    print($"Result: {result}, PI: {PI}")
}
```

**To: `src/main.rs`**
```rust
mod liva_rt { /* ... */ }
mod math;

use math::{add, PI};

fn main() {
    let result = add(5, 3);
    println!("{}", format!("Result: {}, PI: {}", result, PI));
}
```

---

## 🧪 Test Strategy

### Unit Tests

```rust
#[test]
fn test_parse_import_named() {
    // import { add, multiply } from "./math.liva"
}

#[test]
fn test_parse_import_wildcard() {
    // import * as math from "./math.liva"
}

#[test]
fn test_resolve_relative_path() {
    // ./file.liva, ../file.liva, etc.
}

#[test]
fn test_detect_circular_dependency() {
    // Should error on circular imports
}

#[test]
fn test_validate_imported_symbol_exists() {
    // Should error if imported symbol doesn't exist
}

#[test]
fn test_validate_imported_symbol_is_public() {
    // Should error if importing private symbol (_prefix)
}
```

### Integration Tests

```liva
// Test multi-file compilation
// examples/modules/calculator/
```

---

## 📋 Implementation Checklist

### Phase 1: Parser (Days 1-2)
- [ ] Add `ImportDecl` to AST
- [ ] Parse `import { name, ... } from "path"`
- [ ] Parse `import * as name from "path"`
- [ ] Handle multiple imports in braces
- [ ] Parse string literals for paths
- [ ] Add parser tests (10+ tests)

### Phase 2: Module Resolver (Days 3-4)
- [ ] Implement file path resolution
- [ ] Load and parse imported files
- [ ] Build module cache (avoid re-parsing)
- [ ] Extract public/private symbols
- [ ] Build dependency graph
- [ ] Detect circular dependencies
- [ ] Add resolver tests (15+ tests)

### Phase 3: Semantic Analysis (Day 5)
- [ ] Validate import paths exist
- [ ] Validate imported symbols exist
- [ ] Validate imported symbols are public
- [ ] Check for name collisions
- [ ] Update scope resolution for imports
- [ ] Add semantic tests (10+ tests)

### Phase 4: Code Generation (Days 6-7)
- [ ] Generate `mod` declarations
- [ ] Generate `use` statements
- [ ] Add `pub` to public symbols
- [ ] Generate multi-file Rust project
- [ ] Update Cargo.toml generation
- [ ] Handle relative paths in Rust
- [ ] Add codegen tests (10+ tests)

### Phase 5: Integration & Examples (Day 8)
- [ ] Create calculator example project
- [ ] Create multi-module example
- [ ] End-to-end compilation tests
- [ ] Update documentation
- [ ] Update CHANGELOG
- [ ] Update ROADMAP

**Estimated Total:** 8-10 days (60-80 hours)

---

## 🚨 Error Messages

### Missing Import

```
Error [E4001]: Cannot find imported symbol 'nonExistent'
  --> main.liva:1:10
   |
 1 | import { nonExistent } from "./math.liva"
   |          ^^^^^^^^^^^ not found in module
   |
   = help: Available symbols: add, multiply, PI
```

### Private Symbol Import

```
Error [E4002]: Cannot import private symbol '_privateHelper'
  --> main.liva:1:10
   |
 1 | import { _privateHelper } from "./math.liva"
   |          ^^^^^^^^^^^^^^ symbol is private
   |
   = note: Symbols with underscore prefix are private to their module
   = help: Remove the underscore prefix to make it public
```

### Circular Dependency

```
Error [E4003]: Circular dependency detected
  --> main.liva:1:1
   |
 1 | import { funcA } from "./file1.liva"
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: Import chain:
     main.liva
       → file1.liva
         → file2.liva
           → file1.liva (circular!)
```

### File Not Found

```
Error [E4004]: Cannot find module './missing.liva'
  --> main.liva:1:27
   |
 1 | import { func } from "./missing.liva"
   |                      ^^^^^^^^^^^^^^^^ file does not exist
   |
   = help: Check the path is correct relative to the current file
```

---

## 🎯 Success Criteria

- [x] **Specification complete** - All details documented
- [ ] **Parser implemented** - Handles all import syntax
- [ ] **Module resolver working** - Loads multi-file projects
- [ ] **Semantic validation** - Catches all error cases
- [ ] **Code generation** - Produces working Rust projects
- [ ] **Tests passing** - 100% coverage for new features
- [ ] **Examples working** - Calculator project compiles
- [ ] **Documentation complete** - User guide and reference

---

## 📚 Future Enhancements (Post v0.8.0)

### Absolute Imports (v0.8.1)
```liva
import { Http } from "/src/network/http.liva"
```

### Package System (v0.9.0)
```liva
import { Http } from "@liva/http"
import { Json } from "@liva/json"
```

### Re-exports (v0.9.0)
```liva
// Export symbols from another module
import { add, multiply } from "./math.liva"
// Could support: export { add, multiply } from "./math.liva"
```

### Type-Only Imports (v1.0.0)
```liva
import type { UserType } from "./types.liva"
```

---

**Ready to implement! Let's build the module system! 🚀**
