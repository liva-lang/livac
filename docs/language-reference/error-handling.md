# 🛡️ Error Handling - Fallibility System

Liva uses a **fallibility system** with **error binding** instead of traditional try-catch. This makes error handling explicit, type-safe, and composable.

## Philosophy

In Liva:
- Functions can **fail** using the `fail` keyword
- Errors are returned as **values**, not exceptions
- **Error binding** (`let value, err = ...`) makes error handling explicit
- **No hidden control flow** - errors don't skip code unexpectedly

## Basic Error Binding

### Fallible Function

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

main() {
  // Error binding: captures both result and error
  let result, err = divide(10, 2)
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Result: {result}")
  }
}
```

**Output:**
```
Result: 5
```

### Error Case

```liva
main() {
  let result, err = divide(10, 0)
  
  if err != "" {
    print($"Error: {err}")  // "Division by zero"
  } else {
    print($"Result: {result}")
  }
}
```

**Output:**
```
Error: Division by zero
```

## The `fail` Keyword

### Inline Fail (Ternary)

```liva
// One-liner with fail
isAdult(age: number) => age >= 18 ? true : fail "Too young"

main() {
  let adult, err = isAdult(15)
  print($"Error: {err}")  // "Too young"
}
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
    print($"Validation failed: {err}")
  } else {
    print($"Success: {result}")
  }
}
```

**Output:**
```
Validation failed: Password too short
```

## Error Types

### Default Error Type

When you use `fail`, the error is a **string**:

```liva
checkAge(age: number) {
  if age < 0 fail "Age cannot be negative"
  if age > 150 fail "Age too high"
  return $"Valid: {age}"
}

main() {
  let result, err = checkAge(-5)
  print($"Error type: string = {err}")  // err is string
}
```

### Empty String = No Error

If no error occurred, `err` is an empty string `""`:

```liva
main() {
  let result, err = divide(10, 2)
  
  // Check for success
  if err == "" {
    print("Success!")
  }
}
```

## Non-Fallible Functions

### Normal Function Binding

You can use error binding on **any function**, even non-fallible ones:

```liva
// Normal function (never fails)
multiply(a: number, b: number) => a * b

main() {
  // Still works with error binding
  let result, err = multiply(5, 3)
  
  print($"Result: {result}")  // 15
  print($"Error: {err}")      // "" (empty)
}
```

When a function doesn't fail:
- `result` contains the return value
- `err` is always `""`

### Ignoring Errors

Use `_` to ignore the error:

```liva
main() {
  // I know this won't fail
  let result, _ = divide(10, 2)
  print($"Result: {result}")
}
```

Or ignore the result:

```liva
main() {
  // I only care about errors
  let _, err = validateUser("john", "password123")
  
  if err != "" {
    print($"Error: {err}")
  }
}
```

## Error Binding with Concurrency

### Async with Errors

```liva
fetchData(url: string): string {
  if url == "" fail "Empty URL"
  return $"Data from {url}"
}

main() {
  // Error binding with async
  let data, err = async fetchData("https://api.example.com")
  
  if err != "" {
    print($"Async error: {err}")
  } else {
    print($"Async success: {data}")
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
  // Error binding with par
  let result, err = par processData(10)
  
  if err != "" {
    print($"Parallel error: {err}")
  } else {
    print($"Parallel result: {result}")
  }
}
```

### Task with Errors

```liva
main() {
  let task1 = task async fetchData("https://api.example.com")
  let task2 = task par processData(-5)
  
  // Await with error binding
  let data, err1 = await task1
  if err1 != "" {
    print($"Task 1 error: {err1}")
  }
  
  let result, err2 = await task2
  if err2 != "" {
    print($"Task 2 error: {err2}")  // "Negative input"
  }
}
```

## Error Propagation

### Early Return Pattern

```liva
processUser(id: number): string {
  // Validate
  let user, err = fetchUser(id)
  if err != "" {
    return fail $"Failed to fetch: {err}"
  }
  
  // Process
  let processed, err2 = transformUser(user)
  if err2 != "" {
    return fail $"Failed to transform: {err2}"
  }
  
  return processed
}

main() {
  let result, err = processUser(123)
  
  if err != "" {
    print($"Error: {err}")
  } else {
    print($"Success: {result}")
  }
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

main() {
  let result, err = pipeline("raw data")
  
  if err != "" {
    print($"Pipeline failed: {err}")
  } else {
    print($"Pipeline complete: {result}")
  }
}
```

## Multiple Error Handling

### Sequential Errors

```liva
main() {
  let result1, err1 = operation1()
  if err1 != "" {
    print($"Op 1 failed: {err1}")
    return
  }
  
  let result2, err2 = operation2(result1)
  if err2 != "" {
    print($"Op 2 failed: {err2}")
    return
  }
  
  print($"Success: {result2}")
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
  } else {
    print("All users fetched successfully")
  }
}
```

## Best Practices

### 1. Always Check Errors

```liva
// ✅ Good: Check errors
let data, err = fetchData(url)
if err != "" {
  print($"Error: {err}")
  return
}
processData(data)

// ❌ Bad: Ignore errors
let data, _ = fetchData(url)
processData(data)  // Might fail silently
```

### 2. Provide Context

```liva
// ✅ Good: Descriptive error messages
validateInput(input: string): string {
  if input == "" fail "Input cannot be empty"
  if input.length < 3 fail "Input must be at least 3 characters"
  if input.length > 100 fail "Input too long (max 100 characters)"
  return input
}

// ❌ Bad: Generic messages
validateInput(input: string): string {
  if input == "" fail "Error"
  if input.length < 3 fail "Error"
  return input
}
```

### 3. Use Early Returns

```liva
// ✅ Good: Early returns reduce nesting
processData(data: string): string {
  let valid, err = validate(data)
  if err != "" fail err
  
  let clean, err2 = cleanup(valid)
  if err2 != "" fail err2
  
  let final, err3 = finalize(clean)
  if err3 != "" fail err3
  
  return final
}

// ❌ Bad: Deep nesting
processData(data: string): string {
  let valid, err = validate(data)
  if err == "" {
    let clean, err2 = cleanup(valid)
    if err2 == "" {
      let final, err3 = finalize(clean)
      if err3 == "" {
        return final
      } else {
        fail err3
      }
    } else {
      fail err2
    }
  } else {
    fail err
  }
}
```

### 4. Handle Errors at Appropriate Level

```liva
// Low-level function: Just fail
readFile(path: string): string {
  if path == "" fail "Empty path"
  // ... read file or fail
}

// Mid-level function: Add context
loadConfig(path: string): Config {
  let content, err = readFile(path)
  if err != "" fail $"Failed to load config: {err}"
  
  // Parse and return
}

// High-level function: Handle gracefully
main() {
  let config, err = loadConfig("config.toml")
  
  if err != "" {
    print($"Warning: {err}, using defaults")
    config = getDefaultConfig()
  }
  
  run(config)
}
```

## Common Patterns

### Validation Pattern

```liva
validateEmail(email: string): string {
  if email == "" fail "Email cannot be empty"
  if !email.contains("@") fail "Invalid email format"
  return email
}

validatePassword(password: string): string {
  if password.length < 8 fail "Password too short"
  if !password.containsDigit() fail "Password must contain a digit"
  return password
}

registerUser(email: string, password: string): string {
  let validEmail, err1 = validateEmail(email)
  if err1 != "" fail err1
  
  let validPassword, err2 = validatePassword(password)
  if err2 != "" fail err2
  
  // Create user...
  return "User registered"
}
```

### Retry Pattern

```liva
fetchWithRetry(url: string, maxRetries: number): string {
  for i in 0..maxRetries {
    let data, err = async fetchData(url)
    
    if err == "" {
      return data
    }
    
    print($"Attempt {i + 1} failed: {err}")
  }
  
  fail "Max retries exceeded"
}

main() {
  let data, err = fetchWithRetry("https://api.example.com", 3)
  
  if err != "" {
    print($"Failed after retries: {err}")
  } else {
    print($"Success: {data}")
  }
}
```

### Fallback Pattern

```liva
getData(): string {
  // Try primary source
  let data, err = fetchFromPrimary()
  if err == "" return data
  
  print("Primary failed, trying backup...")
  
  // Try backup source
  let backup, err2 = fetchFromBackup()
  if err2 == "" return backup
  
  print("Backup failed, using cache...")
  
  // Try cache
  let cached, err3 = fetchFromCache()
  if err3 == "" return cached
  
  // All failed
  fail "All data sources failed"
}
```

### Transaction Pattern

```liva
transferMoney(from: number, to: number, amount: number): string {
  // Validate
  let valid, err = validateTransfer(from, to, amount)
  if err != "" fail err
  
  // Start transaction
  let tx, err2 = beginTransaction()
  if err2 != "" fail err2
  
  // Debit
  let _, err3 = debit(from, amount)
  if err3 != "" {
    rollback(tx)
    fail $"Debit failed: {err3}"
  }
  
  // Credit
  let _, err4 = credit(to, amount)
  if err4 != "" {
    rollback(tx)
    fail $"Credit failed: {err4}"
  }
  
  // Commit
  let _, err5 = commit(tx)
  if err5 != "" fail $"Commit failed: {err5}"
  
  return "Transfer successful"
}
```

## Error Binding vs Try-Catch

### Liva Style (Error Binding)

```liva
main() {
  let data, err = fetchData()
  if err != "" {
    print($"Error: {err}")
    return
  }
  
  processData(data)
}
```

**Advantages:**
- ✅ Explicit - can't forget to handle errors
- ✅ No hidden control flow
- ✅ Composable with concurrency
- ✅ Type-safe
- ✅ Performance - no stack unwinding

### Traditional Try-Catch (Not in Liva)

```javascript
// Not Liva syntax!
try {
  let data = fetchData()
  processData(data)
} catch (e) {
  print("Error: " + e)
}
```

**Disadvantages:**
- ❌ Easy to forget
- ❌ Hidden control flow (jumps)
- ❌ Complicates concurrency
- ❌ Runtime overhead

## Compilation to Rust

Liva's fallibility system compiles to Rust's `Result<T, String>`:

```liva
// Liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b
```

```rust
// Generated Rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

Error binding:

```liva
// Liva
let result, err = divide(10, 2)
```

```rust
// Generated Rust
let (result, err) = match divide(10, 2) {
    Ok(val) => (val, String::new()),
    Err(e) => (Default::default(), e),
};
```

## Future Enhancements

Planned features for future versions:

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
