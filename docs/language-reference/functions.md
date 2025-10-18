# Functions

Complete reference for function declarations, syntax variations, and best practices in Liva.

## Table of Contents
- [Function Declaration](#function-declaration)
- [Arrow Functions (One-Liners)](#arrow-functions-one-liners)
- [Block Functions](#block-functions)
- [Parameters](#parameters)
- [Return Types](#return-types)
- [Async Inference](#async-inference)
- [Fallibility](#fallibility)
- [Visibility](#visibility)
- [Best Practices](#best-practices)

---

## Function Declaration

### Basic Syntax

Liva supports **two function styles**:

1. **Arrow functions** (one-liners with `=>`)
2. **Block functions** (multi-statement with `{}`)

```liva
// Arrow function (one-liner)
add(a, b) => a + b

// Block function (multi-statement)
calculateTotal(items) {
  let total = 0.0
  for item in items {
    total = total + item.price
  }
  return total
}
```

---

## Arrow Functions (One-Liners)

### Basic Arrow Functions

```liva
// Simple arithmetic
add(a, b) => a + b
multiply(x, y) => x * y
square(n) => n * n

// Boolean logic
isAdult(age) => age >= 18
isEven(n) => n % 2 == 0

// String manipulation
greet(name) => $"Hello, {name}!"
```

### With Type Annotations

```liva
add(a: number, b: number): number => a + b
greet(name: string): string => $"Hello, {name}!"
isPositive(value: float): bool => value > 0.0
```

### Characteristics

- **Single expression**: Must be a single expression
- **Implicit return**: No `return` keyword needed
- **Concise**: Best for simple transformations

---

## Block Functions

### Basic Block Functions

```liva
calculateTotal(items) {
  let total = 0.0
  for item in items {
    total = total + item.price
  }
  return total
}
```

### Explicit Return

```liva
max(a, b) {
  if a > b {
    return a
  }
  return b
}
```

### Early Returns

```liva
processUser(user) {
  if user.age < 18 {
    return "Minor - cannot proceed"
  }
  
  if user.active == false {
    return "Inactive user"
  }
  
  return "Active adult user"
}
```

### Void Functions

Functions without explicit return:

```liva
printMessage(msg: string) {
  print($"[LOG] {msg}")
  // No return statement = returns void
}
```

---

## Parameters

### Basic Parameters

```liva
greet(name) => $"Hello, {name}!"
add(a, b) => a + b
```

### Typed Parameters

```liva
calculateTax(amount: number, rate: float): float {
  return amount * rate
}
```

### Default Parameters

```liva
greet(name: string, greeting: string = "Hello") => $"{greeting}, {name}!"

main() {
  print(greet("Alice"))              // Output: Hello, Alice!
  print(greet("Bob", "Welcome"))     // Output: Welcome, Bob!
}
```

### Multiple Parameters

```liva
formatFullName(first: string, middle: string, last: string): string {
  return $"{first} {middle} {last}"
}

main() {
  let name = formatFullName("John", "Fitzgerald", "Kennedy")
  print(name)  // Output: John Fitzgerald Kennedy
}
```

---

## Return Types

### Explicit Return Types

```liva
add(a: number, b: number): number => a + b
divide(a: float, b: float): float => a / b
getUsername(id: number): string => $"user_{id}"
```

### Inferred Return Types

```liva
// Compiler infers return type
add(a, b) => a + b  // Inferred: number
greet(name) => $"Hello, {name}!"  // Inferred: string
```

### Void Returns

```liva
logMessage(msg: string) {
  print(msg)
  // No explicit return = void
}
```

### Optional Returns

```liva
findUser(id: number): string? {
  if id == 1 {
    return "Alice"
  }
  return null  // Optional return
}
```

### Array Returns

```liva
getNumbers(): [number] => [1, 2, 3, 4, 5]

getUsers(): [{ name: string, age: number }] => [
  { name: "Alice", age: 25 },
  { name: "Bob", age: 30 }
]
```

---

## Async Inference

### Automatic Async Detection

Liva **automatically infers** if a function is async:

```liva
// This function is automatically marked as async
fetchUser(id: number): string {
  let userData = async getFromDatabase(id)
  return userData.name
}
```

**No `async` keyword needed in function declaration!**

### How It Works

The compiler detects async when:
1. Function contains `async` calls
2. Function contains `await` expressions
3. Function calls another async function

### Async Propagation

```liva
// Base async function
fetchData(url: string): string {
  // Async I/O operation
  return async httpGet(url)
}

// Automatically async because it calls fetchData()
processData(url: string): string {
  let data = fetchData(url)
  return data.toUpperCase()
}

// Also async (transitive)
main() {
  let result = processData("https://api.example.com")
  print(result)
}
```

### Manual Await

```liva
main() {
  let userTask = task async fetchUser(1)
  // ... do other work ...
  let user = await userTask  // Explicit await
  print(user)
}
```

---

## Fallibility

### Fallible Functions

Functions that use `fail` are **fallible**:

```liva
// Fallible: returns Result<number, Error>
divide(a: number, b: number): number {
  if b == 0 {
    fail "Division by zero"
  }
  return a / b
}
```

### Ternary with Fail

```liva
checkAge(age: number): string => age >= 18 ? "Adult" : fail "Minor"
```

### Multiple Fail Points

```liva
validateUser(username: string, password: string): string {
  if username == "" {
    fail "Username cannot be empty"
  }
  if password.length < 8 {
    fail "Password too short"
  }
  if password == "12345678" {
    fail "Password too weak"
  }
  return $"User {username} validated"
}
```

### Calling Fallible Functions

```liva
main() {
  // Error binding syntax
  let result, err = divide(10, 2)
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Result: {result}")
  }
}
```

### Non-Fallible Functions

Regular functions never fail:

```liva
add(a: number, b: number): number => a + b

main() {
  let sum, err = add(10, 20)  // err is always ""
  print(sum)  // Output: 30
}
```

---

## Visibility

Functions use **identifier-based visibility**:

### Public Functions

```liva
// Public: starts with letter
calculatePrice(quantity, price) => quantity * price
getUserData(id) => fetchFromDatabase(id)
```

### Protected Functions

```liva
// Protected: starts with single underscore
_validateInput(data) => data != null && data.length > 0
_checkPermissions(user) => user.role == "admin"
```

### Private Functions

```liva
// Private: starts with double underscore
__internalHelper(value) => value * 2 + 1
__secretAlgorithm(data) => /* ... */
```

### Usage

```liva
// In same file/module
main() {
  let price = calculatePrice(5, 10)    // ✅ Public
  let valid = _validateInput("data")   // ✅ Protected (same module)
  let result = __internalHelper(42)    // ✅ Private (same file)
}
```

---

## Best Practices

### Function Naming

```liva
// ✅ Good: Verb-based, descriptive
calculateTotal(items) => /* ... */
fetchUserData(id) => /* ... */
validateEmail(email) => /* ... */

// ❌ Bad: Unclear purpose
doStuff(x) => /* ... */
process(data) => /* ... */
```

### Arrow vs Block

```liva
// ✅ Good: Arrow for simple operations
double(x) => x * 2
isEven(n) => n % 2 == 0

// ✅ Good: Block for complex logic
processOrder(order) {
  validateOrder(order)
  let total = calculateTotal(order.items)
  let tax = calculateTax(total)
  return total + tax
}
```

### Type Annotations

```liva
// ✅ Good: Annotate public APIs
export calculateTax(amount: number, rate: float): float => amount * rate

// ⚠️ Acceptable: Inference for internal functions
_helperSum(a, b) => a + b
```

### Single Responsibility

```liva
// ✅ Good: One responsibility
calculateSubtotal(items) => items.reduce((sum, item) => sum + item.price, 0)
calculateTax(subtotal, rate) => subtotal * rate
calculateTotal(subtotal, tax) => subtotal + tax

// ❌ Bad: Multiple responsibilities
processEverything(items, taxRate) {
  let subtotal = items.reduce((sum, item) => sum + item.price, 0)
  let tax = subtotal * taxRate
  let total = subtotal + tax
  sendToDatabase(total)
  sendEmail(total)
  return total
}
```

### Error Handling

```liva
// ✅ Good: Clear error messages
divide(a: number, b: number): number {
  if b == 0 {
    fail "Cannot divide by zero"
  }
  return a / b
}

// ❌ Bad: Vague errors
divide(a: number, b: number): number {
  if b == 0 {
    fail "Error"
  }
  return a / b
}
```

### Default Parameters

```liva
// ✅ Good: Sensible defaults
createUser(name: string, role: string = "user", active: bool = true) {
  return { name, role, active }
}

// ❌ Bad: Magic numbers without defaults
createConnection(host, port, timeout) {  // What's the default timeout?
  // ...
}
```

---

## Summary

| Feature | Syntax | Example |
|---------|--------|---------|
| **Arrow Function** | `name(params) => expr` | `add(a, b) => a + b` |
| **Block Function** | `name(params) { ... }` | `calculate() { return 42 }` |
| **Typed Params** | `name(x: type)` | `greet(name: string)` |
| **Return Type** | `name(): type` | `add(): number` |
| **Default Param** | `name(x = val)` | `greet(name = "Guest")` |
| **Fallible** | Uses `fail` | `divide(a, b) => b == 0 ? fail "..." : a / b` |
| **Async** | Auto-inferred | Compiler detects async calls |

### Quick Reference

```liva
// Arrow function (one-liner)
add(a, b) => a + b
double(x) => x * 2

// Block function
calculateTotal(items) {
  let sum = 0
  for item in items {
    sum = sum + item.price
  }
  return sum
}

// With types
divide(a: number, b: number): number => a / b

// Fallible
safeDivide(a: number, b: number): number {
  if b == 0 fail "Division by zero"
  return a / b
}

// Calling fallible
let result, err = safeDivide(10, 2)
```

---

**Next**: [Classes →](classes.md)

**See Also**:
- [Error Handling](error-handling.md)
- [Concurrency](concurrency.md)
- [Variables](variables.md)
