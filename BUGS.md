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

**Totals**: 73 bugs tracked, 67 fixed, 6 documented for future work

## Found During Docs Verification Audit (Session v2.0 — 2026-03-26)

> **Context**: Systematic verification of all skill/reference files against compiler source code.
> Discovered 2 compiler bugs by testing documented examples against the actual compiler.

### Bug #95: Bare `or fail` (without message) eats next statement ✅ FIXED
**Severity**: High
**Location**: `parser.rs` L1444-1458 — `or fail` handling in VarDecl parsing
**Status**: FIXED

**Description**: When using `or fail` without a message string, the parser always calls `parse_expression()` after consuming the `fail` token. This causes it to **consume the next statement** as the fail message expression, producing silently broken code.

**Root Cause**: In `parser.rs` L1456, after consuming `or` and `fail`, there's no check for end-of-line/newline/semicolon before calling `parse_expression()`. It unconditionally parses the next available expression.

**Example** (broken):
```liva
main() {
    let data, err = File.read("test.txt")
    let result = data or fail     // ← bare or fail
    print(data)                    // ← THIS gets consumed as the fail message!
}
```

**Generated Rust** (wrong):
```rust
let result = data.unwrap_or_else(|| {
    Err(liva_rt::Error::chain(
        println!("{}", data),      // <-- print(data) was consumed as fail message
        "main", "test.liva:3"
    ))
});
// print(data) is GONE — silently eaten
```

**Workaround**: Always use `or fail "message"`:
```liva
let result = data or fail "Failed to read file"   // ✅ Works correctly
```

**Fix Applied**: In `parser.rs`, after consuming `or fail`, compare the line number of the `fail` token with the next token's line. If they differ (or at end/semicolon/RBrace), use an empty string sentinel instead of calling `parse_expression()`. In `codegen.rs`, detect the empty string sentinel and generate direct error propagation (`return Err(e)`) instead of wrapping in `Error::chain`.

---

### Bug #96: Data class with all-default fields — no-arg constructor not generated ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — data class `new()` generation
**Status**: FIXED

**Description**: When a data class has **all fields with default values** but **no explicit constructor**, the auto-generated `new()` method requires ALL fields as positional arguments. The field defaults are ignored because the auto-generated constructor takes all fields as parameters.

**Example** (broken):
```liva
AppConfig {
    host: string = "localhost"
    port: int = 8080
    debug: bool = false
}

main() {
    let config = AppConfig()    // ❌ Error: expects 3 arguments
}
```

**Workaround**: Add an explicit empty constructor, which allows field defaults to apply:
```liva
AppConfig {
    host: string = "localhost"
    port: int = 8080
    debug: bool = false

    constructor() {}            // ← explicit empty constructor
}

main() {
    let config = AppConfig()    // ✅ Works — host="localhost", port=8080, debug=false
}
```

**Why**: Data classes (no constructor) auto-generate `new(field1, field2, ...)` with ALL fields as parameters. The field defaults only apply when an explicit constructor is present and doesn't assign those fields.

**Fix Applied**: In `codegen.rs`, when generating data class constructor, check if ALL fields have `init` expressions (defaults). If so, generate a no-arg `pub fn new() -> Self` that initializes each field from its default expression. Mixed-default data classes still require all fields as positional args.

---

## Found During Dogfooding v2 — Inventory Manager (Session 18)

8 codegen bugs found via comprehensive 350-line program with 21 test scenarios.

### Bug #75: Map/Set class fields not recognized for method routing ✅ FIXED
**Severity**: High
**Location**: `codegen.rs` — Map/Set routing checks
**Description**: `this.prices.set("x", 1)` generated `.set()` instead of `.insert()` because Map/Set routing only checked `Expr::Identifier`, not `Expr::Member`.
**Fix**: Added `Expr::Member` branch to both Map and Set routing checks. Also registered class Map/Set fields in `map_vars`/`set_vars` before method generation.

### Bug #76: `is_map_get_call` didn't handle `this._field.get()` ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — `is_map_get_call()`
**Description**: `map.get(key) or default` inside class methods generated `||` instead of `.unwrap_or()`.
**Fix**: Added `Expr::Member` access check to `is_map_get_call`.

### Bug #77: String variables not cloned in instance method calls ✅ FIXED
**Severity**: High
**Location**: `codegen.rs` — `generate_method_call_expr()` argument loop
**Description**: `inv.getName(sku)` consumed `sku`, preventing reuse in `inv.getQty(sku)`. Regular function calls already cloned strings, but method calls didn't.
**Fix**: Added string/class-instance variable cloning to the general argument generation loop in `generate_method_call_expr`.

### Bug #78: `or "string"` generates `&str` instead of `.to_string()` ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — user fallible match arm
**Description**: `validate(x) or "FALLBACK"` generated `Err(_) => "FALLBACK"` (type `&str`), but function returns `String`.
**Fix**: Added string literal check to append `.to_string()` in the `Err(_)` match arm.

### Bug #79: `some()`/`every()` use wrong lambda pattern ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — lambda pattern generation
**Description**: `nums.some(n => n > 0)` generated `|&&n|` but `any`/`all` take `FnMut(Self::Item)` not `FnMut(&Self::Item)`, so the correct pattern is `|&n|`.
**Fix**: Separated `some`/`every` from `filter`/`find` in lambda pattern generation.

### Bug #80: for-in-map variables are references, need cloning ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — for-in-map loop body
**Description**: `for k, v in map` yields `(&K, &V)` references from `.iter()`, but loop body treats them as owned.
**Fix**: Added `let key = key.clone(); let val = val.clone();` at start of loop body.

### Bug #81: `map.get or default` at expression level uses `||` ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — `generate_binary_operation()`
**Description**: `let t = config.get("k") or "30"` at expression level generated `||` instead of `.unwrap_or()`.
**Fix**: Added special case in `generate_binary_operation` to detect `BinOp::Or` with `is_map_get_call` left side.

### Bug #82: Map/Set mutating methods don't trigger `&mut self` ✅ FIXED
**Severity**: High
**Location**: `codegen.rs` — `is_mutating_method()`
**Description**: Class methods calling `this.items.set(...)` / `this.tags.add(...)` got `&self` instead of `&mut self`.
**Fix**: Added `"set"`, `"add"`, `"delete"` to `is_mutating_method` list.

### Session 18 Summary
- **Bugs found**: 8 (all fixed)
- **Regression tests**: 7 new snapshot tests
- **Test total**: 322 tests, 0 failures
- **Program**: 350 lines, 21 test scenarios covering Map, Set, Enum, error handling, data classes, interfaces, constants, Math, string/array methods, break/continue

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

## Found During Dogfooding v3 — TODO API REST (Session v1.9)

7 codegen bugs found via a complete REST API (HTTP Server + SQLite + JSON.stringify) ~195 lines.

### Bug #83: Map.get() generates Option<String> instead of String ✅ FIXED
**Severity**: High
**Location**: `codegen.rs` — `generate_map_method_call()`
**Description**: `row.get("name")` generated `HashMap::get().cloned()` → `Option<String>`, but Liva expects plain `String`.
**Fix**: Appended `.unwrap_or_default()` after `.cloned()`. Added `suppress_map_get_unwrap` flag for `or default`/`or "value"` paths.

### Bug #84: DB Connection not thread-safe for async HTTP handlers ✅ FIXED
**Severity**: Critical
**Location**: `codegen.rs` — DB.open codegen
**Description**: `rusqlite::Connection` is not `Clone+Send+Sync`, can't be moved into multiple async route handler closures.
**Fix**: Wrapped `DB.open` result in `Arc<Mutex<>>`. Added `.lock().unwrap()` to `DB.exec`/`DB.query`. Added `db.clone()` before each route handler closure.

### Bug #85: Vec indexing moves HashMap out of Vec ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — index expression clone logic
**Description**: `let row = rows[0]` moves `HashMap` out of `Vec`, but Rust doesn't allow moving out of indexed content.
**Fix**: Added `map_array_vars` check to `needs_clone` logic, generating `.clone()` for array indexing on map array vars.

### Bug #86: DB params consume String variables ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — DB.exec/DB.query params generation
**Description**: `DB.exec(db, sql, [title, description])` generated `vec![title, description].iter().map(|s| s.to_string()).collect()` which consumed the original variables before `.iter()`.
**Fix**: Created `generate_db_params_vec()` helper that generates `vec![a.to_string(), b.to_string()]` directly.

### Bug #87: req.body assigned variables not tracked as strings ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — VarDecl string tracking
**Description**: Variables assigned from `req.body` weren't tracked in `string_vars`, so they weren't cloned when passed to functions.
**Fix**: Added `req.body` detection in VarDecl to register result as `string_vars`.

### Bug #88: axum 0.8 route params use {param} not :param ✅ FIXED
**Severity**: High 
**Location**: `codegen.rs` — route path generation
**Description**: Runtime panic — axum 0.8 uses `/{param}` syntax, not `/:param`. Routes with params failed at startup.
**Fix**: Convert `:param` to `{param}` in route path generation.

### Bug #89: extractJsonField indexOf two-arg not supported ✅ FIXED (Liva source fix)
**Severity**: Low
**Location**: User code (`examples/dogfooding-v3/main.liva`)
**Description**: `indexOf("\"", 1)` — second argument was silently ignored.
**Fix**: Rewrote to use `substring(1).indexOf("\"")` instead.

### Session v1.9 Summary
- **Bugs found**: 7 (all fixed)
- **Regression tests**: 3 snapshot tests updated
- **Test total**: 482 tests, 0 failures
- **Program**: ~195 lines TODO API REST — full CRUD with HTTP Server + SQLite + JSON.stringify
- **All 6 endpoints tested successfully with curl**

Key areas of codegen bugs:
1. Option<T> unwrapping for Map.get()
2. Thread safety for DB connections in async handlers (Arc<Mutex>)
3. Rust ownership/move semantics for Vec indexing and DB params
4. Variable tracking for req.body assignments
5. Runtime compatibility with axum 0.8 path syntax

---

## Found During Self-Hosting Experiment (v2.0)

> **Session**: Implemented a lexer (~660 lines) and parser (~948 lines) for Liva in Liva itself.
> **Date**: 2026-03-23/24
> **Branch**: `feat/self-hosting` (deleted — will restart after fixing these bugs)

### Bug #90: `.length` field collision with codegen `.len()` ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — Member expression generation
**Status**: FIXED

**Description**: When a class has a field literally named `length`, the codegen translates `obj.length` to `obj.len()` (Rust Vec method) instead of accessing the struct field.

**Fix**: Added check at start of `.length` handler: if object is a class instance (via `var_types`) and that class has a `length` field (via `class_fields`), emit `.length` directly. Only translate to `.len()` for arrays/strings.

---

### Bug #91: `array[index].field` generates map-style access ✅ FIXED
**Severity**: High
**Location**: `codegen.rs` — nested Member expression through Index
**Status**: FIXED

**Description**: When accessing a field on an element retrieved by array indexing (`tokens[pos].token`), the codegen generates map-style string indexing instead of struct field access.

**Fix**: Replaced the Bug #51 hardcoded field name list with universal `.clone()` for ALL fields accessed through array-indexed class elements. Primitives (i32, f64, bool) implement Copy so `.clone()` is harmless; String/Vec/struct fields need it.

---

### Bug #92: `let t = array[idx]` for structs causes Rust move error ✅ FIXED
**Severity**: High
**Location**: `codegen.rs` — VarDecl with array index of non-Copy types
**Status**: FIXED

**Description**: Binding a struct from an array index (`let t = tokens[pos]`) generates Rust code without `.clone()`.

**Fix**: The `needs_clone` logic in `Expr::Index` already handled class-typed arrays via `typed_array_vars` + `class_fields.contains_key()`. Verified working. Additionally, variables created from typed array indexing are now tracked as class instances via `var_types` (see Bug #94 fix).

---

### Bug #93: `if expr => break` parsed as lambda ⚠️ INFO
**Severity**: Low
**Location**: Only affects self-hosting parser (not the Rust compiler)
**Status**: Not a compiler bug — self-hosting parser limitation

**Description**: In the self-hosting parser, `if prec <= minPrec => break` is parsed as `prec <= (minPrec => break)` because the parser's Ident handler interprets `ident => expr` as a lambda.

**Note**: The Rust compiler parser handles this correctly. This is documented for reference only — it shows an ambiguity in the `=>` syntax that a simpler parser can't resolve.

---

### Bug #94: String function parameter move issue ✅ FIXED
**Severity**: Medium
**Location**: `codegen.rs` — VarDecl Index tracking + function call argument cloning
**Status**: FIXED

**Description**: `let opTok = toks[i]` where `toks` is `[string]` → `opTok` wasn't tracked as a string variable → not cloned when passed to multiple function calls → Rust move error.

**Fix**: In VarDecl `Expr::Index` handler, now propagates type tracking from `typed_array_vars`: if the base array is `[string]`, the indexed result is tracked in `string_vars`; if `[ClassName]`, tracked in `class_instance_vars` + `var_types`. This enables existing clone logic in `generate_normal_call` to auto-clone the variable.

---

### Session Self-Hosting Summary
- **Bugs found**: 5 (3 codegen, 1 parser info, 1 move semantics)
- **4 fixed**: #90, #91, #92, #94 — all resolved on main
- **1 info**: #93 — self-hosting parser limitation, not a compiler bug
- **Test total**: 497 tests, 0 failures
- **Self-hosting result**: Lexer + Parser fully functional; workarounds no longer needed
