# Console API

The Console API provides functions for interacting with the terminal: logging output, displaying errors/warnings, and reading user input.

## Table of Contents
- [Output Functions](#output-functions)
- [Input Functions](#input-functions)
- [Examples](#examples)

---

## Output Functions

### `console.log(...)`

Prints values to standard output with debug formatting.

**Syntax:**
```liva
console.log(value1, value2, ...)
```

**Parameters:**
- `value1, value2, ...` - Any number of values to print

**Returns:** Nothing (void)

**Examples:**
```liva
console.log("Hello, world!")
console.log("User:", name, "Age:", age)
console.log(42, 3.14, true)
```

**Notes:**
- Uses Rust's debug formatting (`{:?}`)
- Automatically adds newline at the end
- Multiple values are printed separated by spaces

---

### `console.error(...)`

Prints error messages to standard error in **red color**.

**Syntax:**
```liva
console.error(value1, value2, ...)
```

**Parameters:**
- `value1, value2, ...` - Any number of values to print

**Returns:** Nothing (void)

**Examples:**
```liva
console.error("Error: File not found")
console.error("Invalid input:", userInput)
```

**Notes:**
- Uses Rust's display formatting (`{}`)
- Prints to stderr (not stdout) in **red color** using ANSI escape codes
- Color automatically resets after the message
- Useful for error messages that should stand out visually

---

### `console.warn(...)`

Prints warning messages to standard error in **yellow/amber color**.

**Syntax:**
```liva
console.warn(value1, value2, ...)
```

**Parameters:**
- `value1, value2, ...` - Any number of values to print

**Returns:** Nothing (void)

**Examples:**
```liva
console.warn("Deprecated function used")
console.warn("Memory usage high:", memUsage, "MB")
```

**Notes:**
- Prints to stderr in **yellow/amber color** using ANSI escape codes
- No "Warning:" prefix - just colored output
- Color automatically resets after the message
- Uses display formatting for user-friendly output

---

### `console.success(...)`

Prints success messages to standard output in **green color**.

**Syntax:**
```liva
console.success(value1, value2, ...)
```

**Parameters:**
- `value1, value2, ...` - Any number of values to print

**Returns:** Nothing (void)

**Examples:**
```liva
console.success("User created successfully!")
console.success("✓ All tests passed:", testCount, "tests")
```

**Notes:**
- Prints to stdout in **green color** using ANSI escape codes
- Color automatically resets after the message
- Perfect for confirmations, completions, and positive feedback
- Uses display formatting for user-friendly output

---

## Input Functions

### `console.input()` / `console.input(message)`

Reads a line of text from standard input (stdin), with optional prompt message.

**Syntax:**
```liva
// Without prompt
let value = console.input()

// With prompt message
let value = console.input("Enter your name: ")

// With empty prompt (no message displayed)
let value = console.input("")
```

**Parameters:**
- `message` (optional) - String to display as prompt before reading input

**Returns:** `string` - The input line with leading/trailing whitespace removed

**Behavior:**
- **Without message**: Reads input silently (like Python's `input()`)
- **With message**: Displays prompt, then reads input (like Python's `input("prompt")`)
- **With empty string**: Same as without message (no prompt displayed)
- Input is automatically trimmed (removes `\n`, spaces, tabs)
- Blocks execution until user presses Enter

**Examples:**

#### Basic usage with prompt
```liva
main() {
  let name = console.input("What's your name? ")
  console.log($"Hello, {name}!")
}
```

#### Without prompt
```liva
main() {
  console.log("Enter your age:")
  let ageStr = console.input()
  console.log($"You entered: {ageStr}")
}
```

#### With error handling
```liva
main() {
  let numStr = console.input("Enter a number: ")
  let num, err = parseInt(numStr)
  
  if !err {
    console.log($"You entered: {num}")
  } else {
    console.error($"Invalid number: {err}")
  }
}
```

#### Multiple inputs
```liva
main() {
  let firstName = console.input("First name: ")
  let lastName = console.input("Last name: ")
  let age, ageErr = parseInt(console.input("Age: "))
  
  if !ageErr {
    console.log($"Name: {firstName} {lastName}, Age: {age}")
  }
}
```

#### Input validation loop
```liva
main() {
  let validInput = false
  let number = 0
  
  while !validInput {
    let input = console.input("Enter a positive number: ")
    let num, err = parseInt(input)
    
    if !err and num > 0 {
      number = num
      validInput = true
    } else {
      console.error("Invalid input. Try again.")
    }
  }
  
  console.log($"Valid number: {number}")
}
```

**Notes:**
- Always returns a `string` - use `parseInt()` or `parseFloat()` to convert
- Blocking operation - program waits for user input
- Similar to Python's `input()` function
- On Unix/Linux, supports UTF-8 input
- Flush is automatic when prompt message is provided

---

## Examples

### Interactive Calculator
```liva
main() {
  console.log("=== Simple Calculator ===")
  
  let num1, err1 = parseInt(console.input("First number: "))
  let num2, err2 = parseInt(console.input("Second number: "))
  
  if !err1 and !err2 {
    console.log($"\nResults:")
    console.log($"  {num1} + {num2} = {num1 + num2}")
    console.log($"  {num1} - {num2} = {num1 - num2}")
    console.log($"  {num1} * {num2} = {num1 * num2}")
    
    let division, divErr = divide(num1, num2)
    if !divErr {
      console.log($"  {num1} / {num2} = {division}")
    } else {
      console.error($"  Division error: {divErr}")
    }
  } else {
    console.error("Invalid input!")
  }
}

divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b
```

### User Profile Input
```liva
main() {
  console.log("=== Create User Profile ===\n")
  
  let name = console.input("Name: ")
  let email = console.input("Email: ")
  let ageStr = console.input("Age: ")
  
  let age, ageErr = parseInt(ageStr)
  
  if !ageErr {
    let profile = {
      name: name,
      email: email,
      age: age,
      isAdult: age >= 18
    }
    
    console.log("\n=== Profile Created ===")
    console.log($"Name: {profile.name}")
    console.log($"Email: {profile.email}")
    console.log($"Age: {profile.age}")
    console.log($"Adult: {profile.isAdult}")
  } else {
    console.error($"Invalid age: {ageErr}")
  }
}
```

### Menu System
```liva
main() {
  let running = true
  
  while running {
    console.log("\n=== Menu ===")
    console.log("1. Say Hello")
    console.log("2. Calculate")
    console.log("3. Exit")
    
    let choice = console.input("\nSelect option: ")
    
    switch choice {
      case "1": {
        let name = console.input("Your name: ")
        console.log($"Hello, {name}!")
      }
      case "2": {
        let num, err = parseInt(console.input("Enter number: "))
        if !err {
          console.log($"{num} * 2 = {num * 2}")
        }
      }
      case "3": {
        console.log("Goodbye!")
        running = false
      }
      default: console.error("Invalid option")
    }
  }
}
```

---

## Comparison with Other Languages

| Language | Function | With Prompt | Without Prompt |
|----------|----------|-------------|----------------|
| **Liva** | `console.input()` | `console.input("msg")` | `console.input()` |
| Python | `input()` | `input("msg")` | `input()` |
| JavaScript | `prompt()` | `prompt("msg")` | N/A (browser only) |
| Ruby | `gets` | `print + gets` | `gets` |
| Rust | `stdin.read_line()` | Manual | Manual |

---

## See Also
- [String Templates](string-templates.md) - For formatting output messages
- [Error Handling](error-handling.md) - For handling input validation
- [Functions](functions.md) - parseInt() and parseFloat() for type conversion
