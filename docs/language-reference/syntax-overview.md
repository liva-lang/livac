# 📝 Syntax Overview

Complete overview of Liva language syntax.

## Comments

```liva
// Single-line comment

/*
  Multi-line comment
  Can span multiple lines
*/
```

## Variables and Constants

```liva
// Mutable variable
let x = 10
let name = "Alice"
let score: number = 100

// Reassignment
x = 20
name = "Bob"

// Immutable constant
const PI = 3.1416
const MAX_USERS = 100
```

## Functions

### One-Liner Functions

```liva
// Simple expression
add(a, b) => a + b

// With types
square(x: number): number => x * x

// Boolean expression
isAdult(age: number): bool => age >= 18
```

### Block Functions

```liva
// Without types (inferred)
greet(name) {
  print($"Hello, {name}!")
}

// With types
calculateTotal(items: [Item], tax: float): float {
  let total = 0.0
  for item in items {
    total = total + item.price
  }
  return total * (1.0 + tax)
}
```

### Parameters

```liva
// Simple parameters
func(a, b, c) { }

// With types
func(a: number, b: string, c: bool) { }

// Default values (future)
func(a: number, b: number = 10) { }
```

### Return Types

```liva
// Inferred
add(a, b) => a + b

// Explicit
divide(a: number, b: number): number {
  return a / b
}

// Void (no return)
log(message: string) {
  print(message)
  // Implicit return
}
```

## Classes

### Basic Class

```liva
Person {
  // Constructor
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  // Fields
  name: string
  age: number
  
  // Methods
  greet() {
    print($"Hi, I'm {this.name}")
  }
  
  // Method with return type
  isAdult(): bool {
    return this.age >= 18
  }
}
```

### Visibility Modifiers

```liva
User {
  // Public (default)
  name: string
  
  // Protected (pub(super) in Rust)
  _age: number
  
  // Private (not pub in Rust)
  __password: string
  
  // Public method
  getName(): string {
    return this.name
  }
  
  // Protected method
  _getAge(): number {
    return this._age
  }
  
  // Private method
  __validatePassword(): bool {
    return this.__password != ""
  }
}
```

### Creating Instances

```liva
// Constructor syntax
let person1 = Person("Alice", 30)

// Object literal syntax
let person2 = Person {
  name: "Bob",
  age: 25
}
```

## Control Flow

### If Statements

```liva
// Simple if
if condition {
  // code
}

// If-else
if condition {
  // code
} else {
  // code
}

// If-else if-else
if condition1 {
  // code
} else if condition2 {
  // code
} else {
  // code
}

// With logical operators
if age >= 18 and hasId {
  print("Can enter")
}

if isAdmin or isModerator {
  print("Has access")
}
```

### For Loops

```liva
// Range (inclusive..exclusive)
for i in 0..10 {
  print($"Count: {i}")
}

// Array iteration
let items = [1, 2, 3, 4, 5]
for item in items {
  print($"Item: {item}")
}

// With index (future)
for i, item in items {
  print($"{i}: {item}")
}
```

### While Loops

```liva
let counter = 0
while counter < 10 {
  print($"Counter: {counter}")
  counter = counter + 1
}

// Infinite loop (future)
while true {
  // code
  if condition break
}
```

### Switch Statements

```liva
let status = "active"

switch status {
  case "active": print("User is active")
  case "inactive": print("User is inactive")
  case "banned": print("User is banned")
  default: print("Unknown status")
}

// With values
switch age {
  case 0..18: print("Minor")
  case 18..65: print("Adult")
  case 65..: print("Senior")
}
```

## Operators

### Arithmetic

```liva
let a = 10 + 5    // Addition
let b = 10 - 5    // Subtraction
let c = 10 * 5    // Multiplication
let d = 10 / 5    // Division
let e = 10 % 3    // Modulo (remainder)
```

### Comparison

```liva
a == b    // Equal
a != b    // Not equal
a < b     // Less than
a <= b    // Less than or equal
a > b     // Greater than
a >= b    // Greater than or equal
```

### Logical

```liva
// Word operators (preferred)
a and b   // Logical AND
a or b    // Logical OR
not a     // Logical NOT

// Symbol operators (also supported)
a && b    // Logical AND
a || b    // Logical OR
!a        // Logical NOT
```

### Assignment

```liva
x = 10       // Assignment
x += 5       // Add and assign (future)
x -= 3       // Subtract and assign (future)
x *= 2       // Multiply and assign (future)
x /= 4       // Divide and assign (future)
```

## String Templates

```liva
let name = "Alice"
let age = 30

// String interpolation
let message = $"Hello, {name}!"
let info = $"{name} is {age} years old"

// With expressions
let math = $"2 + 2 = {2 + 2}"
let call = $"Result: {calculate(10, 20)}"
```

## Arrays and Objects

### Arrays

```liva
// Array literal
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]

// Access
let first = numbers[0]
let last = numbers[numbers.length - 1]

// Length
print($"Length: {numbers.length}")
```

### Objects

```liva
// Object literal
let person = {
  name: "Alice",
  age: 30,
  email: "alice@example.com"
}

// Access
print($"Name: {person.name}")
print($"Age: {person.age}")

// Nested
let user = {
  info: {
    name: "Bob",
    age: 25
  },
  settings: {
    theme: "dark",
    language: "en"
  }
}

print($"Name: {user.info.name}")
print($"Theme: {user.settings.theme}")
```

### Array of Objects

```liva
let users = [
  { name: "Alice", age: 30 },
  { name: "Bob", age: 25 },
  { name: "Charlie", age: 35 }
]

for user in users {
  print($"{user.name} is {user.age} years old")
}
```

## Concurrency

### Async (I/O-bound)

```liva
// Async call (lazy, awaited on use)
let data = async fetchFromAPI()

// Task handle (explicit await)
let task = task async fetchUser(123)
let user = await task

// Fire and forget
fire async logEvent("started")
```

### Parallel (CPU-bound)

```liva
// Parallel call (lazy, joined on use)
let result = par heavyComputation()

// Task handle (explicit join)
let task = task par calculatePi()
let pi = await task

// Fire and forget
fire par backgroundCleanup()
```

### Data-Parallel Loops

```liva
// Parallel for
let items = [1, 2, 3, 4, 5, 6, 7, 8]
for par item in items with chunk 2 threads 4 {
  process(item)
}

// ParVec (SIMD)
let data = [1, 2, 3, 4, 5, 6, 7, 8]
for parvec lane in data with simdWidth 4 ordered {
  vectorProcess(lane)
}
```

## Error Handling

### Fail Keyword

```liva
// Inline fail (ternary)
divide(a, b) => b == 0 ? fail "Division by zero" : a / b

// Block fail
validateUser(username: string, password: string): string {
  if username == "" fail "Username cannot be empty"
  if password == "" fail "Password cannot be empty"
  if password.length < 8 fail "Password too short"
  return $"User {username} validated"
}
```

### Error Binding

```liva
// Capture both result and error
let result, err = divide(10, 2)

if err != "" {
  print($"Error: {err}")
} else {
  print($"Result: {result}")
}

// With async
let data, err = async fetchData()

// With par
let result, err = par processData()

// With task
let task = task async operation()
let value, err = await task
```

### Ignore Error

```liva
// Ignore error with _
let result, _ = divide(10, 2)

// Ignore result, keep error
let _, err = validate(input)
```

## Types

### Type Annotations

```liva
// Variables
let count: number = 100
let name: string = "Alice"
let active: bool = true

// Function parameters
func(a: number, b: string, c: bool) { }

// Function return type
calculate(x: number): number {
  return x * 2
}

// Class fields
Person {
  name: string
  age: number
  email: string
}
```

### Rust Types

```liva
// Signed integers
let a: i8 = 127
let b: i16 = 32767
let c: i32 = 2147483647
let d: i64 = 9223372036854775807

// Unsigned integers
let e: u8 = 255
let f: u16 = 65535
let g: u32 = 4294967295
let h: u64 = 18446744073709551615

// Floats
let i: f32 = 3.14
let j: f64 = 3.14159265359

// Size types
let k: usize = 100
let l: isize = -50
```

## Literals

### Numbers

```liva
let decimal = 42
let hex = 0xFF
let octal = 0o77
let binary = 0b1010

let float = 3.14
let scientific = 1.23e-4
```

### Strings

```liva
let simple = "Hello"
let template = $"Hello, {name}"
let multiline = "Line 1
Line 2
Line 3"

// Escape sequences
let escaped = "Quote: \" Newline: \n Tab: \t"
```

### Booleans

```liva
let yes = true
let no = false
```

### Arrays

```liva
let empty = []
let numbers = [1, 2, 3]
let strings = ["a", "b", "c"]
let mixed = [1, "two", 3]  // Future: will require explicit types
```

### Objects

```liva
let empty = {}
let person = { name: "Alice", age: 30 }
let nested = { 
  user: { name: "Bob" },
  settings: { theme: "dark" }
}
```

## Keywords

Complete list of Liva keywords:

```
let           // Variable declaration
const         // Constant declaration
if            // Conditional
else          // Alternative branch
for           // Loop
in            // Loop iteration
while         // Loop with condition
switch        // Multi-way branch
case          // Switch case
default       // Switch default
return        // Return from function
async         // Async execution
par           // Parallel execution
task          // Task handle
fire          // Fire and forget
await         // Explicit await/join
fail          // Error return
and           // Logical AND
or            // Logical OR
not           // Logical NOT
true          // Boolean true
false         // Boolean false
this          // Class instance reference
constructor   // Class constructor
use           // Import (future)
import        // Module import (future)
export        // Module export (future)
```

## Reserved for Future Use

```
break         // Loop break
continue      // Loop continue
match         // Pattern matching
enum          // Enumeration
trait         // Interface/trait
impl          // Implementation
type          // Type alias
struct        // Structure
pub           // Public visibility
mod           // Module
fn            // Function (alternative syntax)
async fn      // Async function
```

## Syntax Comparison

### Liva vs TypeScript

```liva
// Liva
add(a: number, b: number): number => a + b

Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  name: string
  age: number
}
```

```typescript
// TypeScript
function add(a: number, b: number): number { return a + b }

class Person {
  name: string;
  age: number;
  constructor(name: string, age: number) {
    this.name = name;
    this.age = age;
  }
}
```

### Liva vs Python

```liva
// Liva
def greet(name: string) {
  print($"Hello, {name}!")
}

for i in 0..10 {
  print(i)
}
```

```python
# Python
def greet(name: str):
    print(f"Hello, {name}!")

for i in range(10):
    print(i)
```

### Liva vs Rust

```liva
// Liva
divide(a: number, b: number) => b == 0 ? fail "Div by zero" : a / b

let result, err = divide(10, 2)
if err != "" {
  print($"Error: {err}")
}
```

```rust
// Rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Div by zero".to_string())
    } else {
        Ok(a / b)
    }
}

match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}
```

## See Also

- **[Types](types.md)** - Type system details
- **[Functions](functions.md)** - Function reference
- **[Classes](classes.md)** - Class reference
- **[Control Flow](control-flow.md)** - Control structures
- **[Concurrency](concurrency.md)** - Async and parallel
- **[Error Handling](error-handling.md)** - Fallibility system

---

**Next:** [Types](types.md)
