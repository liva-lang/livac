# Changelog

All notable changes to the Liva compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.17] - 2026-02-03

### Fixed - String Variables in Constructor Calls 🐛

**String ownership when passing to constructors:**

- ✅ **Bug #32**: String variables are now cloned when passed to constructors
  - **Problem**: `let p = Person(myName); print(myName)` failed because `myName` was moved
  - **Root Cause**: String variables passed to class constructors transferred ownership
  - **Fix**: Clone string variables when passed as constructor arguments
  - This allows reusing string variables after passing them to constructors

**Example fixed:**
```liva
let name = "Alice"
let person = Person(name)
print(name)  // Now works! 'name' was cloned
```

## [0.11.16] - 2026-02-03

### Fixed - Method Calls on Binary Expressions 🐛

**Operator precedence for chained method calls:**

- ✅ **Bug #36**: `(arr.length - 1).toString()` now generates correct code
  - **Problem**: Generated `((arr.len() as i32)) - 1.to_string()` where `.to_string()` binds to `1` not the whole expression
  - **Root Cause**: Method call on binary expression needs extra parentheses for correct precedence
  - **Fix**: Detect when method call object is a Binary expression and wrap in parentheses
  - Generates: `(((arr.len() as i32)) - 1).to_string()`

**Real-world testing (Dogfooding):**
- CSV Parser fully functional with split, forEach, while loops, dynamic indexing, and arithmetic expressions

## [0.11.15] - 2026-02-03

### Fixed - Array Indexing with Variables 🐛

**Integer variables as array indexes:**

- ✅ **Bug #34**: `lines[i]` now works when `i` is an `int` variable
  - **Problem**: `lines[i]` generated `lines[i]` but Rust requires `usize` for Vec indexing
  - **Fix**: Detect array indexing with non-literal indexes and add `as usize`
  - Generates: `lines[i as usize]` for variable indexes

**String array indexing returns clone:**

- Also added `.clone()` when indexing `[string]` arrays
  - **Problem**: `let line = lines[i]` fails because `lines[i]` returns `&String`, not `String`
  - **Fix**: Detect string array indexing and add `.clone()`
  - Generates: `lines[i as usize].clone()`

**Real-world testing (Dogfooding):**
- CSV Parser with split, while loops, and dynamic indexing now fully functional

## [0.11.14] - 2026-02-03

### Fixed - forEach/map on String Arrays 🐛

**Lambda pattern for string arrays:**

- ✅ **Bug #35**: `parts.forEach(p => ...)` on `[string]` arrays now compiles correctly
  - **Problem**: Generated `|&p|` which attempts to move String out of reference
  - **Root Cause**: Variables declared as `[string]` weren't tracked in `typed_array_vars`
  - **Fix**: 
    1. Track array types from explicit type declarations (`let parts: [string]`)
    2. Track `.split()` method results as returning `[string]`
    3. When element type is "string", use `will_use_cloned = true` to generate `|p|` not `|&p|`

**Real-world testing (Dogfooding):**
- String manipulation with split, forEach now fully functional

## [0.11.13] - 2026-02-03

### Fixed - Length Property Chain Method Calls 🐛

**String length with method chaining:**

- ✅ **Bug #31**: `text.length.toString()` now compiles correctly
  - **Problem**: `text.length.toString()` generated `text.len() as i32.to_string()` which fails
  - **Root Cause**: Rust's `as` operator has lower precedence than method calls
  - **Fix**: Wrap the cast in parentheses: `(text.len() as i32).to_string()`
  - Updated all three `.length` handlers in codegen.rs (lines ~3251, ~3280, ~8250)

**Real-world testing (Dogfooding):**
- String manipulation patterns with length conversion to string

## [0.11.12] - 2026-02-03

### Fixed - indexOf on Class Fields 🐛

**String indexOf on this.field access:**

- ✅ **Bug #30**: `this.url.indexOf(query)` now works correctly
  - **Problem**: indexOf on class fields generated `.iter().position()` instead of `.find()`
  - **Root Cause**: Detection logic only checked direct variable names, not member access
  - **Fix**: Expanded `is_string_indexof` to handle `Expr::Member` with `this/self` object
  - Also adds `&` prefix for String variable arguments (Pattern trait requirement)

**Real-world testing (Dogfooding):**
- Bookmark Manager CLI with class-based search functionality

## [0.11.11] - 2026-02-03

### Fixed - Switch with String Patterns 🐛

**Pattern matching with strings:**

- ✅ **Bug #29**: Switch/match with string literals now works correctly
  - **Problem**: `switch level { case "INFO": ... }` failed because `level` (String) couldn't match `"INFO"` (&str)
  - **Fix**: Detect string-based switches (any case is a string literal) and add `.as_str()` to discriminant
  - Generates: `match level.as_str() { "INFO" => ... }`

**Real-world testing (Dogfooding):**
- Log Analyzer CLI - Tests switch/match, File.exists, for loops with strings
- All switch patterns with string literals working!

## [0.11.10] - 2026-02-03

### Fixed - String Indexing & Multi-File Imports 🐛

**String character access:**

- ✅ **Bug #28**: String indexing `s[i]` now works correctly
  - **Problem**: `s[i]` generated `s.get(i)` which doesn't compile for Rust strings (UTF-8)
  - **Fix**: Generates `s.chars().nth(i as usize).map(|c| c.to_string()).unwrap_or_default()`
  - Check for `string_vars` BEFORE calling `generate_expr(object)` to avoid double output
  - Full UTF-8 support for character-by-character string manipulation

**Real-world testing (Dogfooding):**
- Todo API Client with POST/PUT/DELETE - all working!
- Modular App with math.liva + strings.liva imports - fully functional
- String reverse, repeat, padLeft all working with char indexing

## [0.11.9] - 2026-02-03

### Fixed - JSON Null Comparison & Array Methods 🐛

**JsonValue null comparison:**

- ✅ **Bug #25**: JsonValue comparison with `null` now works correctly
  - **Problem**: `coin != null` generated `coin != None` which doesn't compile
  - **Fix**: Translate `jsonVar != null` to `!jsonVar.is_null()`
  - Translate `jsonVar == null` to `jsonVar.is_null()`
  - Works for all JsonValue variables tracked in `json_value_vars`

**JsonValue convenience methods:**

- ✅ **Bug #26**: Added `as_float()` method to JsonValue
  - **Problem**: Users expected `.as_float()` but only `.as_f64()` existed
  - **Fix**: Added `as_float()` returning `f64` directly (unwrapped with 0.0 default)
  - More ergonomic for common use cases

**Vec<JsonValue> length:**

- ✅ **Bug #27**: `Vec<JsonValue>` from `.as_array()` now uses `.len()`
  - **Problem**: `coinList.length` generated `.length()` (JsonValue method) instead of `.len()` (Vec method)
  - **Fix**: Track variables initialized with `.as_array()` in `array_vars`
  - Generates `.len() as i32` for proper type compatibility

**Real-world testing (Dogfooding):**
- Crypto Tracker CLI fully functional with CoinGecko API
- Commands: `price <coin>`, `top`, `search <query>`
- Live data for Bitcoin ($78,359), top 10 cryptos, coin search

## [0.11.8] - 2026-02-02

### Fixed - HTTP Client & JSON Array Access 🐛

**HTTP module naming:**

- ✅ **Bug #23**: `Http.get()` not recognized, only `HTTP.get()` worked
  - **Problem**: Case-sensitive module name matching excluded `Http` variant
  - **Fix**: Added case-insensitive matching for HTTP module methods
  - `Http.get()`, `HTTP.get()`, `http.get()` all now work correctly

**JSON array access improvements:**

- ✅ **Bug #24**: `as_array()` returned `Option<Vec<JsonValue>>` causing type mismatches
  - **Problem**: Calling `.len()` on `as_array()` result failed (private field on Option)
  - **Problem**: Calling `.get(0)` failed (method not found on Option type)
  - **Fix**: `as_array()` now returns `Vec<JsonValue>` directly (empty vec for non-arrays)
  - **Fix**: Array indexing uses `.get(n as usize).cloned().unwrap_or_default()`
  - More ergonomic API - no unwrap needed for array iteration

**Real-world testing (Dogfooding):**
- Weather CLI fully functional with real Open-Meteo API calls
- Geocoding + Weather data with nested JSON parsing
- Tested with London, Tokyo, New York - all working!

## [0.11.7] - 2026-02-02

### Fixed - Class & String Handling Improvements 🐛

**String literal and concatenation fixes:**

- ✅ **Bug #17**: String literals now generate `.to_string()` when initializing variables
  - **Problem**: `let x = "["` inferred as `&str`, caused type mismatch on reassignment
  - **Fix**: All string literal initializations now produce `String` type
  - Prevents `&str vs String` type errors when variable is later reassigned

- ✅ **Bug #18**: Variables initialized with strings now detected in concatenations
  - `expr_is_stringy()` now checks `string_vars` for Identifier expressions
  - Variable names properly sanitized (camelCase → snake_case) for tracking
  - Variables initialized with string concatenations also tracked as strings
  - Enables `format!()` usage for all string concatenations

**Class constructor and method improvements:**

- ✅ **Bug #19**: Constructor body parsing for field assignments
  - **Problem**: `constructor(noteTitle)` with `this.title = noteTitle` generated wrong field names
  - **Fix**: Parse `this.field = value` statements in constructor body
  - Generate correct `Self { field: value, ... }` initializers
  - Supports parameters with different names than fields

- ✅ **Bug #20**: Detect mutating method calls on self fields
  - **Problem**: Methods calling `this.notes.push(note)` used `&self` instead of `&mut self`
  - **Fix**: Detect mutating methods (push/pop/remove/clear/insert/sort/reverse/extend/retain/truncate)
  - Check `this.field.method()` pattern for mutation detection

- ✅ **Bug #22**: forEach lambda pattern for non-Copy types
  - **Problem**: `|&note|` pattern in forEach caused "cannot move out of shared reference"
  - **Fix**: Don't add `&` prefix for class instances in forEach
  - Properly handle typed arrays of objects

**Real-world testing (Dogfooding):**
- Built Task Manager CLI (File I/O + JSON + String handling)
- Built Notes App CLI (Classes + Methods + Arrays + File I/O + JSON)
- Both apps fully functional with add/list/clear commands

## [0.11.6] - 2026-02-02

### Added - System Module & CLI Support 🖥️

**New `Sys` module for CLI applications:**

- ✅ **`Sys.args()`**: Returns command-line arguments as `Vec<String>`
  - `let args = Sys.args()` → first element is program name
  - Special `native_vec_string_vars` tracking for proper indexing

- ✅ **`Sys.env(key)`**: Get environment variable value
  - Returns empty string if not found
  - `let home = Sys.env("HOME")`

- ✅ **`Sys.exit(code)`**: Exit process with status code
  - `Sys.exit(1)` for error exit

### Fixed - JSON Nested Access 🐛

- ✅ **Bug #14**: Nested JSON field access didn't work
  - **Problem**: `issue["user"]["login"]` generated invalid Rust code
  - **Before**: `issue.get_field("user").unwrap_or_default()["login"]` ❌
  - **After**: `issue.get_field("user").unwrap_or_default().get_field("login").unwrap_or_default()` ✅
  - Added detection for nested `Expr::Index` to chain `get_field()` calls

- ✅ **Bug #15**: Variables from JSON indexing not tracked as JsonValue
  - **Problem**: `let items = result["items"]; items.forEach(...)` failed
  - Variables from `result["key"]` now properly tracked in `json_value_vars`
  - Enables correct `forEach`/`map`/`filter` lambda pattern generation

- ✅ **Bug #16**: JSON access with string variable used wrong method
  - **Problem**: `config[key]` where `key: string` generated `.get(key)` instead of `.get_field(&key)`
  - Now detects if index variable is in `string_vars` and uses `get_field()` for object access
  - Works for both `Option<JsonValue>` and direct `JsonValue`

**Real-world testing:**
- Built GitHub CLI helper tool in Liva
- Commands: `user <username>`, `repos <username>`, `issues <owner/repo>`, `search <query>`
- Successfully tested against live GitHub API
- Built Config Tool testing File I/O + JSON combination

## [0.11.5] - 2026-02-02

### Fixed - JSON/HTTP Dogfooding Bug Fixes 🐛

**JsonValue improvements for real API usage:**

- ✅ **Bug #10**: `.as_str()` not found on JsonValue 
  - Changed codegen to use `.as_string().unwrap_or_default()` instead of `.as_str().unwrap_or("")`
  
- ✅ **Bug #11**: JsonValue Display showed JSON quotes around strings
  - Improved Display impl to extract string content without JSON quotes
  - `user["name"]` now displays as `John` instead of `"John"`
  
- ✅ **Bug #12**: Nested JSON bracket access not supported
  - Added `impl Index<&str> for JsonValue` to support `json["a"]["b"]["c"]` chained access
  - Uses Box::leak for safe static references in read-only JSON traversal
  
- ✅ **Bug #13**: JsonValue cannot compare with `true`/`false`
  - Added `impl PartialEq<bool> for JsonValue`
  - Now supports: `if todo["completed"] == true { ... }`
  - Added `impl PartialEq<&str> for JsonValue` for string comparisons

**Test Suite:**
- Created comprehensive API tests with JSONPlaceholder
- Tested: HTTP GET/POST, nested JSON, arrays, boolean comparisons
- All tests passing with real HTTP endpoints

## [0.11.4] - 2026-02-02

### Fixed - Dogfooding Bug Fixes 🐛

**Complete bug fixes from GitHub Dashboard dogfooding session:**

**Array Methods with Non-Copy Types:**
- ✅ `.filter()` and `.find()` now use `.cloned()` for class instances instead of `.copied()`
- ✅ Track typed arrays from array literals containing class constructors
- ✅ Lambda patterns adjusted: use `|x|` with `.cloned()` instead of `|&&x|` with `.copied()`

**Option<T> Handling from .find():**
- ✅ Variables from `.find()` now tracked as `Option<T>` in `option_value_vars`
- ✅ `x != null` now transforms to `x.is_some()` 
- ✅ `x == null` now transforms to `x.is_none()`
- ✅ Field access on Option results auto-unwraps: `found.name` → `found.as_ref().unwrap().name`

**Previous Fixes (v0.11.3):**
- ✅ Private field underscore prefix preserved in snake_case conversion
- ✅ `.length` on strings/arrays generates `.len() as i32`
- ✅ Methods modifying `this.field` generate `&mut self`
- ✅ Assigning from `this.field` auto-clones
- ✅ String templates with ternary expressions (use Display format)
- ✅ JSON.parse error binding tracks `err` in `string_error_vars`

**Summary:** All 9 bugs from dogfooding session now fixed! 🎉

## [0.12.0] - In Development

### Added - Language Server Protocol (LSP) Planning 📝

**Documentation Created:**
- ✅ `LSP_IMPLEMENTATION_PLAN.md` - Complete 9-phase implementation roadmap
  - Phase breakdown with time estimates (8-10 hours total)
  - Success criteria and testing strategy
  - Dependencies and rollout plan
  - ~400 lines of comprehensive planning
  
- ✅ `docs/lsp/LSP_DESIGN.md` - Architecture and design documentation
  - System architecture diagrams
  - Module structure (src/lsp/ with handlers)
  - Data structures (LivaLanguageServer, DocumentState, SymbolTable)
  - LSP capabilities matrix
  - Performance optimization strategies
  - ~600 lines of technical specifications
  
- ✅ `docs/lsp/LSP_USER_GUIDE.md` - End-user documentation
  - Quick start guide
  - Feature walkthroughs (completion, navigation, diagnostics)
  - Configuration options
  - Troubleshooting guide
  - Tips & tricks
  - ~900 lines of user-facing docs
  
- ✅ `docs/lsp/LSP_API.md` - API reference for contributors
  - Complete handler APIs
  - Data structure documentation
  - Code examples
  - Extension points
  - ~900 lines of API documentation

**Implementation Plan:**
- **Phase 1:** LSP Infrastructure (2 hours) - tower-lsp setup, server lifecycle
- **Phase 2:** Document Synchronization (1 hour) - didOpen, didChange, didSave handlers
- **Phase 3:** Diagnostics (1.5 hours) - Real-time error reporting
- **Phase 4:** Autocompletion (2 hours) - Context-aware completions
- **Phase 5:** Go to Definition (1 hour) - Navigation
- **Phase 6:** Find References (1 hour) - Symbol search
- **Phase 7:** Hover Information (0.5 hours) - Type info display
- **Phase 8:** Rename Symbol (1 hour) - Refactoring
- **Phase 9:** VS Code Integration (1 hour) - Client setup

**Key Technologies:**
- `tower-lsp` 0.20 - LSP framework
- `tokio` 1.x - Async runtime
- `dashmap` 5.5 - Concurrent document storage
- JSON-RPC over stdio for communication

**Architecture:**
- Document-centric with AST caching
- Incremental parsing for performance
- Symbol table for fast lookups
- Performance targets: <100ms completion, <500ms diagnostics

**Status:** Planning complete, implementation ready to begin  
**Progress:** 4/4 documentation files complete, 0/9 implementation phases complete

## [0.11.3] - 2025-01-28

### Added - Pattern Matching for Union Types ✨

**Pattern Matching Integration:**
- ✅ Type patterns in switch expressions: `n: int => expr`
- ✅ Automatic type narrowing in each match arm
- ✅ Full exhaustiveness checking for union patterns
- ✅ Wildcard pattern support: `_ => default`

**Syntax:**
```liva
let x: int | string = 42
let result = switch x {
  n: int => n * 2,      // n has type int here
  s: string => s.len()  // s has type string here
}
```

**Implementation:**
- ✅ AST extension: `Pattern::Typed { name, type_ref }`
- ✅ Parser: Recognizes `variable: type` pattern syntax
- ✅ Codegen: Generates proper Rust enum variant matches
  - `Union_i32_String::Int(n) => ...`
  - `Union_i32_String::Str(s) => ...`
- ✅ Semantic validation: Ensures exhaustiveness and type safety

**Multi-Type Unions:**
```liva
let value: int | string | bool = "hello"
switch value {
  n: int => "Number",
  s: string => "String",
  b: bool => "Boolean"
}
```

**Documentation:**
- ✅ Comprehensive pattern matching section in `union-types.md`
- ✅ Examples: type narrowing, exhaustiveness, wildcards
- ✅ Code generation details and best practices

**Phase 7.2 Complete:** Union types are now fully usable with pattern matching support.

## [0.11.2] - 2025-01-28

### Added - Union Types ✨

**Basic Union Types:**
- ✅ Syntax: `int | string`, `T | U | V`
- ✅ Type-safe sum types with automatic variant wrapping
- ✅ Inline union annotations: `let x: int | string = 42`
- ✅ Multi-type unions: `int | string | bool`

**Code Generation:**
- ✅ Generates Rust enums with proper variants: `Union_i32_String { Int(i32), Str(String) }`
- ✅ Auto-wrapping values in correct variants: `42` → `Union_i32_String::Int(42)`
- ✅ Automatic `.to_string()` conversion for string literals
- ✅ Implements `Debug`, `Clone`, and `Display` traits for all unions

**Type Safety:**
- ✅ Union flattening: `(A | B) | C` becomes `A | B | C`
- ✅ Duplicate removal: `int | int | string` becomes `int | string`
- ✅ Full semantic validation
- ✅ Integration with existing type system

**Documentation:**
- ✅ Complete specification in `docs/language-reference/union-types.md`
- ✅ Examples: Result<T>, Option<T>, discriminated unions
- ✅ Comparison with TypeScript, Rust, and Haskell

**Known Limitations:**
- ⚠️ Type aliases with unions (e.g., `type Result<T> = T | Error`) not yet supported at top level
- ⚠️ Pattern matching integration pending (Phase 7.2.6)

## [0.11.1] - 2025-01-28

### Added - Type Aliases ✨

**Basic Type Aliases:**
- ✅ Simple syntax: `type UserId = int`
- ✅ Tuple aliases: `type Point = (int, int)`
- ✅ Complex types: `type Matrix = [[int]]`
- ✅ Inline expansion during compilation (zero runtime overhead)

**Generic Type Aliases:**
- ✅ Single parameter: `type Box<T> = (T,)`
- ✅ Multiple parameters: `type Pair<T, U> = (T, U)`
- ✅ Proper type parameter substitution
- ✅ Nested generic aliases: `type IntBox = Box<int>`

**Type Safety:**
- ✅ Circular reference detection with E0701 error
- ✅ Type parameter count validation with E0702 error
- ✅ Full semantic validation during type checking
- ✅ Integration with existing type system (tuples, arrays, optionals, generics)

**Code Generation:**
- ✅ Type aliases expanded inline in generated Rust code
- ✅ No Rust type aliases generated (simpler codegen, zero overhead)
- ✅ Works with all type annotations (let bindings, parameters, return types)

**Documentation:**
- ✅ Complete specification in `docs/language-reference/type-aliases.md`
- ✅ Examples, best practices, and restrictions
- ✅ Comparison with TypeScript, Rust, and Haskell

## [0.11.0] - 2025-01-28

### Added - Tuple Types & Literals ✨

**Tuple Literals:**
- ✅ New syntax: `(10, 20)` for multi-element tuples
- ✅ Single-element tuples with trailing comma: `(42,)` vs `(42)` (grouped expression)
- ✅ Empty tuples (unit type): `()`
- ✅ Nested tuples: `((1, 2), (3, 4))`
- ✅ Type inference for tuple literals

**Tuple Types:**
- ✅ Type annotations: `(int, int)`, `(string, bool, float)`
- ✅ Function return types: `fn(): (int, int)`
- ✅ Heterogeneous types (mixed types in single tuple)
- ✅ Rust interop: Direct mapping to Rust tuples with zero overhead

**Tuple Member Access:**
- ✅ Numeric property access: `tuple.0`, `tuple.1`, `tuple.2`
- ✅ Works in all expressions: assignments, conditions, string templates
- ⚠️ Chained access requires parentheses: `(matrix.0).0` instead of `matrix.0.0`
  - Root cause: Lexer tokenizes `.0.0` as Dot + FloatLiteral(0.0)
  - Documented workaround in all guides

**Pattern Matching Integration:**
- ✅ Tuple patterns in switch expressions: `(x, y) => ...`, `(0, _) => ...`
- ✅ Binding patterns work: `(x, y) if x > y => ...`
- ✅ Wildcard patterns: `(_, y) => ...`
- ✅ Nested tuple patterns: `((a, b), c) => ...`
- ✅ All pattern types supported (literals, bindings, wildcards, guards)

**Code Generation:**
- ✅ Generates clean Rust tuple syntax: `(i32, i32)`
- ✅ Single-element tuple handling: `(i32,)` in Rust
- ✅ Tuple literal codegen: `(10, 20)`
- ✅ Member access codegen: `.0`, `.1` direct field access
- ✅ Pattern matching codegen: Rust match with tuple destructuring

**Implementation Details:**
- **Phase 1 (AST):** Added `Expr::Tuple` and `TypeRef::Tuple` variants
- **Phase 2 (Parser):** 
  - Tuple literals with comma disambiguation: `(x)` vs `(x,)`
  - Tuple type parsing: `(int, int)`, `(string, bool)`
  - Numeric member access: `IntLiteral` case in `parse_method_name()`
- **Phase 3 (Semantic):**
  - Type inference for tuples: builds `TypeRef::Tuple` from element types
  - Validation: tuple member access with bounds checking
  - Type checking: validates numeric indices, returns element type
- **Phase 4 (Codegen):**
  - Tuple literal generation with single-element comma handling
  - Direct field access generation: `.0`, `.1` instead of get_field()
  - Fixed console.log to pass format strings directly
- **Testing:** 6 comprehensive test files, 5 of 6 passing (83% success rate)

**Test Files:**
1. `test_tuple_literals.liva` ✅ PASSING - Basic creation, empty, single, nested
2. `test_tuple_types.liva` ✅ PASSING - Type annotations
3. `test_tuple_access.liva` ✅ PASSING - Member access (with parentheses for chained)
4. `test_tuple_functions.liva` ❌ FAILING - Return type inference issue
5. `test_tuple_patterns.liva` ✅ PASSING - Switch expression pattern matching
6. `test_tuple_nested.liva` ✅ PASSING - Complex nested structures

**Known Limitations (v0.11.0):**
- ⚠️ Chained tuple access requires parentheses: `(matrix.0).0` instead of `matrix.0.0`
  - Root cause: Lexer tokenizes `.0.0` as Dot + FloatLiteral(0.0) (greedy float tokenization)
  - Workaround documented: Use parentheses for chained access
  
- ⚠️ Tuple destructuring in let bindings broken: `let (x, y) = tuple` fails
  - Parser expects identifier after `let`, doesn't recognize tuple pattern
  - Workaround: Use direct access: `let x = tuple.0, y = tuple.1`
  
- ⚠️ String type annotations cause &str vs String mismatch
  - `getUserInfo(): (string, int, bool)` generates `(String, i32, bool)` but returns `(&str, i32, bool)`
  - Workaround: Use type inference instead of explicit string types in tuples
  
- ⚠️ Return type inference doesn't work for functions without explicit return type
  - Functions without return type default to `f64` instead of inferring tuple type
  - Workaround: Always specify explicit return types for tuple-returning functions

**Documentation:**
- ✅ `TUPLE_IMPLEMENTATION_PLAN.md` - Complete 6-phase implementation plan (518 lines)
- ✅ `docs/language-reference/types.md` - Updated with Tuple Types section
- ✅ `docs/language-reference/pattern-matching.md` - Updated tuple pattern status
- ✅ `docs/language-reference/functions.md` - Added Tuple Returns section
- ✅ `docs/guides/tuples.md` - Comprehensive tutorial (600+ lines)
  - Basic usage, pattern matching, best practices
  - When to use tuples vs structs
  - Common patterns and real-world examples
  - Known limitations and workarounds

**Statistics:**
- **Time:** 4 hours (100% of estimate)
- **Code changes:** 7 files modified (ast.rs, parser.rs, semantic.rs, codegen.rs, ir.rs, lowering.rs, liva_rt.rs)
- **Tests:** 6 test files created, 5 passing (83% success rate)
- **Documentation:** 1,500+ lines (implementation plan, language reference updates, tutorial)
- **Commits:** 1 feature commit (0742d6a)

**Use Cases:**
```liva
// Multiple return values
getCoordinates(): (int, int) {
    return (10, 20)
}

// Pattern matching
let point = (10, 20)
let location = switch point {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}

// Nested tuples
let matrix = ((1, 2), (3, 4))
let elem = (matrix.0).0  // Access with parentheses
```

**Future Work (v0.11.1+):**
- Fix tuple destructuring in let bindings
- Fix chained access without parentheses (lexer improvement)
- Fix string type annotation mismatch
- Fix return type inference for tuples

## [0.10.5] - 2025-01-27

### Added - Or-Patterns & Enhanced Pattern Matching ✨

**Or-Patterns:**
- ✅ New syntax: `1 | 2 | 3 => "value"` matches multiple patterns with one arm
- ✅ Works with integers, strings, and all literal types
- ✅ Significantly reduces code duplication in switch expressions
- ✅ Example: `"Saturday" | "Sunday" => true` for weekend checking
- ✅ Can combine multiple or-patterns in same switch: `1|2|3 => "small", 4|5|6 => "medium"`

**Enhanced Exhaustiveness Checking:**
- ✅ Extended to support or-patterns correctly
- ✅ Integer exhaustiveness (E0902) now processes or-patterns recursively
- ✅ String exhaustiveness (E0903) validates or-patterns properly
- ✅ Type inference improved to detect types inside or-patterns
- ✅ All existing exhaustiveness checks continue to work with or-patterns

**Lexer & Parser Extensions:**
- ✅ Added `|` (Pipe) token to lexer for or-pattern syntax
- ✅ Parser extended with `parse_or_pattern()` method
- ✅ Recursive pattern parsing: `parse_pattern() → parse_or_pattern() → parse_single_pattern()`
- ✅ Tuple and Array pattern AST nodes added (foundation for future work)

**Code Generation:**
- ✅ Or-patterns generate clean Rust match syntax: `1 | 2 | 3 => ...`
- ✅ Display trait updated for all new pattern types
- ✅ Seamless integration with existing codegen infrastructure

**Semantic Validation:**
- ✅ Added pattern binding extraction for future tuple/array validation
- ✅ Added validation framework for nested patterns
- ✅ E0906 error code reserved for incompatible or-pattern bindings (future use with tuples)

**Documentation:**
- ✅ Updated `pattern-matching.md` with or-pattern section
- ✅ Added examples for integer and string or-patterns
- ✅ Documented exhaustiveness behavior with or-patterns
- ✅ Updated version to v0.10.5 across documentation

**Tests:**
- ✅ `test_or_patterns_simple.liva` - Validates or-pattern code generation
- ✅ `test_or_patterns_non_exhaustive.liva` - Validates E0902 with or-patterns
- ✅ All existing exhaustiveness tests continue to pass

**Impact:**
- Makes switch expressions more concise and readable
- Reduces boilerplate when matching multiple values
- Maintains type safety and exhaustiveness guarantees
- Foundation laid for tuple/array destructuring in v0.10.6

## [0.10.4] - 2025-01-27

### Added - Optional Fields & Default Values for JSON Parsing ✨

**Optional Fields with `?` Syntax:**
- ✅ New syntax: `field?: Type` declares optional fields in classes
- ✅ Generates `Option<T>` wrapper in Rust code
- ✅ Auto-adds `#[serde(skip_serializing_if = "Option::is_none")]` attribute
- ✅ Handles missing fields, null values, and present values seamlessly
- ✅ Perfect for real-world APIs with optional/nullable fields

**Default Values with `=` Syntax:**
- ✅ New syntax: `field: Type = value` declares fields with default values
- ✅ Supports all literal types: int, float, string, bool
- ✅ Automatic string conversion: `"text"` → `"text".to_string()` for string fields
- ✅ Works with both default and parameterized constructors
- ✅ Non-parameter fields use their init value in constructors

**Optional Fields with Default Values:**
- ✅ Combined syntax: `field?: Type = value` for optional fields with defaults
- ✅ Generates serde default functions: `fn default_{class}_{field}() -> Option<T>`
- ✅ Adds `#[serde(default = "default_function")]` attribute
- ✅ When JSON missing the field, serde uses default value instead of None
- ✅ Makes defaults available in destructuring patterns automatically

### Fixed - Optional Fields Bug Fixes 🐛

**Constructor Generation:**
- Fixed optional field constructors to generate `None` instead of `String::new()`
- Both default and parameterized constructors now correctly initialize optional fields
- Fixed default values to wrap in `Some()` when field is optional
- String literals in default values automatically converted to `String` type

**Object Destructuring:**
- Fixed optional fields in lambda destructuring for `forEach`, `map`, `filter`, etc.
- Optional fields now correctly unwrap with `.as_ref().map(|v| v.clone()).unwrap_or_default()`
- Required fields correctly use `.clone()` without unnecessary unwrapping
- Added `current_lambda_element_type` to track class types through lambda generation
- Works correctly with parallel operations (`.parvec().forEach`)

**Nested Struct Access:**
- Fixed issue where nested structs were incorrectly treated as JsonValue
- Destructured nested class fields (e.g., `address` from `User`) now correctly identified as class instances
- Member access on nested structs now generates correct code (e.g., `address.zipcode` instead of `address.get_field("zipcode")`)
- Added type tracking for destructured fields that are themselves class types

**Serde Default Integration:**
- Optional fields with default values now generate serde default functions
- Default values correctly applied when field is missing from JSON (not just in constructors)
- Generated code: `#[serde(default = "default_{class}_{field}")]`
- Solves issue where defaults only worked in constructors, not during JSON deserialization

**Real-World Testing:**
- Tested with JSONPlaceholder API integration
- User class with optional `username?: string` field works correctly
- Nested struct access (`address.zipcode`) works correctly in string templates
- Object destructuring in forEach properly handles mixed optional/required fields
- Optional fields with defaults (`algo?: string = "hola"`) show default value when missing from JSON

**Example of Fixed Behavior:**
```liva
User {
    id: u32
    name: string
    username?: string  // ✨ Optional field
}

main() {
    let users: [User], err = async HTTP.get("https://api.example.com/users").json()
    
    // ✅ Now works correctly with optional username
    users.parvec().forEach(({id, name, username}) => {
        console.log($"User {id}: {name} (@{username})")
    })
}
```

**Why Optional Fields Matter:**
- **Type Safety:** Explicitly document which fields can be absent/null
- **No More Crashes:** Missing fields don't cause JSON parse failures
- **Better DX:** Code shows intent - optional vs required fields
- **API Ready:** Handle real-world JSON APIs with nullable fields

**Example Usage:**
```liva
User {
    id: u32          // Required field
    name: String     // Required field
    email?: String   // ✨ Optional - can be null or absent
    age?: u32        // ✨ Optional - can be null or absent
}

main() {
    // Works with all fields present
    let json1 = "{\"id\": 1, \"name\": \"Alice\", \"email\": \"alice@example.com\"}"
    let user1: User, err1 = JSON.parse(json1)
    
    // Works with email missing
    let json2 = "{\"id\": 2, \"name\": \"Bob\"}"
    let user2: User, err2 = JSON.parse(json2)  // ✅ No error!
    
    // Works with email null
    let json3 = "{\"id\": 3, \"name\": \"Carol\", \"email\": null}"
    let user3: User, err3 = JSON.parse(json3)  // ✅ No error!
}
```

**Generated Rust Code:**
```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,  // ✅ Wrapped in Option<T>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
}
```

**Real-World Use Case:**
```liva
// API response with optional fields
Post {
    id: u64
    title: String
    content: String
    publishedAt?: String  // May not be published yet
    authorEmail?: String  // Author may not have public email
    likes?: u32           // New field, older posts don't have it
}

main() {
    let response, err = async HTTP.get("https://api.example.com/posts")
    if err == "" {
        let posts: [Post], parseErr = JSON.parse(response.body)
        // All posts parse successfully, regardless of which fields are present! ✅
    }
}
```

**Implementation Details:**
- **Parser:** Already implemented in v0.10.3 (detects `?` token after field name)
- **AST:** `FieldDecl.is_optional: bool` field tracks optional status
- **Codegen:** `generate_field()` wraps type in `Option<T>` when `is_optional=true`
- **Serde:** Auto-adds skip attribute for efficient serialization
- **Time:** ~45 minutes (as estimated in Phase 7.0.5)

**Files Modified:**
- `src/codegen.rs` - Updated `generate_field()` function (20 lines)
- Tests: `test_optional_fields.liva` (comprehensive 4-case validation)

**Statistics:**
- Code changes: +20 lines in codegen.rs
- Test coverage: 4 test cases (all fields, missing, null, multiple missing)
- Generated code: Clean Option<T> with proper serde attributes

---

## [0.10.3] - 2025-01-26

### Added - Parameter Destructuring 🎯

**Destructuring in Function Parameters:**
- ✅ Array destructuring in parameters: `printPair([first, second]: [int]) { ... }`
- ✅ Object destructuring in parameters: `printUser({name, age}: User) { ... }`
- ✅ Rest patterns in parameters: `processList([head, ...tail]: [int]) { ... }`
- ✅ Full code generation with temporary parameter names
- ✅ Works in both functions and methods
- ✅ Semantic validation for destructured parameters

**Destructuring in Lambda Parameters:**
- ✅ Array destructuring in lambdas: `pairs.forEach(([x, y]) => ...)`
- ✅ Object destructuring in lambdas: `users.forEach(({id, name}) => ...)`
- ✅ Works with all array methods: `forEach`, `map`, `filter`, `reduce`
- ✅ Works with parallel variants: `parvec().forEach(([x, y]) => ...)`
- ✅ Parser recognizes `[x, y] =>` and `{x, y} =>` as lambda starts
- ✅ Codegen inserts destructuring in both regular and special paths

**Example Usage - Array Destructuring:**
```liva
// Function with array destructuring parameter
printPair([first, second]: [int]): int {
    print("First:", first)
    print("Second:", second)
    return first + second
}

main() {
    let nums = [100, 200]
    let sum = printPair(nums)  // First: 100, Second: 200
    print("Sum:", sum)         // Sum: 300
}
```

**Example Usage - Lambda Destructuring:**
```liva
// Array destructuring in forEach
let pairs = [[1, 2], [3, 4], [5, 6]]
pairs.forEach(([x, y]) => {
    print("x=${x}, y=${y}, sum=${x + y}")
})

// Object destructuring in forEach
let users = [
    {id: 1, name: "Alice"},
    {id: 2, name: "Bob"}
]
users.forEach(({id, name}) => {
    print("User #${id}: ${name}")
})

// Works with map
let sums = pairs.map(([a, b]) => a + b)

// Works with filter
let filtered = pairs.filter(([x, y]) => x > 2)
```

**Implementation Details:**
- Parser creates `BindingPattern` for both `Param` and `LambdaParam`
- Both use `pattern: BindingPattern` instead of `name: String`
- Lambda parser updated to recognize destructuring patterns as lambda starts
- Codegen generates temporary names (`_param_0`, `_param_1`, etc.)
- Destructuring code inserted at function/lambda start with `let` statements
- Special array method path (forEach/map/filter) now includes destructuring support
- Semantic analyzer validates patterns and declares variables
- Codegen generates temporary parameter names (`_param_0`, `_param_1`)
- Destructuring code inserted at function/method entry
- Supports nested destructuring (coming soon)

### Changed
- AST: `Param.name: String` → `Param.pattern: BindingPattern`
- All usages of `param.name` migrated to `param.name()` method
- `generate_params()` now handles destructured parameters with temp names

### Technical
- Added `generate_param_destructuring()` for code generation
- Added `parse_param_pattern()` for parsing patterns without type annotations
- Added `declare_param_pattern()` for semantic validation
- Comprehensive design document in `docs/PHASE_6.5.1_PARAM_DESTRUCTURING_DESIGN.md`

## [0.10.2] - 2025-01-26

### Added - Destructuring Patterns 🎯

**Object and Array Destructuring:**
- ✅ Object destructuring: `let {x, y} = point`
- ✅ Object destructuring with rename: `let {name: userName, age: userAge} = person`
- ✅ Array destructuring: `let [first, second] = array`
- ✅ Array destructuring with skip: `let [a, , c] = array`
- ✅ Rest patterns in arrays: `let [head, ...tail] = items`
- ✅ Type annotations with destructuring: `let {x, y}: Point = point`
- ✅ Full semantic validation (field existence, duplicate bindings, type checking)
- ✅ Comprehensive parser, semantic, and codegen support

**Example Usage - Object Destructuring:**
```liva
let point = { x: 10, y: 20 }
let {x, y} = point
print("x:", x, "y:", y)  // x: 10 y: 20

// Rename bindings
let person = { name: "Alice", age: 30 }
let {name: userName, age: userAge} = person
print("userName:", userName)  // userName: Alice
```

**Example Usage - Array Destructuring:**
```liva
let numbers = [1, 2, 3, 4, 5]

// Basic destructuring
let [first, second] = numbers
print("first:", first)  // first: 1

// Skip elements
let [a, , c] = numbers
print("a:", a, "c:", c)  // a: 1 c: 3

// Rest patterns
let [head, ...tail] = numbers
print("head:", head)  // head: 1
// tail is [2, 3, 4, 5]
```

**Implementation Details:**
- New `BindingPattern` enum in AST (Identifier, Object, Array)
- Parser support with proper error handling
- Semantic validation ensures fields exist on known types
- Duplicate binding detection
- Codegen generates temporary variables to avoid move issues
- Works with both JSON objects and Rust structs

See `examples/destructuring_demo.liva` for complete examples.

## [0.10.1] - 2025-01-26

### Added - response.json() Method 🌐

**Ergonomic JSON Parsing from HTTP Responses:**
- ✅ `response.json()` method on Response objects (like JavaScript fetch API)
- ✅ Returns `(JsonValue, String)` tuple for easy error handling
- ✅ Alternative to `JSON.parse(response.body)`
- ✅ Works with typed JSON parsing: `let user: User, err = response.json()`
- ✅ Automatic serde derives for classes used with response.json()
- ✅ Cleaner, more intuitive API for REST consumption

**Example Usage - Basic:**
```liva
let response, err = HTTP.get("https://api.github.com/users/octocat")
if err != "" { return }

// Parse JSON directly from response (like fetch API)
let json, parseErr = response.json()
if parseErr != "" { return }

console.log($"User data: {json}")
```

**Example Usage - Typed:**
```liva
User {
    name: string
    email: string
    company: string
}

let response, err = HTTP.get("https://api.example.com/users/1")
if err != "" { return }

// Automatic deserialization to User class
let user: User, jsonErr = response.json()
if jsonErr != "" { return }

console.log($"User: {user.name} at {user.company}")
```

**Implementation:**
- Runtime (liva_rt.rs): Added `json()` method to Response struct
- Codegen: Extended `is_json_parse_call()` to detect `.json()` methods
- Codegen: Updated `generate_typed_json_parse()` to use `.body` for response.json()
- Codegen: Fixed `is_builtin_conversion_call()` tuple detection logic
- Semantic: Extended JSON.parse validation to include `.json()` calls
- Semantic: Tracks `.json()` calls with type hints for serde derives

### Fixed
- is_builtin_conversion_call() now correctly detects .json() as tuple-returning method
- Moved .json() check to beginning of match statement (was unreachable in else block)

### Documentation
- Updated docs/language-reference/http.md with response.json() documentation (+171 lines)
- Added response.json() examples for basic and typed parsing
- Updated all HTTP examples to use ergonomic response.json() API

### VSCode Extension v0.8.0
- Added 16 new HTTP snippets: httpget, hget, httppost, hpost, httpput, hput, httpdelete, hdel, httpjson, httppostjson, resjson, resjsonc, httptyped, httpstatus, httpfull, restapi
- Updated README with comprehensive HTTP Client documentation
- Added HTTP keywords: http, rest-api, web
- Total snippets: 103 (87 existing + 16 new HTTP snippets)

## [0.10.0] - 2025-01-25

### Added - Typed JSON Parsing (Complete) 🎉

**Type-Safe JSON Parsing with Type Hints:**
- ✅ Parse JSON directly into typed values without `.as_i32().unwrap()` calls
- ✅ Type hints support: `let data: [i32], err = JSON.parse(json)`
- ✅ All Rust primitive types supported: i8-i128, u8-u128, isize, usize, f32, f64, bool, String
- ✅ Arrays of typed values: `[i32]`, `[f64]`, `[String]`, etc.
- ✅ **Custom classes with serde derives (Phase 2)**
- ✅ **Nested classes with recursive dependency tracking (Phase 4)**
- ✅ **Arrays of custom classes**
- ✅ Clean error handling with `(Type, String)` tuple (no Option!)
- ✅ Single binding mode: `let data: [i32] = JSON.parse(json)` (panics on error)

**Example Usage - Primitives and Arrays:**
```liva
// OLD syntax (v0.9.x) - verbose with .unwrap()
let data = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n.as_i32().unwrap() * 2)

// NEW syntax (v0.10.0) - clean and type-safe! ✨
let data: [i32], err = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n * 2)  // No .unwrap() needed!
```

**Example Usage - Custom Classes (Phase 2):**
```liva
User {
    id: u32
    name: String
    age: i32
}

let userJson = "{\"id\": 1, \"name\": \"Alice\", \"age\": 30}"
let user: User, err = JSON.parse(userJson)

if err == "" {
    print($"User: {user.name}, age {user.age}")  // Direct field access!
}
```

**Example Usage - Nested Classes (Phase 4):**
```liva
Address {
    street: String
    city: String
}

User {
    name: String
    address: Address    // Nested class
}

Comment {
    text: String
    author: String
}

Post {
    title: String
    comments: [Comment]  // Array of classes
}

let json = "{\"title\": \"Hello\", \"comments\": [{\"text\": \"Great!\", \"author\": \"Bob\"}]}"
let post: Post, err = JSON.parse(json)
// Both Post and Comment automatically get serde derives!
```

**Phase 1 - Primitives and Arrays (4.5h):**
- Parser: Type hints already supported in let statements
- Semantic: `validate_json_parse_type_hint()` validates serializable types
- Codegen: Generates `serde_json::from_str::<T>` with proper error handling
- Support for all Rust integer types, floats, bool, String
- Arrays: `[T]` maps to `Vec<T>`

**Phase 2 - Custom Classes (1h):**
- AST: Added `needs_serde: bool` to `ClassDecl`
- Semantic: Tracks classes used with JSON.parse in `json_classes` HashSet
- Semantic: `mark_json_classes()` updates AST before codegen
- Codegen: Conditional serde derives: `#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]`
- Codegen: Tracks class instances in `class_instance_vars` for proper member access
- Cargo.toml: Automatically adds `serde = { version = "1.0", features = ["derive"] }`
- Note: Field names must match JSON keys exactly (no automatic camelCase/snake_case conversion)

**Phase 4 - Nested Classes (30min):**
- Semantic: `collect_class_dependencies()` - Recursively finds all class dependencies
- Semantic: `collect_type_dependencies()` - Handles TypeRef (Simple, Array, Optional)
- Semantic: `is_class_type()` - Distinguishes classes from primitives
- All dependent classes automatically get serde derives
- Supports arbitrary nesting depth
- Supports arrays of nested classes: `[Comment]` inside `Post`

**Semantic Validation:**
- Validates that types used with JSON.parse are serializable
- Recursive validation for arrays, optionals, and generics
- Checks class existence for custom types
- Validates nested class dependencies exist

**Code Generation:**
- Generates `serde_json::from_str::<T>(&json)` instead of JsonValue wrapper
- Error handling: `match` expression with Ok/Err branches
- Default values on error: Vec::new(), 0, 0.0, false, String::new(), Default::default()
- Single binding: generates `.expect("JSON parse failed")` for simplicity
- Direct field access for class instances (no `.get_field()`)

**Files Modified:**
- `src/ast.rs`: Added `needs_serde` field to ClassDecl
- `src/semantic.rs`: Added validation and dependency tracking (lines 2687-2840)
- `src/codegen.rs`: Added typed JSON parsing and serde support (lines 119-162, 1540-1720)
- `Cargo.toml`: Template updated to include serde dependency

**Test Files:**
- `test_json_typed_parse.liva`: Primitives and arrays
- `test_json_class_basic.liva`: Simple custom classes
- `test_json_snake_case.liva`: Field name matching demo
- `test_json_nested.liva`: Nested classes (User with Address)
- `test_json_nested_arrays.liva`: Arrays of nested classes (Post with [Comment])

**Documentation:**
- `/docs/language-reference/json.md`: Updated to v0.10.0 with comprehensive type-safe parsing section
- `/docs/guides/json-typed-parsing.md`: New 400+ line guide with examples, best practices, and troubleshooting

**Breaking Changes:**
- None! Old JsonValue syntax still works for untyped parsing

**Known Limitations:**
- Lambda parameters in forEach/map don't track class types (requires full type inference)
- Optional fields (`field?: Type`) not yet supported - use manual Option<T> workaround if needed

**Phase 3 Skipped:**
- Optional fields deferred as general language feature (not JSON-specific)
- `tests/integration/proj_json/test_map.liva`: Updated
- `tests/integration/proj_json/test_parvec_json.liva`: Updated

**Coming in Phase 2:**
- Custom classes with serde derive
- Snake_case field conversion
- Optional fields with `field?: Type`
- Default values with `field: Type = value`
- Nested classes

## [0.9.11] - 2025-01-25

### Fixed - JsonValue Parallel Execution

**JsonValue.parvec() Support:**
- ✅ Fixed parallel execution for JsonValue from JSON.parse()
- JsonValue now converts to Vec with `.to_vec().into_par_iter()` instead of `.par_iter()`
- Lambda patterns correctly use `|x|` (owned) instead of `|&x|` (reference) for JsonValue parallel iteration
- Complete HTTP → JSON → parvec workflow now fully functional

**Code Generation Improvements:**
- Detect `is_direct_json` flag for JsonValue from JSON.parse()
- Par/ParVec adapters: generate `.to_vec().into_par_iter()` for JsonValue
- Lambda pattern generation: extended to handle Par/ParVec with JsonValue (no & prefix)

**Example Usage:**
```liva
// Complete integration: HTTP + JSON + Parallel Processing
async fn fetch_and_process() {
    let res, err = async HTTP.get("https://jsonplaceholder.typicode.com/posts")
    let posts = JSON.parse(res.body)
    
    // Parallel processing of JSON array - NOW WORKS! ✅
    posts.parvec().forEach(post => {
        console.log($"Post {post.id}: {post.title}")
    })
}
```

**Technical Details:**
- JsonValue is a wrapper over serde_json::Value, not a Vec
- `.par_iter()` requires IntoParallelRefIterator trait (not satisfied)
- `.to_vec().into_par_iter()` returns owned values (IntoParIter<JsonValue>)
- Lambda receives owned JsonValue, not reference

## [0.9.10] - 2025-01-25

### Fixed - Parser and Concurrency Support (Phase 6.4.3 - 2h)

**Parser Fix for Reserved Keywords:**
- ✅ Fixed parsing error with `.parvec()`, `.par()`, `.vec()` method calls
- Reserved keywords (Par, Vec, ParVec) can now be used as method names
- Added `parse_method_name()` helper that accepts both identifiers and keyword tokens

**Concurrency Policy Support:**
- ✅ **parvec() works on all arrays**: Parallel execution with Rayon
- ✅ Automatic rayon dependency detection via `ArrayAdapter::Par|ParVec`
- ✅ Correct code generation: `.par_iter()` for parallel, `.collect()` for map
- ✅ Import `use rayon::prelude::*` when parallel execution is detected

**Code Generation Fixes:**
- Map with parallel adapter: generates `.collect::<Vec<_>>()` (no `.cloned()`)
- Filter with parallel adapter: generates `.cloned().collect::<Vec<_>>()`
- Added rayon imports at top level (after liva_rt module)

**Comprehensive Tests:**
- ✅ 10 integration tests in `tests/integration/proj_json/`
  * test_parse_no_error.liva - JSON.parse without binding
  * test_for_in_loop.liva - for...in on JSON
  * test_dot_notation.liva - property access
  * test_foreach_arrow.liva - forEach with arrows
  * test_map.liva - map transformation
  * test_filter.liva - filter selection
  * test_chaining.liva - map then filter
  * test_objects_array.liva - array of objects
  * test_parvec_json.liva - parallel execution
  * test_comprehensive.liva - all features combined

**Example Files:**
- ✅ 4 comprehensive examples in `examples/`
  * json_natural_syntax.liva - v0.9.8 features demo
  * json_arrow_functions.liva - v0.9.9 features demo
  * json_parallel.liva - parvec() demo
  * json_api_processing.liva - Real-world API processing

**Example Usage:**
```liva
main() {
    let data = "[1, 2, 3, 4, 5, 6, 7, 8]"
    let numbers = JSON.parse(data)
    
    // Sequential
    let doubled = numbers.map(n => n.as_i32().unwrap() * 2)
    
    // Parallel 🔥 NEW!
    let par_doubled = numbers.parvec().map(n => n.as_i32().unwrap() * 2)
    
    par_doubled.forEach(n => print($"  {n}"))
}
```

**Technical Details:**
- Parser now distinguishes between identifiers and keyword tokens in method position
- Desugaring phase detects ArrayAdapter usage and sets `ctx.has_parallel = true`
- Cargo.toml generation includes rayon when parallel execution is detected
- Code generator emits appropriate iterator methods based on adapter type

## [0.9.9] - 2025-01-25

### Added - Arrow Functions for JSON Arrays (Phase 6.4.2 - 3h)

**Full Array Method Support for JSON:**
- ✅ **forEach with arrow functions**: `posts.forEach(post => print(post.title))`
- ✅ **map**: `numbers.map(n => n * 2)` - Transform JSON arrays
- ✅ **filter**: `numbers.filter(n => n > 25)` - Filter JSON arrays
- ✅ **find/some/every**: Complete array method support
- ✅ **Chaining**: `posts.filter(p => p.id > 5).forEach(p => print(p.title))`

**Implementation Details:**

**1. JsonValue Iterator Methods:**
- Added `.iter()` → returns `std::vec::IntoIter<JsonValue>` (owned clones)
- Added `.to_vec()` → converts to `Vec<JsonValue>`
- JsonValue already implements `Clone`, so iteration clones values

**2. Lambda Pattern Detection:**
- Tracks which variables are JsonValue via `json_value_vars` HashSet
- Detects when `map`/`filter`/`forEach` is called on JsonValue
- For normal arrays: generates `|&item|` (borrow from iterator)
- For JsonValue: generates `|item|` (owned values from `.iter()`)

**3. Vec<JsonValue> Handling:**
- Results of `.map()`/`.filter()` are `Vec<JsonValue>`
- Tracked separately to handle iteration properly
- Uses `.iter().cloned()` for Vec<JsonValue> to clone elements
- Avoids `.copied()` (which only works for Copy types)

**4. Type Conversion Methods:**
- Added all conversion methods to generated JsonValue:
  * `as_i32()`, `as_f64()`, `as_string()`, `as_bool()`
  * `is_null()`, `is_array()`, `is_object()`
  * `to_json_string()`
- Prevents string literal conversion for `get`/`get_field` methods

**Complete Example:**
```liva
main() {
    // HTTP request (v0.9.6)
    let res, err = async HTTP.get("https://jsonplaceholder.typicode.com/posts")
    
    if res.status == 200 {
        // Natural JSON parsing (v0.9.8)
        let posts = JSON.parse(res.body)
        
        // Arrow functions on JSON arrays (v0.9.9) ✅ NEW!
        posts.forEach(post => {
            print($"Post {post.id}: {post.title}")
        })
        
        // Map and filter work too ✅ NEW!
        let ids = posts.map(p => p.id)
        let filtered = posts.filter(p => p.id > 5)
        
        filtered.forEach(p => print($"Filtered: {p.title}"))
    }
}
```

**Technical Highlights:**
- Smart detection: distinguishes `JsonValue` (direct) from `Vec<JsonValue>` (derived)
- Memory efficient: uses cloning only when necessary
- Iterator consistency: `.iter()` on JsonValue matches `.into_iter()` semantics
- No breaking changes: normal arrays continue working as before

**Performance Notes:**
- JsonValue.iter() clones elements (JsonValue contains serde_json::Value)
- Acceptable for typical JSON use cases (small-medium datasets)
- For large datasets, consider streaming or direct serde_json manipulation

## [0.9.8] - 2025-01-25

### Added - Natural JSON Syntax (Phase 6.4.1 - 2h)

**Ergonomic JSON Improvements:**
- ✅ **JSON.parse without error binding**: `let posts = JSON.parse(body)` - Auto-unwraps or panics on error
- ✅ **for...in loops**: `for post in posts { ... }` - Natural iteration over JSON arrays
- ✅ **Dot notation**: `post.id`, `post.title` - Direct property access instead of brackets

**Implementation Details:**

**1. JSON.parse Auto-unwrap:**
- Detects single-binding pattern in VarDecl: `let posts = JSON.parse(...)`
- Generates: `.0.expect("JSON parse failed")` automatically
- No need for error binding when you want to panic on error

**2. IntoIterator for JsonValue:**
- Implemented `IntoIterator` trait on `JsonValue`
- Returns `std::vec::IntoIter<JsonValue>` for arrays
- Empty iterator for non-arrays
- Embedded in both liva_rt.rs and generated runtime

**3. Dot Notation for Properties:**
- Heuristic detection: if variable is not array/class → treat as JsonValue
- Generates `.get_field("property").unwrap()` automatically
- Works in: assignments, conditions, string templates, function args

**4. Smart Length Detection:**
- `JsonValue.length()` for JSON arrays/objects
- `.len()` for Rust arrays and strings
- Automatic detection based on variable tracking

**Complete Natural Example:**
```liva
main() {
  let res, err = async HTTP.get("https://api.example.com/posts?_limit=5")

  if err != "" {
    console.log($"Error: {err}")
  } else {
    if res.status == 200 {
      let posts = JSON.parse(res.body)  // ✅ No error binding
      for post in posts {                // ✅ for...in loop
        // ✅ Dot notation for properties
        console.log($"Post ID: {post.id}, Title: {post.title}")
      }
    }
  }
}
```

**Comparison:**

Before (v0.9.7):
```liva
let posts, jsonErr = JSON.parse(res.body)
if jsonErr == "" {
    let i = 0
    while i < posts.length {
        let post = posts[i]
        let id = post["id"]
        let title = post["title"]
        print($"Post {id}: {title}")
        i = i + 1
    }
}
```

After (v0.9.8):
```liva
let posts = JSON.parse(res.body)
for post in posts {
    print($"Post {post.id}: {post.title}")
}
```

**Code Changes:**
- Modified VarDecl generation to detect and unwrap JSON.parse
- Added IntoIterator impl to JsonValue (20 lines)
- Enhanced Member expression generation for JsonValue dot notation
- Smart .length() vs .len() detection based on context

## [0.9.7] - 2025-01-25

### Added - JSON Array/Object Support (Phase 6.4 - 3h)

**JsonValue Wrapper:**
- Created `JsonValue` struct wrapping `serde_json::Value` with Liva-friendly interface
- Implements `Display` trait for easy printing and string interpolation
- Provides type-safe methods for common JSON operations

**JSON Methods:**
- `length() -> usize` - Get array/object/string length
- `get(index: usize) -> Option<JsonValue>` - Array element access
- `get_field(key: &str) -> Option<JsonValue>` - Object field access
- `as_i32()`, `as_f64()`, `as_string()`, `as_bool()` - Type conversions
- `is_array()`, `is_object()`, `is_null()` - Type checking

**JSON Operations:**
- ✅ Array indexing: `arr[0]`, `arr[i]` - Access array elements
- ✅ Object key access: `obj["name"]` - Access object fields
- ✅ Length property: `arr.length` - Get array/object size
- ✅ String templates: `print($"Value: {jsonVar}")` - Direct interpolation
- ✅ Iteration support: Use `.length` with `while` loops

**Parser Support (Modified JSON.parse):**
- Changed return type from `(Option<Value>, Option<Error>)` to `(Option<JsonValue>, String)`
- Error messages as strings for consistency with HTTP client
- JsonValue automatically embedded in generated runtime

**Code Generation:**
- Added option_value_vars tracking for variables from tuple-returning functions
- Special handling for JsonValue.length() on Option<JsonValue>
- Heuristic detection of direct JsonValue variables (non-Option)
- String template unwrapping for Option<JsonValue> in interpolations
- Index access generates .get()/.get_field() calls automatically

**Semantic Analysis:**
- Made `.length` validation permissive for identifiers (validated at codegen)
- Allows `.length` on JSON variables without full type inference

**Working Example:**
```liva
main() {
    let res, err = async HTTP.get("https://api.example.com/posts?_limit=5")
    
    if err == "" && res.status == 200 {
        let posts, jsonErr = JSON.parse(res.body)
        
        if jsonErr == "" {
            let i = 0
            while i < posts.length {
                let post = posts[i]
                let id = post["id"]
                let title = post["title"]
                print($"Post {id}: {title}")
                i = i + 1
            }
        }
    }
}
```

**Limitations:**
- Direct `obj["key"]` in string templates (e.g., `$"{obj["key"]}"`) not supported due to parser limitations with nested quotes
- Workaround: use intermediate variables
- No `for...in` loop support yet (use `while` with `.length`)

**Bug Fixes:**
- ✅ Fixed hints.rs panic on empty error codes (added defensive guard)
- ✅ Fixed Option<Struct> consuming with multiple field access (use `.as_ref().unwrap()`)
- ✅ Fixed string template interpolation for option_value_vars

## [0.9.6] - 2025-01-25

### Added - HTTP Client (Phase 6.3 - 5h)

**HTTP Methods:**
- `HTTP.get(url: string) -> (Option<Response>, string)` - GET request
- `HTTP.post(url: string, body: string) -> (Option<Response>, string)` - POST request
- `HTTP.put(url: string, body: string) -> (Option<Response>, string)` - PUT request
- `HTTP.delete(url: string) -> (Option<Response>, string)` - DELETE request

**Response Object:**
- `status: i32` - HTTP status code (200, 404, etc.)
- `statusText: string` - Status text ("OK", "Not Found", etc.)
- `body: string` - Response body as string
- `headers: [string]` - Response headers

**Features:**
- ✅ Async by default using Liva's lazy evaluation (`async HTTP.get()`)
- ✅ Error binding pattern: `let response, err = async HTTP.get(url)`
- ✅ Tuple return type: `(Option<Response>, String)` for success/error
- ✅ 30-second timeout with reqwest
- ✅ TLS support via rustls (no OpenSSL dependency)
- ✅ Comprehensive error handling (network, DNS, timeout, HTTP errors)

**Implementation:**
- Runtime: 150+ lines in liva_rt with LivaHttpResponse struct
- Semantic Analysis: 120+ lines detecting HTTP.*, validation, async/fallible marking
- Parser: Enhanced parse_exec_call() to support `async HTTP.method()` syntax
- Code Generation: 300+ lines across 4 locations for HTTP support
- Dependencies: reqwest 0.11 with rustls-tls features

**Bug Fixes:**
- ✅ Fixed error binding code generation for tuple-returning functions
- ✅ Added returns_tuple field to TaskInfo for correct await generation
- ✅ Enhanced is_builtin_conversion_call() to detect Call wrapping MethodCall
- ✅ Fixed Option<Struct> field access to generate `value.unwrap().field`
- ✅ Prevented String error vars from being tracked as Option<Error>

**Examples:**
```liva
// Simple GET request
let response, err = async HTTP.get("https://api.example.com/data")
if err != "" {
    console.error($"Error: {err}")
} else {
    print($"Status: {response.status}")
    print($"Body: {response.body}")
}

// POST with data
let postResp, postErr = async HTTP.post("https://api.example.com/users", userData)
if postErr == "" {
    print($"Created! Status: {postResp.status}")
}
```

**Time Breakdown:**
- Design & Documentation: 30 min
- Setup & Dependencies: 30 min
- Runtime Implementation: 1.5 hours (all 4 methods)
- Semantic Analysis: 30 min (detection, validation)
- Parser Enhancement: 15 min (async MethodCall)
- Code Generation: 1.5 hours (HTTP calls, embedding, deps)
- Bug Fixes: 1 hour (error binding, tuple handling)
- Testing: 30 min (all methods verified)

**Tests:**
- ✅ test_http_simple.liva - Basic GET with error handling
- ✅ test_http_quick.liva - GET and DELETE
- ✅ examples/manual-tests/test_http_all.liva - Comprehensive (all 4 methods)

## [0.9.5] - 2025-01-24

### Added - Enhanced Pattern Matching (Phase 6.4 - 3.5h)

**Switch Expressions:**
- Switch can now be used as an expression (returns a value)
- Can be used anywhere an expression is valid
- All arms must return the same type

**Pattern Types:**
- **Literal patterns**: `1 => "one"`, `"hello" => greet()`, `true => yes()`
- **Wildcard pattern**: `_ => default_case` (catch-all)
- **Binding patterns**: `n => n * 2` (capture value in variable)
- **Range patterns**: `1..10`, `1..=10`, `..10`, `10..` (inclusive/exclusive)

**Pattern Guards:**
- Add conditional logic with `if` clauses: `x if x < 20 => "teenager"`
- Guards can use any boolean expression
- Guards have access to bound variables

**Exhaustiveness Checking (✅ NEW):**
- ✅ **Bool exhaustiveness**: Compiler ensures both `true` and `false` cases are covered
- Error `E0901`: Non-exhaustive pattern matching on bool
- Accepts wildcard `_` or binding patterns as catch-all
- Helpful error messages with suggestions
- Example:
  ```liva
  // ❌ Error: E0901 - missing 'false' case
  let result = switch flag {
      true => "yes"
  };
  
  // ✅ OK - both cases covered
  let result = switch flag {
      true => "yes",
      false => "no"
  };
  ```

**Implementation:**
- Added `Pattern` enum to AST (Literal, Wildcard, Binding, Range)
- Added `SwitchExpr`, `SwitchArm`, `SwitchBody` to AST
- Added `Token::Underscore` and `Token::DotDotEq` to lexer
- Implemented `parse_switch_expr()` and `parse_pattern()` in parser
- Switch expressions pass through IR as `Unsupported` (handled in codegen)
- Generate Rust `match` expressions with proper pattern translation
- Semantic validation for switch expressions and guards
- ✅ **Exhaustiveness checking** in `check_switch_exhaustiveness()`
- Full await detection for async switch expressions

**Testing:**
- Created `test_switch_expr.liva` with 5 comprehensive test cases
- Created `test_exhaustiveness.liva` with exhaustive patterns
- Created `test_exhaustiveness_error.liva` to verify E0901 error
- Created `test_exhaustiveness_complete.liva` with all scenarios
- All patterns working: literals, ranges, guards, bindings, wildcards
- Tested with integers, strings, booleans
- All 6 tests passing ✅

**Documentation:**
- Complete language reference: `docs/language-reference/pattern-matching.md` (650+ lines)
- Comprehensive design document: `docs/PHASE_6.4_PATTERN_MATCHING_DESIGN.md` (800+ lines)
- Pattern types, guards, exhaustiveness, examples, best practices
- Error codes: E0901 (non-exhaustive bool)

**Examples:**
```liva
// Basic literal matching
let result = switch x {
    1 => "one",
    2 => "two",
    _ => "other"
};

// Range patterns
let grade = switch score {
    90..=100 => "A",
    80..=89 => "B",
    70..=79 => "C",
    _ => "F"
};

// Pattern guards
let category = switch age {
    x if x < 13 => "child",
    x if x < 20 => "teenager",
    x if x < 65 => "adult",
    _ => "senior"
};

// Exhaustiveness checking
let result = switch flag {
    true => "yes",
    false => "no"  // Both cases required!
};
```

**Future Enhancements (v0.9.6+):**
- Full exhaustiveness checking for all types (int, string, enum)
- Tuple/array destructuring patterns
- Enum variant patterns
- Or patterns (`|` operator)

## [0.9.4] - 2025-01-21

### Added - File I/O Operations (Phase 6.2 - 2.5h)

**File API:**
- `File.read(path: string): (string?, Error?)` - Read entire file contents as string
- `File.write(path, content: string): (bool?, Error?)` - Write/overwrite file
- `File.append(path, content: string): (bool?, Error?)` - Append to end of file
- `File.exists(path: string): bool` - Check if file/directory exists
- `File.delete(path: string): (bool?, Error?)` - Delete file from filesystem

**Implementation:**
- Added `generate_file_function_call()` to code generator (82 lines)
- Rust backend using `std::fs`: `read_to_string`, `write`, `OpenOptions`, `Path::exists`, `remove_file`
- Extended `is_builtin_conversion_call()` to recognize File methods (except `exists`)
- Added `option_value_vars` tracking for proper string concatenation with Option types

**Features:**
- Error binding integration for all operations (except `exists`)
- UTF-8 file encoding
- Synchronous I/O (blocking operations)
- Graceful error handling for missing files, permissions, disk full scenarios

**Testing:**
- 5 basic tests in `test_file_simple.liva`
- 27 comprehensive tests in `test_file_complex.liva` covering all operations, edge cases, workflows
- All tests passing ✅

**Documentation:**
- Complete API reference: `docs/language-reference/file-io.md` (450 lines)
- Design document: `docs/PHASE_6.2_FILE_IO_API_DESIGN.md` (430 lines)
- Implementation summary: `docs/PHASE_6.2_FILE_IO_SUMMARY.md` (280 lines)
- Total: 1,160+ lines of documentation

### Fixed
- Option value variables now properly unwrap in string concatenation
- Error binding variables (first in tuple) tracked separately for type-safe string operations

## [0.9.3] - 2025-01-21

### Added - JSON Parsing & Serialization (Phase 6.1 - 4h)

**JSON API:**
- `JSON.parse(json: string): (any?, Error?)` - Parse JSON strings to Liva types
- `JSON.stringify(value: any): (string?, Error?)` - Serialize Liva values to JSON

**Implementation:**
- Added `generate_json_function_call()` to code generator
- Integrated `serde_json` crate for runtime JSON operations
- Extended `is_builtin_conversion_call()` to recognize JSON methods
- Error binding pattern support for both functions

**Type Mapping:**
- JSON → Liva: null→none, bool→bool, number→int/float, string→string, array→array, object→object
- Liva → JSON: Full bidirectional mapping with error handling

**Error Handling:**
- Parse errors: Invalid syntax, unexpected EOF, malformed numbers
- Stringify errors: Unsupported types (functions, tasks), circular references
- All errors use error binding pattern: `let result, err = JSON.parse(str)`

**Examples:**
```liva
// Parse JSON
let data, err = JSON.parse("{\"name\": \"Alice\", \"age\": 30}")
if err { fail err }

// Stringify
let json, err2 = JSON.stringify({name: "Bob", age: 25})
if err2 { fail err2 }
```

**Test Coverage:**
- `test_json_simple.liva` - Basic parse/stringify tests
- Tests valid JSON parsing
- Tests invalid JSON error handling
- Round-trip conversion tests

**Documentation:**
- `docs/language-reference/json.md` - Complete API reference (400 lines)
- Type mapping tables
- Error handling guide
- 4 complete examples

## [0.9.2] - 2025-10-23

### Added - Trait Aliases (Phase 5.10 - 2h)

**Intuitive Trait Aliases:**
- `Numeric` = Add + Sub + Mul + Div + Rem + Neg (all arithmetic)
- `Comparable` = Ord + Eq (equality and ordering)
- `Number` = Numeric + Comparable (complete number operations)
- `Printable` = Display + Debug (formatting)

**Implementation:**
- Added `aliases` HashMap to TraitRegistry
- `register_trait_aliases()` defines 4 built-in aliases
- `is_alias()` checks if constraint is an alias
- `expand_alias()` returns underlying traits
- `expand_constraints()` expands all aliases in a list
- Semantic analyzer expands aliases before registering constraints
- Code generation automatically expands aliases to Rust traits

**Examples:**
```liva
// Simple and intuitive
sum<T: Numeric>(a: T, b: T): T => a + b
max<T: Comparable>(a: T, b: T): T { ... }
clamp<T: Number>(value: T, min: T, max: T): T { ... }

// Mix with granular traits
formatAndCompare<T: Comparable + Display>(a: T, b: T) { ... }
debugCalc<T: Numeric + Printable>(a: T, b: T) { ... }

// Still can use granular for precise control
addOnly<T: Add>(a: T, b: T): T => a + b
```

**Test Coverage:**
- test_trait_aliases.liva (comprehensive test of all aliases)
- Tests mixing aliases with granular traits
- Verifies expansion to correct Rust bounds

**Documentation:**
- Updated generics.md with trait aliases section
- Added comparison table (aliases vs granular)
- Best practices guide
- Examples of mixing approaches

### Philosophy
Liva now offers **the best of both worlds**:
- **Beginners:** Use intuitive aliases (`Numeric`, `Comparable`, `Number`)
- **Advanced:** Use granular traits for precise control (`Add`, `Ord`, etc.)
- **Flexible:** Mix both approaches as needed

## [0.9.1] - 2025-10-23

### Added - Multiple Constraints & Type Arguments (Phase 5.9 - 3h)

**Type Arguments in Function Calls:**
- Added `type_args` field to CallExpr AST
- Parser recognizes `function<Type>(args)` syntax
- Handles both type keywords (float, bool, string) and identifiers
- Lookahead logic to distinguish `<` as type arg vs comparison
- Code generation with Rust turbofish operator `::< Type >`
- Examples: `identity<int>(42)`, `sum<float>(3.5, 2.5)`

**Multiple Constraints with + Operator:**
- Updated `TypeParameter` to use `Vec<String>` for constraints
- Parser supports `<T: Trait1 + Trait2 + Trait3>` syntax
- Semantic analyzer validates ALL constraints in vector
- Code generation emits `<T: Trait1 + Trait2>` format
- Composable constraint system (like Rust/Swift/C#)
- Examples:
  * `clamp<T: Ord + Add + Sub>(value, min, max)`
  * `printIfEqual<T: Eq + Display>(a, b)`
  * `average<T: Add + Div>(a, b, divisor)`

**Available Traits:**
- Arithmetic: Add, Sub, Mul, Div, Rem, Neg
- Comparison: Eq, Ord
- Utilities: Clone, Copy, Display, Debug
- Logical: Not

**Test Coverage:**
- test_multi_constraints.liva (comprehensive multi-trait tests)
- All arithmetic + comparison combinations validated
- Generates correct Rust trait bounds

**Documentation:**
- Updated generics.md with multiple constraints section
- Updated ROADMAP.md with Phase 5.9 completion
- All examples working end-to-end

### Changed
- TypeParameter AST now uses `constraints: Vec<String>` instead of `constraint: Option<String>`
- Display trait for TypeParameter now joins constraints with " + "

## [0.9.0] - 2025-10-23

### Added - Generics System (Phase 5 - CORE COMPLETE, 16.5h - Released! 🎉)

**Phase 5.1: Generic Syntax Design (2h) ✅**
- Complete specification in docs/language-reference/generics.md (785 lines)
- Syntax: `<T>`, `<T: Constraint>`, `<T, U>` for multiple parameters
- Monomorphization strategy (compile-time specialization like Rust)
- Standard library integration plan (Array<T>, Result<T,E>, Option<T>, Map<K,V>, Set<T>)
- Comprehensive examples and edge cases

**Phase 5.2: Parser & AST Extensions (3h) ✅**
- New `TypeParameter` struct with name and optional constraint
- Updated AST nodes: ClassDecl, TypeDecl, FunctionDecl, MethodDecl
- Implemented `parse_type_parameters()` function
- Parser handles `<T>`, `<T: Constraint>`, `<T, U>` syntax
- **Discovery:** Liva has no `class` keyword - classes are `Name<T> { }`
- Fixed codegen to emit proper generic Rust code:
  * `pub struct Name<T: Constraint>`
  * `impl<T: Constraint> Name<T> { }`
- Added `[T]` array type syntax support
- Parser handles type parameters in type annotations (T, U, etc.)
- Added `?` and `!` suffix parsing for Optional and Fallible types
- 11 parser test files with full insta snapshot coverage
- All tests passing (11/11)

**Phase 5.3: Code Generation (2.5h) ✅**
- Generic functions compile and execute correctly! 🎉
  * Example: `identity<T>(value: T): T => value`
  * Test output: Works with int, string, bool types
  * Generated Rust: `fn identity<T>(value: T) -> T { value }`
- Generic classes with single type parameter working! 🎉
  * Example: `Box<T> { value: T }`
  * Generates: `pub struct Box<T> { pub value: T }`
  * Impl blocks: `impl<T> Box<T> { pub fn new(value: T) -> Self { ... } }`
- Generic classes with multiple type parameters working! 🎉
  * Example: `Pair<T, U> { first: T, second: U }`
  * Generates: `pub struct Pair<T, U> { ... }`
  * All combinations tested: int/string, bool/float, string/int
- Array type annotations working! 🎉
  * Example: `firstInt(arr: [int]): int`
  * Generates: `fn first_int(arr: Vec<i32>) -> i32`
  * Tested with firstInt, lastInt, sum functions
- **No codegen changes needed** - infrastructure already supported it!
- Monomorphization delegated to Rust compiler (optimal)

**Known Issue:**
- Field access on method return values generates `["field"]` instead of `.field`
- Workaround: Assign to intermediate variable first

**Working Examples:**
- `examples/test_array_generic.liva` - identity<T> function
- `examples/test_generic_class.liva` - Box<T> class
- `examples/test_generic_methods.liva` - Pair<T,U> class
- `examples/test_array_syntax.liva` - Array type annotations

**Commits:** 8ee5bc1 (specification), ae39b05 (parser tests), d4dc6d2 (array syntax), 72c3878 (working generics!), 677c552 (generic classes), 5669a17 (multiple type params), 2d8c6d3 (docs update), 4b7d0fd (array types)

**Phase 5.4: Standard Library Validation (2h) ✅**
- Test `Option<T>` with generics working! 🎉
  * Created Option<T> class with isSome(), isNone() methods
  * Works with: int, string, bool types
  * File: `examples/test_option_generic.liva`
  * Compiles and executes correctly
- Test `Result<T, E>` with generics working! 🎉
  * Created Result<T,E> class with isSuccess(), isError() methods
  * Works with: Result<int,string>, Result<bool,int>
  * File: `examples/test_result_generic.liva`
  * Compiles and executes correctly

**Important Findings:**

✅ **What Works:**
- Generic classes instantiate correctly with different types
- Multiple type parameters work (`Result<T, E>`)
- Type safety enforced by Rust's type system
- Methods with `&self` work for predicates (bool returns)

⚠️ **Limitations Discovered:**

1. **Ownership Issue:**
   - Methods with `&self` cannot return `T` by value
   - Rust error: "cannot move out of `self.value` which is behind a shared reference"
   - Workaround: Access fields directly instead of getter methods
   - Future solution: Add Clone constraint or make methods consume self

2. **Semantic Analyzer Interference:**
   - Function names like `parseInt` trigger fallible builtin detection
   - Compiler tries to parse string literals instead of calling the function
   - Workaround: Use different names (`parseNumber` instead of `parseInt`)
   - Future solution: Improve semantic analysis to distinguish user functions

3. **VSCode Language Server Bug:**
   - LSP shows parse error on generic class declarations (`Option<T> {`)
   - Actual compiler works fine - error is only in IDE
   - Error message: "Expected LParen" (false positive)
   - Impact: Cosmetic only - doesn't affect compilation

**Commits:** 1594d4d (Option<T>), 17bbef2 (Result<T,E>)

**Phase 5.5: Type System Implementation (1h) ⏸️ PARTIAL**
- Type parameter validation implemented! ✅
  * Added `type_parameters` tracking to SemanticAnalyzer
  * Implemented scope management for type parameters
  * Enhanced `validate_type_ref()` to check T exists in scope
  * Validates type parameters in functions, classes, and methods
  * Methods inherit class type parameters correctly
  * File: `examples/test_type_param_validation.liva`
  * **Status:** Type validation working correctly
- Constraint checking deferred to v0.9.1
  * Advanced features like `T: Clone`, `T: Display` validation
  * Type inference for generic calls (implicit type arguments)
  * Type substitution for complex scenarios
- **Rationale:** Core generics are working. Advanced features can be added incrementally without blocking release.

**Commit:** 2c75280 (type parameter validation)

**Phase 5.7: Documentation & Examples (0.5h) ✅**
- Created comprehensive generics quick-start tutorial
  * File: `docs/guides/generics-quick-start.md` (338 lines)
  * Introduction to generics with motivation
  * Basic generic functions (identity<T>)
  * Generic classes (Box<T>, Pair<T,U>)
  * Array type annotations
  * Option<T> pattern with examples
  * Result<T,E> pattern with examples
  * Best Practices section (Do's and Don'ts)
  * Common Patterns (Stack<T>, Wrapper<T>)
  * Known Limitations clearly documented
  * "What's Next" roadmap for v0.9.1
  * Complete list of working examples
- Updated ROADMAP.md with Phase 5 completion status
- Updated CHANGELOG.md with full release notes

**Commit:** a45acec (tutorial), b6f1f5b (roadmap/changelog updates)

**Phase 5.8: Constraint Checking System (~5h) ✅**
- Implemented complete trait registry system
  * File: `src/traits.rs` (279 lines, 13 built-in traits)
  * Traits: Add, Sub, Mul, Div, Rem (arithmetic operators)
  * Traits: Eq, Ord (comparison operators)
  * Traits: Neg, Not (unary operators)
  * Traits: Clone, Display, Debug, Copy, Default (utility traits)
  * Automatic trait hierarchy (Ord requires Eq, Copy requires Clone)
  * Rust mapping: Add→std::ops::Add<Output=T> + Copy
- Enhanced semantic analyzer with constraint validation
  * `validate_binary_op_constraints()` - validates +, -, *, /, %, ==, !=, <, >, <=, >=
  * `validate_unary_op_constraints()` - validates unary -, !
  * E5001 error: Unknown trait constraint (with suggestions)
  * E5002 error: Missing constraint for operator usage
  * Integrated TraitRegistry into SemanticAnalyzer
- Updated codegen for complete Rust trait bounds
  * Generate bounds: `T: std::ops::Add<Output=T> + Copy`
  * Auto-include Copy for value return types
  * Handle implicit requirements (Ord includes Eq)
  * Updated generate_function() and generate_class()
- Created comprehensive test suite (4 files)
  * test_constraint_arithmetic.liva - All arithmetic operators (+, -, *, /, %, unary-)
  * test_constraint_comparison.liva - Ord tests (max, min, clamp), Eq tests
  * test_constraint_error.liva - E5002 error detection
  * test_generic_stack.liva - Real-world utility functions
- **All tests passing ✅** - Java-level completeness achieved

**Working Examples:**
```liva
// Arithmetic with constraints
sum<T: Add>(a: T, b: T): T => a + b                    // ✅ Works!
modulo<T: Rem>(a: T, b: T): T => a % b                  // ✅ Works!
negate<T: Neg>(value: T): T => -value                   // ✅ Works!

// Comparison with constraints
max<T: Ord>(a: T, b: T): T {                            // ✅ Works!
    if a > b { return a }
    return b
}
clamp<T: Ord>(value: T, min: T, max: T): T { ... }     // ✅ Works!

// Error detection
sum_no_constraint<T>(a: T, b: T): T => a + b           // ❌ E5002: Missing Add constraint
```

**Commit:** 240b814 (constraint checking system complete)

**Summary - v0.9.0 Production Ready:**

✅ **Completed Features:**
- Generic functions: `identity<T>(value: T): T`
- Generic classes: `Box<T>`, `Pair<T, U>`
- **Constraint checking: `sum<T: Add>`, `max<T: Ord>`, `negate<T: Neg>`** 🎉
- Array type annotations: `[int]` → `Vec<i32>`
- Option<T> and Result<T,E> validated and working
- Type parameter validation in semantic analyzer
- **13 built-in traits with automatic validation** 🎉
- 15+ tests passing (parser + integration)
- **4 constraint test files - all passing** 🎉
- 10 working example files

📊 **Statistics:**
- **Time:** 16.5 hours (110% of 15h estimate - exceeded expectations!)
- **Commits:** 18 (all on feature branch)
- **Files created:** 10 examples + 11 parser tests + 2 documentation files + 1 traits module
- **Lines added:** ~2,560 (parser, semantic, codegen, traits, examples, docs, tutorial)
- **Documentation:** 1,123 lines (785 generics.md + 338 quick-start.md)

🎯 **What You Can Do in v0.9.0:**
```liva
// Generic functions
identity<T>(value: T): T => value

// Generic functions with constraints 🎉
sum<T: Add>(a: T, b: T): T => a + b
max<T: Ord>(a: T, b: T): T { if a > b { return a } return b }
negate<T: Neg>(value: T): T => -value

// Generic classes
Box<T> { value: T }
Pair<T, U> { first: T, second: U }
Stack<T: Clone> { items: [T] }

// Array type annotations
sum(numbers: [int]): int { ... }

// Optional types
Option<T> { value: T, hasValue: bool }
Result<T, E> { value: T, error: E }

// All operators with constraints:
// Arithmetic: +, -, *, /, % (Add, Sub, Mul, Div, Rem)
// Comparison: >, <, >=, <=, ==, != (Ord, Eq)
// Unary: -, ! (Neg, Not)
```

⚠️ **Known Limitations (to be addressed in v0.9.1):**
1. Methods with `&self` cannot return `T` by value (use field access)
2. Type inference not implemented (must specify `<T>` explicitly)
3. Multiple constraints syntax `<T: Add + Mul>` not yet supported (use single constraint per function)
4. VSCode LSP shows false positive parse errors (compiler works fine)

**Deferred to v0.9.1:**
- Multiple constraints syntax (`<T: Add + Mul>`)
- Type inference for generic calls
- Advanced type system features

## [0.8.1] - 2025-10-23

**🎉 Phase 5: Enhanced Error Messages - Developer-friendly diagnostics**

Comprehensive error system with "Did you mean?" suggestions, enhanced context, error categorization, intelligent hints, code examples, and documentation links. Quality comparable to Rust and Elm.

### Added - Enhanced Error Messages (Phase 5 - 8h, 100% complete)

**Phase 5.1: "Did You Mean?" Suggestions (~2h) ✅**
- Levenshtein distance algorithm for typo detection
- Smart suggestions for:
  * Undefined variables (max 2 character edits)
  * Undefined functions
  * Undefined types/classes
  * Module import symbols
- `suggestions.rs` module (265 lines)
- Comprehensive test suite (test_suggestions.liva)

**Phase 5.2: Enhanced Error Context (~2h) ✅**
- Show 2 lines before and 2 lines after error location
- Precise token underlining using actual token length (not fixed 3 chars)
- Line numbers for all context lines
- Extended ErrorLocation structure:
  * `length: Option<usize>` - Token length for precise highlighting
  * `context_before: Option<Vec<String>>` - Lines before error
  * `context_after: Option<Vec<String>>` - Lines after error
- get_context_lines() function in semantic analyzer
- Visual improvements with exact caret positioning

**Phase 5.3: Error Categories & Codes (~1h) ✅**
- Organized error codes by category (E0xxx-E7xxx):
  * E0xxx: Lexical errors (invalid tokens, unclosed strings)
  * E1xxx: Syntax errors (grammar violations, unexpected tokens)
  * E2xxx: Semantic errors (undefined symbols, type errors)
  * E3xxx: Control flow errors (invalid return, break, continue)
  * E4xxx: Module errors (import failures, circular dependencies)
  * E5xxx: Concurrency errors (async/parallel misuse)
  * E6xxx: Standard library errors
  * E7xxx: I/O errors
- `error_codes.rs` module (190 lines) with ErrorCategory enum
- Category displayed in error messages: `[Semantic] E2001: ...`
- Complete error reference (ERROR_CODES.md, 316 lines)

**Phase 5.4: Intelligent Hints & Help (~2h) ✅**
- `hints.rs` module (176 lines) with automatic contextual help
- Functions for each error code:
  * `get_hint()` - Actionable advice
  * `get_example()` - Code examples showing correct vs incorrect
  * `get_doc_link()` - Links to documentation
  * `get_common_fixes()` - Common solutions by category
  * `get_tip()` - Additional improvement tips
- Automatic hint injection when manual help not provided
- Coverage for 15+ error codes with plans for more

**Phase 5.5: Documentation (~1h) ✅**
- ERROR_CODES.md (316 lines) - Complete error reference
- ERROR_HANDLING_GUIDE.md (522 lines) - Comprehensive guide
- TROUBLESHOOTING.md (493 lines) - Quick reference
- compiler-internals/enhanced-error-context.md (125 lines)
- Updated README.md with error system showcase
- Best practices and contributing guidelines

**Phase 5.6: VS Code Extension Integration (v0.4.0) ✅**
- Extended JSON error format with Phase 5 fields:
  * `suggestion`, `hint`, `example`, `doc_link`, `category`
- Auto-population of fields in `to_json()` methods
- Builder pattern for error creation:
  * `.with_suggestion()`, `.with_hint()`, `.with_example()`
  * `.with_doc_link()`, `.with_category()`
- Refactored semantic.rs to use builder pattern
- Cleaner, more maintainable error creation

### Changed
- Error message format now includes category badges
- ErrorLocation structure extended with context and length fields
- Error display shows more context (5 lines total vs 1 line)
- Float literals now use `_f64` suffix for type clarity
- Improved error messages with automatic suggestions

### Fixed
- Integration test float literal format (accept both 0.0 and 0_f64)
- async/parallel test with proper function calls

### Statistics
- **21 files changed**: +2,509 insertions, -60 deletions
- **3 new modules**: suggestions.rs, error_codes.rs, hints.rs
- **4 new documentation files**: 1,500+ lines total
- **8 test files**: Comprehensive coverage
- **10 commits**: Feature development complete

### Developer Experience Improvements
**Before:**
- Generic error messages
- No suggestions for typos
- Single line context
- Fixed 3-character underlines

**After:**
- Categorized errors with codes
- "Did you mean?" suggestions
- 5 lines of context (2 before, error, 2 after)
- Precise token-length underlining
- Automatic hints and examples
- Documentation links
- One-click fixes in VS Code

**Example Error:**
```
● E2001: Undefined variable [Semantic]
────────────────────────────────────────────────────────────
  → test.liva:5:12

   3 │     let userName = "Alice"
   4 │     
   5 │     console.log(usrName)
     │                 ^^^^^^^

  ⓘ Cannot find variable 'usrName' in current scope

  💡 Did you mean 'userName'?

  💡 Hint: Check spelling or declare the variable before use

  📝 Example:
     let userName = "value"
     console.log(userName)  // Correct

  📚 https://liva-lang.org/docs/errors/semantic#e2001
────────────────────────────────────────────────────────────
```

### Console API Enhancement
- `console.input()` function for user input
  * `console.input()` - Read without prompt
  * `console.input(message)` - Read with prompt
- ANSI color support:
  * `console.error()` - Red color
  * `console.warn()` - Yellow/amber color  
  * `console.success()` - Green color (NEW)
- Updated documentation and test suite

## [0.8.0] - 2025-10-21

**🚀 Phase 3: Module System - Multi-file projects**

Complete implementation of multi-file project support with JavaScript-style imports, automatic public/private visibility based on naming convention, circular dependency detection, and comprehensive error messages.

#### Added - Module System (Phase 3 - 17h actual, 3.1x faster than estimated)

**Phase 3.1: Design (2h) ✅ Complete**
- Module system specification document (400+ lines)
- Syntax comparison document (4 options evaluated)
- Implementation roadmap (TODO_MODULES.md, 700+ lines)
- Design decisions:
  * Public by default (no prefix)
  * Private with `_` prefix (consistent with Liva)
  * JavaScript-style import syntax
  * Relative paths (`./, ../`)

**Phase 3.2: Parser & AST (2h) ✅ Complete**
- Added `ImportDecl` struct to AST with Display trait
- Added `from` keyword to lexer
- Implemented `parse_import_decl()` method (~60 lines)
- Support for named imports: `import { a, b } from "path"`
- Support for wildcard imports: `import * as name from "path"`
- Handles comma-separated imports with trailing commas
- Comprehensive error handling for malformed imports

**Phase 3.3: Module Resolver (4h) ✅ Complete**
- Created `module.rs` with 400+ lines of infrastructure:
  * **Module struct**: Loads .liva files, extracts public/private symbols
  * **DependencyGraph**: DFS-based cycle detection, topological sort
  * **ModuleResolver**: Recursive loading with caching
- Path resolution for relative imports (`./, ../`)
- Symbol extraction based on `_` prefix
- Circular dependency detection with clear error messages (E4003)
- File not found errors with helpful context (E4004)
- Integration with compiler pipeline:
  * `compile_with_modules()` function
  * Auto-detection of import statements
  * `resolve_all()` returns modules in compilation order
- Unit tests for cycle detection (3 tests)
- Example files: math.liva, operations.liva, utils.liva

**Phase 3.4: Semantic Analysis (3h) ✅ Complete**
- Symbol validation during import resolution
- Check if imported symbols exist in target module
- Private symbol import detection (E4007 error)
- Name collision detection:
  * Import vs local definition (E4008)
  * Import vs import (E4009)
- Module context tracking for semantic analysis
- Integration with existing semantic analyzer

**Phase 3.6: Integration & Polish (in progress) 🔄**
- **Calculator Example** (65 lines, 3 modules):
  * `examples/calculator/calculator.liva` - Main entry point
  * `examples/calculator/basic.liva` - Basic operations (+, -, *, /)
  * `examples/calculator/advanced.liva` - Advanced operations
  * Demonstrates: named imports, public/private visibility
  * Tested: compiles and runs successfully
- **Documentation Updates**:
  * Updated `docs/getting-started/quick-start.md` with module section
  * Created `docs/guides/module-best-practices.md` (500+ lines)
  * Project structure patterns, naming conventions
  * Import patterns, visibility guidelines
  * Common patterns and anti-patterns
  * Performance tips and comprehensive examples
- **Error Message Polish**:
  * Enhanced E4003-E4009 with helpful hints
  * Specific suggestions (e.g., use aliases for collisions)
  * Better context for circular dependencies
  * Actionable guidance for resolving issues
- **Testing**:
  * Multi-module compilation verified
  * Calculator example runs correctly
  * Import syntax examples working
  * Error messages tested

**Phase 3.4: Semantic Analysis (3h) ✅ Complete (original)**
- Extended SemanticAnalyzer with import context:
  * New fields: imported_modules, imported_symbols
  * New function: analyze_with_modules() - accepts module context
  * validate_imports() - iterates all imports in program
  * validate_import() - validates single import declaration
- Import validation checks (180+ lines of code):
  * **E4004**: Module not found - with path resolution
  * **E4006**: Imported symbol not found in module
  * **E4007**: Cannot import private symbol (starts with _)
  * **E4008**: Import conflicts with local definition
  * **E4009**: Import conflicts with another import
- Path resolution:
  * Resolves relative paths (./,  ../)
  * Canonicalizes paths for matching
  * Fallback by filename matching
- Symbol registration:
  * Adds imported symbols to function registry
  * Permissive arity checking (accepts any arg count)
  * Prevents "function not found" errors
- Integration with compile_with_modules():
  * Builds module context map from resolved modules
  * Passes public_symbols and private_symbols
  * Uses analyze_with_modules() instead of analyze_with_source()

**Phase 3.5: Multi-File Code Generation (2h) ✅ Complete**
- Multi-file Rust project generation (180+ lines):
  * **generate_multifile_project()**: Main orchestrator
  * **generate_module_code()**: Per-module code generation
  * **generate_entry_point()**: main.rs with mod declarations
  * **generate_use_statement()**: Import → use conversion
  * **write_multifile_output()**: File writing system
- Import conversion:
  * `import { add } from "./math.liva"` → `use crate::math::add;`
  * `import { a, b } from "./m.liva"` → `use crate::m::{a, b};`
  * Wildcard imports with same-name alias simplified
- Visibility modifiers:
  * Functions without `_` prefix → `pub fn name()`
  * Private functions → `fn name()` (prefix removed)
  * Classes follow same rules
- Module declarations:
  * Automatic `mod` statements in main.rs
  * One .rs file per .liva module
- File structure:
  * src/main.rs - Entry point with mod declarations
  * src/math.rs, src/operations.rs, etc. - Module files
  * Cargo.toml - Project configuration
- Made CodeGenerator.output pub(crate) for access
- Made DesugarContext Clone-able for reuse
- Integration with compile_with_modules() pipeline
- Tested with examples/modules/test_import_syntax.liva:
  * ✅ Generates 4 files (main.rs + 3 modules)
  * ✅ Compiles successfully: `cargo build`
  * ✅ Executes correctly: "10 + 20 = 30"
- Documentation: docs/compiler-internals/multifile-codegen.md (650+ lines)

**Current Status:**
- ✅ Import syntax parsing works
- ✅ Module resolution with cycle detection works
- ✅ Loads all dependencies recursively
- ✅ Returns modules in topological order
- ✅ Import validation complete (all error codes)
- ✅ Symbol existence and visibility checks working
- ✅ Name collision detection working
- ✅ Multi-file Rust project generation working
- ✅ Pub/private visibility correctly applied
- ✅ Import → use conversion functional
- 📋 More examples and polish needed (Phase 3.6)

**Example:**
```liva
// math.liva
add(a: number, b: number): number => a + b
subtract(a: number, b: number): number => a - b
_internal_calc(x: number): number => x * 2  // Private

// main.liva
import { add } from "./math.liva"

main() {
    let result = add(10, 20)
    print($"Result: {result}")
}
```

**Generated Output:**
```
project/
├── Cargo.toml
└── src/
    ├── main.rs      (mod math; use crate::math::add; ...)
    └── math.rs      (pub fn add, pub fn subtract, fn internal_calc)
```

**Progress:**
- ✅ Phase 3.1: Design (2h)
- ✅ Phase 3.2: Parser (2h)
- ✅ Phase 3.3: Module Resolver (4h)
- ✅ Phase 3.4: Semantic Analysis (3h)
- ✅ Phase 3.5: Code Generation (2h)
- 📋 Phase 3.6: Integration & Examples (pending)
- **Total: 13h actual / 53h estimated (83% complete, 4x faster)**

**Next Steps:**
- Phase 3.6: Integration & Examples (9h) - Calculator example, polish, release

---

## [0.7.0] - 2025-10-20

**🎉 Phase 2 Complete: Standard Library - 37 functions implemented in one day!**

### Added - Standard Library (Phase 2)

#### Array Methods (9 methods)
- **`map(fn)`** - Transform each element
  - Sequential: `[1,2,3].map(x => x * 2)` → `[2,4,6]`
  - Uses `.iter().map(|&x| ...).collect()`
- **`filter(fn)`** - Keep elements matching predicate
  - Sequential: `[1,2,3,4,5].filter(x => x > 2)` → `[3,4,5]`
  - Uses `.iter().filter(|&&x| ...).copied().collect()`
- **`reduce(fn, initial)`** - Reduce to single value
  - Example: `[1,2,3,4,5].reduce((acc, x) => acc + x, 0)` → `15`
  - Uses `.iter().fold(initial, |acc, &x| expr)`
- **`forEach(fn)`** - Iterate with side effects
  - Example: `[1,2,3].forEach(x => print(x))`
  - Uses `.iter().for_each(|&x| { ... })`
- **`find(fn)`** - Find first element matching predicate
  - Example: `[1,5,10,15].find(x => x > 10)` → `Some(15)`
  - Returns `Option<T>`, uses `.iter().find(|&&x| pred).copied()`
- **`some(fn)`** - Check if any element matches
  - Example: `[2,4,6].some(x => x % 2 == 0)` → `true`
  - Returns `bool`, uses `.iter().any(|&x| pred)`
- **`every(fn)`** - Check if all elements match
  - Example: `[2,4,6].every(x => x % 2 == 0)` → `true`
  - Returns `bool`, uses `.iter().all(|&x| pred)`
- **`indexOf(value)`** - Find index of value
  - Example: `[10,20,30].indexOf(30)` → `2`
  - Returns `i32` (-1 if not found), uses `.iter().position(|&x| x == value)`
- **`includes(value)`** - Check if array contains value
  - Example: `[10,20,30].includes(20)` → `true`
  - Returns `bool`, uses `.iter().any(|&x| x == value)`

#### String Methods (11 methods)
- **`split(delimiter)`** - Split string into array
  - Example: `"apple,banana,orange".split(",")` → `["apple","banana","orange"]`
  - Returns `Vec<String>`, uses `.split(delim).map(|s| s.to_string()).collect()`
- **`replace(old, new)`** - Replace substring
  - Example: `"hello world".replace("world", "Liva")` → `"hello Liva"`
  - Uses `.replace(old, new)`
- **`toUpperCase()`** - Convert to uppercase
  - Example: `"hello".toUpperCase()` → `"HELLO"`
  - Uses `.to_uppercase()`
- **`toLowerCase()`** - Convert to lowercase
  - Example: `"HELLO WORLD".toLowerCase()` → `"hello world"`
  - Uses `.to_lowercase()`
- **`trim()`** - Remove leading/trailing whitespace
  - Example: `"  hello  ".trim()` → `"hello"`
  - Uses `.trim()`
- **`trimStart()`** - Remove leading whitespace
  - Example: `"  hello".trimStart()` → `"hello"`
  - Uses `.trim_start()`
- **`trimEnd()`** - Remove trailing whitespace
  - Example: `"hello  ".trimEnd()` → `"hello"`
  - Uses `.trim_end()`
- **`startsWith(prefix)`** - Check if starts with prefix
  - Example: `"hello.liva".startsWith("hello")` → `true`
  - Returns `bool`, uses `.starts_with(prefix)`
- **`endsWith(suffix)`** - Check if ends with suffix
  - Example: `"file.pdf".endsWith(".pdf")` → `true`
  - Returns `bool`, uses `.ends_with(suffix)`
- **`substring(start, end)`** - Extract substring
  - Example: `"Hello World".substring(0, 5)` → `"Hello"`
  - Uses slice syntax `[start as usize..end as usize].to_string()`
- **`charAt(index)`** - Get character at index
  - Example: `"Hello".charAt(0)` → `'H'`
  - Uses `.chars().nth(index as usize).unwrap_or(' ')` for UTF-8 safety
- **`indexOf(substring)`** - Find position of substring
  - Example: `"The quick brown fox".indexOf("quick")` → `4`
  - Returns `i32` (-1 if not found), uses `.find(substring).map(|i| i as i32).unwrap_or(-1)`
  - Disambiguated from array `indexOf` by argument type detection

#### Math Functions (9 functions)
- **`Math.sqrt(x)`** - Square root
  - Example: `Math.sqrt(16.0)` → `4.0`
  - Uses `x.sqrt()` method on f64
- **`Math.pow(base, exp)`** - Power/exponentiation
  - Example: `Math.pow(5.0, 2.0)` → `25.0`
  - Uses `base.powf(exp)` method on f64
- **`Math.abs(x)`** - Absolute value
  - Example: `Math.abs(-10.5)` → `10.5`
  - Uses `x.abs()` method with parentheses for unary expressions
- **`Math.floor(x)`** - Round down to integer
  - Example: `Math.floor(3.7)` → `3`
  - Returns `i32`, uses `x.floor() as i32`
- **`Math.ceil(x)`** - Round up to integer
  - Example: `Math.ceil(3.2)` → `4`
  - Returns `i32`, uses `x.ceil() as i32`
- **`Math.round(x)`** - Round to nearest integer
  - Example: `Math.round(3.5)` → `4`, `Math.round(3.4)` → `3`
  - Returns `i32`, uses `x.round() as i32`
- **`Math.min(a, b)`** - Minimum of two values
  - Example: `Math.min(10.5, 20.3)` → `10.5`
  - Uses `a.min(b)` method on f64
- **`Math.max(a, b)`** - Maximum of two values
  - Example: `Math.max(10.5, 20.3)` → `20.3`
  - Uses `a.max(b)` method on f64
- **`Math.random()`** - Random float between 0.0 and 1.0
  - Example: `Math.random()` → `0.8025414370953201` (varies)
  - Uses `rand::random::<f64>()`, automatically adds `rand` crate dependency

#### Type Conversion Functions (3 functions)
- **`parseInt(str)`** - Parse string to integer with error binding
  - Example: `let num, err = parseInt("42")` → `(42, None)`
  - Example: `let num, err = parseInt("abc")` → `(0, Some("Invalid integer format"))`
  - Returns tuple `(i32, Option<Error>)` using Liva's error binding pattern
  - Uses Rust's `.parse::<i32>()`  internally
- **`parseFloat(str)`** - Parse string to float with error binding
  - Example: `let value, err = parseFloat("3.14")` → `(3.14, None)`
  - Example: `let value, err = parseFloat("xyz")` → `(0.0, Some("Invalid float format"))`
  - Returns tuple `(f64, Option<Error>)` using Liva's error binding pattern
  - Uses Rust's `.parse::<f64>()` internally
- **`toString(value)`** - Convert any value to string
  - Example: `toString(42)` → `"42"`
  - Example: `toString(3.14)` → `"3.14"`
  - Example: `toString(true)` → `"true"`
  - Uses `format!("{}", value)` with Rust's Display trait
  - Works with all primitive types (Int, Float, Bool)

#### Console/IO Functions (6 functions - Hybrid Approach)
- **`print(...args)`** - Simple output for end users
  - Format: Display `{}` (clean, no quotes on strings)
  - Example: `print("Hello")` → `Hello`
  - Example: `print($"Name: {name}")` → `Name: Alice`
  - Uses `println!("{}", ...)` for user-facing output
  - Best for: Final output, status messages, simple scripts
- **`console.log(...args)`** - Debug output for developers
  - Format: Debug `{:?}` (shows structure, quotes strings)
  - Example: `console.log("Hello")` → `"Hello"` (with quotes)
  - Example: `console.log([1,2,3])` → `[1, 2, 3]`
  - Uses `println!("{:?}", ...)` for stdout
  - Best for: Debugging, data inspection, development
- **`console.error(...args)`** - Print to stderr
  - Format: Display `{}` (clean, readable error messages)
  - Example: `console.error("File not found!")` → `File not found!`
  - Uses `eprintln!("{}", ...)` for error output
  - Useful for separating errors from normal output
- **`console.warn(...args)`** - Print warning to stderr
  - Format: Display `{}` (clean, readable warning messages)
  - Example: `console.warn("Deprecated feature")` → `Warning: Deprecated feature`
  - Uses `eprintln!("Warning: {}", ...)` with prefix
  - Writes to stderr with "Warning: " prefix
- **`console.readLine()`** - Read line from stdin
  - Example: `let input = console.readLine()`
  - Generates inline block with `std::io::stdin().read_line()`
  - Returns trimmed string
  - Blocks until user provides input
- **`console.prompt(message)`** - Display message and read input
  - Example: `let name = console.prompt("Enter name: ")`
  - Generates inline block with `print!()` + `flush()` + `read_line()`
  - Returns trimmed string
  - Combines prompt display + input reading in one call

**Design Decision: Hybrid I/O Approach**
- **`print()`** - Simple function for beginners and user-facing output
  - Uses Display format `{}` for clean, readable output
  - Strings without quotes: `"Hello"` → `Hello`
  - Best for final results and status messages
- **`console.*`** - Professional namespace for debugging and development
  - Uses Debug format `{:?}` for detailed inspection
  - Strings with quotes: `"Hello"` → `"Hello"`
  - Arrays formatted: `[1, 2, 3]`
  - Organized under single namespace for discoverability
  - Familiar to JavaScript/Node.js developers

### Changed
- **`print()` now uses Display format `{}`** - Clean output for end users (no quotes)
- **`console.log()` uses Debug format `{:?}`** - Shows data structure for debugging
- **`console.error()` and `console.warn()` use Display format `{}`** - Readable error messages
- Extended `generate_method_call_expr()` in codegen.rs to handle string and console methods
- Added `generate_string_method_call()` function for string-specific code generation
- Added `generate_math_function_call()` function for Math namespace functions
- Added `generate_console_function_call()` function for console.* methods
- Added `parseInt()`, `parseFloat()`, `toString()`, `readLine()`, and `prompt()` as built-in functions
- Added `is_builtin_conversion_call()` helper to detect conversion functions
- Fixed VarDecl code generation to properly destructure tuples from built-in conversions
- Fixed method name sanitization - custom methods now convert camelCase to snake_case
- Method call detection now differentiates between array, string, Math, and console methods
- `indexOf` method now supports both arrays (numeric search) and strings (substring search)
- Float literals now generate with `_f64` suffix for explicit typing
- Added `has_random` flag to `DesugarContext` for dependency detection
- Auto-detection of `Math.random()` usage in desugaring phase
- Cargo.toml generation now includes `rand` crate when `Math.random()` is used

### Technical Details
- Array methods use iterator patterns for efficient processing
- String methods map directly to Rust standard library methods
- Math functions use namespace style (`Math.*`) and map to Rust f64 methods
- Console functions use namespace style (`console.*`) and map to println!/eprintln! macros
- Type conversion functions use error binding pattern: `(value, Option<Error>)` tuples
- parseInt/parseFloat return default values (0 or 0.0) on error with error message
- toString uses Rust's Display trait for universal type conversion
- readLine/prompt generate inline blocks with stdin operations
- All methods tested with comprehensive test suites
- Reused existing `MethodCall` and `CallExpr` AST nodes (no parser changes required)
- Fixed precedence issue with `abs()` by wrapping unary expressions in parentheses
- **Critical Fix**: Error binding variables now destructure correctly from built-in functions

### Tests
- Created 6 test files for array methods
- Created 4 test files for string methods
- Created 2 test files for Math functions (basic and comprehensive)
- Created 1 test file for Type Conversion functions (3 functions)
- Created 1 test file for Console/IO functions (3 console functions tested)
- Created 1 comprehensive comparison file (print vs console.log)
- All 37 functions (9 array + 11 string + 9 Math + 3 conversion + 5 I/O) implemented
- 35 functions verified working (readLine/prompt require interactive testing)

### Documentation
- Complete documentation for all stdlib functions in `docs/language-reference/stdlib/`
- Hybrid I/O approach extensively documented (print vs console.*)
- Updated README.md with Standard Library examples
- Updated ROADMAP.md with design decisions
- Created comparison examples showing format differences

---

## [0.6.1] - 2025-10-20

### Fixed
- Removed 26 compiler warnings across the codebase
  - Fixed unreachable code in codegen.rs after early returns
  - Fixed unreachable pattern in lowering.rs
  - Prefixed unused variables with `_`
  - Marked intentionally unused code with `#[allow(dead_code)]`
- Fixed `ir_codegen_string_templates` test
  - Implemented variable type tracking for correct format specifiers
  - Use `{}` for Display types (identifiers, literals, member access)
  - Use `{:?}` for Debug types (arrays, objects)
- Fixed error variable formatting in string templates
  - Added `.unwrap_or("")` when error variables used in templates
  - Prevents `Option<&str>` Display trait errors
- Fixed double semicolons in fire calls
  - Removed trailing semicolon from fire call closures
- Removed illegal class inheritance from test examples
  - Fixed `proj_comprehensive` test: replaced `Empleado : Persona` with composition
  - Clarified distinction between interface implementation (legal) and class inheritance (illegal)

### Changed
- All tests now pass (178 tests total)
  - 82 codegen tests
  - 50 desugar tests
  - 11 integration tests
  - 9 lexer tests
  - 21 parser tests
  - 6 property tests
  - 17 semantics tests
  - 3 doc tests
- Zero compiler warnings
- Improved code quality and consistency

### Documentation
- Updated TODO.md with detailed Phase 1 consolidation progress
- Skipped semantic unit tests restoration (incompatible with current AST)
- Verified all documentation correctly describes interface-only inheritance model

## [0.6.0] - 2025-10-19

### BREAKING CHANGES

#### Removed `protected` Visibility
- **Rationale:** Liva doesn't support class inheritance, only interface implementation
- **Migration:**
  - Old `_protectedField` → Now private (same syntax, different meaning)
  - Old `__privateField` → Now use `_privateField`
  - Protected methods no longer have special semantics

**Before (v0.5.x):**
```liva
Person {
  name: string        // public
  _age: number        // protected (accessible in subclasses)
  __ssn: string       // private (class-only)
}
```

**After (v0.6.0):**
```liva
Person {
  name: string        // public
  _age: number        // private (class-only)
}
```

### Added
- Interface implementation support
  - Classes can implement interfaces using `:` syntax
  - Interfaces are pure contracts (only method signatures, no fields)
  - Multiple interface implementation supported

### Changed
- Visibility model simplified to public/private only
- `_` prefix now means private (was protected)
- `__` prefix removed (no longer needed)

### Migration Guide

#### Class Inheritance → Composition
If you were using class inheritance patterns:

**Before:**
```liva
// This was never officially supported but might have worked
Empleado : Persona {
  empresa: string
}
```

**After:**
```liva
// Use composition instead
Empleado {
  persona: Persona
  empresa: string
  
  init(nombre: string, edad: number, empresa: string) {
    this.persona = Persona(nombre, edad)
    this.empresa = empresa
  }
}
```

#### Interfaces (Still Supported)
Interfaces remain unchanged:

```liva
// Interface (only signatures)
Animal {
  makeSound(): string
  getName(): string
}

// Implementation (has fields + implementations)
Dog : Animal {
  name: string
  
  constructor(name: string) {
    this.name = name
  }
  
  makeSound() => "Woof!"
  getName() => this.name
}
```

---

[Unreleased]: https://github.com/liva-lang/livac/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/liva-lang/livac/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/liva-lang/livac/releases/tag/v0.6.0
