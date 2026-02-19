# 🛡️ Error Handling - Fallibility System

Liva uses a **fallibility system** with **error binding** instead of traditional try-catch. Functions can **fail** using the `fail` keyword, errors are returned as **values** (not exceptions), and **error binding** (`let value, err = ...`) keeps handling explicit with no hidden control flow.

## Basic Error Binding

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

main() {
  let result, err = divide(10, 2)

  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Result: {result}")  // "Result: 5"
  }
}
```

Error case:

```liva
main() {
  let result, err = divide(10, 0)
  if err != "" {
    print($"Error: {err}")  // "Division by zero"
  }
}
```

## The `fail` Keyword

### Inline Fail (Ternary)

```liva
isAdult(age: number) => age >= 18 ? true : fail "Too young"
```

### Block Fail

```liva
validateUser(username: string, password: string): string {
  if username == "" fail "Username cannot be empty"
  if password == "" fail "Password cannot be empty"
  if password.length < 8 fail "Password too short"

  return $"User {username} validated"
}

main() {
  let result, err = validateUser("john", "pass")
  if err != "" {
    print($"Validation failed: {err}")  // "Password too short"
  }
}
```

## Error Types

When you use `fail`, the error is a **string**. If no error occurred, `err` is `""`:

```liva
main() {
  let result, err = divide(10, 2)
  if err == "" {
    print("Success!")
  }
}
```

## Non-Fallible Functions

Error binding works on **any function**. For non-fallible ones, `err` is always `""`:

```liva
multiply(a: number, b: number) => a * b

main() {
  let result, err = multiply(5, 3)
  print($"Result: {result}")  // 15
  print($"Error: {err}")      // "" (empty)
}
```

### Ignoring Errors

```liva
let result, _ = divide(10, 2)   // Ignore error
let _, err = validateUser("john", "password123")  // Ignore result
```

## Error Binding with Concurrency

### Async with Errors

```liva
fetchData(url: string): string {
  if url == "" fail "Empty URL"
  return $"Data from {url}"
}

main() {
  let data, err = async fetchData("https://api.example.com")
  if err != "" {
    print($"Async error: {err}")
  }
}
```

### Parallel with Errors

```liva
processData(n: number): number {
  if n < 0 fail "Negative input"
  return n * n
}

main() {
  let result, err = par processData(10)
  if err != "" {
    print($"Parallel error: {err}")
  }
}
```

### Task with Errors

```liva
main() {
  let task1 = task async fetchData("https://api.example.com")
  let task2 = task par processData(-5)

  let data, err1 = await task1
  if err1 != "" { print($"Task 1 error: {err1}") }

  let result, err2 = await task2
  if err2 != "" { print($"Task 2 error: {err2}") }
}
```

## Error Propagation

### Early Return Pattern

```liva
processUser(id: number): string {
  let user, err = fetchUser(id)
  if err != "" fail $"Failed to fetch: {err}"

  let processed, err2 = transformUser(user)
  if err2 != "" fail $"Failed to transform: {err2}"

  return processed
}
```

### Chain of Operations

```liva
pipeline(data: string): string {
  let step1, err1 = validate(data)
  if err1 != "" fail $"Step 1: {err1}"

  let step2, err2 = transform(step1)
  if err2 != "" fail $"Step 2: {err2}"

  let step3, err3 = save(step2)
  if err3 != "" fail $"Step 3: {err3}"

  return "Pipeline success"
}
```

## `or fail` — Error Propagation Shorthand

The `or fail` operator provides a concise way to propagate errors without explicit error binding:

```liva
let response = HTTP.get(url) or fail "Connection error"
let content = File.read("config.json") or fail "Cannot read config"
let data = JSON.parse(content) or fail "Invalid JSON"
```

Equivalent to:

```liva
let response, err = HTTP.get(url)
if err != "" { fail "Connection error" }
```

### Chained Pipeline

```liva
// Concise with or fail
pipeline(data: string): string {
  let step1 = validate(data) or fail "Validation failed"
  let step2 = transform(step1) or fail "Transform failed"
  let step3 = save(step2) or fail "Save failed"
  return "Pipeline success"
}
```

> **Note:** When you need to inspect or log the error value, use the traditional `let value, err = ...` pattern.

## `or default` — Default Value on Error

```liva
let config = loadConfig("app.toml") or default defaultConfig()
let port = parsePort(input) or default 8080
```

## Best Practices & Common Patterns

### Always Check Errors

```liva
// ✅ Good
let data, err = fetchData(url)
if err != "" {
  print($"Error: {err}")
  return
}
processData(data)

// ❌ Bad: silent failure
let data, _ = fetchData(url)
processData(data)
```

### Provide Descriptive Messages

```liva
validateInput(input: string): string {
  if input == "" fail "Input cannot be empty"
  if input.length < 3 fail "Input must be at least 3 characters"
  if input.length > 100 fail "Input too long (max 100 characters)"
  return input
}
```

### Handle Errors at Appropriate Levels

```liva
// Low-level: Just fail
readFile(path: string): string {
  if path == "" fail "Empty path"
}

// Mid-level: Add context
loadConfig(path: string): Config {
  let content, err = readFile(path)
  if err != "" fail $"Failed to load config: {err}"
}

// High-level: Handle gracefully
main() {
  let config, err = loadConfig("config.toml")
  if err != "" {
    print($"Warning: {err}, using defaults")
    config = getDefaultConfig()
  }
  run(config)
}
```

### Validation Pattern

```liva
registerUser(email: string, password: string): string {
  let validEmail, err1 = validateEmail(email)
  if err1 != "" fail err1

  let validPassword, err2 = validatePassword(password)
  if err2 != "" fail err2

  return "User registered"
}
```

### Retry Pattern

```liva
fetchWithRetry(url: string, maxRetries: number): string {
  for i in 0..maxRetries {
    let data, err = async fetchData(url)
    if err == "" { return data }
    print($"Attempt {i + 1} failed: {err}")
  }
  fail "Max retries exceeded"
}
```

### Fallback Pattern

```liva
getData(): string {
  let data, err = fetchFromPrimary()
  if err == "" return data

  let backup, err2 = fetchFromBackup()
  if err2 == "" return backup

  let cached, err3 = fetchFromCache()
  if err3 == "" return cached

  fail "All data sources failed"
}
```

### Parallel Error Collection

```liva
main() {
  let task1 = task async fetchUser(1)
  let task2 = task async fetchUser(2)
  let task3 = task async fetchUser(3)

  let errors = []

  let user1, err1 = await task1
  if err1 != "" errors.push($"User 1: {err1}")

  let user2, err2 = await task2
  if err2 != "" errors.push($"User 2: {err2}")

  let user3, err3 = await task3
  if err3 != "" errors.push($"User 3: {err3}")

  if errors.length > 0 {
    print($"Errors occurred: {errors}")
  }
}
```

## Compile-Time Validation

The compiler enforces that calls to fallible functions **must use error binding** (E0701).

| Error Code | Description | Fix |
|------------|-------------|-----|
| **E0701** | Fallible function called without error binding | Use `let result, err = func(...)` |

Applies to all contexts: variable assignments, string templates, expression statements, binary operations, and function arguments.

```liva
divide(a, b) {
  if b == 0.0 fail "Division by zero"
  return a / b
}

main() {
  // ❌ ERROR E0701: Missing error binding
  let x = divide(20.0, 4.0)

  // ✅ Correct
  let result, err = divide(10.0, 2.0)
  if err != "" {
    print($"Error: {err}")
  }
}
```

The compiler detects fallible functions by scanning for `fail` statements. Non-fallible functions (no `fail`) can be called normally.

The VS Code extension (v0.3.2+) provides real-time validation with red squiggly lines (300ms debounce).

## Future Enhancements

- **Custom error types**: `fail MyError("message")`
- **Error variants**: `fail NotFound | InvalidInput`
- **Result helpers**: `unwrap()`, `expect()`, `?` operator
- **Error context**: Automatic stack traces

## See Also

- **[Functions](functions.md)** - Function declarations and types
- **[Concurrency](concurrency.md)** - Error handling with async/par
- **[Error Handling Guide](../guides/error-handling-patterns.md)** - Best practices
- **[Error System](../compiler-internals/error-system.md)** - Compiler error codes

---

**Next:** [Visibility](visibility.md)
