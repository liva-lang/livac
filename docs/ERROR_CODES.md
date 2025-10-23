# Liva Error Codes Reference

This document provides a comprehensive reference of all error codes in the Liva compiler.

## Error Code Categories

Error codes in Liva follow a structured numbering system:

- **E0xxx**: General semantic errors (0000-0999)
- **E1xxx**: Lexer errors (1000-1999)
- **E2xxx**: Parser errors (2000-2999)
- **E3xxx**: Code generation errors (3000-3999)
- **E4xxx**: Module system errors (4000-4999)
- **E5xxx**: Type system errors (5000-5999)
- **E6xxx**: Concurrency errors (6000-6999)
- **E7xxx**: Error handling errors (7000-7999)

---

## E0xxx: General Semantic Errors

### E0000 - Generic Error
**Category:** General  
**Severity:** Error  
**Description:** Generic error used when no specific error code applies.

### E0001 - Interface Not Implemented
**Category:** Semantic  
**Severity:** Error  
**Description:** A class declares it implements an interface but doesn't provide all required methods.

**Example:**
```liva
interface Drawable {
  draw()
}

class Circle : Drawable {
  // Missing draw() method
}
```

### E0002 - Method Signature Mismatch
**Category:** Semantic  
**Severity:** Error  
**Description:** Method implementation doesn't match the interface signature.

**Example:**
```liva
interface Calculator {
  add(a: number, b: number): number
}

class BasicCalc : Calculator {
  add(a: number): number {  // Wrong signature
    return a
  }
}
```

---

## E1xxx: Lexer Errors

### E1000 - Lexer Error
**Category:** Lexing  
**Severity:** Error  
**Description:** Error during tokenization of source code (invalid characters, malformed tokens, etc.)

---

## E2xxx: Parser Errors

### E2000 - Parse Error
**Category:** Parsing  
**Severity:** Error  
**Description:** Syntax error during parsing (unexpected token, missing semicolon, etc.)

**Example:**
```liva
let x = 5
let y x + 10  // Missing '='
```

### E2001 - Invalid Execution Modifier
**Category:** Parsing  
**Severity:** Error  
**Description:** Invalid execution modifier in lambda expression.

### E2002 - Duplicate Execution Modifier
**Category:** Parsing  
**Severity:** Error  
**Description:** Duplicate execution modifier in lambda expression.

### E2003 - Invalid Loop Policy
**Category:** Parsing  
**Severity:** Error  
**Description:** Invalid policy modifier in for loop.

---

## E3xxx: Code Generation Errors

### E3000 - Code Generation Error
**Category:** Codegen  
**Severity:** Error  
**Description:** Error during IR to Rust code generation.

### E3001 - Compilation Failed
**Category:** Codegen  
**Severity:** Error  
**Description:** Generated Rust code failed to compile.

### E3002 - Build Failed
**Category:** Codegen  
**Severity:** Error  
**Description:** Cargo build process failed.

---

## E4xxx: Module System Errors

### E4003 - Invalid Module Path
**Category:** Modules  
**Severity:** Error  
**Description:** Module path is invalid or cannot be resolved.

**Example:**
```liva
import { add } from "../nonexistent/module"
```

### E4004 - Module Not Found
**Category:** Modules  
**Severity:** Error  
**Description:** Module file could not be found.

**Example:**
```liva
import { subtract } from "math"  // math.liva doesn't exist
```

### E4005 - Compilation Failed
**Category:** Modules  
**Severity:** Error  
**Description:** Module compilation failed during import.

### E4006 - Symbol Not Found
**Category:** Modules  
**Severity:** Error  
**Description:** Imported symbol doesn't exist in module.

**Example:**
```liva
import { multiply } from "math"  // multiply not exported
```

**Suggestion:** "Did you mean 'add'?"

### E4007 - Invalid Import Syntax
**Category:** Modules  
**Severity:** Error  
**Description:** Import statement has invalid syntax.

### E4008 - Empty Import List
**Category:** Modules  
**Severity:** Error  
**Description:** Import list cannot be empty.

**Example:**
```liva
import { } from "module"  // Empty import
```

### E4009 - Module Not Exported
**Category:** Modules  
**Severity:** Error  
**Description:** Attempting to import from module that doesn't export symbols.

---

## E5xxx: Type System Errors

### E5001 - Type Mismatch
**Category:** Types  
**Severity:** Error  
**Description:** Type incompatibility detected.

---

## E6xxx: Concurrency Errors

### E0401 - Invalid Concurrent Execution
**Category:** Concurrency  
**Severity:** Error  
**Description:** Invalid combination of concurrent execution modifiers.

### E0402 - Unsafe Concurrent Access
**Category:** Concurrency  
**Severity:** Error  
**Description:** Potentially unsafe concurrent access pattern detected.

### E0510 - Non-Send Capture
**Category:** Concurrency  
**Severity:** Warning  
**Description:** Move lambda captures value that may not be Send-safe for parallel execution.

### E0511 - Non-Sync Capture
**Category:** Concurrency  
**Severity:** Warning  
**Description:** Lambda captures value that may not be Sync-safe for parallel execution.

### E0602 - Duplicate Execution Modifier
**Category:** Concurrency  
**Severity:** Error  
**Description:** Duplicate execution modifiers on the same call.

**Example:**
```liva
async async fetchData()  // Duplicate 'async'
```

### E0603 - Not Awaitable
**Category:** Concurrency  
**Severity:** Error  
**Description:** Expression cannot be awaited.

**Examples:**
- Awaiting `par` call (completes eagerly)
- Awaiting literal value
- Awaiting non-async expression

### E0604 - Await Multiple Times
**Category:** Concurrency  
**Severity:** Error  
**Description:** Expression or handle awaited more than once.

### E0605 - Await in Parallel Loop
**Category:** Concurrency  
**Severity:** Error  
**Description:** `await` is not allowed inside `for par` or `for parvec` loops.

---

## E7xxx: Error Handling Errors

### E0701 - Fallible Function Without Binding
**Category:** Error Handling  
**Severity:** Error  
**Description:** Fallible function called without error binding.

**Example:**
```liva
divide(10, 0)  // divide can fail
```

**Fix:**
```liva
let result, err = divide(10, 0)
```

### E0702 - Invalid Chunk Size
**Category:** Error Handling  
**Severity:** Error  
**Description:** `chunk` option must be a positive integer.

### E0703 - Invalid Prefetch Size
**Category:** Error Handling  
**Severity:** Error  
**Description:** `prefetch` option must be a positive integer.

### E0704 - Invalid Thread Count
**Category:** Error Handling  
**Severity:** Error  
**Description:** `threads` option must be a positive integer.

### E0705 - SIMD Without Vector Policy
**Category:** Error Handling  
**Severity:** Error  
**Description:** `simdWidth` option requires `for vec` or `for parvec` policy.

### E0706 - Invalid SIMD Width
**Category:** Error Handling  
**Severity:** Error  
**Description:** `simdWidth` value must be a positive integer.

---

## Error Severity Levels

- **Error**: Compilation cannot proceed
- **Warning**: Potential issue but compilation can continue
- **Note**: Informational message

---

## Getting Help

For more information about specific errors:

1. **Error Code Search**: Use the error code (e.g., E0701) to search this document
2. **Compiler Output**: Read the full error message with context and suggestions
3. **Documentation**: Check language reference at `docs/language-reference/`
4. **Examples**: See `examples/manual-tests/` for error demonstrations

---

## Contributing

When adding new error codes:

1. Choose appropriate category range
2. Add entry to this document
3. Include description, example, and fix
4. Add test case in `tests/`
5. Update error message with helpful context
