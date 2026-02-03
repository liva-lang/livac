# Liva Compiler Bugs - Dogfooding Session

## Found During GitHub Dashboard Project Testing

### Bug #1: Private field underscore prefix not preserved in Rust ✅ FIXED
**Severity**: High
**Location**: Code generation (Rust backend) - `sanitize_name()` function
**Status**: FIXED in v0.11.3

**Description**: When a Liva class has fields with underscore prefix (e.g., `_isDirty`, `_isRunning`), the generated Rust code transforms them incorrectly.

**Fix Applied**: Modified `sanitize_name()` function to preserve leading underscore when converting from camelCase to snake_case.

**Example**:
```liva
App {
    _isRunning: bool
    _commandCount: number
}
```

**Expected Rust** (now works correctly):
```rust
pub struct App {
    pub _is_running: bool,
    pub _command_count: i32,
}
```

---

### Bug #2: `.length` not translated to `.len()` for strings ✅ FIXED
**Severity**: High
**Location**: Code generation (Rust backend) - `generate_expr()` Member handling
**Status**: FIXED in v0.11.3

**Description**: Liva's `.length` property on strings should translate to Rust's `.len()` method.

**Fix Applied**: Added `string_vars: HashSet<String>` tracking in CodeGenerator to identify string-typed variables. Modified Member expression handling to generate `.len()` instead of `.length()` when the object is a known string variable.

**Example**:
```liva
let currentLen = str.length
```

**Expected Rust** (now works correctly):
```rust
let current_len = str.len();
```

---

### Bug #3: Methods modifying `self` fields generated with `&self` instead of `&mut self` ✅ FIXED
**Severity**: High
**Location**: Code generation (Rust backend) - `generate_params()` function
**Status**: FIXED in v0.11.3

**Description**: When a method modifies `this.field`, the generated Rust used `&self` (immutable reference) instead of `&mut self`.

**Fix Applied**: Added `method_modifies_self()` helper function that analyzes method body for assignments to `this.field`. Modified `generate_params()` to detect when a method needs `&mut self`.

**Example**:
```liva
App {
    commandCount: number
    
    run() {
        this.commandCount = this.commandCount + 1  // Modifies self
    }
}
```

**Expected Rust** (now works correctly):
```rust
impl App {
    pub fn run(&mut self) {  // &mut self, not &self
        self.command_count = self.command_count + 1;
    }
}
```

---

### Bug #4: Assigning `self.field` to local variable needs `.clone()` ✅ FIXED
**Severity**: High
**Location**: Code generation (Rust backend) - `generate_stmt()` for VarDecl
**Status**: FIXED in v0.11.3

**Description**: When assigning a String field from `self` to a local variable, the generated Rust code tried to move the value, which is not allowed behind `&mut self`.

**Fix Applied**: Added `expr_is_self_field()` helper function to detect `this.field` accesses. Modified `generate_stmt()` for VarDecl to automatically add `.clone()` when assigning from a self field.

**Example**:
```liva
run() {
    let user = this.username  // Needs clone in Rust
    showUser(user)
}
```

**Expected Rust** (now works correctly):
```rust
fn run(&mut self) {
    let user = self.username.clone();  // Auto-cloned
    show_user(user);
}
```

---

### Bug #5: String concatenation produces wrong types
**Severity**: High
**Location**: Code generation (Rust backend)

**Description**: When concatenating strings with `+` operator, the generated Rust code produces type mismatches between `String` and `&str`.

**Example**:
```liva
json = json + "\"defaultUser\":\"" + this.defaultUser + "\","
```

**Expected**: Type-safe string concatenation

**Actual**: Type error: expected `&str`, found `String`

---

### Bug #4: Error type comparison with empty string
**Severity**: High  
**Location**: Code generation (Rust backend)

**Description**: The `let value, err = ...` pattern generates code that compares `Option<Error>` with `""` string.

**Example**:
```liva
let content, err = File.read(path)
if err != "" {
    ...
}
```

**Expected Rust**:
```rust
if err.is_some() { ... }
// or
if let Some(e) = err { ... }
```

**Actual Rust**:
```rust
if err != "" {  // Error: cannot compare Option<Error> with &str
```

---

### Bug #5: Array literals with non-primitive types don't implement Copy
**Severity**: Medium
**Location**: Code generation (Rust backend)

**Description**: When using `.filter()` on arrays of objects, the generated Rust uses `.copied()` which requires the `Copy` trait.

**Example**:
```liva
let popular = repos.filter(r => r.isPopular())
```

**Actual Rust generates**:
```rust
repos.iter().filter(|&&r| r.is_popular()).copied().collect::<Vec<_>>();
// Error: Copy trait not implemented
```

**Fix needed**: Use `.cloned()` or remove `.copied()` for reference-based iteration.

---

### Bug #6: Integer type mismatches (i32 vs usize)
**Severity**: Medium
**Location**: Code generation (Rust backend)

**Description**: Liva uses `number` which maps to `i32`, but array lengths and indices in Rust are `usize`.

**Example**:
```liva
pluralize("repo", repos.length)
```

**Actual Rust**:
```rust
pluralize("repo".to_string(), repos.len())
// Error: expected i32, found usize
```

---

### Bug #7: String template with complex expressions
**Severity**: Medium
**Location**: Parser or code generation

**Description**: String templates `$"..."` with complex expressions may produce incorrect Rust code.

**Workaround**: Use string concatenation with `+` instead of templates.

---

### Bug #8: JSON.parse returns Option but code treats as value
**Severity**: Medium
**Location**: Code generation (Rust backend)

**Description**: `JSON.parse` returns an Option, but field access like `parsed.get_field()` is called directly on the Option.

---

### Bug #9: Substring/slice syntax generates wrong types
**Severity**: Medium
**Location**: Code generation (Rust backend)

**Description**: `str.substring(0, maxLen - 3)` generates Rust code with type mismatches in slice indices.

---

## Parser Issues (Not Bugs, Documentation Errors)

### Issue #1: `switch` syntax differs from documentation
**Documentation says**:
```liva
switch value {
    "x" => doX(),
    "y" => doY(),
    _ => default()
}
```

**Actual syntax**:
```liva
switch value {
    case "x": doX()
    case "y": doY()
    default: doDefault()
}
```

### Issue #2: `const` not supported at module level
Module-level constants are not parsed. Constants must be inside functions as `let`.

### Issue #3: `not` operator not supported
Use `!` instead of `not` for negation.

---

## Summary

**Total bugs found**: 23
**Fixed**: 23 ✅ ALL FIXED!

### Session 1 (v0.11.3):
- ✅ Bug #1: Private field underscore prefix
- ✅ Bug #2: `.length` not translated to `.len()` for strings
- ✅ Bug #3: Methods modifying self fields generated with `&self` instead of `&mut self`
- ✅ Bug #4: Assigning `self.field` to local variable needs `.clone()`
- ✅ Bug #5: `.filter()`/`.find()` using `.cloned()` for non-Copy types
- ✅ Bug #6: `.length` returns `i32` (cast from `usize`)
- ✅ Bug #7: String templates with ternary expressions (use single quotes)
- ✅ Bug #8: JSON.parse error binding now tracks `err` in string_error_vars
- ✅ Bug #9: `.find()` Option handling - `x != null` → `x.is_some()`, field access via `.as_ref().unwrap()`

### Session 2 (v0.11.5):
- ✅ Bug #10: `.as_str()` not found on JsonValue - changed to `.as_string().unwrap_or_default()`
- ✅ Bug #11: JsonValue Display showed quotes around strings - improved Display impl to extract string value
- ✅ Bug #12: Nested JSON bracket access `json["a"]["b"]` not supported - added Index<&str> impl
- ✅ Bug #13: JsonValue cannot compare with `true/false` - added PartialEq<bool> impl

### Session 3 (v0.11.6):
- ✅ Bug #14: Nested JSON field access chained `get_field()` calls
- ✅ Bug #15: Variables from JSON indexing tracked in `json_value_vars`
- ✅ Bug #16: JSON access with string variable uses correct method

### Session 4 (v0.11.7):
- ✅ Bug #17: String literals generate `.to_string()` on variable init
- ✅ Bug #18: String variables detected in concatenations via `expr_is_stringy()`
- ✅ Bug #19: Constructor body parsing for `this.field = value` statements
- ✅ Bug #20: Detect mutating methods (push/pop/etc) for `&mut self`
- ✅ Bug #22: forEach lambda without `&` prefix for non-Copy class instances

### Session 5 (v0.11.8):
- ✅ Bug #23: `Http.get()` not recognized, only `HTTP.get()` was working
- ✅ Bug #24: `as_array()` returned `Option<Vec<JsonValue>>` causing `.len()` and `.get()` failures; now returns `Vec<JsonValue>` directly and array indexing uses `.cloned()`

### Session 6 (v0.11.9):
- ✅ Bug #25: JsonValue comparison with `null` now uses `.is_null()` - `coin != null` → `!coin.is_null()`
- ✅ Bug #26: Added `as_float()` method to JsonValue returning `f64` directly (unwrapped)
- ✅ Bug #27: `Vec<JsonValue>` from `.as_array()` uses `.len()` instead of `.length()`

### Session 7 (v0.11.10):
- ✅ Bug #28: String indexing `s[i]` now uses `.chars().nth(i)` for UTF-8 safety

### Known Limitations (not bugs):
- `_` placeholder for ignored values in tuple destructuring not yet supported
- Use `and`/`or` keywords instead of `&&`/`||`
- Top-level functions don't use `fn` keyword (only inside classes)
- `match` keyword is `switch` in Liva with `case:/default:` syntax

**Critical (High severity)**: 4 (all fixed!)
**Medium severity**: 24 (all fixed!)
**Documentation issues**: 3

Most bugs were in the Rust code generation phase, particularly around:
1. Type handling (String vs &str, i32 vs usize)
2. Field naming with underscores
3. Standard library method translation
4. Error type handling
5. Borrow checker issues with self fields
6. Option<T> handling for .find() results
7. JsonValue wrapper methods and traits
8. Case sensitivity in module names (Http vs HTTP)
9. Null comparison for JsonValue types
10. UTF-8 string indexing
