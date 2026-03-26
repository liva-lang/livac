# Liva Error Codes Reference

## Error Code Ranges

| Range | Category | Description |
|-------|----------|-------------|
| **E0000-E0099** | Semantic | Core semantic errors (interfaces, variables, types) |
| **E0300-E0399** | Destructuring | Destructuring & pattern binding errors |
| **E0400-E0699** | Concurrency | Async, parallel, Send/Sync errors |
| **E0700-E0799** | Error Handling | Fallibility, loop options |
| **E0900-E0999** | Pattern Matching | Exhaustiveness checks |
| **E1xxx** | Lexer | Tokenization errors |
| **E2xxx** | Parser | Syntax parsing errors |
| **E3xxx** | Codegen | Code generation & build errors |
| **E4xxx** | Module | Module import & resolution errors |
| **E5xxx** | Type System | Type checking & inference errors |
| **E9xxx** | Interop | Rust interop errors |

## E0xxx: General Semantic Errors

| Code | Name | Description |
|------|------|-------------|
| E0000 | Generic Error | No specific code applies |
| E0001 | Interface Not Implemented / Variable Already Defined | Missing interface methods or duplicate variable |
| E0002 | Method Signature Mismatch / Constant Already Defined | Method doesn't match interface or duplicate constant |
| E0003 | Class Is Not an Interface | Trying to implement a non-interface |
| E0004 | Circular Type Alias | Type alias references itself |
| E0005 | Invalid `.length` Access | `.length` only on strings, bytes, arrays |
| E0006 | Invalid HTTP Call | Wrong number of arguments to HTTP method |
| E0007 | Unknown HTTP Method | Only: `HTTP.get()`, `.post()`, `.put()`, `.delete()` |

## E0300-E0399: Destructuring Errors

| Code | Description |
|------|-------------|
| E0301 | Field not found on destructured type |
| E0302 | Duplicate binding in destructuring pattern |
| E0303 | Array destructure on non-array type |
| E0304 | Tuple destructure on non-tuple type |
| E0310 | Duplicate function parameter name |
| E0311 | Parameter field not found in destructuring |
| E0312 | Duplicate binding in parameter destructuring |

## E0400-E0699: Concurrency Errors

| Code | Description |
|------|-------------|
| E0401 | Invalid concurrent execution combination |
| E0402 | Unsafe concurrent access pattern |
| E0510 | Non-Send capture in parallel lambda |
| E0511 | Non-Sync capture in parallel lambda |
| E0602 | Duplicate execution modifier (`async async`) |
| E0603 | Not awaitable (e.g., `await 42`, `await par`) |
| E0604 | Await same expression/handle multiple times |
| E0605 | `await` inside `for par`/`for parvec` |

## E0700-E0799: Error Handling & Loop Options

| Code | Description |
|------|-------------|
| E0701 | Fallible function called without error binding — use `let val, err = f()` or `f() or fail` |
| E0702 | Invalid `chunk` size (must be positive int) |
| E0703 | Invalid `prefetch` size |
| E0704 | Invalid `threads` count |
| E0705 | `simdWidth` requires `for vec`/`for parvec` |
| E0706 | Invalid `simdWidth` value |

## E0900-E0999: Pattern Matching Exhaustiveness

| Code | Description |
|------|-------------|
| E0901 | Non-exhaustive bool match — missing `true` or `false` |
| E0902 | Non-exhaustive integer match — needs `_` wildcard |
| E0903 | Non-exhaustive string match — needs `_` wildcard |
| E0904 | Non-exhaustive enum match — missing variant(s). Cover all variants or add `_` |
| E0906 | Incompatible or-pattern bindings — `\|` alternatives must bind same variables |

## E1xxx–E2xxx: Lexer & Parser Errors

| Code | Description |
|------|-------------|
| E1000 | Lexer error (invalid characters, unclosed strings/comments) |
| E2000 | Parse error (unexpected token, missing punctuation) |
| E2001 | Invalid execution modifier |
| E2002 | Duplicate execution modifier |
| E2003 | Invalid loop policy |
| E2004 | Undefined interface |

## E3xxx: Code Generation Errors

| Code | Description |
|------|-------------|
| E3000 | IR to Rust codegen error |
| E3001 | Generated Rust code failed to compile |
| E3002 | Cargo build failed |

## E4xxx: Module System Errors

| Code | Description |
|------|-------------|
| E4003 | Invalid module path |
| E4004 | Module file not found |
| E4005 | Imported module failed to compile |
| E4006 | Symbol not found (compiler suggests similar names) |
| E4007 | Invalid import syntax |
| E4008 | Empty import list |
| E4009 | Symbol is private (`_` prefixed) |

## E5xxx: Type System Errors

| Code | Description |
|------|-------------|
| E5001 | Type mismatch |
| E5002 | Missing trait constraint — add the required bound (e.g., `<T: Ord>`) |
| E5003 | Wrong number of type arguments for generic type alias |

## E9xxx: Interop Errors

| Code | Description |
|------|-------------|
| E9002 | Cannot override internal crate version. Internal crates: `tokio`, `serde`, `serde_json`, `reqwest`, `rayon`, `rand`. Use `features [...]` instead. |

## Severity Levels

| Level | Meaning |
|-------|---------|
| **Error** | Compilation cannot proceed |
| **Warning** (W prefix) | Potential issue, compilation continues |
| **Note** | Informational message |
