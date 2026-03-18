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
  email: string
  age: number
  
  constructor(email: string, age: number) {
    if email == "" fail "Email required"
    if age < 0 fail "Age must be positive"
    
    this.email = email
    this.age = age
  }
}
```

### Small, Focused Classes

```liva
// ✅ Good: Single responsibility
User {
  name: string
  email: string
  
  constructor(name: string, email: string) {
    this.name = name
    this.email = email
  }
  
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
  _count: number
  
  constructor() {
    this._count = 0
  }
  
  increment() {
    this._count = this._count + 1
  }
  
  getCount() => this._count
}

// ❌ Bad: Direct field access
Counter {
  count: number
  
  constructor() {
    this.count = 0  // Public field
  }
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
  logger: Logger
  
  constructor(logger: Logger) {
    this.logger = logger
  }
  
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

**⭐ Available since v1.2.0** | **🔄 Auto-detected since v1.3.0** (no `data` keyword needed)

A class with fields but **no explicit constructor** is automatically a data class. The compiler auto-generates a positional constructor, `PartialEq`, and `Display`.

> **Breaking change (v1.3.0):** The `data` keyword has been removed. Classes are now auto-detected as data classes based on their structure — consistent with Liva's philosophy of avoiding unnecessary keywords.

### Basic Syntax

```liva
// No keyword needed — just declare fields without a constructor
Point {
    x: number
    y: number
}

let p = Point(10, 20)
print(p)  // "Point { x: 10, y: 20 }" (auto Display)
print(p == Point(10, 20))  // true (auto PartialEq)
```

This is equivalent to writing a full class with constructor, fields, `PartialEq`, and `Display` manually — but in far fewer lines.

### Data Class with Methods

You can add methods and the class is still auto-detected:

```liva
Color {
    r: number
    g: number
    b: number

    sum() => r + g + b
}

let c = Color(255, 128, 0)
print(c.sum())  // 383
```

### What Auto-Detection Generates

| Feature | Class with constructor | Class without constructor (data class) |
|---------|----------------------|----------------------------------------|
| **Constructor** | Your custom logic | ✅ Auto-generated positional `new(field1, field2, ...)` |
| **`PartialEq`** | Not derived | ✅ Auto-derived (structural equality) |
| **`Display`** | Not derived | ✅ Auto-derived (`ClassName { field: value, ... }`) |
| **Methods** | ✅ Supported | ✅ Supported |

### The Rule

| Has explicit `constructor()`? | Result |
|------|--------|
| **No** (fields only, or fields + methods) | ✅ **Data class** — auto constructor, PartialEq, Display |
| **Yes** | Regular class — you control initialization |

### When to Use Each

```liva
// ✅ Data class (auto-detected: no constructor)
Coordinate {
    lat: float
    lon: float
}

// ✅ Regular class (has constructor for validation)
User {
  email: string
  age: number
  
  constructor(email: string, age: number) {
    if email == "" fail "Email required"
    this.email = email
    this.age = age
  }
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
| **Data class** | `ClassName { fields }` (no constructor) | `Point { x: number, y: number }` |

---

## Quick Reference

```liva
// Basic class (has constructor → regular class)
Person {
  name: string
  age: number
  
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  isAdult() => this.age >= 18
  
  birthday() {
    this.age = this.age + 1
  }
}

// Instantiation
let alice = Person("Alice", 30)

// Interface
Animal {
  makeSound(): string
  getName(): string
}

// Implementation
Dog : Animal {
  name: string
  
  constructor(name: string) {
    this.name = name
  }
  
  makeSound() => "Woof!"
  getName() => this.name
}

// Data class (no constructor → auto-detected)
Point {
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
