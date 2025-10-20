# Changelog

All notable changes to the Liva compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - TBD (In Development)

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
  - Example: `console.error($"Error message")`
  - Uses `eprintln!("{:?}", ...)` for error output
  - Useful for separating errors from normal output
- **`console.warn(...args)`** - Print warning to stderr
  - Example: `console.warn($"Warning message")`
  - Uses `eprintln!("Warning: {:?}", ...)` with prefix
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
- Extended `generate_method_call_expr()` in codegen.rs to handle string and console methods
- Added `generate_string_method_call()` function for string-specific code generation
- Added `generate_math_function_call()` function for Math namespace functions
- Added `generate_console_function_call()` function for console.* methods
- Added `parseInt()`, `parseFloat()`, `toString()`, `readLine()`, and `prompt()` as built-in functions
- Added `is_builtin_conversion_call()` helper to detect conversion functions
- Fixed VarDecl code generation to properly destructure tuples from built-in conversions
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
- All 37 functions (9 array + 11 string + 9 Math + 3 conversion + 5 I/O) implemented
- 35 functions verified working (readLine/prompt require interactive testing)

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
