# Generics in Liva

**Version:** 0.9.0  
**Status:** Specification  
**Last Updated:** 2025-10-23

---

## Overview

Generics enable **parametric polymorphism** in Liva, allowing you to write code that works with multiple types while maintaining type safety. This document specifies the syntax, semantics, and implementation details of Liva's generic system.

---

## Table of Contents

1. [Motivation](#motivation)
2. [Syntax](#syntax)
3. [Generic Functions](#generic-functions)
4. [Generic Classes](#generic-classes)
5. [Generic Interfaces](#generic-interfaces)
6. [Type Constraints](#type-constraints)
7. [Type Inference](#type-inference)
8. [Standard Library](#standard-library)
9. [Implementation Notes](#implementation-notes)
10. [Examples](#examples)

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

### Single Constraint

Use `:` to specify a constraint (interface the type must implement):

```liva
fn max<T: Comparable<T>>(a: T, b: T): T {
    if a.compareTo(b) > 0 {
        return a
    }
    return b
}
```

### Multiple Constraints (Future)

```liva
// Future syntax - not in v0.9.0
fn process<T: Comparable<T> & Serializable>(value: T) {
    // T must implement both Comparable and Serializable
}
```

### Where Clauses (Future)

```liva
// Future syntax - not in v0.9.0
fn complexFunction<T, U>(t: T, u: U)
    where T: Comparable<T>,
          U: Mapper<T, string> {
    // Complex constraints
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

## Standard Library

### Array<T>

```liva
class Array<T> {
    fn push(item: T)
    fn pop(): T
    fn get(index: int): T
    fn length(): int
    
    fn map<U>(fn: (T) => U): Array<U>
    fn filter(fn: (T) => bool): Array<T>
    fn reduce<U>(fn: (U, T) => U, initial: U): U
}

let numbers: Array<int> = [1, 2, 3]
let doubled = numbers.map(x => x * 2)
```

### Option<T>

```liva
interface Option<T> {
    fn isSome(): bool
    fn isNone(): bool
    fn unwrap(): T
    fn unwrapOr(default: T): T
}

class Some<T> : Option<T> {
    value: T
    
    fn isSome(): bool { return true }
    fn isNone(): bool { return false }
    fn unwrap(): T { return this.value }
    fn unwrapOr(default: T): T { return this.value }
}

class None<T> : Option<T> {
    fn isSome(): bool { return false }
    fn isNone(): bool { return true }
    fn unwrap(): T { fail "Called unwrap on None" }
    fn unwrapOr(default: T): T { return default }
}
```

### Result<T, E>

```liva
interface Result<T, E> {
    fn isOk(): bool
    fn isErr(): bool
    fn unwrap(): T
    fn unwrapOr(default: T): T
}

class Ok<T, E> : Result<T, E> {
    value: T
    
    fn isOk(): bool { return true }
    fn isErr(): bool { return false }
    fn unwrap(): T { return this.value }
    fn unwrapOr(default: T): T { return this.value }
}

class Err<T, E> : Result<T, E> {
    error: E
    
    fn isOk(): bool { return false }
    fn isErr(): bool { return true }
    fn unwrap(): T { fail "Called unwrap on Err" }
    fn unwrapOr(default: T): T { return default }
}
```

### Map<K, V>

```liva
class Map<K, V> {
    fn set(key: K, value: V)
    fn get(key: K): Option<V>
    fn has(key: K): bool
    fn remove(key: K): bool
    fn keys(): Array<K>
    fn values(): Array<V>
    fn entries(): Array<[K, V]>
}

let ages: Map<string, int> = Map()
ages.set("Alice", 30)
ages.set("Bob", 25)
```

### Set<T>

```liva
class Set<T> {
    fn add(value: T)
    fn has(value: T): bool
    fn remove(value: T): bool
    fn size(): int
    fn values(): Array<T>
}

let numbers: Set<int> = Set()
numbers.add(1)
numbers.add(2)
numbers.add(1)  // Duplicate ignored
```

---

## Implementation Notes

### Monomorphization

Liva uses **monomorphization** (similar to Rust):

1. Each unique type argument combination generates a specialized version
2. `identity<int>` and `identity<string>` compile to separate Rust functions
3. No runtime overhead - all generic code is specialized at compile time

### Rust Mapping

```liva
// Liva
fn identity<T>(value: T): T {
    return value
}

// Generated Rust
fn identity<T>(value: T) -> T {
    value
}
```

```liva
// Liva
class Box<T> {
    value: T
}

// Generated Rust
struct Box<T> {
    value: T,
}
```

### Constraints to Traits

```liva
// Liva
fn max<T: Comparable<T>>(a: T, b: T): T {
    // ...
}

// Generated Rust
fn max<T: Comparable>(a: T, b: T) -> T
where
    T: Comparable,
{
    // ...
}
```

---

## Examples

### Example 1: Generic Stack

```liva
class Stack<T> {
    items: T[]
    
    constructor() {
        this.items = []
    }
    
    fn push(item: T) {
        this.items.push(item)
    }
    
    fn pop(): Option<T> {
        if this.items.length() == 0 {
            return None<T>()
        }
        return Some(this.items.pop())
    }
    
    fn peek(): Option<T> {
        if this.items.length() == 0 {
            return None<T>()
        }
        return Some(this.items[this.items.length() - 1])
    }
    
    fn isEmpty(): bool {
        return this.items.length() == 0
    }
}

// Usage
let intStack = Stack<int>()
intStack.push(1)
intStack.push(2)
intStack.push(3)

let top = intStack.pop()  // Some(3)
let next = intStack.peek()  // Some(2)
```

### Example 2: Generic Tree

```liva
class TreeNode<T> {
    value: T
    left: Option<TreeNode<T>>
    right: Option<TreeNode<T>>
    
    constructor(value: T) {
        this.value = value
        this.left = None<TreeNode<T>>()
        this.right = None<TreeNode<T>>()
    }
    
    fn insert(value: T) {
        // Insertion logic
    }
    
    fn search(value: T): bool {
        // Search logic
        return false
    }
}
```

### Example 3: Generic Repository Pattern

```liva
interface Repository<T> {
    fn save(entity: T): Result<T, string>
    fn findById(id: int): Option<T>
    fn findAll(): Array<T>
    fn delete(id: int): bool
}

class UserRepository : Repository<User> {
    users: Map<int, User>
    
    constructor() {
        this.users = Map()
    }
    
    fn save(user: User): Result<User, string> {
        this.users.set(user.id, user)
        return Ok(user)
    }
    
    fn findById(id: int): Option<User> {
        return this.users.get(id)
    }
    
    fn findAll(): Array<User> {
        return this.users.values()
    }
    
    fn delete(id: int): bool {
        return this.users.remove(id)
    }
}
```

### Example 4: Type-safe Builder Pattern

```liva
class QueryBuilder<T> {
    filters: Array<string>
    
    constructor() {
        this.filters = []
    }
    
    fn where(condition: string): QueryBuilder<T> {
        this.filters.push(condition)
        return this
    }
    
    fn execute(): Array<T> {
        // Execute query and return results
        return []
    }
}

let users = QueryBuilder<User>()
    .where("age > 18")
    .where("country = 'US'")
    .execute()
```

---

## Design Decisions

### 1. Angle Bracket Syntax

**Choice:** `<T>` (like TypeScript, Rust, C++)

**Rationale:**
- Industry standard
- Familiar to most developers
- Clear visual separation from function parameters

**Alternatives considered:**
- Square brackets `[T]` - conflicts with array syntax
- Parentheses `(T)` - conflicts with function calls

### 2. Type Parameter Naming

**Convention:** Single uppercase letters for simple cases

- `T` - generic Type
- `K` - Key (in maps/dictionaries)
- `V` - Value (in maps/dictionaries)
- `E` - Error type
- `U` - second generic type (when T is taken)

**Full names for domain-specific generics:**
- `TEntity`, `TResult`, `TState`

### 3. Constraints Syntax

**Choice:** `<T: Constraint>` (like Rust)

**Rationale:**
- Clear and readable
- Consistent with interface implementation syntax
- Allows for future `where` clauses

### 4. Monomorphization vs Type Erasure

**Choice:** Monomorphization (generate specialized code for each type)

**Rationale:**
- Zero runtime overhead
- Better performance
- Simpler implementation (maps directly to Rust generics)
- Type safety preserved

**Trade-offs:**
- Larger binary size (acceptable for v0.9.0 scope)
- Longer compilation times (acceptable for small projects)

---

## Future Enhancements

### v0.10.0+

1. **Multiple constraints:** `<T: Comparable & Serializable>`
2. **Where clauses:** For complex constraints
3. **Associated types:** `interface Iterator { type Item; }`
4. **Default type parameters:** `class Array<T = int>`
5. **Const generics:** `class Matrix<T, const N: int>`
6. **Higher-kinded types:** Types that take types as parameters

---

## Migration Guide

### From v0.8.1 to v0.9.0

**Arrays:**

```liva
// Before (v0.8.1)
let numbers: int[] = [1, 2, 3]

// After (v0.9.0) - both work
let numbers: int[] = [1, 2, 3]
let numbers: Array<int> = [1, 2, 3]
```

**Option/Result types:**

```liva
// Before (v0.8.1) - using tuples
let value, err = parseInt("42")

// After (v0.9.0) - using Result<T, E>
let result: Result<int, string> = parseInt("42")
if result.isOk() {
    print(result.unwrap())
}
```

---

## Error Codes

- **E5001:** Generic type parameter not found
- **E5002:** Wrong number of type arguments
- **E5003:** Type constraint not satisfied
- **E5004:** Cannot infer type parameter
- **E5005:** Conflicting type parameter bounds
- **E5006:** Recursive type parameter constraint
- **E5007:** Generic type used without type arguments

---

## Best Practices

1. **Use descriptive names for complex generics:**
   ```liva
   // Good
   class Repository<TEntity>
   
   // Less clear
   class Repository<T>
   ```

2. **Prefer type inference when possible:**
   ```liva
   // Good
   let box = Box(42)
   
   // Unnecessary
   let box = Box<int>(42)
   ```

3. **Add constraints only when needed:**
   ```liva
   // Good - minimal constraint
   fn sort<T: Comparable<T>>(arr: T[])
   
   // Bad - over-constrained
   fn sort<T: Comparable<T> & Serializable & Debug>(arr: T[])
   ```

4. **Document generic type parameters:**
   ```liva
   /// Generic stack data structure
   /// @param T The type of elements in the stack
   class Stack<T> {
       // ...
   }
   ```

---

**Next Steps:**
1. Implement AST extensions for generic types
2. Update parser to handle generic syntax
3. Implement type substitution in semantic analyzer
4. Generate Rust generic code in codegen
5. Update standard library to use generics

---

**References:**
- Rust Generics: https://doc.rust-lang.org/book/ch10-00-generics.html
- TypeScript Generics: https://www.typescriptlang.org/docs/handbook/2/generics.html
- C++ Templates: https://en.cppreference.com/w/cpp/language/templates
