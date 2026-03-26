# Module System Best Practices

## Project Structure

### Small Projects (1-3 modules)

```
my_project/
├── main.liva        # Entry point
├── utils.liva       # Utility functions
└── config.liva      # Configuration
```

### Medium Projects (4-10 modules)

Group by feature:

```
calculator/
├── calculator.liva       # Entry point
├── basic.liva           # Basic operations
├── advanced.liva        # Advanced operations
└── formatting.liva      # Output formatting
```

### Large Projects (nested modules)

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

## Import Patterns

**Named imports (preferred)** — import only what you need:

```liva
import { add, subtract } from "./math.liva"
import { format, parse } from "./string_utils.liva"
```

Always use relative paths. Group: stdlib first, then project modules.

## File Organization

- One feature per module
- 5–15 exports per module
- Public functions: no prefix. Private: `_` prefix
- Module files: snake_case (`user_service.liva`)

## Anti-Patterns

- **Circular dependencies** — extract shared code to a third module
- **God modules** — split `helpers.liva` into `math_helpers.liva`, `string_helpers.liva`, etc.
- **Importing everything** — import only what you use
- **Mixed responsibilities** — separate models, logic, and I/O into different files

## Module Structure

```liva
// math.liva

// === Public API ===
add(a: number, b: number): number => a + b
subtract(a: number, b: number): number => a - b

// === Private Helpers ===
_validateNonZero(x: number): bool => x != 0
```
