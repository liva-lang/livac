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
- **Status:** ✅ FIXED (Rust compiler)
- **Fix:** Changed `if is_data {` to `if has_fields {` in codegen.rs — Display impl now generated for ALL classes with fields, not just data classes.
- **Impact:** Cross-module Display errors when types without Display are used as fields

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
- **Status:** ✅ FIXED (Rust compiler)
- **Fix:** Added `is_string_iterable` detection in `Stmt::For` handler. When iterable is a string var, emits `.chars()` + `let ch = ch.to_string();` inside the loop body.
- **Impact:** String iteration inside class methods generates `for ch in s.clone()` but `String` is not iterable in Rust

### BUG-006: `or <value>` doesn't unwrap `Option<T>` from user functions
- **Status:** ✅ FIXED (Rust compiler)
- **Fix:** (a) Added `in_optional_function` tracking — `return x` in `T?` functions wraps with `Some()`, `return null` emits `None`. (b) Fixed `or` handler for `Expr::Call` — generic function calls now generate `.unwrap_or()` instead of silently dropping the default.
- **Impact:** `let x = fn() or default` where fn returns `T?` now correctly unwraps the Option

### BUG-007: No type narrowing after null checks
- **Status:** ✅ FIXED (Rust compiler)
- **Fix:** Added `extract_option_null_check()` helper. `if x != null { body }` now generates `if let Some(x) = x { body }` — the variable is unwrapped inside the block. Also tracks `optional_returning_functions` to auto-detect Option variables from function calls.
- **Impact:** After `if x != null { ... }`, x is now the unwrapped type inside the block

### BUG-008: switch/case on optional types doesn't wrap patterns in `Some()`
- **Status:** ✅ FIXED (Rust compiler)
- **Fix:** Added `is_option_discriminant` detection in switch handler. String switches on Option vars use `.as_deref()` instead of `.as_str()`, case patterns wrapped in `Some()`, null cases generate `None`.
- **Impact:** `switch opt_var { case "x": ... case null: ... }` now generates correct `match opt_var.as_deref() { Some("x") => ..., None => ... }`

### BUG-009: Struct fields used in multiple for-loops cause move errors
- **Status:** NOT A BUG (`.clone()` in for-loops already handles this)
- **Detail:** Tested in Rust compiler — `.clone()` on for-loop iterables prevents move errors. Was a self-hosting codegen issue only.

### BUG-010: Enum variant constructor in codegen generates snake_case
- **Status:** ✅ FIXED (codegen.liva)
- **Impact:** `Color.Red` generates `color.red` instead of `Color::Red`
- **Detail:** The self-hosting codegen applies `_rustName()` to enum variant access, which snake-cases it
- **Fix:** Added `_isPascalCase(s)` helper. `Expr.Member` uses `::` for PascalCase objects. `_genCall` preserves PascalCase for constructors. `_rustName` skips snake_case for PascalCase names.

### BUG-011: `idx = idx + 1` generates `.push_str()` in multi-module codegen
- **Status:** NOT REPRODUCIBLE in Rust compiler
- **Detail:** Tested multi-module builds with numeric counters — no `.push_str()` generated. Was a self-hosting codegen issue only.

### BUG-012: Quotes inside `//` comments break the lexer
- **Status:** NOT A BUG in Rust compiler
- **Detail:** The logos regex `//[^\n]*` correctly handles quotes in comments. Was a self-hosting Liva lexer issue only.

### BUG-013: Struct field access from imported types generates `.get_field()`
- **Status:** NOT REPRODUCIBLE in Rust compiler
- **Detail:** Tested complex multi-module builds with imported struct types — field access generates correctly. Was a self-hosting codegen issue only.

### BUG-014: Integer comparison generates `.as_str()` in multi-module codegen
- **Status:** NOT REPRODUCIBLE in Rust compiler
- **Detail:** Tested multi-module with integer comparisons — generates correctly. Same as BUG-011, was a self-hosting codegen issue only.

### BUG-015: Parameters used multiple times generate `borrow of moved value` (E0382)
- **Status:** NOT REPRODUCIBLE in Rust compiler
- **Detail:** Tested multi-use parameters in single and multi-file modes — works fine. Was a self-hosting codegen issue only.

---

## Summary

| Bug | Description | Status | Scope |
|-----|-------------|--------|-------|
| BUG-001 | `_prefix` visibility in multi-module | ✅ FIXED | Self-hosting |
| BUG-002 | Missing ref patterns in array comprehension | ✅ FIXED | Self-hosting |
| BUG-003 | Codegen refactored to one-liner `add` functions | ✅ FIXED | Self-hosting |
| BUG-004 | Classes with constructor don't get Display impl | ✅ FIXED | Rust compiler |
| BUG-005 | `for ch in s` doesn't work for string iteration | ✅ FIXED | Rust compiler |
| BUG-006 | `or` doesn't unwrap `Option<T>` from functions | ✅ FIXED | Rust compiler |
| BUG-007 | No type narrowing after null checks | ✅ FIXED | Rust compiler |
| BUG-008 | switch/case on optional types no `Some()` wrap | ✅ FIXED | Rust compiler |
| BUG-009 | Multi-loop struct field move errors | NOT A BUG | Self-hosting only |
| BUG-010 | Enum variant constructor generates snake_case | ✅ FIXED | Self-hosting |
| BUG-011 | `idx + 1` generates `.push_str()` in multi-module | NOT REPRODUCIBLE | Self-hosting only |
| BUG-012 | Quotes inside `//` comments break lexer | NOT A BUG | Self-hosting only |
| BUG-013 | Imported struct field → `.get_field()` | NOT REPRODUCIBLE | Self-hosting only |
| BUG-014 | Integer comparison → `.as_str()` | NOT REPRODUCIBLE | Self-hosting only |
| BUG-015 | Multi-use params → `borrow of moved value` | NOT REPRODUCIBLE | Self-hosting only |

**Result:** Of 15 bugs found during self-hosting, 8 were actual Rust compiler bugs (all now FIXED), 1 was self-hosting codegen only (FIXED), and 6 were not reproducible in the Rust compiler.
