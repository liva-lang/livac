# Types: Primitives & Basics

Liva has a strong, static type system with full type inference.

## Table of Contents

1. [Type Philosophy](#type-philosophy)
2. [Basic Types](#basic-types)
3. [Type Inference](#type-inference)
4. [Collection Types](#collection-types)
5. [Object Types](#object-types)
6. [Function Types](#function-types)
7. [Optional Types](#optional-types-future)
8. [Union Types](#union-types-future)
9. [Type Aliases](#type-aliases-future)
10. [Generic Types](#generic-types-future)

---

## Type Philosophy

- **Strong typing**: No implicit conversions that lose information
- **Type inference**: Types are inferred when obvious
- **Explicit when needed**: Annotations for clarity and APIs
- **Rust compatibility**: All Rust primitive types available

## Basic Types

### Number Types

```liva
// Default number (i32)
let count = 42
let score: number = 100

// Floating point (f64)
let pi = 3.1416
let temp: float = 21.5

// Explicit Rust types
let tiny: i8 = 127
let big: u64 = 1000000
let precise: f32 = 3.14159
```

### Signed Integers

| Type | Size | Range |
|------|------|-------|
| `i8` | 8-bit | -128 to 127 |
| `i16` | 16-bit | -32,768 to 32,767 |
| `i32` | 32-bit | -2,147,483,648 to 2,147,483,647 |
| `i64` | 64-bit | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 |
| `i128` | 128-bit | Very large range |
| `isize` | Platform | 32-bit or 64-bit depending on platform |

**Alias:** `number` = `i32`

### Unsigned Integers

| Type | Size | Range |
|------|------|-------|
| `u8` | 8-bit | 0 to 255 |
| `u16` | 16-bit | 0 to 65,535 |
| `u32` | 32-bit | 0 to 4,294,967,295 |
| `u64` | 64-bit | 0 to 18,446,744,073,709,551,615 |
| `u128` | 128-bit | Extremely large |
| `usize` | Platform | Used for array sizes/indices |

### Floating Point

| Type | Size | Precision | Range |
|------|------|-----------|-------|
| `f32` | 32-bit | ~7 decimal digits | ±3.4e38 |
| `f64` | 64-bit | ~15 decimal digits | ±1.7e308 |

**Alias:** `float` = `f64`

### Boolean

```liva
let isActive: bool = true
let isDone = false

// Boolean expressions
let canVote = age >= 18
let hasAccess = isAdmin or isModerator
```

### Character

```liva
let letter: char = 'A'
let emoji: char = '😀'
let newline: char = '\n'
```

**Note:** Liva `char` is a Unicode scalar value (like Rust), not just ASCII.

### String

```liva
// String type
let name: string = "Alice"
let message = "Hello, World!"

// String templates
let greeting = $"Hello, {name}!"

// Multi-line strings
let poem = "Roses are red,
Violets are blue,
Liva is awesome,
And so are you!"
```

**Note:** Liva `string` maps to Rust's `String` (heap-allocated, growable).

## Type Inference

Liva infers types automatically in most cases:

```liva
// Inferred as i32
let count = 42

// Inferred as f64
let pi = 3.14159

// Inferred as String
let name = "Alice"

// Inferred as bool
let isActive = true

// Inferred from function return type
sum(a: number, b: number): number => a + b
let result = sum(10, 20)  // result: number
```

### When to Use Type Annotations

Use explicit types when:

1. **API boundaries** - Public functions and class fields
2. **Ambiguity** - When the type isn't obvious
3. **Documentation** - For clarity
4. **Specific precision** - When you need a specific type

```liva
// ✅ Good: Clear API
calculateArea(width: float, height: float): float {
  return width * height
}

// ✅ Good: Disambiguate
let count: u32 = 100  // Explicitly unsigned

// ❌ Unnecessary: Type is obvious
let x: number = 42    // Just use: let x = 42
```

## Collection Types

### Arrays

Fixed-size, homogeneous collections:

```liva
// Inferred as array of numbers
let numbers = [1, 2, 3, 4, 5]

// Explicit type
let scores: [number] = [85, 90, 78]

// Access
let first = numbers[0]
print($"Length: {numbers.length}")
```

### Vectors (Dynamic Arrays)

Growable arrays:

```liva
// Create vector
let mut items = vec![1, 2, 3]

// Add elements
items.push(4)
items.push(5)

// Access
let first = items[0]
print($"Length: {items.length}")
```

## Object Types

### Object Literals

```liva
// Anonymous object
let person = {
  name: "Alice",
  age: 30,
  email: "alice@example.com"
}

// Access fields
print($"Name: {person.name}")
print($"Age: {person.age}")
```

### Classes

Classes define custom types:

```liva
// Define class type
User {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  greet() {
    print($"Hello, I'm {this.name}")
  }
}

// Use the type
let user: User = User("Bob", 25)
```

## Function Types

Functions have types too:

```liva
// Function with explicit type signature
add(a: number, b: number): number => a + b

// Function as parameter (future feature)
apply(f: (number, number) -> number, a: number, b: number): number {
  return f(a, b)
}
```

## Optional Types (Future)

Planned for future versions:

```liva
// Maybe type
let maybeValue: number? = null

// Check before use
if maybeValue != null {
  print($"Value: {maybeValue}")
}
```

## Union Types (Future)

Planned for future versions:

```liva
// Union of types
let value: number | string = "hello"

// Type narrowing
if value is string {
  print($"String: {value}")
} else {
  print($"Number: {value}")
}
```

## Type Aliases (Future)

Create aliases for complex types:

```liva
// Type alias
type UserId = number
type Callback = (string) -> void

let id: UserId = 123
```

## Generic Types (Future)

Planned for future versions:

```liva
// Generic class
Box<T> {
  value: T
  
  constructor(v: T) {
    this.value = v
  }
  
  get(): T {
    return this.value
  }
}

// Generic function
identity<T>(x: T): T => x

let box = Box<number>(42)
let value = identity<string>("hello")
```

---

## See Also

- **[Types: Advanced](types-advanced.md)** - Tuple types, type conversions, type checking rules, and more
- **[Variables & Constants](variables.md)** - Variable declarations
- **[Functions](functions-basics.md)** - Function type signatures
- **[Classes](classes-basics.md)** - Custom types
- **[Collections](collections.md)** - Arrays and data structures
- **[Generics](generics-basics.md)** - Generic type parameters
