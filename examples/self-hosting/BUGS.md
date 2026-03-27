# Bugs & Language Issues

> Found during self-hosting v3 (livac-liva)
> Started: 2026-03-27

---

### BUG-001: Variables inside `test()` blocks not mutable
- **Status:** OPEN
- **Impact:** Compilation error when calling `&mut self` methods on variables
- **Detail:** In regular functions, `let x = ...` generates `let mut x = ...` in Rust. Inside `test()` blocks, it generates `let x = ...` (immutable). This causes Rust errors when calling methods that take `&mut self`.
- **Repro:** Any `let x = Foo(...)` followed by `x.mutatingMethod()` inside a `test()` block
- **Workaround:** Use helper functions outside test blocks for operations requiring mutable variables

### BUG-002: Mutation detection misses `self._field` inside switch/case blocks
- **Status:** OPEN
- **Impact:** Methods that modify `self._field` inside switch cases are generated as `&self` instead of `&mut self`
- **Detail:** The compiler checks for `self._field = ...` to decide `&self` vs `&mut self`, but doesn't detect mutations inside `switch/case` blocks.
- **Workaround:** Call `this._advance()` (already `&mut self`) instead of direct `this._pos = this._pos + 1`

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
