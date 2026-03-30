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
- **Status:** OPEN (codegen.liva issue, not compiler)
- **Impact:** `Color.Red` generates `color.red` instead of `Color::Red`
- **Detail:** The self-hosting codegen applies `_rustName()` to enum variant access, which snake-cases it
- **Fix needed:** Handle `Member` access on enum types differently — preserve PascalCase
