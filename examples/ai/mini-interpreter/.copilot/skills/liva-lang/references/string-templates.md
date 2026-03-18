# String Templates

Complete reference for string interpolation and template strings in Liva.

## Table of Contents
- [Basic Syntax](#basic-syntax)
- [Expression Interpolation](#expression-interpolation)
- [Special Characters](#special-characters)
- [Multi-line Strings](#multi-line-strings)
- [Best Practices](#best-practices)

---

## Basic Syntax

### Template String Prefix

String templates use the `$"..."` prefix:

```liva
let name = "Alice"
let greeting = $"Hello, {name}!"
print(greeting)  // Output: Hello, Alice!
```

### vs Regular Strings

```liva
// Regular string (no interpolation)
let str1 = "Hello, {name}!"  // Output: Hello, {name}!

// Template string (interpolation)
let str2 = $"Hello, {name}!"  // Output: Hello, Alice!
```

---

## Expression Interpolation

### Simple Variables

```liva
let name = "Bob"
let age = 25

let message = $"My name is {name} and I'm {age} years old"
print(message)  // Output: My name is Bob and I'm 25 years old
```

### Arithmetic Expressions

```liva
let a = 10
let b = 20

let result = $"The sum of {a} and {b} is {a + b}"
print(result)  // Output: The sum of 10 and 20 is 30
```

### Function Calls

```liva
getUsername(id: number): string => $"user_{id}"

let userId = 42
let message = $"Welcome, {getUsername(userId)}!"
print(message)  // Output: Welcome, user_42!
```

### Member Access

```liva
let user = { name: "Alice", age: 30 }

let profile = $"User: {user.name}, Age: {user.age}"
print(profile)  // Output: User: Alice, Age: 30
```

### Method Calls

```liva
let person = Person("Charlie", 35)

let intro = $"Hello! {person.greet()}"
print(intro)  // Output: Hello! I'm Charlie
```

### Complex Expressions

```liva
let price = 99.99
let tax = 0.15

let total = $"Total: ${price * (1 + tax)}"
print(total)  // Output: Total: $114.9885
```

### Nested Interpolation

```liva
let items = [
  { name: "Laptop", price: 999 },
  { name: "Mouse", price: 29 }
]

for item in items {
  print($"Item: {item.name}, Price: ${item.price}")
}
// Output:
// Item: Laptop, Price: $999
// Item: Mouse, Price: $29
```

---

## Special Characters

### Escaping Braces

To include literal `{` or `}` in template strings:

```liva
// Use double braces
let code = $"function() {{ return 42; }}"
print(code)  // Output: function() { return 42; }
```

### Newlines

```liva
let multiline = $"Line 1\nLine 2\nLine 3"
print(multiline)
// Output:
// Line 1
// Line 2
// Line 3
```

### Quotes

```liva
let message = $"She said, \"Hello!\""
print(message)  // Output: She said, "Hello!"

// Or use single quotes for the whole string
let message2 = $'He said, "Hi!"'
print(message2)  // Output: He said, "Hi!"
```

### Backslashes

```liva
let path = $"C:\\Users\\Alice\\Documents"
print(path)  // Output: C:\Users\Alice\Documents
```

---

## Multi-line Strings

### Single-Line Template

```liva
let message = $"Hello, {name}! Welcome to Liva."
```

### Multi-Line (with \n)

```liva
let letter = $"Dear {name},\n\nThank you for your purchase.\n\nBest regards,\nThe Team"
print(letter)
// Output:
// Dear Alice,
//
// Thank you for your purchase.
//
// Best regards,
// The Team
```

---

## Best Practices

### Use for Readable Output

```liva
// ✅ Good: String template for readability
let message = $"User {user.name} (ID: {user.id}) logged in at {timestamp}"

// ❌ Bad: Manual concatenation
let message = "User " + user.name + " (ID: " + user.id + ") logged in at " + timestamp
```

### Keep Expressions Simple

```liva
// ✅ Good: Simple expression
let greeting = $"Hello, {name}!"

// ⚠️ Acceptable but harder to read
let result = $"Total: {items.reduce((sum, item) => sum + item.price, 0)}"

// ✅ Better: Extract to variable
let total = items.reduce((sum, item) => sum + item.price, 0)
let result = $"Total: {total}"
```

### Use for Logging

```liva
// ✅ Good: Structured log messages
print($"[LOG] User {userId} performed action '{action}' at {timestamp}")

// ✅ Good: Error messages
fail $"Invalid age: {age}. Must be between {minAge} and {maxAge}"
```

### Type Coercion

All interpolated values are automatically converted to strings:

```liva
let count = 42
let isActive = true
let price = 99.99

let info = $"Count: {count}, Active: {isActive}, Price: {price}"
print(info)  // Output: Count: 42, Active: true, Price: 99.99
```

---

## Summary

### Syntax

```liva
$"text {expression} more text"
```

### Features

| Feature | Example | Output |
|---------|---------|--------|
| Variable | `$"Hello {name}"` | `Hello Alice` |
| Expression | `$"Sum: {a + b}"` | `Sum: 30` |
| Function | `$"User: {getUser()}"` | `User: Alice` |
| Member | `$"Age: {user.age}"` | `Age: 25` |
| Escape | `$"Code: {{ x }}"` | `Code: { x }` |
| Newline | `$"Line 1\nLine 2"` | `Line 1` <br> `Line 2` |

### Quick Reference

```liva
// Basic interpolation
let name = "Alice"
let age = 25
let greeting = $"Hello, {name}! You are {age} years old."

// Expressions
let result = $"The sum is {10 + 20}"

// Function calls
let message = $"Welcome, {getUsername(id)}!"

// Object properties
let profile = $"Name: {user.name}, Email: {user.email}"

// Complex expressions
let total = $"Total: ${price * (1 + tax)}"

// Logging
print($"[INFO] Processing user {userId} at {timestamp}")

// Error messages
fail $"Validation failed: {field} must be at least {minLength} characters"
```

---

**Next**: [Collections →](collections.md)

**See Also**:
- [Variables](variables.md)
- [Functions](functions.md)
- [Operators](operators.md)
