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

## When to Split a Class Across Files (`extend`)

Use `extend ClassName { ... }` when a single class grows past ~300–500 LOC
or mixes several concerns the owner module wants to keep separate. Convention:

- **Owner file** carries the canonical name (`emitter.liva`) and holds the
  fields, constructor, and a few cross-cutting methods.
- **Extension files** use the suffix pattern `<owner>_<concern>.liva`:
  - `emitter_expr.liva` — `extend Emitter { _emitExpr*() ... }`
  - `emitter_stmt.liva` — `extend Emitter { _emitStmt*() ... }`
  - `emitter_class.liva` — `extend Emitter { _emitClass*() ... }`
- Each extension file imports the owner type explicitly:
  `import { Emitter } from "./emitter"`.
- One sentinel free function per extension (`emitterExprExtensionLoaded(): bool`)
  keeps the module reachable when only the `extend` block is used (no other
  imported symbols).

The compiler merges every `extend` back into the owner `impl` at codegen time
(zero runtime overhead). Helpers defined alongside an `extend` block are
automatically reachable from the owner via a synthetic wildcard import —
you don't need to re-export them.

See [class-extensions.md](../language-reference/class-extensions.md) for the
full reference (E0910–E0913).

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
