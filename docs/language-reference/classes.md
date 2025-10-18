# Classes

Complete reference for object-oriented programming in Liva: classes, constructors, fields, methods, and inheritance.

## Table of Contents
- [Class Declaration](#class-declaration)
- [Constructors](#constructors)
- [Fields](#fields)
- [Methods](#methods)
- [Visibility](#visibility)
- [Instantiation](#instantiation)
- [Inheritance](#inheritance)
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
    this.name = name        // Public
    this._email = ""        // Protected
    this.__password = password  // Private
  }
  
  name: string       // Public
  _email: string     // Protected
  __password: string // Private
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
    this.__validateAmount(amount)
    this.balance = this.balance + amount
  }
  
  // Protected method
  _logTransaction(type: string, amount: number) {
    print($"[{type}] ${amount}")
  }
  
  // Private method
  __validateAmount(amount: number) {
    if amount <= 0 fail "Invalid amount"
  }
}
```

### Visibility Rules

| Prefix | Visibility | Access |
|--------|-----------|--------|
| None | **Public** | Everywhere |
| `_` | **Protected** | Same module + subclasses |
| `__` | **Private** | Same class only |

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

## Inheritance

### Basic Inheritance

```liva
// Base class
Animal {
  constructor(name: string) {
    this.name = name
  }
  
  name: string
  
  speak() => "Some sound"
}

// Derived class
Dog: Animal {
  constructor(name: string, breed: string) {
    // Call parent constructor
    super(name)
    this.breed = breed
  }
  
  breed: string
  
  // Override method
  speak() => "Woof!"
  
  // New method
  fetch() => $"{this.name} is fetching"
}
```

### Syntax

```liva
ChildClass: ParentClass {
  // ...
}
```

### Calling Parent Constructor

```liva
Employee: Person {
  constructor(name: string, age: number, employeeId: string) {
    super(name, age)  // Call parent constructor
    this.employeeId = employeeId
  }
  
  employeeId: string
}
```

### Method Overriding

```liva
Vehicle {
  constructor(brand: string) {
    this.brand = brand
  }
  
  brand: string
  
  describe() => $"A vehicle of brand {this.brand}"
}

Car: Vehicle {
  constructor(brand: string, model: string) {
    super(brand)
    this.model = model
  }
  
  model: string
  
  // Override parent method
  describe() => $"A {this.brand} {this.model}"
}
```

### Accessing Parent Methods

```liva
Employee: Person {
  constructor(name: string, age: number, department: string) {
    super(name, age)
    this.department = department
  }
  
  department: string
  
  // Call parent method
  introduce() {
    let greeting = super.greet()
    return $"{greeting}. I work in {this.department}."
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

```liva
// ✅ Good: Composition
Logger {
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

// ⚠️ Use inheritance sparingly
BaseService {
  log(msg: string) { }
}

UserService: BaseService {
  createUser(name: string) {
    this.log($"Creating user")
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
| **Inheritance** | `Child: Parent { }` | `Dog: Animal { }` |
| **Super Call** | `super(args)` | `super(name, age)` |
| **Visibility** | `name` / `_name` / `__name` | Public / Protected / Private |

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

// Inheritance
Employee: Person {
  constructor(name: string, age: number, id: string) {
    super(name, age)
    this.id = id
  }
  
  id: string
}
```

---

**Next**: [Control Flow →](control-flow.md)

**See Also**:
- [Functions](functions.md)
- [Visibility](visibility.md)
- [Variables](variables.md)
