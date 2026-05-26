# Linter — Static Analysis Warnings

> **Subcommand:** `livac lint <file>`
> **Version:** v1.8+
> **Purpose:** Detect code smells without blocking compilation.

---

## Usage

```bash
# Lint a file (human-readable output)
livac lint main.liva

# JSON output (for IDE integration)
livac lint main.liva --json
```

### Example output

```
warning [W001]: Unused variable
  --> main.liva:5
      5 |     let y = 10
   = Variable 'y' is declared but never used
   help: Prefix with underscore to suppress: _y

warning [W003]: Unreachable code
  --> main.liva:14
     14 |     console.log("unreachable")
   = Code after 'return' will never be executed
   help: Remove unreachable code or restructure the logic

2 warnings emitted
```

---

## Warning Codes

### W001 — Unused variable

Emitted when a local variable (`let`/`const`) or loop variable (`for`) is declared but never referenced.

```liva
main() {
    let x = 42        // W001: 'x' never used
    console.log("hi")
}
```

**Suppress** by prefixing with `_`:

```liva
main() {
    let _x = 42       // ✅ no warning
    console.log("hi")
}
```

**Not emitted for:**
- Function parameters (they are part of the public interface).
- Variables prefixed with `_`.
- The wildcard `_` in destructuring.

---

### W002 — Unused import

Emitted when an imported symbol is never used in the file.

```liva
import { add, subtract } from "./math.liva"   // W002: 'subtract' unused

main() {
    console.log(add(1, 2))
}
```

**Fix** by removing the unused import:

```liva
import { add } from "./math.liva"   // ✅
```

**Not emitted for:**
- Wildcard imports (`import * from "..."`).
- Types referenced in type annotations.

---

### W003 — Unreachable code

Emitted when statements appear after `return`, `fail`, `break`, or `continue`.

```liva
main() {
    return "done"
    console.log("never executed")   // W003
}
```

```liva
process(): string {
    fail "error"
    return "never"   // W003
}
```

**Note:** Only the first unreachable statement per block is reported. Branches inside separate `if`/`else` arms are checked independently.

---

### W004 — Comparison is always true/false

Emitted when a comparison can be evaluated statically.

#### Case 1: variable compared with itself

```liva
if x == x { ... }   // W004: always true
if x != x { ... }   // W004: always false
```

#### Case 2: distinct literals compared

```liva
if 42 == 99 { ... }   // W004: always false
if "a" != "b" { ... } // W004: always true
```

#### Case 3: equal literals compared

```liva
if true == true { ... }   // W004: always true
if 42 == 42 { ... }       // W004: always true
```

---

### W005 — Shadowed variable

Emitted when a binding (let / const / for-variable) re-uses the name of an
identifier already declared in an enclosing scope, including function
parameters.

```liva
main() {
    let x = 1
    if true {
        let x = 2        // W005: 'x' shadows outer binding
        console.log(x)
    }
}
```

Disabling: prefix the inner binding with `_`, or rename it.

---

### W006 — Empty block

Emitted when an `if`, `else`, `while`, or `for` body has no statements.
Usually a leftover from refactoring or an unfinished placeholder.

```liva
main() {
    if ready {
                         // W006: empty 'if' block
    }
}
```

Disabling: add a comment inside the block to make the intent explicit,
or remove the construct.

---

### W007 — Unused parameter

Emitted when a function or method declares a parameter whose name never
appears in the body.

```liva
greet(name, age) {       // W007: 'age' is never used
    console.log(name)
}
```

Disabling: prefix the parameter with `_` (e.g. `_age`). `self` is never
flagged. Parameters of empty-bodied stubs (interface impls) are not
treated specially — wrap with `_` if appropriate.

---

## JSON Output

For IDE integration, pass `--json`:

```bash
livac lint main.liva --json
```

```json
[
  {
    "code": "W001",
    "title": "Unused variable",
    "message": "Variable 'y' is declared but never used",
    "file": "main.liva",
    "line": 5,
    "column": null,
    "source_line": "    let y = 10",
    "help": "Prefix with underscore to suppress: _y"
  }
]
```

---

## Behavior

- **Non-blocking:** warnings are informational; `livac build`/`livac run` still succeed.
- **Exit code:** `livac lint` always returns `0` even with warnings. It returns `1` only if the file fails to parse.
- **`_` suppresses W001:** following the Rust/Liva convention — variables prefixed with `_` are intentionally ignored.
- **One warning per block for W003:** only the first unreachable statement in a block is reported.
