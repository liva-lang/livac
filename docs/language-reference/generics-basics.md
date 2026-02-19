# Generics: Basics

**Version:** 0.9.0  
**Status:** Specification  
**Last Updated:** 2025-10-23

---

## Table of Contents

1. [Overview](#overview)
2. [Motivation](#motivation)
3. [Syntax](#syntax)
4. [Generic Functions](#generic-functions)
5. [Generic Classes](#generic-classes)
6. [Generic Interfaces](#generic-interfaces)
7. [Type Constraints](#type-constraints)
8. [Type Inference](#type-inference)

---

## Overview

Generics enable **parametric polymorphism** in Liva, allowing you to write code that works with multiple types while maintaining type safety. This document specifies the syntax, semantics, and implementation details of Liva's generic system.

---

## Motivation

Without generics, you need to write duplicate code for different types:

```liva
// Without generics - code duplication
fn firstInt(arr: int[]): int {
    return arr[0]
}

fn firstString(arr: string[]): string {
    return arr[0]
}

fn firstFloat(arr: float[]): float {
    return arr[0]
}
```

With generics, write once, use everywhere:

```liva
// With generics - code reuse
fn first<T>(arr: T[]): T {
    return arr[0]
}

let num = first([1, 2, 3])        // T = int
let str = first(["a", "b", "c"])  // T = string
let flt = first([1.5, 2.5, 3.5])  // T = float
```

---

## Syntax

### Type Parameters

Type parameters are declared using angle brackets `<T>`:

```liva
<T>              // Single type parameter
<T, U>           // Multiple type parameters
<K, V>           // Convention: K for keys, V for values
<T: Comparable>  // Type parameter with constraint
```

### Type Arguments

Type arguments are provided when using a generic type:

```liva
Array<int>           // Array of integers
Map<string, int>     // Map from strings to integers
Result<User, Error>  // Result with User or Error
```

---

## Generic Functions

### Basic Syntax

```liva
fn identity<T>(value: T): T {
    return value
}

let x = identity(42)        // T inferred as int
let y = identity("hello")   // T inferred as string
```

### Multiple Type Parameters

```liva
fn pair<T, U>(first: T, second: U): [T, U] {
    return [first, second]
}

let p = pair(42, "answer")  // [int, string]
```

### Generic Array Functions

```liva
fn map<T, U>(arr: T[], fn: (T) => U): U[] {
    let result: U[] = []
    for item in arr {
        result.push(fn(item))
    }
    return result
}

let numbers = [1, 2, 3]
let strings = map(numbers, x => toString(x))  // ["1", "2", "3"]
```

### Return Type Inference

```liva
fn wrap<T>(value: T) {  // Return type inferred as T
    return value
}
```

---

## Generic Classes

### Basic Syntax

```liva
class Box<T> {
    value: T
    
    constructor(value: T) {
        this.value = value
    }
    
    fn get(): T {
        return this.value
    }
    
    fn set(newValue: T) {
        this.value = newValue
    }
}

let intBox = Box(42)
let strBox = Box("hello")
```

### Multiple Type Parameters

```liva
class Pair<K, V> {
    key: K
    value: V
    
    constructor(key: K, value: V) {
        this.key = key
        this.value = value
    }
    
    fn getKey(): K {
        return this.key
    }
    
    fn getValue(): V {
        return this.value
    }
}

let entry = Pair("name", "Alice")  // Pair<string, string>
```

### Generic Methods

```liva
class Container<T> {
    items: T[]
    
    constructor() {
        this.items = []
    }
    
    fn add(item: T) {
        this.items.push(item)
    }
    
    fn map<U>(fn: (T) => U): Container<U> {
        let result = Container<U>()
        for item in this.items {
            result.add(fn(item))
        }
        return result
    }
}
```

---

## Generic Interfaces

### Basic Syntax

```liva
interface Comparable<T> {
    fn compareTo(other: T): int
}

class Person : Comparable<Person> {
    name: string
    age: int
    
    fn compareTo(other: Person): int {
        return this.age - other.age
    }
}
```

### Multiple Type Parameters

```liva
interface Mapper<T, U> {
    fn map(value: T): U
}

class StringToInt : Mapper<string, int> {
    fn map(value: string): int {
        let result, err = parseInt(value)
        return result
    }
}
```

---

## Type Constraints

### Trait Aliases (Recommended - Simple and Intuitive) ✨ New in v0.9.1

For common use cases, use **trait aliases** that group related traits:

```liva
// Numeric: All arithmetic operations
sum<T: Numeric>(a: T, b: T): T => a + b
multiply<T: Numeric>(a: T, b: T): T => a * b
negate<T: Numeric>(value: T): T => -value

// Comparable: Equality and ordering  
max<T: Comparable>(a: T, b: T): T {
    if a > b { return a }
    return b
}

// Number: Numeric + Comparable (complete number operations)
clamp<T: Number>(value: T, min_val: T, max_val: T): T {
    if value < min_val { return min_val }
    if value > max_val { return max_val }
    return value
}

// Printable: Display + Debug
showValue<T: Printable>(value: T) {
    console.log(value)
}
```

**Built-in Trait Aliases:**

| Alias | Expands To | Use Case |
|-------|-----------|----------|
| `Numeric` | `Add + Sub + Mul + Div + Rem + Neg` | Arithmetic operations |
| `Comparable` | `Ord + Eq` | Comparisons and equality |
| `Number` | `Numeric + Comparable` | Complete number operations |
| `Printable` | `Display + Debug` | Formatting and debugging |

### Granular Traits (For Fine Control)

When you need precise control, use individual traits:

```liva
// Only addition (more restrictive than Numeric)
addOnly<T: Add>(a: T, b: T): T => a + b

// Only ordering (more restrictive than Comparable)
lessThan<T: Ord>(a: T, b: T): bool => a < b

// Specific combination
sumAndCompare<T: Add + Ord>(a: T, b: T): T {
    let sum_val = a + b
    if sum_val > a { return sum_val }
    return a
}
```

**Available Individual Traits:**
- **Arithmetic:** `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`
- **Comparison:** `Eq`, `Ord`
- **Utilities:** `Clone`, `Copy`, `Display`, `Debug`
- **Logical:** `Not`

### Mixing Aliases and Granular Traits

You can combine trait aliases with individual traits:

```liva
// Comparable + Display (alias + granular)
formatAndCompare<T: Comparable + Display>(a: T, b: T): string {
    if a == b { return $"Equal: {a}" }
    if a > b { return $"{a} > {b}" }
    return $"{a} < {b}"
}

// Numeric + Printable (two aliases)
debugCalculation<T: Numeric + Printable>(a: T, b: T): T {
    console.log($"Calculating {a} + {b}")
    return a + b
}

// Number + Display (alias + granular)
formatNumber<T: Number + Display>(value: T): string {
    return $"Number: {value}"
}
```

### Best Practices

**✅ Do:**
- Use `Numeric` for general arithmetic
- Use `Comparable` for general comparisons
- Use `Number` when you need both arithmetic and comparison
- Use granular traits when you need precise control
- Mix aliases and granular traits for complex requirements

**❌ Don't:**
- Use granular traits when an alias suffices (less intuitive)
- Over-constrain (only require what you actually use)

### Where Clauses (Future)

```liva
// Future syntax - not in v0.9.1
fn complexFunction<T, U>(t: T, u: U)
    where T: Ord + Display,
          U: Add + Clone {
    // Complex constraints with where clause
}
```

---

## Type Inference

Liva infers type arguments from usage context:

### From Function Arguments

```liva
fn identity<T>(value: T): T {
    return value
}

let x = identity(42)        // T = int (inferred)
let y = identity("hello")   // T = string (inferred)
```

### From Variable Type

```liva
let box: Box<int> = Box(42)  // T = int (from declaration)
```

### From Return Type

```liva
fn makeBox<T>(value: T): Box<T> {
    return Box(value)
}

let b: Box<string> = makeBox("hello")  // T = string (inferred)
```

### Explicit Type Arguments

When inference fails or you want to be explicit:

```liva
let empty = Array<int>()     // Explicit type argument
let result = identity<string>("test")  // Explicit, though unnecessary
```

---

## See Also

- **[Generics: Advanced Patterns](generics-advanced.md)** - Standard library generics, implementation notes, examples, and design decisions
- **[Types: Primitives & Basics](types-primitives.md)** - Basic type system
- **[Types: Advanced](types-advanced.md)** - Advanced type features
- **[Classes](classes-basics.md)** - Class definitions
- **[Interfaces](classes-interfaces.md)** - Interface definitions
