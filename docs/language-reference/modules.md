# Module System (v0.8.0)

> **Status:** 🚧 In Development (Phase 3.3 Complete)  
> **Available:** feature/modules-v0.8.0 branch

Liva's module system allows you to organize your code across multiple files, making it easier to build larger applications with reusable components.

## Overview

The module system in Liva is designed to be:

- **Simple**: JavaScript-style import syntax that's easy to learn
- **Consistent**: Public by default, private with `_` prefix (like Liva functions)
- **Safe**: Circular dependency detection and clear error messages
- **Efficient**: Modules are loaded once and cached

## Basic Usage

### Creating a Module

Any `.liva` file is a module. Functions, classes, and constants are automatically exported if they don't start with `_`:

**math.liva:**
```liva
// Public function - automatically exported
add(a, b) {
    ret a + b
}

subtract(a, b) {
    ret a - b
}

// Private function - not exported (starts with _)
_internal_helper(x) {
    ret x * 2
}
```

### Importing from a Module

Use the `import` statement to bring functions from another module into scope:

**main.liva:**
```liva
import { add, subtract } from "./math.liva"

main() {
    let sum = add(10, 20)
    let diff = subtract(30, 10)
    
    print($"Sum: {sum}")      // Output: Sum: 30
    print($"Diff: {diff}")    // Output: Diff: 20
}
```

## Import Syntax

### Named Imports

Import specific functions, classes, or constants:

```liva
// Single import
import { add } from "./math.liva"

// Multiple imports
import { add, subtract, multiply } from "./math.liva"

// Trailing comma is allowed
import { 
    add, 
    subtract, 
    multiply,
} from "./math.liva"
```

### Wildcard Imports

Import all public symbols as a namespace:

```liva
import * as math from "./math.liva"

main() {
    let result = math.add(5, 10)
    print($"Result: {result}")
}
```

### Multiple Import Statements

You can import from multiple files:

```liva
import { add, subtract } from "./math.liva"
import { log, error } from "./logger.liva"
import * as utils from "./utils.liva"

main() {
    let sum = add(10, 20)
    log($"Result: {sum}")
}
```

## Path Resolution

### Relative Paths

Liva supports relative paths using `./` and `../`:

```liva
// Same directory
import { add } from "./math.liva"

// Subdirectory
import { helper } from "./utils/helper.liva"

// Parent directory
import { config } from "../config.liva"

// Two levels up
import { constants } from "../../constants.liva"
```

### Path Rules

- Paths must end with `.liva`
- Paths are resolved relative to the importing file
- Use `/` as path separator (works on all platforms)

## Visibility Rules

### Public by Default

Functions, classes, and constants are **public** (exported) by default:

```liva
// ✅ Public - will be exported
add(a, b) { ret a + b }

// ✅ Public - will be exported
Person {
    constructor(name) {
        this.name = name
    }
    name: string
}

// ✅ Public - will be exported
const PI = 3.14159
```

### Private with `_` Prefix

Prefix with `_` to make a symbol private (not exported):

```liva
// ❌ Private - NOT exported
_helper(x) { ret x * 2 }

// ❌ Private - NOT exported
_InternalConfig {
    value: number
}

// ❌ Private - NOT exported
const _SECRET = "hidden"
```

### Why Public by Default?

This design choice makes Liva:

- **Beginner-friendly**: No need to learn export syntax first
- **Concise**: Less boilerplate in most cases
- **Consistent**: Matches Liva's existing `_` prefix convention for private members

## Example: Calculator Project

Here's a complete example of a multi-file project:

**Project Structure:**
```
calculator/
├── main.liva
├── math.liva
└── operations.liva
```

**math.liva** - Basic math operations:
```liva
add(a, b) {
    ret a + b
}

subtract(a, b) {
    ret a - b
}

// Private helper
_validate(n) {
    ret n > 0
}
```

**operations.liva** - Advanced operations:
```liva
import { add } from "./math.liva"

multiply(a, b) {
    ret a * b
}

square(x) {
    ret multiply(x, x)
}

// Uses imported function
increment(x) {
    ret add(x, 1)
}
```

**main.liva** - Entry point:
```liva
import { add, subtract } from "./math.liva"
import { multiply, square } from "./operations.liva"

main() {
    let a = 10
    let b = 5
    
    print($"Add: {add(a, b)}")           // 15
    print($"Subtract: {subtract(a, b)}") // 5
    print($"Multiply: {multiply(a, b)}") // 50
    print($"Square: {square(a)}")        // 100
}
```

**Compile and run:**
```bash
livac main.liva --run
```

## Error Handling

### Circular Dependencies

Liva detects circular dependencies and reports them clearly:

**error.liva:**
```liva
import { bar } from "./b.liva"

foo() {
    bar()
}
```

**b.liva:**
```liva
import { foo } from "./a.liva"

bar() {
    foo()  // ❌ Circular dependency!
}
```

**Error message:**
```
● E4003: Circular dependency detected
────────────────────────────────────────────────────────────
  Import chain:
  → /path/to/a.liva
  → /path/to/b.liva
  → /path/to/a.liva

  ⓘ Circular dependencies are not allowed. 
    Consider restructuring your code to break the cycle.
────────────────────────────────────────────────────────────
```

### File Not Found

Clear error when an imported file doesn't exist:

```liva
import { foo } from "./missing.liva"  // ❌ File doesn't exist
```

**Error message:**
```
● E4004: Cannot find module
────────────────────────────────────────────────────────────
  Cannot find module: ./missing.liva

  ⓘ Check that:
    - The file exists
    - The path is correct
    - The file ends with .liva
────────────────────────────────────────────────────────────
```

## Current Limitations (v0.8.0-dev)

The module system is under active development. Current limitations:

- ❌ **Symbol validation not yet implemented** - Imported symbols aren't validated yet
- ❌ **Multi-file code generation pending** - Currently only entry point is compiled
- ❌ **No absolute imports** - Only relative paths supported
- ❌ **No package manager** - Can't import from external packages yet

These features are planned for v0.8.0 final release and v0.9.0.

## Implementation Status

### ✅ Phase 3.1: Design (Complete)
- Module system specification
- Syntax design and comparison
- Implementation roadmap

### ✅ Phase 3.2: Parser & AST (Complete)
- Import statement parsing
- AST structures for imports
- Lexer support for `from` keyword

### ✅ Phase 3.3: Module Resolver (Complete)
- Module loading from files
- Dependency graph construction
- Circular dependency detection
- Path resolution
- Symbol extraction (public/private)

### ⏳ Phase 3.4: Semantic Analysis (In Progress)
- Validate imported symbols exist
- Check symbol visibility
- Detect name collisions
- Import-aware scope resolution

### 📋 Phase 3.5: Code Generation (Planned)
- Multi-file Rust project generation
- Module structure generation
- Import/export translation

## Best Practices

### 1. Organize by Feature

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

### 2. Keep Modules Focused

Each module should have a single, clear purpose:

```liva
// ✅ Good - focused on math operations
// math.liva
add(a, b) { ret a + b }
subtract(a, b) { ret a - b }

// ❌ Bad - mixing unrelated functions
// stuff.liva
add(a, b) { ret a + b }
logMessage(msg) { print(msg) }
validateEmail(email) { ret true }
```

### 3. Use Descriptive Names

```liva
// ✅ Good
import { calculateTotal } from "./billing.liva"
import { validateUser } from "./auth.liva"

// ❌ Bad
import { calc } from "./b.liva"
import { check } from "./a.liva"
```

### 4. Minimize Dependencies

Keep dependency chains shallow:

```
// ✅ Good - 2 levels
main.liva → math.liva → constants.liva

// ⚠️  Avoid - 5 levels
main.liva → a.liva → b.liva → c.liva → d.liva → e.liva
```

## Next Steps

To use the module system today:

1. **Checkout the feature branch:**
   ```bash
   git checkout feature/modules-v0.8.0
   ```

2. **Build the compiler:**
   ```bash
   cargo build --release
   ```

3. **Try the examples:**
   ```bash
   ./target/release/livac examples/modules/main.liva
   ```

4. **Report issues:**
   - Found a bug? Open an issue on GitHub
   - Have a suggestion? Let us know!

## See Also

- [Module System Specification](../design/MODULE_SYSTEM_SPEC.md) - Complete technical specification
- [Module Syntax Comparison](../design/MODULE_SYNTAX_COMPARISON.md) - Design alternatives
- [TODO: Modules](../../TODO_MODULES.md) - Implementation progress
- [Functions](functions.md) - Function syntax and visibility
- [Classes](classes.md) - Class definitions and methods
