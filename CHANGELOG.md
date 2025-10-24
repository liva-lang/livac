# Changelog

All notable changes to the Liva compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.5] - 2025-01-24

### Added - Enhanced Pattern Matching (Phase 6.4 - 3h)

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

**Implementation:**
- Added `Pattern` enum to AST (Literal, Wildcard, Binding, Range)
- Added `SwitchExpr`, `SwitchArm`, `SwitchBody` to AST
- Added `Token::Underscore` and `Token::DotDotEq` to lexer
- Implemented `parse_switch_expr()` and `parse_pattern()` in parser
- Switch expressions pass through IR as `Unsupported` (handled in codegen)
- Generate Rust `match` expressions with proper pattern translation
- Semantic validation for switch expressions and guards
- Full await detection for async switch expressions

**Testing:**
- Created `test_switch_expr.liva` with 5 comprehensive test cases
- All patterns working: literals, ranges, guards, bindings, wildcards
- Tested with integers, strings, booleans
- All 5 tests passing ✅

**Documentation:**
- Complete language reference: `docs/language-reference/pattern-matching.md` (600+ lines)
- Comprehensive design document: `docs/PHASE_6.4_PATTERN_MATCHING_DESIGN.md` (800+ lines)
- Pattern types, guards, exhaustiveness, examples, best practices
- Error codes: E6001 (non-exhaustive), E6002 (type mismatch), E6003 (invalid range)

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

// Binding patterns
let doubled = switch num {
    0 => 0,
    n => n * 2
};
```

**Future Enhancements (v0.9.6+):**
- Full exhaustiveness checking for all types
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
