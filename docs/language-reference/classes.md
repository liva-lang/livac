# Classes

Complete reference for object-oriented programming in Liva: classes, constructors, fields, methods, and inheritance.

## Table of Contents
- [Class Declaration](#class-declaration)
- [Constructors](#constructors)
- [Fields](#fields)
- [Methods](#methods)
- [Visibility](#visibility)
- [Instantiation](#instantiation)
- [Interfaces](#interfaces)
- [Best Practices](#best-practices)

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

## Visibility

### Field Visibility

```liva
User {
  constructor(name: string, password: string) {
    this.name = name              // Public
    this._password = password     // Private
  }
  
  name: string       // Public
  _password: string  // Private
}
```

### Method Visibility

```liva
BankAccount {
  constructor(balance: number) {
    this.balance = balance
  }
  
  balance: number
  
  // Public method
  deposit(amount: number) {
    this._validateAmount(amount)
    this.balance = this.balance + amount
  }
  
  // Private method
  _logTransaction(type: string, amount: number) {
    print($"[{type}] ${amount}")
  }
  
  // Private method
  _validateAmount(amount: number) {
    if amount <= 0 fail "Invalid amount"
  }
}
```

### Visibility Rules

| Prefix | Visibility | Access |
|--------|-----------|--------|
| None | **Public** | Everywhere |
| `_` | **Private** | Same class only |

---

## Instantiation

### Constructor Call Syntax

```liva
// Direct constructor call
let person1 = Person("Alice", 30)

// Object literal syntax
let person2 = Person {
  name: "Bob",
  age: 25
}
```

### With Type Annotations

```liva
let user: User = User("alice", "password123")
```

### Array of Instances

```liva
let people = [
  Person("Alice", 25),
  Person("Bob", 30),
  Person("Charlie", 35)
]

for person in people {
  print(person.fullName())
}
```

---

## Interfaces

### What is an Interface?

An **interface** defines a contract: a set of method signatures that a class must implement. Interfaces have **only method signatures** (no fields, no implementations).

```liva
// Interface: only method signatures, no fields
Drawable {
  draw(): void
  getBounds(): string
}

// Class implementing interface: has fields and implementations
Circle : Drawable {
  radius: float
  
  draw() => println("Drawing a circle")
  getBounds() => $"Circle(radius={this.radius})"
}
```

### Syntax

```liva
// Interface definition (no fields, only signatures)
InterfaceName {
  methodName(params): returnType
  anotherMethod(): returnType
}

// Class implementing interface
ClassName : InterfaceName {
  // Fields
  field: type
  
  // Constructor
  constructor(params) { }
  
  // Method implementations (must implement all interface methods)
  methodName(params): returnType { }
  anotherMethod(): returnType { }
}
```

### Distinguishing Interfaces from Classes

| Feature | Interface | Class |
|---------|-----------|-------|
| **Fields** | ❌ No fields allowed | ✅ Can have fields |
| **Method Bodies** | ❌ Only signatures | ✅ Must have implementations |
| **Constructor** | ❌ No constructor | ✅ Has constructor |
| **Purpose** | Define contract | Implement behavior |

```liva
// This is an INTERFACE (only signatures)
Animal {
  makeSound(): string
  getName(): string
}

// This is a CLASS (has fields)
Dog : Animal {
  name: string
  
  constructor(name: string) {
    this.name = name
  }
  
  makeSound() => "Woof!"
  getName() => this.name
}
```

### Multiple Interfaces

A class can implement **multiple interfaces** using comma-separated syntax:

```liva
// Multiple interfaces
Drawable {
  draw(): void
}

Named {
  getName(): string
}

Comparable {
  compareTo(other: Self): int
}

// Class implementing multiple interfaces
Circle : Drawable, Named, Comparable {
  radius: float
  name: string
  
  constructor(name: string, radius: float) {
    this.name = name
    this.radius = radius
  }
  
  draw() => println($"Drawing {this.name}")
  getName() => this.name
  compareTo(other: Circle) => (this.radius - other.radius) as int
}
```

### Interface Examples

#### Simple Interface

```liva
// Interface
Greetable {
  greet(): string
}

// Implementation
Person : Greetable {
  name: string
  
  constructor(name: string) {
    this.name = name
  }
  
  greet() => $"Hello, I'm {this.name}"
}

Robot : Greetable {
  id: number
  
  constructor(id: number) {
    this.id = id
  }
  
  greet() => $"Greetings, I am Robot {this.id}"
}
```

#### Interface with Multiple Methods

```liva
// Interface
Serializable {
  toJSON(): string
  fromJSON(json: string): Self
  validate(): bool
}

// Implementation
User : Serializable {
  name: string
  email: string
  
  constructor(name: string, email: string) {
    this.name = name
    this.email = email
  }
  
  toJSON() => $"{\"name\":\"{this.name}\",\"email\":\"{this.email}\"}"
  
  fromJSON(json: string): User {
    // Parse JSON and create User
    return User("parsed", "parsed@example.com")
  }
  
  validate() => this.email != "" && this.name != ""
}
```

#### Interface for Polymorphism

```liva
// Interface
Shape {
  area(): float
  perimeter(): float
}

// Implementations
Circle : Shape {
  radius: float
  
  constructor(radius: float) {
    this.radius = radius
  }
  
  area() => 3.14159 * this.radius * this.radius
  perimeter() => 2.0 * 3.14159 * this.radius
}

Rectangle : Shape {
  width: float
  height: float
  
  constructor(width: float, height: float) {
    this.width = width
    this.height = height
  }
  
  area() => this.width * this.height
  perimeter() => 2.0 * (this.width + this.height)
}

// Function accepting any Shape
calculateTotalArea(shapes: [Shape]): float {
  let total = 0.0
  for shape in shapes {
    total = total + shape.area()
  }
  return total
}

// Usage
let shapes = [
  Circle(5.0),
  Rectangle(4.0, 6.0)
]
let total = calculateTotalArea(shapes)
```

### Interface Composition

Interfaces can reference other interfaces:

```liva
// Base interfaces
Readable {
  read(): string
}

Writable {
  write(data: string): void
}

// Class implementing both
File : Readable, Writable {
  path: string
  content: string
  
  constructor(path: string) {
    this.path = path
    this.content = ""
  }
  
  read() => this.content
  
  write(data: string) {
    this.content = data
  }
}
```

### Semantic Validation

When implementing an interface, the compiler validates:

1. ✅ **All methods are implemented**
   ```liva
   Animal {
     makeSound(): string
     getName(): string
   }
   
   // ❌ Error: Missing getName() implementation
   Dog : Animal {
     makeSound() => "Woof!"
   }
   ```

2. ✅ **Method signatures match**
   ```liva
   Comparable {
     compareTo(other: Self): int
   }
   
   // ❌ Error: Wrong return type (should be int, not bool)
   Point : Comparable {
     compareTo(other: Point): bool => true
   }
   ```

3. ✅ **Parameter types match**
   ```liva
   Processor {
     process(data: string): string
   }
   
   // ❌ Error: Wrong parameter type (should be string, not int)
   DataProcessor : Processor {
     process(data: int): string => ""
   }
   ```

### Rust Mapping

Interfaces compile to Rust traits:

```liva
// Liva interface
Animal {
  makeSound(): string
}

Dog : Animal {
  name: string
  makeSound() => "Woof!"
}
```

```rust
// Generated Rust code
trait Animal {
    fn make_sound(&self) -> String;
}

struct Dog {
    name: String,
}

impl Animal for Dog {
    fn make_sound(&self) -> String {
        "Woof!".to_string()
    }
}
```

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

## Summary

| Feature | Syntax | Example |
|---------|--------|---------|
| **Class** | `ClassName { }` | `Person { }` |
| **Constructor** | `constructor(params) { }` | `constructor(name: string) { }` |
| **Field** | `fieldName: type` | `name: string` |
| **Method** | `methodName() { }` | `greet() => "Hi"` |
| **Interface** | `InterfaceName { signatures }` | `Animal { makeSound(): string }` |
| **Implements** | `Class : Interface { }` | `Dog : Animal { }` |
| **Multiple** | `Class : I1, I2 { }` | `Dog : Animal, Named { }` |
| **Visibility** | `name` / `_name` | Public / Private |

### Quick Reference

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
```

---

**Next**: [Control Flow →](control-flow.md)

**See Also**:
- [Functions](functions.md)
- [Visibility](visibility.md)
- [Variables](variables.md)
