# Variables and Constants

Complete reference for variable declarations and constant definitions in Liva.

## Table of Contents
- [Variable Declarations](#variable-declarations)
- [Constants](#constants)
- [Type Annotations](#type-annotations)
- [Initialization](#initialization)
- [Scoping Rules](#scoping-rules)
- [Error Binding](#error-binding)
- [Best Practices](#best-practices)

---

## Variable Declarations

### Basic Syntax

```liva
let name = "Alice"
let age = 25
let height = 1.75
let isActive = true
```

Variables are declared using the `let` keyword with **type inference** by default.

### Type Annotations (Optional)

```liva
let name: string = "Alice"
let age: number = 25
let height: float = 1.75
let isActive: bool = true
```

Type annotations are **optional** but recommended for:
- Public APIs
- Complex types
- Documentation clarity

### Mutability

All variables declared with `let` are **mutable**:

```liva
let counter = 0
counter = counter + 1  // ✅ Valid
counter = 5           // ✅ Valid
```

Liva does not have immutable variables at the syntax level. Use `const` for compile-time constants.

---

## Constants

### Basic Syntax

```liva
const PI = 3.14159
const MAX_USERS = 100
const APP_VERSION = "v1.0.0"
```

Constants:
- Must be assigned at declaration
- **Cannot be reassigned**
- Use `SCREAMING_SNAKE_CASE` by convention
- Are evaluated at **compile time** when possible

### Type Annotations

```liva
const MAX_CONNECTIONS: number = 1000
const API_ENDPOINT: string = "https://api.example.com"
const DEBUG_MODE: bool = false
```

### Differences from `let`

| Feature | `let` | `const` |
|---------|-------|---------|
| Mutability | Mutable | Immutable |
| Reassignment | ✅ Allowed | ❌ Forbidden |
| Type Annotation | Optional | Optional |
| Initialization | Required | Required |
| Scoping | Block-scoped | Block-scoped |

---

## Type Annotations

### When to Use

```liva
// ✅ Good: Public API function
calculatePrice(quantity: number, price: float): float => quantity * price

// ✅ Good: Complex return type
fetchUsers(): [{ name: string, age: number }] => [/* ... */]

// ⚠️ Acceptable: Simple inference
let sum = 10 + 20  // Inferred as number
```

### Supported Type Syntax

```liva
// Primitive types
let age: number = 25
let height: float = 1.75
let name: string = "Alice"
let isActive: bool = true
let initial: char = 'A'

// Array types
let numbers: [number] = [1, 2, 3]
let names: [string] = ["Alice", "Bob"]

// Optional types
let maybeAge: number? = null
let userName: string? = "Alice"

// Fallible types (for error binding)
let result: number! = divide(10, 2)  // May fail
```

---

## Initialization

### Required Initialization

All variables must be initialized:

```liva
let x = 10        // ✅ Valid
let y: number     // ❌ Compile error: uninitialized variable
```

### Multiple Assignment (Destructuring)

```liva
// Array destructuring
let a, b = [10, 20]
print($"{a}, {b}")  // Output: 10, 20

// Error binding (fallible functions)
let result, err = divide(10, 2)
if err != "" {
  print($"Error: {err}")
} else {
  print($"Result: {result}")
}
```

---

## Scoping Rules

### Block Scope

Variables are **block-scoped** (like JavaScript `let`):

```liva
main() {
  let x = 10
  
  if true {
    let y = 20
    print(x)  // ✅ Accessible: 10
    print(y)  // ✅ Accessible: 20
  }
  
  print(x)  // ✅ Accessible: 10
  print(y)  // ❌ Compile error: y not in scope
}
```

### Function Scope

```liva
calculate() {
  let result = 42
  return result
}

main() {
  print(result)  // ❌ Compile error: result not defined
}
```

### Shadowing

Variables can shadow outer scopes:

```liva
main() {
  let x = 10
  
  if true {
    let x = 20  // Shadows outer x
    print(x)    // Output: 20
  }
  
  print(x)      // Output: 10
}
```

### Loop Scope

```liva
main() {
  for i in 1..5 {
    let squared = i * i
    print(squared)
  }
  
  print(i)       // ❌ Compile error: i not in scope
  print(squared) // ❌ Compile error: squared not in scope
}
```

---

## Error Binding

### Basic Error Binding

For **fallible functions** (those using `fail`):

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

main() {
  let result, err = divide(10, 2)
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Result: {result}")
  }
}
```

### How It Works

1. **Two bindings**: `result, err`
2. **Error binding** (`err`): Contains error message or empty string
3. **Result binding** (`result`): Contains return value or default value

### Error Binding with Async/Par

Error binding works seamlessly with concurrency:

```liva
// Async error binding (lazy await on first use)
let asyncResult, asyncErr = async divide(20, 4)
print($"Async result: {asyncResult}")  // Implicitly awaits here

// Parallel error binding (lazy join on first use)
let parResult, parErr = par heavyComputation(100)
print($"Par result: {parResult}")  // Implicitly joins here
```

### Ignoring Errors

Use `_` or a dummy variable to ignore errors:

```liva
let value, _ = divide(10, 2)  // Ignore error
print(value)
```

### Non-Fallible Functions

For regular functions (no `fail`), `err` is always empty:

```liva
multiply(a: number, b: number) => a * b

main() {
  let result, err = multiply(5, 3)  // err will be ""
  print($"Result: {result}")  // Output: Result: 15
}
```

---

## Best Practices

### Naming Conventions

```liva
// ✅ Good: Descriptive names
let userCount = 0
let isAuthenticated = true
let totalPrice = 99.99

// ❌ Bad: Single letters (except loops)
let u = 0
let x = true
let p = 99.99

// ✅ Good: Loop variables
for i in 1..10 { }
for item in items { }
```

### Constants

```liva
// ✅ Good: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT = 3
const API_BASE_URL = "https://api.example.com"

// ❌ Bad: lowercase or camelCase
const maxRetryCount = 3
const apiBaseUrl = "https://api.example.com"
```

### Type Annotations

```liva
// ✅ Good: Annotate public APIs
calculateTax(amount: number): number => amount * 0.15

// ✅ Good: Annotate complex types
fetchUsers(): [{ name: string, age: number, active: bool }] {
  // ...
}

// ⚠️ Acceptable: Let inference work
let sum = 10 + 20
let name = "Alice"
```

### Error Handling

```liva
// ✅ Good: Always check errors for fallible calls
let result, err = divide(a, b)
if err != "" {
  print($"Error: {err}")
  return
}
// Use result safely here

// ❌ Bad: Ignoring errors without reason
let result, _ = divide(a, b)  // Only if you're sure it won't fail
```

### Initialization

```liva
// ✅ Good: Initialize close to usage
main() {
  // ... some code
  let sum = calculate()
  print(sum)
}

// ❌ Bad: Unnecessary early initialization
main() {
  let sum = 0  // Not used for 50 lines
  // ... lots of code
  sum = calculate()
  print(sum)
}
```

---

## Summary

| Feature | Syntax | Notes |
|---------|--------|-------|
| **Variable** | `let x = 10` | Mutable, block-scoped |
| **Constant** | `const MAX = 100` | Immutable, block-scoped |
| **Type Annotation** | `let x: number = 10` | Optional |
| **Error Binding** | `let val, err = fallibleFn()` | For fallible functions |
| **Destructuring** | `let a, b = [1, 2]` | Multiple assignment |
| **Shadowing** | Allowed | Inner scope shadows outer |

### Quick Reference

```liva
// Variables
let age = 25
let name: string = "Alice"
let x, y = [10, 20]

// Constants
const PI = 3.14159
const MAX: number = 100

// Error binding
let result, err = divide(10, 2)
if err != "" {
  print($"Error: {err}")
}

// With concurrency
let asyncVal, asyncErr = async fetchUser(1)
let parVal, parErr = par heavyCompute(100)
```

---

**Next**: [Functions →](functions.md)

**See Also**:
- [Error Handling](error-handling.md)
- [Types](types.md)
- [Concurrency](concurrency.md)
