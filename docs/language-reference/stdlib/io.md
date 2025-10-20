# Console/IO Functions

> **Status:** ✅ Complete  
> **Version:** v0.7.0  
> **Design:** Hybrid approach - `print()` for simplicity + `console.*` for power

Built-in functions for console output and user input.

---

## 🎯 Design Philosophy: Hybrid I/O Approach

Liva provides **two complementary ways** to handle I/O:

### 1. **`print()` - Simple & Direct**
- **Purpose:** Quick output for beginners and simple scripts
- **Format:** Display format (`{}`) - clean, user-facing output
- **Use case:** Final output, user messages, simple debugging
- **Example:** `print("Hello")`, `print($"Name: {name}")`

### 2. **`console.*` - Professional & Organized**
- **Purpose:** Structured logging and advanced I/O
- **Format:** Debug format (`{:?}`) - detailed, developer-facing output
- **Use case:** Development, debugging, structured logging, data inspection
- **Functions:** `console.log()`, `console.error()`, `console.warn()`, `console.readLine()`, `console.prompt()`
- **Inspired by:** JavaScript/Node.js console API

### When to Use Which?

```liva
// ✅ Use print() for:
print("Hello, World!")                    // Simple greetings
print($"Total: ${total}")                 // User-facing output
print($"Processing complete!")            // Status messages

// ✅ Use console.* for:
console.log(data)                         // Inspect complex data structures
console.error($"Error: {errorMsg}")       // Explicit error output
console.warn($"Deprecated API used")      // Warnings
let input = console.prompt("Name: ")      // Interactive input
```

---

## 📚 Available Functions

### print(...args)

**Simple output function for user-facing messages.**

**Format:** Display (`{}`) - Clean, no extra formatting

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Generates:** `println!("{}", ...)` (Rust)

**Examples:**

```liva
// Simple string
print("Hello, World!")
// Output: Hello, World!

// With string template
let name = "Alice"
print($"Hello, {name}!")
// Output: Hello, Alice!

// Multiple values
let x = 42
let y = 3.14
print($"X = {x}, Y = {y}")
// Output: X = 42, Y = 3.14
```

**Best for:**
- User-facing output
- Simple status messages
- Clean, formatted text
- Quick scripts and examples

---

### console.log(...args)

**Debug-friendly output function for developers.**

**Format:** Debug (`{:?}`) - Shows internal structure, quotes strings, formats arrays

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Generates:** `println!("{:?}", ...)` (Rust)

**Examples:**

```liva
// Simple message
console.log("Hello, World!")
// Output: "Hello, World!"  (note the quotes)

// Inspect variables
let x = 42
console.log(x)
// Output: 42

// Inspect arrays
let nums = [1, 2, 3, 4, 5]
console.log(nums)
// Output: [1, 2, 3, 4, 5]

// Inspect objects
let user = { name: "Alice", age: 25 }
console.log(user)
// Output: { name: "Alice", age: 25 }

// Multiple values
console.log("Name:", "Alice", "Age:", 25)
// Output: "Name:" "Alice" "Age:" 25
```

**Best for:**
- Debugging and development
- Inspecting data structures
- Development logging
- Seeing exact value representations

---

### console.error(...args)

**Print error messages to standard error (stderr).**

**Format:** Debug (`{:?}`)

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Stream:** stderr (separate from normal output)

**Generates:** `eprintln!("{:?}", ...)` (Rust)

**Examples:**

```liva
// Simple error
console.error("File not found!")
// Output (stderr): "File not found!"

// Error with code
let code = 404
console.error($"Error {code}: Not found")
// Output (stderr): "Error 404: Not found"

// Error with context
let filename = "config.json"
console.error($"Cannot read file: {filename}")
// Output (stderr): "Cannot read file: config.json"
```

**Best for:**
- Error messages
- Exception handling
- Failure notifications
- Separation of errors from normal output

---

### console.warn(...args)

**Print warning messages to stderr with "Warning: " prefix.**

**Format:** Debug (`{:?}`)

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Stream:** stderr

**Prefix:** Automatically adds "Warning: " before message

**Generates:** `eprintln!("Warning: {:?}", ...)` (Rust)

**Examples:**

```liva
// Simple warning
console.warn("This feature is deprecated")
// Output (stderr): Warning: "This feature is deprecated"

// Conditional warning
if age < 18 {
    console.warn($"User is under 18: {age}")
}
// Output (stderr): Warning: "User is under 18: 15"

// API warning
console.warn("Using legacy authentication method")
// Output (stderr): Warning: "Using legacy authentication method"
```

**Best for:**
- Deprecation warnings
- Non-critical issues
- Development hints
- API usage warnings

---

### console.readLine() -> String

**Read a line of text from standard input (stdin).**

**Parameters:**
- None

**Returns:**
- String containing the input line (trimmed of whitespace)

**Blocks:** Yes - waits for user input

**Examples:**

```liva
// Simple input
console.log("What is your name?")
let name = console.readLine()
console.log($"Hello, {name}!")

// Process input
let line = console.readLine()
let num, err = parseInt(line)
if err == "" {
    console.log($"You entered: {num}")
} else {
    console.error($"Invalid number: {err}")
}

// Loop until valid input
let valid = false
let age = 0

while !valid {
    console.log("Enter your age:")
    let input = console.readLine()
    let num, err = parseInt(input)
    
    if err == "" {
        age = num
        valid = true
    } else {
        console.error("Please enter a valid number")
    }
}
```

**Best for:**
- Reading user input
- Interactive programs
- Line-by-line processing
- When you need control over prompts

---

### console.prompt(message: String) -> String

**Display a prompt message and read input in one call.**

**Parameters:**
- `message` - The prompt message to display (no automatic newline)

**Returns:**
- String containing the input line (trimmed of whitespace)

**Blocks:** Yes - waits for user input

**Examples:**

```liva
// Simple prompt
let name = console.prompt("Enter your name: ")
console.log($"Hello, {name}!")

// Get and validate age
let ageStr = console.prompt("Enter your age: ")
let age, err = parseInt(ageStr)
if err == "" {
    console.log($"You are {age} years old")
} else {
    console.error($"Invalid age: {err}")
}

// Multiple prompts
let firstName = console.prompt("First name: ")
let lastName = console.prompt("Last name: ")
let email = console.prompt("Email: ")

console.log($"Name: {firstName} {lastName}")
console.log($"Email: {email}")
```

**Best for:**
- Interactive prompts
- Quick input collection
- Single-line Q&A
- Cleaner code than separate print + readLine

---

## 🆚 print() vs console.log() Comparison

| Feature | `print()` | `console.log()` |
|---------|-----------|-----------------|
| **Format** | Display `{}` | Debug `{:?}` |
| **Strings** | `Hello` | `"Hello"` (with quotes) |
| **Arrays** | N/A (not formatted) | `[1, 2, 3]` |
| **Objects** | N/A (not formatted) | `{ name: "Alice" }` |
| **Use Case** | User output | Development/debugging |
| **Audience** | End users | Developers |
| **Best For** | Final messages | Data inspection |

### Side-by-Side Examples

```liva
let name = "Alice"
let age = 25
let nums = [1, 2, 3]

// print() - Clean, user-facing
print("Welcome!")
// Output: Welcome!

print($"Hello, {name}!")
// Output: Hello, Alice!

print($"Age: {age}")
// Output: Age: 25

// console.log() - Debug, developer-facing
console.log("Welcome!")
// Output: "Welcome!"  (with quotes)

console.log(name)
// Output: "Alice"  (with quotes)

console.log(age)
// Output: 25

console.log(nums)
// Output: [1, 2, 3]  (array format)

console.log($"Name: {name}, Age: {age}")
// Output: "Name: Alice, Age: 25"
```

---

## 💡 Usage Patterns

### Logging Levels

```liva
// Information (stdout)
print("Application started")
console.log("Application started")

// Warnings (stderr)
console.warn("Low disk space")

// Errors (stderr)
console.error("Failed to connect to database")
```

### Interactive Applications

```liva
main() {
    // Welcome message (user-facing)
    print("=== User Registration ===")
    print("")
    
    // Get user info with prompts
    let name = console.prompt("Enter your name: ")
    let email = console.prompt("Enter your email: ")
    let ageStr = console.prompt("Enter your age: ")
    
    // Validate input
    let age, err = parseInt(ageStr)
    if err != "" {
        console.error($"Invalid age: {err}")
        return
    }
    
    // Debug output (developer-facing)
    console.log("User data collected:")
    console.log(name)
    console.log(email)
    console.log(age)
    
    // Final confirmation (user-facing)
    print("")
    print($"Welcome, {name}!")
    print($"Email: {email}")
    print($"Age: {age}")
}
```

### Error Handling Pattern

```liva
main() {
    let filename = console.prompt("Enter filename: ")
    
    let content, err = readFile(filename)
    if err != "" {
        // Error output (stderr)
        console.error($"Error reading file: {err}")
        console.error($"File: {filename}")
        return
    }
    
    // Success output (stdout)
    print($"File loaded successfully: {filename}")
    console.log(content)  // Debug: show content
}
```

### Development vs Production

```liva
const DEBUG = true

main() {
    // User-facing output (always shown)
    print("Processing data...")
    
    // Developer output (only in debug mode)
    if DEBUG {
        console.log("Data structure:")
        console.log(data)
        console.log("Processing started at:")
        console.log(timestamp)
    }
    
    // Process data...
    
    // User-facing result
    print("Processing complete!")
    
    // Developer metrics
    if DEBUG {
        console.log($"Total items: {count}")
        console.log($"Duration: {duration}ms")
    }
}
```

---

## 📝 Technical Notes

### Output Streams

- **stdout (Standard Output)**:
  - `print()` → `println!("{}", ...)`
  - `console.log()` → `println!("{:?}", ...)`
  - Use for normal program output
  - Can be redirected: `program > output.txt`

- **stderr (Standard Error)**:
  - `console.error()` → `eprintln!("{:?}", ...)`
  - `console.warn()` → `eprintln!("Warning: {:?}", ...)`
  - Use for errors and warnings
  - Can be redirected separately: `program 2> errors.txt`
  - Shows in terminal even when stdout is redirected

### Format Specifiers

- **Display Format (`{}`)**:
  - Used by `print()`
  - Clean, user-friendly output
  - Strings without quotes
  - Example: `"Hello"` → `Hello`

- **Debug Format (`{:?}`)**:
  - Used by `console.*` functions
  - Developer-friendly, shows structure
  - Strings with quotes
  - Arrays and objects formatted
  - Example: `"Hello"` → `"Hello"`

### Input Functions

- **`console.readLine()`**:
  - Reads complete line (until newline)
  - Blocks until user presses Enter
  - Trims leading/trailing whitespace
  - Returns empty string on empty input

- **`console.prompt(message)`**:
  - Combines output + input in one call
  - Message displayed without newline
  - User types on same line
  - Equivalent to: `print(message)` + `console.readLine()`

### String Templates

Both `print()` and `console.*` support string templates:

```liva
let name = "Alice"
let age = 25

// With print()
print($"Name: {name}, Age: {age}")
// Output: Name: Alice, Age: 25

// With console.log()
console.log($"Name: {name}, Age: {age}")
// Output: "Name: Alice, Age: 25"  (with quotes)
```

### Migration Guide

If you're used to other languages:

**From Python:**
```python
# Python
print("Hello")
name = input("Name: ")

# Liva
print("Hello")                    # Same!
let name = console.prompt("Name: ")  # Use console.prompt
```

**From JavaScript:**
```javascript
// JavaScript
console.log("Hello");
const name = prompt("Name: ");

// Liva
console.log("Hello")              // Same!
let name = console.prompt("Name: ")  # Same!
```

**From Rust:**
```rust
// Rust
println!("{}", "Hello");
println!("{:?}", data);

// Liva
print("Hello")                    // Simpler!
console.log(data)                 // Simpler!
```

---

## 🧪 Testing

All console/IO functions have been tested:
- ✅ `print()` - stdout output with Display format
- ✅ `console.log()` - stdout output with Debug format
- ✅ `console.error()` - stderr output
- ✅ `console.warn()` - stderr output with "Warning: " prefix
- ✅ `console.readLine()` - implemented (requires interactive testing)
- ✅ `console.prompt()` - implemented (requires interactive testing)

Test file: `test_io.liva`

**Note:** `console.readLine()` and `console.prompt()` require interactive user input and cannot be tested in automated test suites.

---

## 🎯 Best Practices

### 1. Choose the Right Function

```liva
// ✅ Good: User-facing messages with print()
print("Welcome to MyApp!")
print($"Processing {count} items...")

// ✅ Good: Debug output with console.log()
console.log(data)
console.log($"State: {state}")

// ❌ Avoid: Debug output for end users
console.log("Welcome to MyApp!")  // Shows quotes

// ❌ Avoid: Complex data with print()
print(complexObject)  // Won't format nicely
```

### 2. Use Appropriate Streams

```liva
// ✅ Good: Errors to stderr
console.error($"Connection failed")

// ✅ Good: Warnings to stderr
console.warn($"Deprecated API")

// ❌ Avoid: Errors to stdout
print($"Error: Connection failed")  // Mixed with normal output
```

### 3. Consistent Prompting

```liva
// ✅ Good: Use console.prompt for input
let name = console.prompt("Enter name: ")

// ✅ Also good: Manual prompt for multi-line questions
print("What is your name?")
print("(Please enter your full name)")
let name = console.readLine()

// ❌ Avoid: Mixing styles
print("Enter name: ")
let name = console.readLine()  // Less clear than prompt()
```

### 4. Error Handling

```liva
// ✅ Good: Errors to stderr, success to stdout
let result, err = doSomething()
if err != "" {
    console.error($"Operation failed: {err}")
    return
}
print($"Operation successful: {result}")

// ❌ Avoid: All output to stdout
let result, err = doSomething()
if err != "" {
    print($"Operation failed: {err}")  // Can't be filtered
    return
}
print($"Operation successful: {result}")
```

---

## 📝 See Also

- [Type Conversion Functions](./conversions.md) - For parsing user input
- [String Methods](./strings.md) - For processing input strings
- [Standard Library Overview](./README.md)
