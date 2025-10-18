# ⚠️ Documentation Status Notice

## Important: Outdated Reference Documents

The following documents in `docs_old/` were written during **early development** (April 2025) and **may not reflect the current implementation**:

### Potentially Outdated Documents

1. **`docs_old/Liva_v0.6_spec.md`**
   - ⚠️ May be missing recent features
   - ⚠️ Some syntax may have evolved
   - ✅ Core concepts are still valid

2. **`docs_old/Liva_v0.6_EBNF_AST.md`**
   - ⚠️ AST structure may have changed
   - ⚠️ Some tokens may be missing
   - ✅ General grammar structure is accurate

3. **`docs_old/Liva_v0.6_Desugaring.md`**
   - ⚠️ Transformation rules may have evolved
   - ⚠️ IR-based codegen not fully reflected
   - ✅ Basic transformation concepts valid

## Features NOT in Old Specs (but ARE implemented)

Based on inspection of `src/lexer.rs` and `src/ast.rs`, these features are **implemented in code** but may not be documented in old specs:

### Additional Keywords (Implemented)
```rust
// In lexer, but may not be in old spec:
- move        // Move semantics
- seq         // Sequential execution
- vec         // Vector type
- simdWidth   // SIMD width policy
- prefetch    // Prefetch policy
- reduction   // Reduction policy
- schedule    // Schedule policy
- detect      // Auto-detect
- auto        // Auto mode
- safe        // Safe mode
- fast        // Fast mode
- static      // Static schedule
- dynamic     // Dynamic schedule
```

### Error Binding Features
The current implementation supports:
- ✅ `let value, err = fallibleFunc()`
- ✅ Works with async: `let value, err = async fallibleFunc()`
- ✅ Works with par: `let value, err = par fallibleFunc()`
- ✅ Error type is always `String`

This is NEWER than the old spec described!

### Concurrency Policies (Fully Implemented)
The old spec may not fully document:
```liva
// Data-parallel for with policies
for par item in items with chunk 2 threads 4 ordered {
    process(item)
}

// SIMD with policies
for parvec lane in data with simdWidth 4 ordered {
    vectorProcess(lane)
}
```

**Policy options:**
- `chunk N` - Items per worker
- `threads N` - Thread count
- `simdWidth N` - SIMD vector width
- `ordered` - Preserve order
- `unordered` - Allow reordering
- `prefetch` - Prefetch data
- `reduction` - Reduction operation
- `schedule static|dynamic|auto` - Scheduling strategy
- `detect safe|fast|auto` - Safety detection

## Source of Truth

**For accurate, up-to-date information, always refer to:**

1. **Source Code** (Authoritative):
   - `src/lexer.rs` - All tokens and keywords
   - `src/ast.rs` - AST structure
   - `src/parser.rs` - Parsing rules
   - `src/semantic.rs` - Semantic rules
   - `src/codegen.rs` - Code generation behavior

2. **New Documentation** (October 2025):
   - `docs/language-reference/` - User-facing language guide
   - `docs/compiler-internals/` - Implementation details
   - These are based on the ACTUAL implementation

3. **Test Files** (Examples of what works):
   - `tests/` - Snapshot tests showing real behavior
   - `main.liva` - Working example program
   - `test_*.liva` - Various test cases

## What to Trust

### ✅ Trust (Accurate)

**New Documentation:**
- `docs/language-reference/syntax-overview.md`
- `docs/language-reference/types.md`
- `docs/language-reference/concurrency.md`
- `docs/language-reference/error-handling.md`
- `docs/compiler-internals/architecture.md`
- `docs/compiler-internals/grammar.md`

**Source Code:**
- Everything in `src/` - This is the ground truth
- Test files showing working examples

**Working Examples:**
- `main.liva` - Comprehensive working program
- Test files in root directory

### ⚠️ Verify Before Trusting

**Old Specs:**
- `docs_old/Liva_v0.6_spec.md` - Core concepts OK, details may be outdated
- `docs_old/Liva_v0.6_EBNF_AST.md` - Grammar structure OK, may miss features
- `docs_old/Liva_v0.6_Desugaring.md` - Basic rules OK, IR not covered

**When in doubt:**
1. Check the source code in `src/`
2. Look at test files that actually compile
3. Refer to new documentation in `docs/`

## Discrepancies Found

### Example 1: Policy Keywords
**Old spec:** May not mention `prefetch`, `reduction`, `schedule`
**Reality:** All implemented in lexer.rs

### Example 2: Error Binding
**Old spec:** May not fully describe error binding with concurrency
**Reality:** Fully integrated: `let value, err = async|par func()`

### Example 3: AST Structure
**Old spec:** AST nodes may differ
**Reality:** Check `src/ast.rs` for actual structure

### Example 4: Keywords
**Old spec:** Lists ~30 keywords
**Reality:** Lexer has 50+ tokens including policy keywords

## Recommendation for Developers

**Development Workflow:**
1. **Read new docs** in `docs/` for high-level understanding
2. **Check source code** in `src/` for exact behavior
3. **Look at tests** in `tests/` for working examples
4. **Use old specs** in `docs_old/` only for historical context

**When Writing New Features:**
1. Update the source code first
2. Update new documentation in `docs/`
3. Add test cases showing the feature works
4. (Optional) Note differences from old specs

## Timeline

- **April 2025**: Old specs written (`docs_old/`)
- **May-September 2025**: Major implementation work
  * IR system added
  * Concurrency system completed
  * Error binding system refined
  * Policy system expanded
- **October 2025**: New documentation written (`docs/`)
  * Based on current implementation
  * Reflects actual behavior
  * Up-to-date with all features

## Verification Status

| Feature | Old Spec | Source Code | New Docs | Tests |
|---------|----------|-------------|----------|-------|
| Basic syntax | ✅ | ✅ | ✅ | ✅ |
| async/par | ✅ | ✅ | ✅ | ✅ |
| Error binding | ⚠️ | ✅ | ✅ | ✅ |
| Policy keywords | ❌ | ✅ | ⚠️ | ✅ |
| IR system | ❌ | ✅ | ✅ | ✅ |
| Auto-async | ✅ | ✅ | ✅ | ✅ |

Legend:
- ✅ Documented/Implemented
- ⚠️ Partially documented
- ❌ Not documented

## Action Items

### For New Users
- **Start with:** `docs/getting-started/quick-start.md`
- **Reference:** `docs/language-reference/`
- **Avoid:** Old specs until you know the current state

### For Contributors
- **Read:** Source code in `src/`
- **Update:** New docs when changing features
- **Add:** Tests showing new features work
- **Check:** Old specs for historical design decisions

### For Maintainers
- **TODO:** Verify all policy keywords are documented
- **TODO:** Complete documentation of all lexer tokens
- **TODO:** Document all AST node types
- **TODO:** Add migration guide from old spec to current

## Summary

**Trust the code and new docs, verify the old specs.**

The compiler has evolved significantly since April 2025. The old specification documents provide valuable historical context and design rationale, but should not be considered authoritative for current syntax or behavior.

**When in doubt:** `cargo run -- example.liva --verbose` tells you exactly what the compiler does!

---

**Last Updated:** October 18, 2025  
**Compiler Version:** v0.6 (current)  
**Old Specs Version:** v0.6 (April 2025 snapshot)
