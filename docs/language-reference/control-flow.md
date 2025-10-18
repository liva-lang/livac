# Control Flow

Complete reference for control flow statements in Liva: conditionals, loops, switches, and exception handling.

## Table of Contents
- [If Statements](#if-statements)
- [While Loops](#while-loops)
- [For Loops](#for-loops)
- [Switch Statements](#switch-statements)
- [Try-Catch](#try-catch)
- [Return Statements](#return-statements)
- [Fail Statements](#fail-statements)
- [Best Practices](#best-practices)

---

## If Statements

### Basic Syntax

```liva
if age >= 18 {
  print("Adult")
}
```

### If-Else

```liva
if age >= 18 {
  print("Adult")
} else {
  print("Minor")
}
```

### If-Else-If

```liva
if score >= 90 {
  print("A")
} else if score >= 80 {
  print("B")
} else if score >= 70 {
  print("C")
} else {
  print("F")
}
```

### Optional Parentheses

```liva
// With parentheses (optional)
if (age >= 18) {
  print("Adult")
}

// Without parentheses (preferred)
if age >= 18 {
  print("Adult")
}
```

### Single-Statement If

```liva
// Single statement (no braces needed)
if age < 18 fail "Minor not allowed"

// With braces (also valid)
if age < 18 {
  fail "Minor not allowed"
}
```

### If as Expression (Ternary)

```liva
let status = age >= 18 ? "Adult" : "Minor"
let max = a > b ? a : b
```

---

## While Loops

### Basic While

```liva
let counter = 0
while counter < 5 {
  print($"Count: {counter}")
  counter = counter + 1
}
```

### Infinite Loop

```liva
while true {
  let input = readInput()
  if input == "quit" {
    return
  }
  processInput(input)
}
```

### Condition with Complex Logic

```liva
while isConnected and retryCount < maxRetries {
  try {
    sendData()
    return
  } catch (err) {
    retryCount = retryCount + 1
    wait(1000)
  }
}
```

---

## For Loops

### Basic For Loop

```liva
for i in 1..10 {
  print($"Iteration {i}")
}
```

### Range Syntax

```liva
// Inclusive range: 1, 2, 3, 4, 5
for i in 1..6 {
  print(i)
}

// From variable
let start = 0
let end = 5
for i in start..end {
  print(i)
}
```

### Iterating Arrays

```liva
let names = ["Alice", "Bob", "Charlie"]

for name in names {
  print($"Hello, {name}")
}
```

### Iterating Objects

```liva
let users = [
  { name: "Alice", age: 25 },
  { name: "Bob", age: 30 }
]

for user in users {
  print($"{user.name} is {user.age} years old")
}
```

### Data-Parallel For Policies

#### Sequential (Default)

```liva
for item in items {
  process(item)
}

// Explicit sequential
for seq item in items {
  process(item)
}
```

#### Parallel (CPU-Bound)

```liva
for par item in items {
  heavyComputation(item)
}

// With options
for par item in items with chunk 2 threads 4 {
  process(item)
}
```

#### Vectorized (SIMD)

```liva
for vec value in values {
  compute(value)
}

// With SIMD width
for vec value in values with simdWidth 4 {
  compute(value)
}
```

#### Parallel + Vectorized

```liva
for parvec value in values with simdWidth 4 ordered {
  process(value)
}
```

**See [Concurrency Guide](concurrency.md) for details on data-parallel policies.**

---

## Switch Statements

### Basic Switch

```liva
switch userType {
  case "admin": print("Admin user")
  case "user": print("Regular user")
  case "guest": print("Guest user")
  default: print("Unknown user")
}
```

### Multiple Statements per Case

```liva
switch status {
  case "pending": {
    print("Processing...")
    processOrder()
  }
  case "completed": {
    print("Done!")
    sendNotification()
  }
  default: {
    print("Unknown status")
  }
}
```

### Switch with Expressions

```liva
switch temperature {
  case t if t < 0: print("Freezing")
  case t if t < 20: print("Cold")
  case t if t < 30: print("Warm")
  default: print("Hot")
}
```

### No Fall-Through

Liva **does not support fall-through** (unlike C/Java). Each case is independent:

```liva
switch day {
  case "Monday": print("Start of week")    // Does NOT fall through
  case "Tuesday": print("Second day")
  default: print("Other day")
}
```

---

## Try-Catch

### Basic Try-Catch

```liva
try {
  let result = riskyOperation()
  print($"Success: {result}")
} catch (err) {
  print($"Error: {err}")
}
```

### With Multiple Statements

```liva
try {
  let file = openFile("data.txt")
  let content = readFile(file)
  processContent(content)
} catch (err) {
  print($"File operation failed: {err}")
  logError(err)
}
```

### Nested Try-Catch

```liva
try {
  let data = fetchData()
  
  try {
    saveToDatabase(data)
  } catch (err) {
    print($"Database error: {err}")
  }
} catch (err) {
  print($"Fetch error: {err}")
}
```

### Try-Catch vs Error Binding

```liva
// Try-catch (traditional exception handling)
try {
  let result = divide(10, 0)
} catch (err) {
  print($"Error: {err}")
}

// Error binding (fallibility system - preferred)
let result, err = divide(10, 0)
if err != "" {
  print($"Error: {err}")
}
```

**Prefer error binding for Liva-style error handling.**

---

## Return Statements

### Basic Return

```liva
calculateSum(a, b) {
  return a + b
}
```

### Early Return

```liva
processUser(user) {
  if user == null {
    return "Invalid user"
  }
  
  if user.age < 18 {
    return "Minor"
  }
  
  return "Valid adult user"
}
```

### Void Return

```liva
logMessage(msg: string) {
  print($"[LOG] {msg}")
  return  // Optional for void functions
}
```

### Return in Loops

```liva
findUser(id: number, users): User? {
  for user in users {
    if user.id == id {
      return user
    }
  }
  return null
}
```

---

## Fail Statements

### Basic Fail

```liva
divide(a: number, b: number): number {
  if b == 0 {
    fail "Division by zero"
  }
  return a / b
}
```

### Fail with Interpolation

```liva
validateAge(age: number) {
  if age < 0 {
    fail $"Invalid age: {age}"
  }
  if age < 18 {
    fail $"User must be 18+ (got {age})"
  }
}
```

### Fail in If Statement

```liva
checkPermissions(user) {
  if user.role != "admin" fail "Unauthorized"
  // Continue if authorized
}
```

### Fail in Ternary

```liva
getDiscount(age: number): float => age >= 65 ? 0.2 : age < 18 ? fail "No discount for minors" : 0.0
```

---

## Best Practices

### Prefer Early Returns

```liva
// ✅ Good: Early returns
processOrder(order) {
  if order == null return "Invalid order"
  if order.total < 0 return "Invalid total"
  if order.items.length == 0 return "Empty order"
  
  // Main logic here
  return "Order processed"
}

// ❌ Bad: Nested if statements
processOrder(order) {
  if order != null {
    if order.total >= 0 {
      if order.items.length > 0 {
        // Main logic deeply nested
        return "Order processed"
      }
    }
  }
  return "Invalid"
}
```

### Use Error Binding Over Try-Catch

```liva
// ✅ Good: Error binding
let result, err = divide(a, b)
if err != "" {
  print($"Error: {err}")
  return
}
print($"Result: {result}")

// ⚠️ Acceptable: Try-catch for compatibility
try {
  let result = divide(a, b)
  print($"Result: {result}")
} catch (err) {
  print($"Error: {err}")
}
```

### Switch vs If-Else-If

```liva
// ✅ Good: Switch for discrete values
switch userRole {
  case "admin": handleAdmin()
  case "user": handleUser()
  case "guest": handleGuest()
  default: handleUnknown()
}

// ✅ Good: If-else-if for ranges
if score >= 90 {
  grade = "A"
} else if score >= 80 {
  grade = "B"
} else {
  grade = "C"
}
```

### For Loop Policies

```liva
// ✅ Good: Use appropriate policy
for seq item in ioOperations {  // I/O bound: sequential
  writeToFile(item)
}

for par item in computations {  // CPU bound: parallel
  heavyComputation(item)
}

for vec value in numbers {  // SIMD: vectorized
  mathOperation(value)
}
```

---

## Summary

| Statement | Syntax | Use Case |
|-----------|--------|----------|
| **If** | `if cond { }` | Conditional execution |
| **If-Else** | `if cond { } else { }` | Binary choice |
| **If-Else-If** | `if cond { } else if { }` | Multiple conditions |
| **While** | `while cond { }` | Repeat while true |
| **For** | `for item in items { }` | Iterate collection |
| **For Range** | `for i in 1..10 { }` | Iterate range |
| **For Parallel** | `for par item in items { }` | Parallel iteration |
| **Switch** | `switch val { case x: }` | Multiple discrete cases |
| **Try-Catch** | `try { } catch (e) { }` | Exception handling |
| **Return** | `return value` | Exit function |
| **Fail** | `fail "error"` | Raise error |

### Quick Reference

```liva
// If statement
if age >= 18 {
  print("Adult")
} else {
  print("Minor")
}

// While loop
let i = 0
while i < 5 {
  print(i)
  i = i + 1
}

// For loop
for item in items {
  print(item)
}

// For range
for i in 1..10 {
  print(i)
}

// Parallel for
for par item in items with threads 4 {
  processItem(item)
}

// Switch
switch status {
  case "ok": print("Success")
  case "error": print("Failed")
  default: print("Unknown")
}

// Try-catch
try {
  riskyOperation()
} catch (err) {
  print($"Error: {err}")
}

// Fail
if invalid fail "Error message"
```

---

**Next**: [Operators →](operators.md)

**See Also**:
- [Concurrency](concurrency.md) - Data-parallel for policies
- [Error Handling](error-handling.md) - Fail and error binding
- [Functions](functions.md)
