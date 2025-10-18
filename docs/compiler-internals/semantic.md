# Semantic Analysis

The semantic analyzer validates type correctness, infers async functions, detects fallibility, and performs program-wide validation.

## Location

**File**: `src/semantic.rs` (2,067 lines)

## Overview

The semantic analyzer performs **three passes**:

1. **Definition Collection**: Gather types, functions, signatures
2. **Async Inference**: Detect which functions are async (transitive)
3. **Validation**: Type checking, fallibility checks, awaitable validation

## Key Responsibilities

### 1. Async Inference

**Automatically detects** if a function is async:

```rust
// Compiler infers this is async
fetchUser(id: number): string {
  let data = async httpGet($"/users/{id}")  // Contains async call
  return data.body
}
```

**Fixed-point iteration**: Repeats until no new async functions found.

### 2. Fallibility Detection

Tracks functions containing `fail`:

```rust
divide(a: number, b: number): number {
  if b == 0 fail "Division by zero"  // Detected as fallible
  return a / b
}
```

Fallible functions:
- Return `Result<T, liva_rt::Error>` in codegen
- Can use error binding: `let val, err = divide(10, 2)`

### 3. Type Checking

- **Function signatures**: Parameter types, return types
- **Variable types**: Tracked in symbol table
- **Type compatibility**: Ensures assignments are valid

### 4. Awaitable Validation

Enforces rules for `task` and `fire`:

- **Task handles** must be awaited before function returns
- **Fire calls** cannot be awaited
- **Pending tasks** tracked across scopes

### 5. Data-Parallel Policy Validation

Validates `for` loop policies and options:

```rust
// ❌ Invalid: seq cannot use simdWidth
for seq x in items with simdWidth 4 { }  // Error!

// ✅ Valid: parvec can use simdWidth
for parvec x in items with simdWidth 4 { }
```

**Rules enforced**:
- `seq`: No parallel/SIMD options
- `par`: Can use `chunk`, `threads`, `schedule`, `prefetch`, `reduction`, `detect`
- `vec`: Can use `simdWidth`, `ordered`
- `parvec`: Combines `par` and `vec` options

### 6. Error Binding Validation

Validates error binding patterns:

```rust
// ✅ Valid: Fallible function with 2 bindings
let result, err = divide(10, 2)

// ❌ Invalid: Non-fallible with 2 bindings
let result, err = add(10, 2)  // add() doesn't fail

// ✅ Valid: Can ignore error
let result, _ = divide(10, 2)
```

## Architecture

### SemanticAnalyzer

```rust
pub struct SemanticAnalyzer {
    async_functions: HashSet<String>,
    fallible_functions: HashSet<String>,
    types: HashMap<String, TypeInfo>,
    functions: HashMap<String, FunctionSignature>,
    external_modules: HashSet<String>,
    current_scope: Vec<HashMap<String, Option<TypeRef>>>,
    awaitable_scopes: Vec<HashMap<String, AwaitableInfo>>,
    source_file: String,
    source_code: String,
    source_map: Option<SourceMap>,
}
```

### TypeInfo

```rust
struct TypeInfo {
    name: String,
    fields: HashMap<String, (Visibility, TypeRef)>,
    methods: HashMap<String, (Visibility, bool)>, // (visibility, is_async)
}
```

### FunctionSignature

```rust
struct FunctionSignature {
    params: Vec<Option<TypeRef>>,
    return_type: Option<TypeRef>,
    is_async: bool,
    defaults: Vec<bool>,
}
```

## Public API

```rust
pub fn analyze(program: Program, source_file: &str, source_code: &str) 
    -> Result<Program>
```

Returns validated and annotated `Program`.

## Error Codes

| Code | Description |
|------|-------------|
| **E3001** | Async call outside async context |
| **E3002** | Unawaited task handle |
| **E3003** | Awaiting non-awaitable value |
| **E3004** | Fire call cannot be awaited |
| **E3005** | Await in data-parallel loop |
| **E3006** | Invalid policy option combination |
| **E3007** | Invalid chunk size |
| **E3008** | Invalid thread count |
| **E3009** | Invalid SIMD width |
| **E3010** | Error binding mismatch |

## Examples

### Async Inference

**Input AST**:
```liva
fetchData(url: string): string {
  let response = async httpGet(url)
  return response.body
}
```

**After Semantic Analysis**:
- `fetchData.is_async_inferred = true`
- Added to `async_functions` set

### Fallibility Detection

**Input AST**:
```liva
validateAge(age: number) {
  if age < 0 fail "Negative age"
  if age > 150 fail "Too old"
}
```

**After Semantic Analysis**:
- `validateAge.contains_fail = true`
- Added to `fallible_functions` set

### Error Binding Validation

**Valid**:
```liva
let result, err = divide(10, 2)  // ✅ fallible function, 2 bindings
```

**Invalid**:
```liva
let result, err = add(10, 2)  // ❌ E3010: Non-fallible with 2 bindings
```

## Summary

- **2,067 Lines**: Comprehensive semantic validation
- **Three Passes**: Collect → Infer → Validate
- **Async Inference**: Transitive, fixed-point algorithm
- **Fallibility Tracking**: Detects `fail` usage
- **Policy Validation**: Data-parallel correctness
- **Rich Errors**: E3xxx codes with source context

**Next**: [IR (Intermediate Representation) →](ir.md)

**See Also**:
- [Error System](error-system.md)
- [Concurrency](../language-reference/concurrency.md)
- [Error Handling](../language-reference/error-handling.md)
