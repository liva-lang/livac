# Classes: Basics

Fundamentals of object-oriented programming in Liva: class declarations, constructors, fields, methods, and method references.

## Table of Contents
- [Class Declaration](#class-declaration)
- [Constructors](#constructors)
- [Fields](#fields)
- [Methods](#methods)
- [Method References (`::` Syntax)](#method-references--syntax)

---

## Class Declaration

### Basic Syntax

```liva
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  greet() => $"Hello, I'm {this.name}"
}
```

### Components

1. **Class name**: PascalCase by convention
2. **Constructor**: Initializes instance
3. **Fields**: Instance variables
4. **Methods**: Instance functions

---

## Constructors

### Basic Constructor

```liva
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
}
```

### Constructor with Validation

```liva
User {
  constructor(username: string, password: string) {
    if username == "" {
      fail "Username cannot be empty"
    }
    if password.length < 8 {
      fail "Password must be at least 8 characters"
    }
    
    this.username = username
    this.password = password
  }
  
  username: string
  password: string
}
```

### Multiple Constructors

Liva supports **one constructor per class**. Use default parameters or factory functions for variants:

```liva
// Default parameters in constructor
Rectangle {
  constructor(width: number = 1.0, height: number = 1.0) {
    this.width = width
    this.height = height
  }
  
  width: number
  height: number
}

// Or factory functions
createSquare(size: number): Rectangle {
  return Rectangle(size, size)
}
```

---

## Fields

### Field Declarations

Fields are declared **after the constructor**:

```liva
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  email: string?  // Optional field
}
```

### Field Initialization

Fields are initialized in the constructor:

```liva
Counter {
  constructor() {
    this.count = 0
  }
  
  count: number
}
```

### Default Values ✅ Available in v0.10.4

Fields can have default values that are automatically used in constructors:

```liva
User {
  constructor(name: string) {
    this.name = name
    // Other fields use their defaults automatically
  }
  
  name: string
  age: int = 18           // Default value
  role: string = "user"   // Default value
  active: bool = true     // Default value
}

let user = User.new("Alice")
// age = 18, role = "user", active = true
```

**Supported Default Types:**
- `int`, `i8`-`i128`, `u8`-`u128`: Integer literals
- `float`, `f32`, `f64`: Float literals
- `string`, `String`: String literals (auto-converted to `String`)
- `bool`: `true` or `false`

**Default Constructor:**
When no constructor is defined, fields with defaults are used:

```liva
Config {
  host: string = "localhost"
  port: int = 8080
  debug: bool = false
}

let config = Config.new()
// host = "localhost", port = 8080, debug = false
```

**Optional Fields with Defaults:**
Combine `?` (optional) with `=` (default) for JSON deserialization:

```liva
Settings {
  theme: string = "dark"          // Required with default
  fontSize: int = 14              // Required with default
  autoSave?: bool = true          // Optional with default
}

let settings = Settings.new()
// theme = "dark", fontSize = 14, autoSave = Some(true)
```

See [JSON Typed Parsing](./json.md) for how defaults work with JSON deserialization.

### Computed Fields

Use methods for computed values:

```liva
Person {
  constructor(firstName: string, lastName: string) {
    this.firstName = firstName
    this.lastName = lastName
  }
  
  firstName: string
  lastName: string
  
  // Computed property via method
  fullName() => $"{this.firstName} {this.lastName}"
}
```

---

## Methods

### Basic Methods

```liva
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  // Arrow method (one-liner)
  isAdult() => this.age >= 18
  
  // Block method
  birthday() {
    this.age = this.age + 1
    print($"{this.name} is now {this.age}")
  }
}
```

### Methods with Parameters

```liva
BankAccount {
  constructor(balance: number) {
    this.balance = balance
  }
  
  balance: number
  
  deposit(amount: number) {
    if amount <= 0 fail "Amount must be positive"
    this.balance = this.balance + amount
  }
  
  withdraw(amount: number) {
    if amount > this.balance fail "Insufficient funds"
    this.balance = this.balance - amount
  }
  
  getBalance() => this.balance
}
```

### Methods with Return Types

```liva
Calculator {
  constructor() { }
  
  add(a: number, b: number): number => a + b
  
  multiply(a: number, b: number): number => a * b
  
  divide(a: number, b: number): number {
    if b == 0 fail "Division by zero"
    return a / b
  }
}
```

### Async Methods

Methods are **automatically inferred as async**:

```liva
UserService {
  constructor(apiUrl: string) {
    this.apiUrl = apiUrl
  }
  
  apiUrl: string
  
  // Automatically async (calls async function)
  fetchUser(id: number): string {
    let response = async httpGet($"{this.apiUrl}/users/{id}")
    return response.body
  }
}
```

---

## Method References (`::` Syntax)

**⭐ New in v1.1.0**

Pass an instance method as a callback using `object::method` syntax. The method is bound to the specific instance.

### Basic Usage

```liva
Formatter {
    prefix: string
    constructor(prefix: string) { this.prefix = prefix }
    format(s: string) => $"{this.prefix}: {s}"
}

main() {
    let names = ["Alice", "Bob", "Charlie"]
    let fmt = Formatter("Hello")

    // Method reference: pass fmt.format as callback
    let greetings = names.map(fmt::format)
    // Result: ["Hello: Alice", "Hello: Bob", "Hello: Charlie"]

    greetings.forEach(print)
}
```

### How It Works

`object::method` creates a closure that calls the instance method on the given object:

```liva
// These two are equivalent:
let greetings = names.map(fmt::format)
let greetings = names.map(name => fmt.format(name))
```

### Supported Array Methods

Method references work with all callback-accepting array methods:

```liva
let checker = Validator(3)

names.forEach(logger::log)         // Side effects
let labels = names.map(fmt::format) // Transform
let valid = names.filter(checker::isValid)  // Filter
let found = names.find(checker::matches)    // Search
let any = names.some(checker::isValid)      // Test any
let all = names.every(checker::isValid)     // Test all
```

> **Note:** The referenced method must accept a single argument matching the array element type. For multi-argument or complex expressions, use the standard lambda syntax.

---

## See Also

- [Classes: Interfaces & Visibility](classes-interfaces.md) — Visibility rules, instantiation, and interfaces
- [Classes: Best Practices & Data Classes](classes-data.md) — Best practices, patterns, and `data` class sugar
- [Functions](functions.md)
- [Variables](variables.md)
