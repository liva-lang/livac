# Console/IO Functions

> **Status:** ✅ Complete  
> **Version:** v0.7.0

Built-in functions for console output and user input.

---

## 📚 Available Functions (5/5)

### console.log(...args)

Print messages to standard output (stdout).

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Examples:**

```liva
// Simple message
console.log($"Hello, World!")

// Multiple values
console.log($"Name:", "Alice", $"Age:", 25)

// With variables
let x = 42
console.log($"The answer is {x}")
```

---

### console.error(...args)

Print error messages to standard error (stderr).

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Examples:**

```liva
// Error message
console.error($"File not found!")

// Error with code
let code = 404
console.error($"Error {code}: Not found")

// Multiple values
console.error($"Error:", "Connection failed")
```

---

### console.warn(...args)

Print warning messages to standard error (stderr) with "Warning: " prefix.

**Parameters:**
- `...args` - Variable number of arguments to print

**Returns:**
- None (void)

**Examples:**

```liva
// Warning message
console.warn($"This feature is deprecated")

// Conditional warning
if age < 18 {
    console.warn($"User is under 18")
}
```

---

### readLine() -> String

Read a line of text from standard input (stdin).

**Parameters:**
- None

**Returns:**
- String containing the input line (trimmed of whitespace)

**Examples:**

```liva
// Read user input
let input = readLine()
console.log($"You entered: {input}")

// Process input
let line = readLine()
let num, err = parseInt(line)
if err == "" {
    console.log($"You entered the number: {num}")
} else {
    console.error($"Invalid number")
}
```

---

### prompt(message: String) -> String

Display a message and read a line from standard input.

**Parameters:**
- `message` - The prompt message to display

**Returns:**
- String containing the input line (trimmed of whitespace)

**Examples:**

```liva
// Get user name
let name = prompt("Enter your name: ")
console.log($"Hello, {name}!")

// Get age
let ageStr = prompt("Enter your age: ")
let age, err = parseInt(ageStr)
if err == "" {
    console.log($"You are {age} years old")
} else {
    console.error($"Invalid age")
}
```

---

## 💡 Usage Patterns

### Logging Different Message Types

```liva
// Information
console.log($"Application started")

// Warnings
console.warn($"Low disk space")

// Errors
console.error($"Failed to connect to database")
```

### Interactive Input

```liva
// Simple input
console.log($"What is your name?")
let name = readLine()

// With prompt
let city = prompt("Enter your city: ")

// Validate input
let ageStr = prompt("Enter your age: ")
let age, err = parseInt(ageStr)
if err != "" {
    console.error($"Invalid age: {err}")
}
```

### Error Handling with User Input

```liva
let valid = false
let number = 0

while !valid {
    let input = prompt("Enter a number: ")
    let num, err = parseInt(input)
    
    if err == "" {
        number = num
        valid = true
    } else {
        console.error($"Invalid input: {err}")
    }
}

console.log($"You entered: {number}")
```

---

## 📝 Notes

- **Output Streams**:
  - `console.log()` writes to stdout
  - `console.error()` and `console.warn()` write to stderr
  - Useful for separating normal output from error messages

- **Input Functions**:
  - `readLine()` reads a complete line (until newline)
  - `prompt()` displays a message before reading
  - Both functions trim leading/trailing whitespace
  - Both block until user provides input

- **String Templates**:
  - Use `$"..."` for string templates in console functions
  - Variables can be embedded with `{variable}`

- **vs print()**:
  - `print()` is a simpler function for basic output
  - `console.log()` is more explicit and follows JavaScript conventions
  - Both write to stdout

---

## 🧪 Testing

All console/IO functions have been tested:
- ✅ console.log() - stdout output
- ✅ console.error() - stderr output
- ✅ console.warn() - stderr output with "Warning: " prefix
- ✅ readLine() - implemented (requires interactive testing)
- ✅ prompt() - implemented (requires interactive testing)

Test file: `test_io.liva`

**Note:** `readLine()` and `prompt()` require interactive user input and cannot be tested in automated test suites.

---

## 📝 See Also

- [Type Conversion Functions](./conversions.md) - For parsing user input
- [String Methods](./strings.md) - For processing input strings
- [Standard Library Overview](./README.md)
