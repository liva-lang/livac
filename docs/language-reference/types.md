# 📐 Types and Type System

Liva has a strong, static type system with full type inference.

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

## Type Conversions

### Explicit Conversions

```liva
// String to number
let str = "123"
let num = parseInt(str)
print($"Number: {num}")

// Number to string
let n = 42
let s = toString(n)
print($"String: {s}")

// Float conversions
let f = parseFloat("3.14")
print($"Float: {f}")
```

### Type Casting (Rust-style)

```liva
// Explicit cast
let x: i32 = 42
let y: i64 = x as i64
let z: f64 = x as f64
```

## Type Checking Rules

### Assignment Compatibility

```liva
// ✅ Compatible
let x: number = 42
let y: i32 = 42

// ❌ Incompatible (future: will error)
let a: number = 3.14  // float can't go into number without cast
```

### Function Argument Matching

```liva
sum(a: number, b: number): number => a + b

// ✅ Correct
let result = sum(10, 20)

// ❌ Wrong type (future: will error)
let bad = sum(10, 3.14)  // f64 doesn't match i32
```

### Return Type Checking

```liva
// Function must return declared type
calculate(): number {
  let result = 10 + 20
  return result  // ✅ i32 matches number
}

// ❌ Wrong return type (future: will error)
getAge(): number {
  return "25"  // string doesn't match number
}
```

## Type Inference in Different Contexts

### Variable Declarations

```liva
// From literal
let x = 42        // i32
let y = 3.14      // f64
let s = "hello"   // String
let b = true      // bool

// From expression
let sum = 10 + 20           // i32
let product = 3.5 * 2.0     // f64
let message = $"Hi {name}"  // String
```

### Function Returns

```liva
// Inferred from return expression
double(x) {
  return x * 2  // If x is i32, return is i32
}

// Explicit return type (better for APIs)
triple(x: number): number {
  return x * 3
}
```

### Array Elements

```liva
// All elements must have same type
let nums = [1, 2, 3]        // [i32; 3]
let strs = ["a", "b", "c"]  // [String; 3]

// ❌ Mixed types not allowed (future: will error)
let mixed = [1, "two", 3.0]
```

## Nullability and Error Types

Liva doesn't have `null` or `undefined`. Instead:

### Error Binding

```liva
// Functions can fail
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

// Error binding captures both result and error
let result, err = divide(10, 2)

if err != "" {
  print($"Error: {err}")
} else {
  print($"Result: {result}")
}
```

Error values are **strings** (`""`= no error).

## Platform-Specific Types

### Size Types

```liva
// Platform-dependent size (32-bit or 64-bit)
let index: usize = 0
let offset: isize = -10

// Use for array indexing
let items = [1, 2, 3, 4, 5]
let i: usize = 2
let item = items[i]
```

### Pointer-Sized Types

- `usize`: Unsigned, pointer-sized (for array sizes, indices)
- `isize`: Signed, pointer-sized (for offsets)

## Type System Roadmap

Current and planned features:

| Feature | Status |
|---------|--------|
| Basic primitives | ✅ v0.6 |
| Arrays | ✅ v0.6 |
| Objects/Classes | ✅ v0.6 |
| Type inference | ✅ v0.6 (basic) |
| Rust type compatibility | ✅ v0.6 |
| Explicit type checking | 🚧 v0.7 |
| Optional types (`?`) | 📋 v0.8 |
| Union types (`\|`) | 📋 v0.8 |
| Generic types (`<T>`) | 📋 v0.8 |
| Type aliases | 📋 v0.8 |
| Traits/Interfaces | 📋 v0.9 |

## Rust Interop

All Rust primitive types work in Liva:

```liva
// Use any Rust type
let byte: u8 = 255
let huge: u128 = 340282366920938463463374607431768211455
let ptr: usize = 0x1000

// Rust standard types (future)
use rust "std::collections::HashMap" as HashMap

let map: HashMap<string, number> = HashMap.new()
```

## Best Practices

### 1. Use Type Inference

```liva
// ✅ Good: Let the compiler infer
let count = 42
let name = "Alice"

// ❌ Unnecessary: Type is obvious
let count: number = 42
let name: string = "Alice"
```

### 2. Annotate Public APIs

```liva
// ✅ Good: Clear API contract
calculateTotal(items: [Item], tax: float): float {
  // ...
}

// ❌ Bad: Unclear API
calculateTotal(items, tax) {
  // What types?
}
```

### 3. Choose Appropriate Precision

```liva
// ✅ Good: Use smallest type that fits
let age: u8 = 25        // 0-255 is enough
let count: u32 = 1000   // Reasonable range

// ❌ Bad: Overkill
let age: u128 = 25      // Way too large
```

### 4. Be Explicit with Conversions

```liva
// ✅ Good: Clear intent
let x: i32 = 42
let y: f64 = x as f64

// ❌ Bad: Implicit conversion (doesn't work in Liva)
let y: f64 = x
```

## See Also

- **[Variables & Constants](variables.md)** - Variable declarations
- **[Functions](functions.md)** - Function type signatures
- **[Classes](classes.md)** - Custom types
- **[Collections](collections.md)** - Arrays and data structures
- **[Type Conversions API](../api/conversions.md)** - Conversion functions

---

**Next:** [Variables & Constants](variables.md)
