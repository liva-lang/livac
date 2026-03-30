# Bugs & Language Issues

> Found during self-hosting v3 (livac-liva)
> Started: 2026-03-27

---

### BUG-001: Variables inside `test()` blocks not mutable
- **Status:** ✅ FIXED (v1.5.0+)
- **Fix:** Added `collect_mutated_vars_in_block` pre-analysis to `generate_test`, `generate_test_case`, and `generate_test_lifecycle` — same as `generate_function` already does.
- **Detail:** `mutated_vars` set was never populated for test blocks, so all variables got `let x` instead of `let mut x`.

### BUG-002: Mutation detection misses `self._field` inside switch/case blocks
- **Status:** ✅ FIXED (v1.5.0+)
- **Fix:** Added `Stmt::Switch`, `Stmt::TryCatch`, `Stmt::Block` cases to `stmt_modifies_self()` and `Expr::Switch` to `expr_modifies_self()`. The `_ => false` wildcard was silently ignoring mutations inside match arms.
- **Detail:** `method_modifies_self()` scanner didn't recurse into switch/case/try-catch blocks.

### BUG-003: Field access on enum variant payload uses `get_field()` instead of direct access
- **Status:** ✅ FIXED (v1.5.0+)
- **Fix:** Pre-populate `class_fields` from all modules in `generate_module_code` and `generate_entry_point`. Also relax `register_pattern_bindings` gate to accept enum types.
- **Detail:** `class_fields` was only populated for the local module in `generate_program()`, so imported class types weren't recognized. Pattern-bound variables from enum matching fell through to the JSON `get_field()` path.

### BUG-004: Classes with explicit `constructor()` don't get Display impl
- **Status:** OPEN
- **Impact:** Cross-module Display errors when types without Display are used as fields
- **Detail:** Data classes (no constructor) get auto-generated Display. Classes with explicit `constructor()` do NOT get Display. Then other modules that reference them in Display impls fail.
- **Workaround:** Remove explicit constructors; use data class pattern

### NOTE-001: No `!` unwrap operator for optionals
- **Status:** LANGUAGE GAP (not a bug)
- **Impact:** Cannot write `optionalVar!` to unwrap. Must use `or <default>` or restructure code.
- **Workaround:** Use `or <value>` / `or fail` or restructure with switch/if

### NOTE-002: No optional chaining operator `?.`
- **Status:** FEATURE REQUEST
- **Impact:** Cannot write `user?.profile?.name` for safe navigation through nullable chains.
- **Expected:** `obj?.field` → returns `null` if `obj` is null, otherwise `obj.field`
- **Workaround:** Nested `if x != null` checks or `or <default>` at each step
- **Rust target:** Could generate `.as_ref().map(|x| x.field)` chains

### BUG-005: `for ch in s` doesn't work when `s: string`
- **Status:** OPEN
- **Impact:** String iteration inside class methods generates `for ch in s.clone()` but `String` is not iterable in Rust
- **Workaround:** Use `while i < s.length { let ch = s.charAt(i) ... }`
- **Expected:** The compiler should generate `for ch in s.chars()` for string iteration

### BUG-006: `or <value>` doesn't unwrap `Option<T>` fields
- **Status:** OPEN (LANGUAGE LIMITATION)
- **Impact:** `x or fallback` where x is `T?` class field generates `x || fallback` (boolean OR) instead of `x.unwrap_or_else(|| fallback)`
- **Detail:** `or` only works in `let x = fallible_fn() or default` context, not for general `Option<T>` unwrapping
- **Workaround:** Use sentinel values (TypeRef.None, Expr.None) instead of optional types

### BUG-007: No type narrowing after null checks
- **Status:** OPEN (LANGUAGE LIMITATION)
- **Impact:** After `if x != null { ... }`, x is still `Option<T>` in the generated Rust, not `T`
- **Workaround:** Use sentinel approach — avoid optional types entirely

### BUG-008: switch/case on optional types doesn't wrap patterns in `Some()`
- **Status:** OPEN
- **Impact:** `switch t` where `t: TypeRef?` generates `match t { TypeRef::Named { name } => ... }` instead of `match t { Some(TypeRef::Named { name }) => ... }`
- **Workaround:** Don't use optional types with switch

### BUG-009: Struct fields used in multiple for-loops cause move errors
- **Status:** OPEN
- **Impact:** `for f in cls.fields { ... }; for f in cls.fields { ... }` — second loop gets "use of moved value"
- **Workaround:** Collect all data in a single pass through the fields

### BUG-010: Enum variant constructor in codegen generates snake_case
- **Status:** ✅ FIXED (codegen.liva)
- **Impact:** `Color.Red` generates `color.red` instead of `Color::Red`
- **Detail:** The self-hosting codegen applies `_rustName()` to enum variant access, which snake-cases it
- **Fix:** Added `_isPascalCase(s)` helper. `Expr.Member` uses `::` for PascalCase objects. `_genCall` preserves PascalCase for constructors. `_rustName` skips snake_case for PascalCase names.

### BUG-011: `idx = idx + 1` generates `.push_str()` in multi-module codegen
- **Status:** OPEN (compiler bug)
- **Impact:** In class methods compiled from multi-module projects, `n = n + 1` where n is a number generates `n.push_str(&1.to_string())` instead of `n = n + 1`
- **Detail:** The compiler's type inference for `+` operator incorrectly chooses string concatenation over numeric addition in certain class method contexts. Only affects multi-file builds via `import`.
- **Workaround:** Use `for ... in` loops instead of `while idx < arr.length { idx = idx + 1 }`. Avoid mixing numeric counters with string operations in the same scope.

### BUG-012: Quotes inside `//` comments break the lexer
- **Status:** OPEN (compiler bug)
- **Impact:** A comment like `// example: ["foo"]` causes E1000 lexer error
- **Detail:** The lexer appears to parse `"..."` inside `//` comments as string literals, corrupting lexer state. Comments with double quotes cause subsequent code to fail parsing.
- **Workaround:** Avoid using `"` characters in `//` comments.

### BUG-013: Struct field access from imported types generates `.get_field()`
- **Status:** OPEN (compiler bug, pre-existing)
- **Impact:** When iterating over struct fields from an imported module, `field.name` generates `field.get_field("name")` instead of `field.name`
- **Detail:** Multi-module compilation loses type information for imported struct types. The compiler falls back to dynamic `get_field()` access instead of direct field access. Affects `for field in variant.fields { field.name }` patterns.
- **Workaround:** Works in test runner context (doesn't prevent test execution), but fails in standalone `build`. Only affects certain struct access patterns in multi-module builds.

### BUG-014: Integer comparison generates `.as_str()` in multi-module codegen
- **Status:** OPEN (compiler bug)
- **Impact:** `while idx < arr.length` generates `idx.as_str() < (arr.len() as i32)` — calls `.as_str()` on an integer
- **Detail:** Same root cause as BUG-011. The compiler's type inference in multi-module class methods sometimes types integer variables as strings, producing `.as_str()` on integers for comparisons and `.push_str()` for arithmetic.
- **Workaround:** Use `for item in arr` loops instead of index-based while loops.

### BUG-015: Parameters used multiple times generate `borrow of moved value` (E0382)
- **Status:** OPEN (compiler bug)
- **Impact:** A function parameter used more than once in the body (e.g., `name` used in two different expressions) compiles to Rust that moves the value on first use, causing E0382 on subsequent uses
- **Detail:** The Rust codegen doesn't insert `.clone()` for multi-use string/struct parameters in all necessary places. Triggers when a param is passed to a function AND used in another expression within the same scope.
- **Workaround:** Manually assign param to a local variable (`let x = param`) for each additional use.

---

## Summary

| Bug | Description | Status | Scope |
|-----|-------------|--------|-------|
| BUG-001 | `_prefix` visibility in multi-module | ✅ FIXED | Self-hosting |
| BUG-002 | Missing ref patterns in array comprehension | ✅ FIXED | Self-hosting |
| BUG-003 | Codegen refactored to one-liner `add` functions | ✅ FIXED | Self-hosting |
| BUG-004 | Class method `this.` prefix missing in generated Rust | ✅ FIXED | Self-hosting |
| BUG-005 | `for i, char in s.chars` generates wrong variable references | ✅ FIXED | Self-hosting |
| BUG-006 | Enum variant struct constructor generates tuple syntax | ✅ FIXED | Self-hosting |
| BUG-007 | Named-field enum pattern in switch generates tuple pattern | ✅ FIXED | Self-hosting |
| BUG-008 | `for i, item in array` doesn't desugar to `.iter().enumerate()` | ✅ FIXED | Self-hosting |
| BUG-009 | Multi-line methods in enum break parser | ✅ FIXED | Self-hosting |
| BUG-010 | Enum variant constructor generates snake_case | ✅ FIXED | Self-hosting |
| BUG-011 | `idx + 1` generates `.push_str()` in multi-module | OPEN | Compiler |
| BUG-012 | Quotes inside `//` comments break lexer | OPEN | Compiler |
| BUG-013 | Imported struct field → `.get_field()` | OPEN | Compiler |
| BUG-014 | Integer comparison → `.as_str()` | OPEN | Compiler |
| BUG-015 | Multi-use params → `borrow of moved value` | OPEN | Compiler |

**Blocking:** BUG-011 through BUG-015 block enum variant construction in self-hosting codegen (Shape::Circle { radius: 3.14 }).
