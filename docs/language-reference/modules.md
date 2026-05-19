# Module System

> Basic import syntax is in SKILL.md. This file covers path resolution, wildcard imports, visibility rules, error codes, circular dependency handling, and extensionless imports.

## Import Syntax

```liva
// Named imports
import { add, subtract } from "./math.liva"
import { add, subtract } from "./math"       // Extension optional (v2.0+)

// Wildcard import (namespace)
import * as math from "./math.liva"
math.add(5, 10)

// Multiple files
import { add } from "./math.liva"
import { log } from "./logger.liva"
import * as utils from "./utils.liva"
```

## Path Resolution Rules

- Paths resolved **relative to the importing file** (not project root)
- `.liva` extension is optional — `"./math"` and `"./math.liva"` both work
- Use `/` as separator (cross-platform)
- Only relative paths supported (`./`, `../`) — no absolute imports

```liva
import { helper } from "./utils/helper.liva"   // Subdirectory
import { config } from "../config.liva"         // Parent directory
import { constants } from "../../constants"     // Two levels up, extensionless
```

## Visibility Rules

**Public by default.** `_` prefix = private (not exported).

```liva
// ✅ Exported
add(a, b) => a + b
Person { name: string }
const PI = 3.14159

// ❌ Not exported
_helper(x) => x * 2
_InternalConfig { value: number }
const _SECRET = "hidden"
```

## Error Codes

| Code | Error | Trigger |
|------|-------|---------|
| E4003 | Circular dependency | Module A imports B, B imports A |
| E4004 | Module not found | File doesn't exist |
| E4006 | Symbol not found | Imported name doesn't exist in target module |
| E4007 | Private symbol | Attempting to import `_`-prefixed symbol |
| E4008 | Conflicts with local | Import name matches a local function/class |
| E4009 | Duplicate import | Same symbol imported from multiple modules |

## Circular Dependencies

Detected and reported with full chain:

```
● E4003: Circular dependency detected
  Import chain:
  → /path/to/a.liva
  → /path/to/b.liva
  → /path/to/a.liva

  ⓘ Restructure code to break the cycle.
```

## Module Caching

Modules are loaded once and cached — multiple imports of the same file share a single module instance.

## What Gets Exported

Any `.liva` file is a module. These symbols auto-export if not `_`-prefixed:
- Functions
- Classes / Data classes
- Constants (`const`)

## Project Structure Convention

```
project/
├── main.liva
├── user/
│   ├── user.liva
│   ├── auth.liva
│   └── profile.liva
└── utils/
    ├── math.liva
    └── string.liva
```

Keep modules focused on a single responsibility. Keep dependency chains shallow (2–3 levels).

## Splitting a Class Across Files

For large classes (hundreds of methods), Liva supports `extend ClassName { ... }`:
the owner module declares fields + constructor + core methods, and any other
module that imports the class can add more methods via `extend`. The compiler
merges them into a single `impl` at compile time.

```liva
// shapes.liva  — owner
Circle { radius: number; constructor(r: number) { this.radius = r } }

// shapes_area.liva  — extension
import { Circle } from "./shapes"
extend Circle { area(): number { return 3.14159 * this.radius * this.radius } }
```

See [class-extensions.md](./class-extensions.md) for the full reference.
