# Generics: Advanced Patterns

**Version:** 0.9.0  
**Status:** Specification  
**Last Updated:** 2025-10-23

---

## Table of Contents

1. [Standard Library](#standard-library)
2. [Implementation Notes](#implementation-notes)
3. [Examples](#examples)
4. [Design Decisions](#design-decisions)
5. [Future Enhancements](#future-enhancements)
6. [Migration Guide](#migration-guide)
7. [Error Codes](#error-codes)
8. [Best Practices](#best-practices)

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

## See Also

- **[Generics: Basics](generics-basics.md)** - Generic syntax, functions, classes, interfaces, constraints, and type inference
- **[Types: Primitives & Basics](types-primitives.md)** - Basic type system
- **[Types: Advanced](types-advanced.md)** - Advanced type features
- **[Classes](classes-basics.md)** - Class definitions
- **[Interfaces](classes-interfaces.md)** - Interface definitions

---

**References:**
- Rust Generics: https://doc.rust-lang.org/book/ch10-00-generics.html
- TypeScript Generics: https://www.typescriptlang.org/docs/handbook/2/generics.html
- C++ Templates: https://en.cppreference.com/w/cpp/language/templates
