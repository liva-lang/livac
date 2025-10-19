# ✅ TODO - Phase 1: Consolidation & Quality (v0.6.1)

> **Goal:** Production-ready v0.6 with zero warnings and 100% test coverage  
> **Branch:** `fix/consolidation-v0.6.1`  
> **Started:** 2025-10-19

---

## 📋 Task Checklist

### 1. Fix Compiler Warnings (~30 min)

#### 1.1 Run Cargo Fix
- [ ] `cd livac`
- [ ] `cargo fix --lib -p livac --allow-dirty`
- [ ] Review and commit changes

#### 1.2 Clean Unused Imports
- [ ] Fix `src/semantic.rs` - Remove `colored::Colorize`
- [ ] Fix `src/liva_rt.rs` - Remove `JoinHandle`
- [ ] Search for other unused imports: `grep "unused import" target/warnings.txt`

#### 1.3 Fix Unreachable Code
- [ ] Fix `src/codegen.rs` line 4608-4610
  ```rust
  // Current (line 4608):
  return generate_with_ast(program, ctx);
  
  let ir_gen = IrCodeGenerator::new(&ctx);  // ← Unreachable!
  ```
- [ ] Remove or move unreachable code

#### 1.4 Fix Unused Variables
- [ ] `src/codegen.rs` - `has_methods`, `async_kw`, `type_params`, `class_name`, `key`, `condition`, `class`
- [ ] Prefix with `_` if intentionally unused: `let _unused_var = ...`
- [ ] Or remove if truly unnecessary

#### 1.5 Verify Zero Warnings
- [ ] Run `cargo build 2>&1 | grep warning`
- [ ] Confirm output is empty
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Fix any clippy warnings

**Success Criteria:** `cargo build` produces 0 warnings ✅

---

### 2. Fix Failing Test (~15 min)

#### 2.1 Identify Failure
- [ ] Run `cargo test --test codegen_ir_tests ir_codegen_string_templates`
- [ ] Note the exact error/diff

#### 2.2 Review Snapshot
- [ ] Open `tests/snapshots/codegen_ir_tests__ir_string_templates.snap`
- [ ] Compare with actual output
- [ ] Determine if snapshot is outdated or code is wrong

#### 2.3 Fix or Accept
- [ ] **If snapshot is outdated:** `cargo insta accept`
- [ ] **If code is wrong:** Fix code generation logic
- [ ] Re-run test to confirm pass

#### 2.4 Verify
- [ ] Run `cargo test --test codegen_ir_tests`
- [ ] Confirm all codegen IR tests pass

**Success Criteria:** `cargo test` shows 0 failures ✅

---

### 3. Restore Semantic Unit Tests (~1 hour)

#### 3.1 Review Commented Tests
- [ ] Open `src/semantic.rs`
- [ ] Find commented `#[cfg(test)]` block (around line 1766)
- [ ] Review what tests were commented out

#### 3.2 Update Test Signatures
All tests need to create `SemanticAnalyzer` with:
```rust
SemanticAnalyzer::new(source_file: String, source_code: String)
```

- [ ] Update `test_expr_contains_async_variants`
  ```rust
  let mut analyzer = SemanticAnalyzer::new(
      "test.liva".to_string(),
      "test code".to_string()
  );
  ```

- [ ] Update `test_is_builtin_type_matches`
  ```rust
  let analyzer = SemanticAnalyzer::new(
      "test.liva".to_string(),
      "".to_string()
  );
  ```

- [ ] Update any other tests using `SemanticAnalyzer::new()`

#### 3.3 Fix VarDecl Tests
VarDecl struct changed - now has `bindings` instead of `name`:
```rust
// Old:
ast::VarDecl {
    name: "v".into(),
    type_ref: None,
}

// New:
ast::VarDecl {
    bindings: vec![...],
    is_fallible: false,
}
```

- [ ] Update VarDecl test fixtures

#### 3.4 Uncomment Tests
- [ ] Uncomment `#[cfg(test)]`
- [ ] Uncomment `mod tests {`
- [ ] Uncomment all test functions
- [ ] Uncomment closing `}`

#### 3.5 Run Tests
- [ ] `cargo test --lib semantic`
- [ ] Fix any remaining compilation errors
- [ ] Ensure all tests pass

**Success Criteria:** All semantic unit tests pass ✅

---

### 4. Audit Inheritance Usage (~30 min)

#### 4.1 Search for Inheritance Patterns
- [ ] `grep -r "Class.*:.*Class" tests/`
- [ ] `grep -r ":" tests/ | grep -v "interface"`
- [ ] Note any files with inheritance (e.g., `Empleado : Persona`)

#### 4.2 Review Each Instance
For each match:
- [ ] Is it an interface implementation? ✅ Keep
- [ ] Is it class inheritance? ❌ Remove/rewrite

#### 4.3 Fix Inheritance Examples
Example from `tests/semantics/ok_visibility.liva`:
```liva
// ❌ BAD (inheritance):
Empleado : Persona {
  salario: number
}

// ✅ GOOD (composition):
Empleado {
  persona: Persona
  salario: number
}

// ✅ GOOD (interface):
Empleado : Worker {  // Worker is an interface
  salario: number
  work() => ...
}
```

- [ ] Rewrite inheritance examples
- [ ] Update associated tests
- [ ] Update snapshots if needed

#### 4.4 Check Documentation
- [ ] `grep -r "inheritance" docs/`
- [ ] Ensure no docs mention class inheritance
- [ ] Confirm interfaces are clearly distinguished

**Success Criteria:** Zero class inheritance patterns in codebase ✅

---

### 5. Update CHANGELOG (~15 min)

#### 5.1 Create CHANGELOG.md (if doesn't exist)
- [ ] Create `CHANGELOG.md` in root
- [ ] Follow [Keep a Changelog](https://keepachangelog.com/) format

#### 5.2 Document v0.6.1
```markdown
## [0.6.1] - 2025-10-19

### Fixed
- Removed 26 compiler warnings
- Fixed unreachable code in codegen.rs
- Restored semantic.rs unit tests
- Fixed failing ir_codegen_string_templates test
- Removed all class inheritance examples

### Changed
- All tests now pass (110+ integration tests)
- Zero compiler warnings
- Improved code quality
```

#### 5.3 Document v0.6.0 Breaking Changes
```markdown
## [0.6.0] - 2025-10-19

### BREAKING CHANGES
- **Removed `protected` visibility**
  - Single underscore `_` now means private (was protected)
  - Double underscore `__` syntax removed (no longer needed)
  - Rationale: No class inheritance = no need for protected
  
### Migration Guide
**Before (v0.5.x):**
```liva
class User {
  name: string        // public
  _email: string      // protected
  __password: string  // private
}
```

**After (v0.6.x):**
```liva
class User {
  name: string      // public
  _password: string // private (single underscore)
}
```

### Added
- Interface-based design with `:` syntax
- Real-time interface validation in VS Code
- 110+ comprehensive integration tests
- Complete documentation (23 files)

### Fixed
- 68 files updated for visibility changes
- All test snapshots updated
```

#### 5.4 Commit
- [ ] `git add CHANGELOG.md`
- [ ] `git commit -m "docs: Add CHANGELOG for v0.6.0 and v0.6.1"`

**Success Criteria:** CHANGELOG documents all changes ✅

---

### 6. Final Verification

#### 6.1 Run Full Test Suite
- [ ] `cargo test`
- [ ] Confirm: All tests pass ✅
- [ ] Confirm: 0 failures ✅

#### 6.2 Lint Check
- [ ] `cargo clippy -- -D warnings`
- [ ] Confirm: No warnings ✅

#### 6.3 Format Check
- [ ] `cargo fmt --check`
- [ ] If fails: `cargo fmt`
- [ ] Commit formatting changes

#### 6.4 Build Check
- [ ] `cargo build --release`
- [ ] Confirm: Builds successfully ✅
- [ ] Confirm: 0 warnings ✅

#### 6.5 Documentation Check
- [ ] Verify README.md is up to date
- [ ] Verify all doc links work
- [ ] Check that examples compile

#### 6.6 VSCode Extension Check
- [ ] Open VSCode with Liva extension
- [ ] Create test `.liva` file
- [ ] Verify syntax highlighting works
- [ ] Verify IntelliSense works
- [ ] Verify interface validation works

**Success Criteria:** Everything works perfectly ✅

---

## 🎯 Completion Checklist

- [ ] Task 1: Fix Compiler Warnings ✅
- [ ] Task 2: Fix Failing Test ✅
- [ ] Task 3: Restore Semantic Unit Tests ✅
- [ ] Task 4: Audit Inheritance Usage ✅
- [ ] Task 5: Update CHANGELOG ✅
- [ ] Task 6: Final Verification ✅

---

## 📝 Notes

- Keep branch `fix/consolidation-v0.6.1` up to date
- Commit frequently with clear messages
- Run tests after each major change
- Update this file as you complete tasks

---

## 🚀 After Completion

1. **Merge to main:**
   ```bash
   git checkout main
   git merge fix/consolidation-v0.6.1
   git push origin main
   ```

2. **Tag release:**
   ```bash
   git tag -a v0.6.1 -m "Release v0.6.1: Consolidation & Quality"
   git push origin v0.6.1
   ```

3. **Update roadmap:**
   - Mark Phase 1 as complete ✅
   - Move to Phase 2 (Standard Library)

4. **Celebrate! 🎉**

---

**Started:** 2025-10-19  
**Completed:** _pending_  
**Time Spent:** _pending_
