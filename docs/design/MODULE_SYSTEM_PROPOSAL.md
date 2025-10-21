# 📦 Module System Design Proposal - Phase 3 (v0.8.0)

> **Status:** 🎨 Design Phase  
> **Date:** 2025-10-20  
> **Author:** Fran Nadal (with AI assistance)

---

## 🎯 Goals

1. **Multi-file projects** - Split code across multiple files
2. **Namespace management** - Avoid naming conflicts
3. **Encapsulation** - Control what's public/private
4. **Reusability** - Share code between projects
5. **Simple & intuitive** - Easy to learn and use

---

## 🔍 Syntax Comparison - Other Languages

### JavaScript/TypeScript (ES6 Modules)
```javascript
// export
export function greet(name) { ... }
export class User { ... }
export const MAX_SIZE = 100
export default Calculator

// import
import { greet, User } from './utils.js'
import Calculator from './calc.js'
import * as utils from './utils.js'
```

**Pros:** 
- Very familiar to web developers
- Explicit named exports
- Default exports for single exports
- Wildcard imports

**Cons:**
- Default exports can be confusing
- Multiple ways to do the same thing

---

### Python
```python
# export (everything is exported by default, _ prefix for private)
def greet(name): ...
class User: ...
MAX_SIZE = 100
_internal_helper = 42  # private by convention

# import
from utils import greet, User
from calc import Calculator
import utils
import utils as u
from utils import *
```

**Pros:**
- Simple: everything is exported by default
- `_` prefix for private = consistent with Liva
- Multiple import styles

**Cons:**
- No explicit exports (implicit is less clear)
- Wildcard imports discouraged

---

### Rust
```rust
// export
pub fn greet(name: &str) { ... }
pub struct User { ... }
pub const MAX_SIZE: usize = 100;
fn internal_helper() { ... }  // private by default

// import
use utils::{greet, User};
use calc::Calculator;
use utils::*;
mod utils;  // declare module
```

**Pros:**
- Explicit with `pub` keyword
- Private by default (secure)
- Module tree structure
- Very clear what's public

**Cons:**
- Requires `mod` declarations
- More verbose

---

### Go
```go
// export (capitalized = public, lowercase = private)
func Greet(name string) { ... }  // public
type User struct { ... }          // public
const MaxSize = 100               // public
func internalHelper() { ... }     // private

// import
import "utils"
import . "utils"  // dot import
import _ "utils"  // side effects only
```

**Pros:**
- No keywords needed (capitalization = visibility)
- Very clean syntax

**Cons:**
- Case sensitivity is crucial
- Not obvious for beginners

---

## 🎨 Proposed Options for Liva

### Option 1: JavaScript-style (Explicit Exports)

**Syntax:**
```liva
// ============ math.liva ============
export fn add(a: Int, b: Int) -> Int {
    return a + b
}

export fn multiply(a: Int, b: Int) -> Int {
    return a * b
}

fn _internalHelper() {
    // private function
}

export class Calculator {
    // ...
}

export const PI = 3.14159

// ============ main.liva ============
import { add, multiply } from "./math.liva"
import { Calculator } from "./math.liva"

let result = add(5, 3)
let calc = Calculator()
```

**Pros:**
- Familiar to JS/TS developers
- Explicit exports (clear intent)
- Works well with existing Liva syntax

**Cons:**
- Requires `export` keyword everywhere
- More typing

---

### Option 2: Python-style (Implicit Exports + `_` prefix)

**Syntax:**
```liva
// ============ math.liva ============
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn multiply(a: Int, b: Int) -> Int {
    return a * b
}

fn _internalHelper() {
    // private by convention (not exported)
}

class Calculator {
    // ...
}

const PI = 3.14159

// ============ main.liva ============
from "./math.liva" import { add, multiply }
from "./math.liva" import { Calculator }

let result = add(5, 3)
let calc = Calculator()
```

**Pros:**
- Less typing (no `export` keyword)
- Consistent with `_` prefix for private
- Clean syntax

**Cons:**
- Everything public by default (less secure)
- Not as explicit

---

### Option 3: Rust-style (Explicit `pub` keyword)

**Syntax:**
```liva
// ============ math.liva ============
pub fn add(a: Int, b: Int) -> Int {
    return a + b
}

pub fn multiply(a: Int, b: Int) -> Int {
    return a * b
}

fn internalHelper() {
    // private by default (no pub)
}

pub class Calculator {
    // ...
}

pub const PI = 3.14159

// ============ main.liva ============
use "./math.liva"::{add, multiply}
use "./math.liva"::Calculator

let result = add(5, 3)
let calc = Calculator()
```

**Pros:**
- Private by default (more secure)
- Very explicit and clear
- Familiar to Rust developers

**Cons:**
- Different import syntax (`use` vs `import`)
- More keywords to learn

---

### Option 4: Hybrid (Mix of JS + Python + Liva conventions)

**Syntax:**
```liva
// ============ math.liva ============
// Public functions (no keyword needed, not prefixed with _)
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn multiply(a: Int, b: Int) -> Int {
    return a * b
}

// Private functions (prefixed with _)
fn _internalHelper() {
    // private
}

// Public class
class Calculator {
    // Public fields/methods
    fn calculate() { ... }
    
    // Private fields/methods (prefixed with _)
    fn _validate() { ... }
}

const PI = 3.14159      // public
const _EPSILON = 0.001  // private

// ============ main.liva ============
import { add, multiply } from "./math.liva"
import { Calculator, PI } from "./math.liva"
// or import all:
import * as math from "./math.liva"

let result = add(5, 3)
let calc = Calculator()
console.log(PI)
// math.add(5, 3) if using wildcard import
```

**Pros:**
- Familiar `import` syntax (like JS/TS)
- Uses existing `_` prefix convention for private
- No new keywords needed
- Public by default (easier for beginners)
- Wildcard imports for convenience

**Cons:**
- Relies on naming convention for privacy
- Everything public by default (need discipline)

---

## 🎯 Recommendation: **Option 4 (Hybrid)**

**Why?**

1. **Consistency with current Liva:**
   - Already uses `_` prefix for private fields/methods
   - No new visibility keywords needed
   - Keeps the language simple

2. **Familiar to most developers:**
   - `import` syntax from JavaScript/TypeScript
   - `from` alternative for those who prefer it
   - Wildcard imports (`import * as`) for convenience

3. **Beginner-friendly:**
   - Public by default (less mental overhead)
   - Clear naming convention with `_` prefix
   - Simple to learn

4. **Flexible:**
   - Named imports: `import { add, multiply } from "./math.liva"`
   - Wildcard imports: `import * as math from "./math.liva"`
   - Single import: `import { Calculator } from "./math.liva"`

---

## 📝 Detailed Syntax Specification (Option 4)

### Import Statements

```liva
// Named imports
import { functionName } from "./path/to/file.liva"
import { Class1, Class2 } from "./classes.liva"
import { fn1, fn2, CONSTANT } from "./utils.liva"

// Wildcard import (namespace)
import * as moduleName from "./module.liva"
moduleName.functionName()

// Mixed imports
import { main, helper } from "./app.liva"

// Alternative syntax (Python-style)
from "./math.liva" import { add, multiply }
```

### Export Rules (Implicit)

```liva
// ✅ Public (exported) - no _ prefix
fn publicFunction() { }
class PublicClass { }
const PUBLIC_CONSTANT = 42

// ❌ Private (not exported) - _ prefix
fn _privateFunction() { }
class _InternalHelper { }
const _INTERNAL_CONSTANT = 42
```

### File Structure

```
project/
├── main.liva           # Entry point
├── utils.liva          # Utilities module
├── models/
│   ├── user.liva       # User model
│   └── product.liva    # Product model
└── services/
    └── api.liva        # API service
```

### Import Examples

```liva
// main.liva
import { greet } from "./utils.liva"
import { User } from "./models/user.liva"
import { Product } from "./models/product.liva"
import * as api from "./services/api.liva"

fn main() {
    greet("Alice")
    
    let user = User("Alice", 25)
    let product = Product("Widget", 9.99)
    
    api.fetchData()
}
```

---

## 🔄 Module Resolution

### Relative Paths
```liva
import { X } from "./file.liva"        // Same directory
import { Y } from "./sub/file.liva"    // Subdirectory
import { Z } from "../file.liva"       // Parent directory
import { W } from "../../file.liva"    // Up two levels
```

### Absolute Paths (from project root)
```liva
import { X } from "/src/utils.liva"    // From project root
```

### Standard Library (future)
```liva
import { Http } from "@liva/http"      // Standard library
import { Json } from "@liva/json"      // Standard library
```

---

## 🛠️ Implementation Strategy

### Phase 1: Basic Imports (Week 1)
- [x] Parse `import { name } from "path"` syntax
- [x] Parse wildcard imports `import * as name from "path"`
- [x] Resolve relative file paths
- [x] Load and parse imported files
- [x] Build module dependency graph
- [x] Detect circular dependencies

### Phase 2: Symbol Resolution (Week 1)
- [x] Track exported symbols (non-`_` prefixed)
- [x] Validate imported symbols exist
- [x] Handle namespace imports (`moduleName.function()`)
- [x] Check for naming conflicts

### Phase 3: Code Generation (Week 2)
- [x] Generate Rust `mod` declarations
- [x] Generate `use` statements for imports
- [x] Add `pub` to exported items
- [x] Handle multi-file Cargo project structure

### Phase 4: Testing & Docs (Week 2)
- [x] Comprehensive test suite
- [x] Multi-file example projects
- [x] Documentation and guides

---

## 🤔 Questions for Discussion

1. **Import syntax preference:**
   - JavaScript-style: `import { X } from "./file.liva"` ✅ (recommended)
   - Python-style: `from "./file.liva" import { X }`
   - Both supported?

2. **Visibility:**
   - `_` prefix for private (implicit) ✅ (recommended)
   - `export` keyword for public (explicit)
   - `pub` keyword (Rust-style)

3. **Wildcard imports:**
   - Allow `import * as name from "path"` ✅ (recommended)
   - Disallow (only named imports)

4. **File extensions:**
   - Always require `.liva` extension ✅ (recommended)
   - Allow omitting extension: `import { X } from "./file"`

5. **Re-exports:**
   - Support `export { X } from "./other.liva"` for re-exporting?
   - Keep it simple (no re-exports initially)

6. **Default exports:**
   - Support `export default X` (like JS)?
   - No default exports (only named) ✅ (recommended)

---

## 📊 Comparison Matrix

| Feature | Option 1 (JS) | Option 2 (Python) | Option 3 (Rust) | Option 4 (Hybrid) ✅ |
|---------|---------------|-------------------|-----------------|---------------------|
| Public by default | ❌ | ✅ | ❌ | ✅ |
| Uses `_` convention | ❌ | ✅ | ❌ | ✅ |
| Explicit keywords | `export` | None | `pub` | None |
| Familiar syntax | ✅ | ✅ | ❌ | ✅ |
| Secure by default | ✅ | ❌ | ✅ | ❌ |
| Beginner-friendly | ✅ | ✅ | ❌ | ✅ |
| Consistent with Liva | ❌ | ⚠️ | ❌ | ✅ |

---

## 🎯 Next Steps

1. **Review and approve** this design document
2. **Create detailed spec** for parser changes
3. **Update AST** with `ImportDecl` nodes
4. **Implement parser** for import statements
5. **Build module resolver** for file loading
6. **Update semantic analysis** for cross-file checking
7. **Generate multi-file Rust projects**
8. **Write comprehensive tests**
9. **Document everything**

---

**What do you think? Should we go with Option 4 (Hybrid), or do you prefer a different approach?** 🤔
