# 🎨 Module System - Syntax Quick Reference

## 📋 4 Options Side-by-Side

### Option 1: JavaScript-style (Explicit `export`)

```liva
// ===== math.liva =====
export fn add(a, b) { return a + b }
export class Calculator { }
fn _private() { }  // private (not exported)

// ===== main.liva =====
import { add, Calculator } from "./math.liva"
```

✅ Explicit, familiar to JS/TS devs  
❌ Requires `export` keyword everywhere

---

### Option 2: Python-style (Implicit, `_` = private)

```liva
// ===== math.liva =====
fn add(a, b) { return a + b }        // public (exported)
class Calculator { }                 // public
fn _private() { }                    // private (not exported)

// ===== main.liva =====
from "./math.liva" import { add, Calculator }
```

✅ Less typing, uses `_` convention  
❌ Everything public by default

---

### Option 3: Rust-style (Explicit `pub`)

```liva
// ===== math.liva =====
pub fn add(a, b) { return a + b }    // public
pub class Calculator { }             // public
fn private() { }                     // private (default)

// ===== main.liva =====
use "./math.liva"::{add, Calculator}
```

✅ Private by default (secure), very explicit  
❌ Different keywords (`pub`, `use`)

---

### Option 4: Hybrid ⭐ (Recommended) ✅ SELECTED

```liva
// ===== math.liva =====
add(a, b) { return a + b }           // public (no _ prefix)
class Calculator { }                 // public
_private() { }                       // private (_ prefix)

// ===== main.liva =====
import { add, Calculator } from "./math.liva"
// or:
import * as math from "./math.liva"
math.add(5, 3)
```

✅ **Uses existing `_` convention from Liva**  
✅ Familiar `import` syntax (like JS/TS)  
✅ No new keywords needed  
✅ Simple and beginner-friendly  
❌ Public by default (need discipline)

---

## 🎯 Key Features (Option 4 - Hybrid)

### Import Styles

```liva
// Named imports
import { add, multiply } from "./math.liva"

// Wildcard (namespace)
import * as math from "./math.liva"

// Mixed
import { User, Product } from "./models.liva"

// Python alternative (both work)
from "./math.liva" import { add }
```

### Visibility Rules (Simple!)

```liva
// ✅ PUBLIC (exported) - no underscore
publicFunction() { }
class PublicClass { }
const PUBLIC = 42

// ❌ PRIVATE (not exported) - has underscore
_privateFunction() { }
class _InternalHelper { }
const _PRIVATE = 42
```

### Path Resolution

```liva
import { X } from "./file.liva"       // Same dir
import { Y } from "./sub/file.liva"   // Subdir
import { Z } from "../file.liva"      // Parent
import { W } from "/src/file.liva"    // From root (future)
```

---

## 🤔 Quick Questions

**1. Import syntax?**
- A) `import { X } from "./file.liva"` ⭐ (JS-style, recommended)
- B) `from "./file.liva" import { X }` (Python-style)
- C) Support both A and B

**2. Visibility?**
- A) `_` prefix = private ⭐ (uses existing convention)
- B) `export` keyword = public (explicit)
- C) `pub` keyword = public (Rust-style)

**3. Wildcard imports?**
- A) Yes, allow `import * as name from "path"` ⭐ (recommended)
- B) No, only named imports

**4. File extension?**
- A) Always require `.liva` ⭐ (recommended, explicit)
- B) Allow omitting: `import { X } from "./file"`

**5. Re-exports?**
- A) Support later (Phase 4)
- B) Not needed ⭐ (keep simple initially)

**6. Default exports?**
- A) No default exports ⭐ (only named, simpler)
- B) Support `export default X` (like JS)

---

## 📊 My Recommendation: **Option 4 (Hybrid)** ⭐

**Why?**

1. ✅ **Consistent:** Uses existing `_` prefix convention (like fields/methods)
2. ✅ **Familiar:** `import` syntax known from JS/TS
3. ✅ **Simple:** No new keywords (`export`, `pub`, etc.)
4. ✅ **Flexible:** Named + wildcard imports
5. ✅ **Beginner-friendly:** Public by default, easy to learn

**Example Project:**

```
calculator/
├── main.liva
├── math.liva
├── operations/
│   ├── basic.liva
│   └── advanced.liva
└── utils/
    └── helpers.liva
```

```liva
// main.liva
import { add, multiply } from "./math.liva"
import * as ops from "./operations/basic.liva"
import { formatNumber } from "./utils/helpers.liva"

fn main() {
    let result = add(5, 3)
    print($"Result: {formatNumber(result)}")
}
```

---

## 💬 Your Turn!

**¿Qué opción prefieres?**

- Option 1: JavaScript-style (`export` keyword)
- Option 2: Python-style (implicit exports)
- Option 3: Rust-style (`pub` keyword)
- **Option 4: Hybrid** ⭐ (my recommendation)

**¿Algún cambio que quieras hacer?**

- Sintaxis de import diferente?
- Reglas de visibilidad diferentes?
- Otras características?

**Dime qué piensas y empezamos la implementación!** 🚀
