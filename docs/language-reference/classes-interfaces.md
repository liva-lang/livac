# Classes: Interfaces & Visibility

Visibility rules, instantiation patterns, and interface contracts in Liva.

## Table of Contents
- [Visibility](#visibility)
- [Instantiation](#instantiation)
- [Interfaces](#interfaces)
  - [What is an Interface?](#what-is-an-interface)
  - [Syntax](#syntax)
  - [Distinguishing Interfaces from Classes](#distinguishing-interfaces-from-classes)
  - [Multiple Interfaces](#multiple-interfaces)
  - [Interface Examples](#interface-examples)
  - [Interface Composition](#interface-composition)
  - [Semantic Validation](#semantic-validation)
  - [Rust Mapping](#rust-mapping)

---

## Visibility

### Field Visibility

```liva
User {
  name: string       // Public
  _password: string  // Private
  
  constructor(name: string, password: string) {
    this.name = name              // Public
    this._password = password     // Private
  }
}
```

### Method Visibility

```liva
BankAccount {
  balance: number
  
  constructor(balance: number) {
    this.balance = balance
  }
  
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
  
  // Methods (must implement all interface methods)
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

## See Also

- [Classes: Basics](classes-basics.md) — Class declarations, constructors, fields, methods, and method references
- [Classes: Best Practices & Data Classes](classes-data.md) — Best practices, patterns, and `data` class sugar
- [Functions](functions.md)
- [Visibility](visibility.md)
