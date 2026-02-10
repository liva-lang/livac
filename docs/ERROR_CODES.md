# Liva Error Codes Reference

This document provides a comprehensive reference of all error codes in the Liva compiler.

## Error Code Categories

Error codes in Liva follow a structured numbering system:

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

---

## E0xxx: General Semantic Errors

### E0000 - Generic Error
Generic error used when no specific error code applies.

### E0001 - Interface Not Implemented / Variable Already Defined
A class declares it implements an interface but doesn't provide all required methods, or a variable is declared multiple times in the same scope.

```liva
Animal {
    makeSound(): string
}

Dog : Animal {
    // Error: Missing makeSound() method
}
```

### E0002 - Method Signature Mismatch / Constant Already Defined
Method implementation doesn't match the interface signature, or a constant is redeclared.

### E0003 - Class Is Not an Interface
Attempting to implement a class that is not an interface.

### E0004 - Circular Type Alias
A type alias references itself, creating a circular definition.

```liva
type Node = Node  // Error: Circular reference
```

### E0005 - Invalid `.length` Access
`.length` is only available on strings, bytes, and arrays.

```liva
let x = 42
let n = x.length  // Error: number has no .length
```
**Fix:** Use `.count()` for iterables, or check the type supports `.length`.

### E0006 - Invalid HTTP Call
HTTP method called with wrong number of arguments.

```liva
let resp, err = async HTTP.get()  // Error: requires 1 argument
```
**Fix:**
```liva
let resp, err = async HTTP.get("https://api.example.com")
let resp, err = async HTTP.post("https://api.example.com", body)
```

### E0007 - Unknown HTTP Method
Calling an HTTP method that doesn't exist.

**Available:** `HTTP.get()`, `HTTP.post()`, `HTTP.put()`, `HTTP.delete()`

---

## E0xxx: Destructuring Errors (E0300-E0399)

### E0301 - Field Not Found
Field doesn't exist on the type being destructured.

### E0302 - Duplicate Binding
Same binding name used twice in a destructuring pattern.

### E0303 - Array Destructure Mismatch
Attempting to array-destructure a non-array type.

### E0304 - Tuple Destructure Mismatch
Attempting to tuple-destructure a non-tuple type.

### E0310 - Duplicate Parameter Name
Function parameter declared multiple times.

```liva
add(x: number, x: number) => x + x  // Error: duplicate 'x'
```

### E0311 - Parameter Field Not Found
Field not found when destructuring a parameter.

### E0312 - Parameter Duplicate Binding
Duplicate binding in a parameter destructuring pattern.

---

## E0xxx: Concurrency Errors (E0400-E0699)

### E0401 - Invalid Concurrent Execution
Invalid combination of concurrent execution modifiers.

### E0402 - Unsafe Concurrent Access
Potentially unsafe concurrent access pattern detected.

### E0510 - Non-Send Capture
Move lambda captures value that may not be `Send`-safe for parallel execution.

### E0511 - Non-Sync Capture
Lambda captures value that may not be `Sync`-safe for parallel execution.

### E0602 - Duplicate Execution Modifier
Duplicate execution modifiers on the same call.

```liva
async async fetchData()  // Error: duplicate 'async'
```

### E0603 - Not Awaitable
Expression cannot be awaited.

```liva
let x = await 42          // Error: cannot await a literal
let y = await par calc()  // Error: par completes eagerly
```

### E0604 - Await Multiple Times
Expression or handle awaited more than once.

### E0605 - Await in Parallel Loop
`await` is not allowed inside `for par` or `for parvec` loops.

---

## E0xxx: Error Handling & Loop Options (E0700-E0799)

### E0701 - Fallible Function Without Binding
Fallible function called without error binding.

```liva
divide(10, 0)  // Error: divide can fail
```
**Fix:**
```liva
let result, err = divide(10, 0)
if err { print($"Error: {err}") }
```

### E0702 - Invalid Chunk Size
`chunk` option must be a positive integer.

### E0703 - Invalid Prefetch Size
`prefetch` option must be a positive integer.

### E0704 - Invalid Thread Count
`threads` option must be a positive integer.

### E0705 - SIMD Without Vector Policy
`simdWidth` option requires `for vec` or `for parvec` policy.

### E0706 - Invalid SIMD Width
`simdWidth` value must be a positive integer.

---

## E0xxx: Pattern Matching Exhaustiveness (E0900-E0999)

### E0901 - Non-Exhaustive Bool Match
Boolean pattern matching doesn't cover all cases.

```liva
let msg = switch flag {
    true => "yes"
    // Error: missing 'false' case
}
```

### E0902 - Non-Exhaustive Integer Match
Integer pattern matching is not exhaustive — requires wildcard `_`.

```liva
let msg = switch num {
    0 => "zero",
    1 => "one"
    // Error: integers are infinite, need _ wildcard
}
```

### E0903 - Non-Exhaustive String Match
String pattern matching is not exhaustive — requires wildcard `_`.

### E0906 - Incompatible Or-Pattern Bindings
Or-patterns (`|`) must bind the same variables in all alternatives.

---

## E1xxx: Lexer Errors

### E1000 - Lexer Error
Error during tokenization (invalid characters, unclosed strings/comments).

---

## E2xxx: Parser Errors

### E2000 - Parse Error
Syntax error during parsing (unexpected token, missing punctuation).

### E2001 - Invalid Execution Modifier
Invalid execution modifier in expression.

### E2002 - Duplicate Execution Modifier
Duplicate execution modifier in expression.

### E2003 - Invalid Loop Policy
Invalid policy modifier in for loop.

### E2004 - Undefined Interface
Interface referenced in class declaration doesn't exist.

---

## E3xxx: Code Generation Errors

### E3000 - Code Generation Error
Error during IR to Rust code generation.

### E3001 - Compilation Failed
Generated Rust code failed to compile with `rustc`.

### E3002 - Build Failed
Cargo build process failed.

---

## E4xxx: Module System Errors

### E4003 - Invalid Module Path
Module path is invalid or cannot be resolved.

### E4004 - Module Not Found
Module file could not be found at the specified path.

### E4005 - Module Compilation Failed
An imported module failed to compile.

### E4006 - Symbol Not Found
Imported symbol doesn't exist in the module. Compiler suggests similar names.

### E4007 - Invalid Import Syntax
Import statement has invalid syntax.

### E4008 - Empty Import List
Import list cannot be empty.

### E4009 - Module Not Exported
Symbol exists but is private (prefixed with `_`).

---

## E5xxx: Type System Errors

### E5001 - Type Mismatch
Type incompatibility detected during type checking.

### E5002 - Missing Trait Constraint
Generic type parameter used with an operator that requires a trait bound.

```liva
max<T>(a: T, b: T): T {
    if a > b { return a }  // Error: T needs Ord constraint
    return b
}
```
**Fix:** Add constraint: `max<T: Ord>(a: T, b: T): T`

### E5003 - Type Argument Count Mismatch
Wrong number of type arguments provided for a generic type alias.

```liva
type Pair<A, B> = (A, B)
let p: Pair<int> = (1, 2)  // Error: expects 2 type args, got 1
```

---

## Error Severity Levels

| Level | Meaning |
|-------|---------|
| **Error** | Compilation cannot proceed |
| **Warning** (W prefix) | Potential issue, compilation continues |
| **Note** | Informational message |

---

## Getting Help

1. **Error Code Search**: Use the error code (e.g., E0701) to search this document
2. **Compiler Output**: Full error message includes context, suggestions, and doc links
3. **Language Reference**: See `docs/language-reference/` for detailed guides
4. **Quick Reference**: See `docs/QUICK_REFERENCE.md` for syntax overview
