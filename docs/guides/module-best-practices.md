# Module System Best Practices

**Liva v0.8.0+**

This guide covers best practices for organizing code with Liva's module system.

---

## Table of Contents

1. [Project Structure](#project-structure)
2. [Naming Conventions](#naming-conventions)
3. [Visibility Guidelines](#visibility-guidelines)
4. [Import Patterns](#import-patterns)
5. [Common Patterns](#common-patterns)
6. [Anti-Patterns](#anti-patterns)
7. [Performance Tips](#performance-tips)

---

## Project Structure

### Small Projects (1-3 modules)

**Flat structure:**
```
my_project/
├── main.liva        # Entry point
├── utils.liva       # Utility functions
└── config.liva      # Configuration
```

**Example:**
```liva
// utils.liva
formatNumber(n: number): string => $"Number: {n}"

// config.liva
APP_NAME: string => "My App"
VERSION: string => "1.0.0"

// main.liva
import { formatNumber } from "./utils.liva"
import { APP_NAME } from "./config.liva"

main() {
    print($"{APP_NAME} - {formatNumber(42)}")
}
```

### Medium Projects (4-10 modules)

**Group by feature:**
```
calculator/
├── calculator.liva       # Entry point
├── basic.liva           # Basic operations
├── advanced.liva        # Advanced operations
└── formatting.liva      # Output formatting
```

### Future: Large Projects (nested modules)

**Note:** Nested directories are planned for future versions.

```
app/
├── main.liva
├── models/
│   ├── user.liva
│   └── session.liva
└── services/
    ├── auth.liva
    └── api.liva
```

---

## Naming Conventions

### Module Files

✅ **Good:**
- Lowercase with underscores: `user_service.liva`
- Clear and descriptive: `math_operations.liva`
- Short but meaningful: `auth.liva`, `db.liva`

❌ **Avoid:**
- Uppercase: `UserService.liva`
- Special characters: `user-service.liva`
- Generic names: `helpers.liva`, `stuff.liva`

### Function Names

✅ **Public functions** (no `_` prefix):
```liva
// Good - clear, descriptive
calculateTotal(items: array): number => ...
validateEmail(email: string): bool => ...
formatDate(date: string): string => ...
```

✅ **Private functions** (`_` prefix):
```liva
// Good - internal implementation details
_parseTokens(input: string): array => ...
_validateInput(value: number): bool => ...
_formatInternal(data: string): string => ...
```

❌ **Avoid:**
```liva
// Too generic
doStuff() => ...
process() => ...

// Unclear purpose
x() => ...
fn1() => ...
```

---

## Visibility Guidelines

### Default to Public

Only make functions private when they are truly internal:

✅ **Good:**
```liva
// Public API - no prefix
add(a: number, b: number): number => a + b
multiply(a: number, b: number): number => a * b

// Internal helper - with prefix
_validateNumbers(a: number, b: number): bool => {
    a != 0 && b != 0
}
```

### When to Use Private (`_` prefix)

Use `_` prefix for:
- Implementation details
- Helper functions
- Functions that might change frequently
- Functions with unclear/unstable APIs

✅ **Example:**
```liva
// Public stable API
processData(input: string): string {
    let cleaned = _removeWhitespace(input)
    let formatted = _applyFormat(cleaned)
    formatted
}

// Private implementation (can change without breaking users)
_removeWhitespace(s: string): string => ...
_applyFormat(s: string): string => ...
```

### Organizing Exports

Group public functions by category:

```liva
// math.liva

// === Basic Operations ===
add(a: number, b: number): number => a + b
subtract(a: number, b: number): number => a - b

// === Advanced Operations ===
power(base: number, exp: number): number => ...
sqrt(x: number): number => ...

// === Private Helpers ===
_validateNonZero(x: number): bool => x != 0
```

---

## Import Patterns

### Named Imports (Preferred)

✅ **Import only what you need:**
```liva
import { add, subtract } from "./math.liva"
```

Benefits:
- Clear dependencies
- Smaller generated code
- Easier to refactor

### Multiple Imports

✅ **Group related imports:**
```liva
// Good - organized by module
import { add, subtract, multiply, divide } from "./math.liva"
import { format, parse } from "./string_utils.liva"
```

✅ **Split if too many:**
```liva
// Better for many imports
import { add, subtract, multiply } from "./math.liva"
import { divide, modulo, power } from "./math.liva"
```

### Wildcard Imports

✅ **Use for namespace-style access:**
```liva
import * as math from "./math.liva"

main() {
    let sum = math::add(10, 5)  // Future: namespace access
}
```

**Note:** Currently, wildcard imports make the module available but don't require a use statement since `mod math;` already provides access.

### Relative Paths

✅ **Always use relative paths:**
```liva
import { helper } from "./utils.liva"      // Same directory
import { config } from "../config.liva"    // Parent directory (future)
```

---

## Common Patterns

### Barrel Files (Re-exports)

**Future feature:** Re-export symbols from multiple modules.

```liva
// operations.liva (planned)
export { add, subtract } from "./basic.liva"
export { power, sqrt } from "./advanced.liva"

// main.liva (future)
import { add, power } from "./operations.liva"
```

### Configuration Modules

✅ **Centralize configuration:**
```liva
// config.liva
APP_NAME: string => "My App"
VERSION: string => "1.0.0"
DEBUG: bool => true

// main.liva
import { APP_NAME, VERSION } from "./config.liva"
```

### Utility Modules

✅ **Group related utilities:**
```liva
// string_utils.liva
trim(s: string): string => s.trim()
capitalize(s: string): string => s.toUpperCase()

// array_utils.liva
sum(arr: array): number => arr.reduce((a, x) => a + x, 0)
max(arr: array): number => arr.reduce((a, x) => a > x ? a : x, arr[0])
```

### Feature Modules

✅ **One feature per module:**
```liva
// auth.liva
login(username: string, password: string): bool => ...
logout(): void => ...
isAuthenticated(): bool => ...

// api.liva
fetchData(url: string): string => ...
postData(url: string, data: string): bool => ...
```

---

## Anti-Patterns

### ❌ Circular Dependencies

**Don't do this:**
```liva
// a.liva
import { funcB } from "./b.liva"
funcA() => funcB()

// b.liva
import { funcA } from "./a.liva"  // ERROR: Circular dependency!
funcB() => funcA()
```

✅ **Solution:** Extract shared code to a third module:
```liva
// shared.liva
sharedLogic() => ...

// a.liva
import { sharedLogic } from "./shared.liva"

// b.liva
import { sharedLogic } from "./shared.liva"
```

### ❌ Importing Everything

**Don't do this:**
```liva
// Using only 2 functions from module with 20 exports
import { fn1, fn2, fn3, fn4, fn5, fn6, ..., fn20 } from "./utils.liva"
```

✅ **Solution:** Import only what you need:
```liva
import { fn1, fn2 } from "./utils.liva"
```

### ❌ Too Many Responsibilities

**Don't do this:**
```liva
// utils.liva - everything mixed together
calculateTax() => ...
formatString() => ...
connectDatabase() => ...
validateEmail() => ...
```

✅ **Solution:** Split by responsibility:
```liva
// tax.liva
calculateTax() => ...

// string_utils.liva
formatString() => ...

// db.liva
connectDatabase() => ...

// validation.liva
validateEmail() => ...
```

### ❌ God Modules

**Don't do this:**
```liva
// helpers.liva - 100+ random functions
helper1() => ...
helper2() => ...
... (98 more)
```

✅ **Solution:** Organize by domain:
```liva
// math_helpers.liva
// string_helpers.liva
// validation_helpers.liva
```

---

## Performance Tips

### Import Placement

✅ **Put imports at the top:**
```liva
// Good
import { add } from "./math.liva"
import { format } from "./utils.liva"

main() {
    ...
}
```

### Selective Imports

✅ **Import specific symbols:**
```liva
// Better
import { add, subtract } from "./math.liva"

// Worse (imports everything)
import * as math from "./math.liva"
```

### Module Organization

✅ **Keep modules focused:**
- Each module should have a clear, single purpose
- Aim for 5-15 exports per module
- Split large modules into smaller ones

---

## Summary

### ✅ Do's

- Use descriptive module names
- Default to public visibility
- Import only what you need
- Organize by feature/domain
- Keep modules focused
- Use relative paths
- Document public APIs

### ❌ Don'ts

- Create circular dependencies
- Use generic names (`helpers.liva`)
- Make everything private
- Create god modules
- Import everything
- Mix unrelated functionality

---

## Examples

### Good Project Structure

```
calculator/
├── calculator.liva      # Entry point with main()
├── basic.liva          # add, subtract, multiply, divide
└── advanced.liva       # power, sqrt, percentage

// calculator.liva
import { add, subtract } from "./basic.liva"
import { power } from "./advanced.liva"

main() {
    let sum = add(10, 5)
    let squared = power(sum, 2)
    print($"Result: {squared}")
}
```

### Bad Project Structure

```
bad_project/
├── main.liva
├── stuff.liva          # Too generic
├── helpers.liva        # God module
└── utils.liva          # Another god module
```

---

## See Also

- [Module System Reference](../language-reference/modules.md)
- [Import Validation](../compiler-internals/import-validation.md)
- [Multi-File Codegen](../compiler-internals/multifile-codegen.md)

---

**Last Updated:** 2024-10-21  
**Liva Version:** v0.8.0
