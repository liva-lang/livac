# Classes: Best Practices & Data Classes

Design guidelines for Liva classes and the `data` class sugar syntax for concise value types.

## Table of Contents
- [Best Practices](#best-practices)
  - [Class Naming](#class-naming)
  - [Field Naming](#field-naming)
  - [Constructor Validation](#constructor-validation)
  - [Small, Focused Classes](#small-focused-classes)
  - [Immutability](#immutability)
  - [Composition Over Inheritance](#composition-over-inheritance)
- [Data Classes](#data-classes)
  - [Basic Syntax](#basic-syntax)
  - [Data Class with Methods](#data-class-with-methods)
  - [What `data` Auto-Generates](#what-data-auto-generates)
- [Summary](#summary)
- [Quick Reference](#quick-reference)

---

## Best Practices

### Class Naming

```liva
// ✅ Good: PascalCase nouns
Person { }
BankAccount { }
UserService { }

// ❌ Bad: lowercase or verbs
person { }
processUser { }
```

### Field Naming

```liva
// ✅ Good: camelCase
firstName: string
accountBalance: number
isActive: bool

// ❌ Bad: snake_case or PascalCase
first_name: string
AccountBalance: number
```

### Constructor Validation

```liva
// ✅ Good: Validate in constructor
User {
  constructor(email: string, age: number) {
    if email == "" fail "Email required"
    if age < 0 fail "Age must be positive"
    
    this.email = email
    this.age = age
  }
  
  email: string
  age: number
}
```

### Small, Focused Classes

```liva
// ✅ Good: Single responsibility
User {
  constructor(name: string, email: string) {
    this.name = name
    this.email = email
  }
  
  name: string
  email: string
  
  validate() => this.email != "" && this.name != ""
}

UserRepository {
  save(user: User) { }
  find(id: number): User { }
}

// ❌ Bad: God class
User {
  // Fields
  name: string
  email: string
  
  // Validation
  validate() { }
  
  // Database operations
  save() { }
  find(id: number) { }
  
  // Email operations
  sendEmail() { }
  
  // Logging
  log() { }
}
```

### Immutability

```liva
// ✅ Good: Provide methods to change state
Counter {
  constructor() {
    this.__count = 0
  }
  
  __count: number
  
  increment() {
    this.__count = this.__count + 1
  }
  
  getCount() => this.__count
}

// ❌ Bad: Direct field access
Counter {
  constructor() {
    this.count = 0  // Public field
  }
  
  count: number
}
```

### Composition Over Inheritance

Prefer **composition** (embedding objects) over **inheritance** (extending classes):

```liva
// ✅ Good: Composition
Logger {
  constructor() { }
  
  log(msg: string) {
    print($"[LOG] {msg}")
  }
}

UserService {
  constructor(logger: Logger) {
    this.logger = logger
  }
  
  logger: Logger
  
  createUser(name: string) {
    this.logger.log($"Creating user: {name}")
    // ...
  }
}

// ✅ Also good: Interfaces for contracts
Loggable {
  log(msg: string): void
}

UserService : Loggable {
  constructor() { }
  
  log(msg: string) {
    print($"[UserService] {msg}")
  }
  
  createUser(name: string) {
    this.log($"Creating user: {name}")
  }
}
```

---

## Data Classes

**⭐ Available since v1.2.0**

The `data` keyword provides sugar syntax to declare simple value-oriented classes without boilerplate. A `data` class auto-generates a constructor, field declarations, `PartialEq`, and `Display`.

### Basic Syntax

```liva
// data keyword auto-generates constructor, fields, PartialEq, and Display
data Point {
    x: number
    y: number
}

let p = Point(10, 20)
print(p)  // "Point { x: 10, y: 20 }" (auto Display)
print(p == Point(10, 20))  // true (auto PartialEq)
```

This is equivalent to writing a full class with constructor, fields, `PartialEq`, and `Display` manually — but in far fewer lines.

### Data Class with Methods

You can add methods to a `data` class just like a regular class:

```liva
data Color {
    r: number
    g: number
    b: number

    sum() => r + g + b
}

let c = Color(255, 128, 0)
print(c.sum())  // 383
```

### What `data` Auto-Generates

| Feature | Regular Class | `data` Class |
|---------|--------------|-------------|
| **Constructor** | Must write manually | ✅ Auto-generated from fields |
| **Fields** | Must declare explicitly | ✅ Declared in body, auto-wired |
| **`PartialEq`** | Not derived | ✅ Auto-derived (structural equality) |
| **`Display`** | Not derived | ✅ Auto-derived (`ClassName { field: value, ... }`) |
| **Methods** | ✅ Supported | ✅ Supported |

### When to Use `data`

- **Use `data`** for simple value types: points, colors, configs, DTOs
- **Use regular classes** when you need custom constructor logic, validation, or complex initialization

```liva
// ✅ Perfect for data class
data Coordinate {
    lat: float
    lon: float
}

// ❌ Needs regular class (validation in constructor)
User {
  constructor(email: string, age: number) {
    if email == "" fail "Email required"
    this.email = email
    this.age = age
  }
  
  email: string
  age: number
}
```

---

## Summary

| Feature | Syntax | Example |
|---------|--------|---------|
| **Class** | `ClassName { }` | `Person { }` |
| **Constructor** | `constructor(params) { }` | `constructor(name: string) { }` |
| **Field** | `fieldName: type` | `name: string` |
| **Method** | `methodName() { }` | `greet() => "Hi"` |
| **Method ref** | `object::method` | `names.map(fmt::format)` |
| **Interface** | `InterfaceName { signatures }` | `Animal { makeSound(): string }` |
| **Implements** | `Class : Interface { }` | `Dog : Animal { }` |
| **Multiple** | `Class : I1, I2 { }` | `Dog : Animal, Named { }` |
| **Visibility** | `name` / `_name` | Public / Private |
| **Data class** | `data ClassName { fields }` | `data Point { x: number, y: number }` |

---

## Quick Reference

```liva
// Basic class
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  isAdult() => this.age >= 18
  
  birthday() {
    this.age = this.age + 1
  }
}

// Instantiation
let alice = Person("Alice", 30)
let bob = Person {
  name: "Bob",
  age: 25
}

// Interface
Animal {
  makeSound(): string
  getName(): string
}

// Implementation
Dog : Animal {
  constructor(name: string) {
    this.name = name
  }
  
  name: string
  
  makeSound() => "Woof!"
  getName() => this.name
}

// Data class
data Point {
  x: number
  y: number
}

let p = Point(10, 20)
print(p)              // "Point { x: 10, y: 20 }"
print(p == Point(10, 20))  // true
```

---

**Next**: [Control Flow →](control-flow.md)

---

## See Also

- [Classes: Basics](classes-basics.md) — Class declarations, constructors, fields, methods, and method references
- [Classes: Interfaces & Visibility](classes-interfaces.md) — Visibility rules, instantiation, and interfaces
- [Functions](functions.md)
- [Visibility](visibility.md)
- [Variables](variables.md)
