# Variables and Constants

Complete reference for variable declarations and constant definitions in Liva.

## Table of Contents
- [Variable Declarations](#variable-declarations)
- [Constants](#constants)
- [Type Annotations](#type-annotations)
- [Initialization](#initialization)
- [Destructuring](#destructuring) ⭐ ENHANCED!
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

---

## Destructuring

### Multiple Assignment (Destructuring)

**⭐ Enhanced in v0.10.2 and v0.10.3**

#### Array Destructuring

Extract multiple values from arrays:

```liva
// Simple array destructuring
let [a, b] = [10, 20]
print($"{a}, {b}")  // Output: 10, 20

// With more elements
let [x, y, z] = [1, 2, 3]
print($"x={x}, y={y}, z={z}")  // Output: x=1, y=2, z=3

// Skip elements with empty slots
let [first, , third] = [1, 2, 3]
print($"{first}, {third}")  // Output: 1, 3
```

#### Object Destructuring

Extract fields from objects:

```liva
class User {
    id: int
    name: string
    email: string
}

// Extract specific fields
let {id, name} = User { id: 1, name: "Alice", email: "alice@example.com" }
print($"User {id}: {name}")  // Output: User 1: Alice

// Field renaming
let {name: userName, email: userEmail} = user
print($"{userName} <{userEmail}>")
```

#### Rest Patterns

Capture remaining elements:

```liva
// Array rest pattern
let [head, ...tail] = [1, 2, 3, 4, 5]
print($"First: {head}")      // Output: First: 1
print($"Rest: {tail}")       // Output: Rest: [2, 3, 4, 5]

// Object rest pattern
let {id, ...rest} = user
print($"ID: {id}, Other data: {rest}")
```

#### Nested Destructuring

Destructure nested structures:

```liva
// Nested arrays
let [[a, b], [c, d]] = [[1, 2], [3, 4]]
print($"{a}, {b}, {c}, {d}")  // Output: 1, 2, 3, 4

// Nested objects
let {address: {city, country}} = user
print($"{city}, {country}")
```

#### Error Binding (Fallible Functions)

Special syntax for functions that can fail:

```liva
// Error binding syntax
let result, err = divide(10, 2)
if err != "" {
  print($"Error: {err}")
} else {
  print($"Result: {result}")
}

// With async/parallel
let data, error = async fetchUserData(userId)
if error != "" {
  print($"Failed to fetch: {error}")
}
```

#### Type Annotations with Destructuring

```liva
// Array with type
let [x, y]: [int] = [10, 20]

// Object with type
let {id, name}: User = getUser(1)

// Rest pattern with type
let [first, ...rest]: [string] = ["a", "b", "c"]
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
| **Array Destructuring** | `let [a, b] = [1, 2]` | Extract array elements |
| **Object Destructuring** | `let {id, name} = user` | Extract object fields |
| **Rest Pattern** | `let [head, ...tail] = arr` | Capture remaining elements |
| **Shadowing** | Allowed | Inner scope shadows outer |

### Quick Reference

```liva
// Variables
let age = 25
let name: string = "Alice"

// Array destructuring
let [x, y] = [10, 20]
let [first, , third] = [1, 2, 3]
let [head, ...tail] = [1, 2, 3, 4]

// Object destructuring
let {id, name} = user
let {name: userName} = user

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
