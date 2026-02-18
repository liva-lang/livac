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

### Bug #5: String concatenation produces wrong types ✅ FIXED
**Severity**: High
**Location**: Code generation (Rust backend)
**Status**: FIXED in v0.11.7 (Bug #18: `expr_is_stringy()` detection)

**Description**: When concatenating strings with `+` operator, the generated Rust code produces type mismatches between `String` and `&str`.

**Fix Applied**: String concatenation now uses `format!("{}{}", ...)` instead of `+` operator.

---

### Bug #4b: Error type comparison with empty string ✅ FIXED
**Severity**: High  
**Location**: Code generation (Rust backend)
**Status**: FIXED in v0.11.3 (Bug #8: error binding vars tracked)

**Description**: The `let value, err = ...` pattern generates code that compares `Option<Error>` with `""` string.

**Fix Applied**: Error binding variables are tracked in `error_binding_vars`. Comparisons with `""` are transformed to `.is_some()`/`.is_none()`.

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

### Bug #7: String template with complex expressions ✅ FIXED
**Severity**: Medium
**Location**: Parser or code generation
**Status**: FIXED in v0.11.3 (Bug #7: string templates with ternary expressions)

**Description**: String templates `$"..."` with complex expressions may produce incorrect Rust code.

---

### Bug #8: JSON.parse returns Option but code treats as value ✅ FIXED
**Severity**: Medium
**Location**: Code generation (Rust backend)
**Status**: FIXED in v0.11.3 (Bug #8: JSON.parse error binding tracking)

**Description**: `JSON.parse` returns an Option, but field access like `parsed.get_field()` is called directly on the Option.

---

### Bug #9: Substring/slice syntax generates wrong types ✅ FIXED
**Severity**: Medium
**Location**: Code generation (Rust backend)
**Status**: FIXED in v1.2.0 (Bug #55: substring/charAt expression precedence)

**Description**: `str.substring(0, maxLen - 3)` generates Rust code with type mismatches in slice indices. Fixed by wrapping arguments in parentheses before `as usize` cast.

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

**Total bugs found**: 39
**Fixed**: 39 ✅ ALL FIXED!

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

### Session 8 (v0.11.11):
- ✅ Bug #29: Switch/match with string literals - discriminant now uses `.as_str()` to match `&str` cases

### Session 9 (v0.11.12):
- ✅ Bug #30: `indexOf` on class fields (`this.field.indexOf(query)`) - now correctly detected as string method and generates `.find(&query)` with proper reference

### Session 10 (v0.11.13-v0.11.19):
- ✅ Bug #31: `array.length.toString()` - wrap cast in parens: `(len as i32).to_string()`
- ✅ Bug #32: String variables cloned when passed to constructors
- ✅ Bug #34: Array indexing with int variables adds `as usize` and `.clone()` for strings
- ✅ Bug #35: forEach on `[string]` uses `|p|` not `|&p|` - track string array types
- ✅ Bug #36: Method calls on binary expressions wrap in parens: `(a + b).method()`
- ✅ Bug #37: `join()` keeps `&str` argument, doesn't add `.to_string()`
- ✅ Bug #38: JSON `asString()`, `asBool()`, etc. add `.unwrap_or_default()` for direct values
- ✅ Bug #39: `JSON.stringify` without error binding extracts value with `.0.unwrap_or_default()`

### Session 11 (v0.11.22):
- ✅ Bug #40: Wildcard imports (`import * as alias`) generate incorrect code
  - Was generating: `alias.function()` (field access syntax)
  - Now generates: `module::function()` (Rust module path syntax)
  - Added `module_aliases` HashMap to CodeGenerator to track alias → module_name mappings
  - String literals in module function calls now properly convert with `.to_string()`

### Session 12 - Generics & Parallel Dogfooding (v0.11.22):

**Generics Issues (All Fixed in v0.11.25):**
- ✅ Bug #41: `Vec<T>::pop()` returns `Option<T>`, but Liva's `pop(): T` expects direct value - FIXED v0.11.25
  - Added `.expect("pop from empty array")` suffix for pop() method calls in codegen
  - Added `Stmt::VarDecl` case to `collect_mutated_vars_in_stmt` for mutation detection
  
- ✅ Bug #42: Generic array indexing `items[len - 1]` uses `i32` but should be `usize` - FIXED v0.11.25
  - Now wraps entire index expression in parentheses before adding `as usize`
  - `self.items[len - 1]` → `self.items[(len - 1) as usize]`
  - Also handles `Expr::Member` (self.items) not just `Expr::Identifier`
  
- ✅ Bug #43: Variables calling mutating methods (`push`/`pop`) not detected as needing `mut` - FIXED v0.11.23
  - `let stack = Stack()` now correctly becomes `let mut stack` when `stack.push(x)` is called
  - Fixed: sanitize names in `collect_mutated_vars_in_expr` to match VarDecl lookup

- ✅ Bug #44: Trait `Eq` generates `PartialEq + Copy` but `String` doesn't implement `Copy` - FIXED v0.11.25
  - Changed from `Copy` to `Clone` in trait bounds for Eq, Ord, Neg, Not traits
  - Now generates `PartialEq + Clone` which works with String and other non-Copy types

- ✅ Bug #45: Generic getter methods `get(): T` generate `.clone()` without `Clone` bound - FIXED v0.11.25
  - Extended `expr_is_self_field()` to also detect `this.items[i]` patterns
  - Added `.clone()` suffix for array indexing on self fields

- ✅ Bug #46: Generic methods returning `T` need automatic `Clone` bound inference - FIXED v0.11.25
  - Added `infer_type_param_bounds()` function in codegen
  - Analyzes methods to detect when they return `T` from `this.field` or `this.items[i]`
  - Automatically adds `Clone` bound to type parameters when needed

**Parallel Operations Issues:**
- ✅ Bug #47: `par_iter().filter(|x| x % 2 == 0)` - missing dereference `*x` - FIXED v0.11.23
  - Fixed: `needs_lambda_pattern` now true for parallel adapters
  - Generates `filter(|&&x| ...)` with proper dereference

- ✅ Bug #48: `par_iter().reduce(initial, |acc, x|)` - Rayon's `fold` needs `|| initial` - FIXED v0.11.23
  - Fixed: Generates `fold(|| identity, |acc, x| ...).reduce(|| identity, |a, b| a + b)`
  - Added `.copied()` for Copy types before fold

- ✅ Bug #49: `par_iter().filter(|x| x > 3)` - comparison with `&&T` not `T` - FIXED v0.11.23
  - Same fix as Bug #47

- ✅ Bug #50: Regular `filter()` also has dereference issue with `&&T` - FIXED v0.11.23
  - Fixed: Track primitive type arrays in `typed_array_vars`
  - Array literals like `[1,2,3]` now tracked as "i32" type
  - Generates `filter(|&&x| ...)` with `.copied().collect()` for Copy types

**Field Access Issues:**
- ✅ Bug #51: Array indexing then field access generates JSON-style access - FIXED v0.11.23
  - Fixed: Check if array is typed with class elements
  - Generates `results[0].value` instead of `results[0]["value"]`
  - Added `.clone()` for String fields to avoid move errors
  
- ✅ Bug #52: `number / number` with `float` return type doesn't cast - FIXED v0.11.24
  - Problem: `return x / y` with `-> float` generated `x / y` (integer division)
  - Fixed: Track `current_return_type` in CodeGenerator
  - When return type is `f64` and expression contains division, cast operands to f64
  - `return x / y` → `return (x) as f64 / (y) as f64`
  - Complex expressions like `(a + b) / 2` also work correctly

- ✅ Bug #53: Field access in string templates uses `get_field()` for array items - FIXED v0.11.23
  - Was fixed by Bug #51 fix - typed arrays generate direct field access
  - `$"{results[0].value}"` → `results[0].value` (correct)
  
- ✅ Bug #54: Generic fields in string templates need `Display` bound - FIXED v0.11.25
  - Added `block_uses_type_in_template()` and `expr_uses_type_in_template()` functions
  - Detects when `$"...{this.value}..."` uses a generic field
  - Automatically adds `std::fmt::Display` bound to type parameter
  - Example: `Box<T>` with `$"Box({this.value})"` → `impl<T: std::fmt::Display> Box<T>`

**What Works Well:**
- ✅ Basic generics: `Box<T>`, `Pair<A,B>`, `Triple<X,Y,Z>`
- ✅ Nested generics: `Box(Pair(1, "one"))`
- ✅ Generic class field access (direct, not via array indexing)
- ✅ Generic constructors with type inference
- ✅ Generic factory functions returning specific instantiations
- ✅ Regular and parallel filter() with proper dereference patterns
- ✅ Parallel reduce with correct Rayon fold+reduce pattern
- ✅ Array indexing with direct field access for typed arrays
- ✅ Importing generic classes from other modules
- ✅ Automatic Clone bound inference for methods returning T from this.field
- ✅ Automatic Display bound inference for generic fields in string templates
- ✅ Array indexing with (expr) as usize for non-literal indexes
- ✅ Parallel `map()` operations work perfectly
- ✅ Regular `reduce()` works fine
- ✅ Combining parallel map with subsequent operations
- ✅ Generic classes with different type instantiations in same file

### Known Limitations (not bugs):
- `_` placeholder for ignored values in tuple destructuring not yet supported
- Use `and`/`or` keywords instead of `&&`/`||`
- Top-level functions don't use `fn` keyword (only inside classes)
- `match` keyword is `switch` in Liva with `case:/default:` syntax
- Inclusive range `1..=10` has parser issues in some contexts

### Session 13 - Edge Case Dogfooding (v1.2.0):

**Substring/charAt Expression Precedence:**
- ✅ Bug #55: `substring(start, maxLen - 3)` generates `max_len - 3 as usize` (wrong precedence)
  - Fixed: Wrap arguments in parentheses before `as usize` cast: `(max_len - 3) as usize`
  - Affects both `substring()` and `charAt()` with expression arguments

**forEach on Function Parameters:**
- ✅ Bug #56: `forEach` on `[string]` function parameters generates `|&s|` (move error on String)
  - Fixed: `generate_params()` now tracks `TypeRef::Array` parameters in `typed_array_vars` and `array_vars`
  - Enables correct iterator pattern selection for string array parameters

**Array Literals with Strings:**
- ✅ Bug #57: `let words = ["Hello", "world"]` generates `vec!["Hello", "world"]` (Vec<&str> not Vec<String>)
  - Fixed: String literals in `Expr::ArrayLiteral` now get `.to_string()` suffix
  - Generates `vec!["Hello".to_string(), "world".to_string()]`

**char.toString() Concatenation:**
- ✅ Bug #58: `first.toString() + second.toString()` uses `+` instead of `format!()`
  - Fixed: `expr_is_stringy()` now detects `.toString()`, `.toUpperCase()`, `.toLowerCase()`, `.trim()` method calls
  - String concatenation with `+` correctly generates `format!("{}{}", ...)`

**Class Field Array Operations:**
- ✅ Bug #59: `this.items.filter(item => item == query)` fails with `&&String == String`
  - Root cause: `get_base_var_name()` didn't handle `Expr::Member` (this.field)
  - Fixed: `get_base_var_name()` now extracts property name from Member expressions
  - Also: Class fields registered in tracking maps (array_vars, typed_array_vars, string_vars) before method generation

**Filter Lambda Comparison Types:**
- ✅ Bug #60: `filter(|&item| item == query)` fails: `&String == String`
  - Fixed: Added `ref_lambda_params: HashSet<String>` to track lambda params declared with `&` prefix
  - When a `ref_lambda_param` is used in `==`/`!=` comparison, automatically adds `*` dereference
  - Generates `*item == query` (derefs &String to String)

**Print Array Variables:**
- ✅ Bug #61: `print(reversed)` where `reversed` is `Vec<i32>` from function return uses `{}`
  - Fixed: Added `array_returning_functions: HashSet<String>` to track functions that return `[T]`
  - Variables assigned from array-returning function calls are now tracked in `array_vars`
  - Print handler uses `{:?}` (Debug format) for array variables

**Filter Result Indexing:**
- ✅ Bug #62: `found[0]` on filter result `Vec<String>` fails: cannot move out of index
  - Fixed: Propagate element type from source array through filter/map results
  - When `this.items.filter(...)` produces a result, and `items` is `[string]`, the result is also tracked as string array
  - Array indexing on string arrays now generates `.clone()` suffix

### Session 15 - Student Grade Tracker Dogfooding (v1.2.0):

**Parser Issues:**
- ✅ Bug #63: `return` without value followed by `}` — parser tried to parse `}` as return expression
  - Fixed: `parse_simple_statement()` and `parse_statement()` now check `is_at_end() || Semicolon || RBrace` for empty return
  - `return` in void functions no longer requires semicolon before `}`

- ✅ Bug #64: `continue` inside `if` block fails when top-level `const` is uppercase
  - Root cause: `parse_call()` sees uppercase identifier (`LIMIT`) followed by `{` and interprets `LIMIT { continue }` as a struct literal
  - Fixed: Added lookahead check — verify `{ }` or `{ ident: expr }` pattern before committing to struct literal parsing
  - Now `const LIMIT = 10` followed by `if x > LIMIT { continue }` works correctly

**Semantic Analysis Issues:**
- ✅ Bug #65: `.length` on `Member`/`MethodCall` expressions rejected
  - Fixed: Added `Expr::Member { .. } => true` and `Expr::MethodCall(_) => true` to `expr_supports_length()`
  - Now `this.items.length` and `getItems().length` work correctly

**Code Generation Issues:**
- ✅ Bug #66: Data class `Display` impl had unescaped braces in `write!()` format string
  - `write!(f, "Grade { subject: {}, score: {} }")` fails because `{` is literal but interpreted as format placeholder
  - Fixed: Changed to `push_str()` with `{{{{` and `}}}}` for literal braces in struct-style Display output

- ✅ Bug #67: Data class constructor was `new()` with no parameters
  - `Grade::new(subject, score)` failed because `new()` didn't take any args
  - Fixed: Added data class branch that generates `pub fn new(field1: Type1, field2: Type2, ...) -> Self`

- ✅ Bug #68: Switch expression string literal arms returned `&str` instead of `String`
  - `"A" => "Excellent"` generated `&str` while other arms returned `String`
  - Fixed: Added `.to_string()` for `Expr::Literal(Literal::String(_))` arms in `generate_switch_expr`

- ✅ Bug #69: `this._grades[i].score` generated bracket notation `["score"]` instead of `.score`
  - Array element field access on `self.field[i].prop` used JSON-style `["prop"]`
  - Fixed: Extended `typed_array_vars` check in `Expr::Index` handler to also check `Expr::Member` base objects

- ✅ Bug #70: Methods using `fail` didn't generate `Result` return type
  - `fail "error"` inside a method didn't wrap return type in `Result<T, liva_rt::Error>`
  - Fixed: Added `method.contains_fail` check in `generate_method` to wrap return type and set `in_fallible_function`

- ✅ Bug #71: Methods didn't pre-analyze mutated variables
  - Variables assigned inside method bodies weren't marked as `mut`
  - Fixed: Added `mutated_vars` analysis to `generate_method` (same as `generate_function`)

- ✅ Bug #74: For loops consumed collections due to Rust ownership
  - `for item in items { ... }` moved `items`, preventing reuse in later code
  - Fixed: Added `.clone()` for `Expr::Identifier` and `Expr::Member` iterables (not ranges/method calls)
  - Also fixed duplicate `generate_expr` call that was generating the iterable expression twice

**Critical (High severity)**: 4 (all fixed!)
**Medium severity**: 55 (all fixed!)
**Documentation issues**: 4

**Totals**: 71 bugs tracked, 65 fixed, 6 documented for future work

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
11. Switch/match with string discriminants
12. String indexOf detection on class fields
13. Generic type bounds (Clone, Display)
14. Parallel iterator reference handling
