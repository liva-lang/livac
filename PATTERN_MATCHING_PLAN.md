# Phase 6.4: Exhaustive Pattern Matching - Implementation Plan

## Goal
Complete pattern matching exhaustiveness checking for all types, not just bool.

## Current Status
✅ **Already Implemented:**
- Switch expressions with 4 pattern types (literal, wildcard, binding, range)
- Pattern guards with if conditions
- Exhaustiveness checking for `bool` type (E0901)
- Basic type inference from patterns

## Tasks to Complete

### 1. Exhaustiveness for Integer Types (1 hour)
**Goal:** Detect non-exhaustive integer patterns

**Strategy:**
- For small ranges (e.g., 0-10), enumerate all values
- For large ranges, require wildcard or suggest range patterns
- Detect overlapping ranges
- Check if all values in a range are covered

**Examples to handle:**
```liva
// Non-exhaustive - missing cases
let x = switch num {
    0 => "zero",
    1 => "one"
    // Error: missing 2, 3, 4, ... or wildcard
};

// Exhaustive with ranges
let x = switch num {
    0..=10 => "small",
    11..=100 => "medium",
    _ => "large"  // Required for numbers > 100
};

// Overlapping ranges - should warn
let x = switch num {
    0..=50 => "low",
    25..=75 => "mid",  // Warning: overlaps with previous
    _ => "high"
};
```

**Implementation:**
- Add `check_int_exhaustiveness()` method
- Collect all literal and range patterns
- Build set of covered values
- For ranges > 100 values, require wildcard
- Emit E0902 error for non-exhaustive integers

---

### 2. Exhaustiveness for String Literals (30 min)
**Goal:** Warn when string patterns might not be exhaustive

**Strategy:**
- String type is infinite, can't be exhaustive without wildcard
- Always require `_` or binding pattern for strings
- Allow multiple string literals but require catch-all

**Examples:**
```liva
// Non-exhaustive - no wildcard
let x = switch status {
    "active" => 1,
    "inactive" => 2
    // Error: no wildcard for other strings
};

// Exhaustive with wildcard
let x = switch status {
    "active" => 1,
    "inactive" => 2,
    _ => 0
};
```

**Implementation:**
- Add `check_string_exhaustiveness()` method
- Check for wildcard/binding pattern
- Emit E0903 error if missing

---

### 3. Tuple/Array Destructuring Patterns (1.5 hours)
**Goal:** Support pattern matching on tuples and arrays

**New syntax:**
```liva
let result = switch pair {
    [0, 0] => "origin",
    [0, y] => "y-axis",
    [x, 0] => "x-axis",
    [x, y] => "other"  // Binding pattern - exhaustive
};

let result = switch tuple {
    (1, 2) => "one-two",
    (x, y) if x == y => "equal",
    (x, y) => "other"
};
```

**Implementation:**
- Extend `Pattern` enum with `Array` and `Tuple` variants
- Parse array/tuple patterns in parser
- Semantic analysis for nested patterns
- Code generation for destructuring in match
- Exhaustiveness checking for fixed-size collections

**AST Changes:**
```rust
pub enum Pattern {
    Literal(Literal),
    Wildcard,
    Binding(String),
    Range(Box<Literal>, Box<Literal>),
    Array(Vec<Pattern>),        // NEW
    Tuple(Vec<Pattern>),        // NEW
}
```

---

### 4. Or Patterns (30 min)
**Goal:** Allow multiple patterns in one arm with `|`

**Syntax:**
```liva
let result = switch x {
    1 | 2 | 3 => "small",
    4 | 5 | 6 => "medium",
    _ => "large"
};

let status = switch code {
    200 | 201 | 204 => "success",
    400 | 404 => "client error",
    500 | 502 | 503 => "server error",
    _ => "unknown"
};
```

**Implementation:**
- Extend `Pattern` enum with `Or(Vec<Pattern>)` variant
- Parse `|` operator in patterns
- Semantic check: all sub-patterns must bind same variables
- Code generation: expand to multiple Rust match arms
- Exhaustiveness: collect all sub-patterns

**AST Changes:**
```rust
pub enum Pattern {
    // ... existing variants
    Or(Vec<Pattern>),           // NEW: pattern1 | pattern2 | pattern3
}
```

---

### 5. Testing (30 min)
Create comprehensive test files:

- `test_exhaustive_int.liva` - Integer exhaustiveness
- `test_exhaustive_string.liva` - String patterns
- `test_array_patterns.liva` - Array destructuring
- `test_tuple_patterns.liva` - Tuple matching
- `test_or_patterns.liva` - Or patterns
- `test_overlapping_ranges.liva` - Range overlap warnings

---

### 6. Documentation (30 min)
Update documentation:

- `docs/language-reference/pattern-matching.md`
  - Add exhaustiveness section for all types
  - Document array/tuple patterns
  - Document or patterns
  - Add examples for each

- `CHANGELOG.md`
  - Add v0.10.5 entry
  - List all new features

- `ROADMAP.md`
  - Mark 6.4 as complete

---

## Error Codes

### New Error Codes:
- **E0901**: Non-exhaustive bool pattern (already exists)
- **E0902**: Non-exhaustive integer pattern (NEW)
- **E0903**: Non-exhaustive string pattern - requires wildcard (NEW)
- **E0904**: Non-exhaustive array/tuple pattern (NEW)
- **E0905**: Overlapping range patterns (NEW - warning)
- **E0906**: Incompatible patterns in or-pattern (NEW)

---

## Timeline

| Task | Estimated Time |
|------|---------------|
| 1. Integer exhaustiveness | 1 hour |
| 2. String exhaustiveness | 30 min |
| 3. Tuple/Array patterns | 1.5 hours |
| 4. Or patterns | 30 min |
| 5. Testing | 30 min |
| 6. Documentation | 30 min |
| **TOTAL** | **4.5 hours** |

---

## Priority Order

**Phase 1** (Essential - 2 hours):
1. Integer exhaustiveness
2. String exhaustiveness
3. Testing basic exhaustiveness
4. Documentation updates

**Phase 2** (Advanced - 2.5 hours):
5. Tuple/Array patterns
6. Or patterns
7. Comprehensive testing
8. Full documentation

---

## Success Criteria

✅ All primitive types have exhaustiveness checking  
✅ Helpful error messages with suggestions  
✅ Array/tuple destructuring in patterns  
✅ Or patterns for concise matching  
✅ Comprehensive test suite (6+ tests)  
✅ Complete documentation  
✅ All tests passing  
✅ Ready to merge to main as v0.10.5
