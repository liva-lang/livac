# Functions: Advanced

Reference for async inference, fallibility, visibility, function references, and best practices in Liva.

## Table of Contents
- [Async Inference](#async-inference)
- [Fallibility](#fallibility)
- [Visibility](#visibility)
- [Function References](#function-references) ⭐ NEW!
- [Best Practices](#best-practices)
- [Summary](#summary)

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

### Usage

```liva
// In same file/module
main() {
  let price = calculatePrice(5, 10)    // ✅ Public
  let valid = _validateInput("data")   // ✅ Private (same module)
}
```

---

## Function References

**⭐ New in v1.1.0**

Pass function names or instance methods directly as callbacks, without writing a lambda wrapper.

### Point-Free Function References

When an array method expects a single-argument callback, you can pass the function name directly:

```liva
double(x) => x * 2
isPositive(n) => n > 0

main() {
    let nums = [1, 2, 3, 4, 5]

    // Point-free: pass function name directly
    nums.forEach(print)              // instead of: nums.forEach(x => print(x))
    let doubled = nums.map(double)   // instead of: nums.map(x => double(x))
    let pos = nums.filter(isPositive) // instead of: nums.filter(x => isPositive(x))
    let strs = nums.map(toString)    // instead of: nums.map(x => toString(x))
}
```

**Supported methods:** `forEach`, `map`, `filter`, `find`, `some`, `every`

Also works with `for =>` one-liner loops:

```liva
for item in items => print         // instead of: for item in items => print(item)
for item in items => process       // instead of: for item in items => process(item)
```

### Method References with `::`

Reference an instance method using `object::method` syntax. The method is bound to the specific instance:

```liva
Formatter {
    prefix: string
    constructor(prefix: string) { this.prefix = prefix }
    format(s: string) => $"{this.prefix}: {s}"
}

main() {
    let names = ["Alice", "Bob", "Charlie"]
    let fmt = Formatter("Hello")

    // Method reference: binds fmt.format as a callback
    let greetings = names.map(fmt::format)
    // Result: ["Hello: Alice", "Hello: Bob", "Hello: Charlie"]

    greetings.forEach(print)

    // Also works with forEach, filter, find, some, every
    let validator = Validator(3)
    let valid = names.filter(validator::isValid)
}
```

### When to Use

| Scenario | Syntax | Example |
|----------|--------|---------|
| Built-in function | bare name | `items.forEach(print)` |
| User-defined function | bare name | `nums.map(double)` |
| Instance method | `object::method` | `names.map(fmt::format)` |
| Complex expression | lambda | `nums.map(x => x * 2 + 1)` |
| Multi-argument | lambda | `nums.reduce((a, b) => a + b, 0)` |

> **Note:** Function references work for single-argument callbacks only. For multi-argument or complex expressions, use the standard lambda syntax `x => expr`.

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
| **Point-free** | bare name | `items.forEach(print)` |
| **Method ref** | `object::method` | `names.map(fmt::format)` |

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

**Previous**: [← Functions: Basics](functions-basics.md)

**Next**: [Classes →](classes.md)

**See Also**:
- [Functions: Basics](functions-basics.md) — Declaration, arrow functions, block functions, parameters, return types
- [Error Handling](error-handling.md)
- [Concurrency](concurrency.md)
- [Variables](variables.md)
