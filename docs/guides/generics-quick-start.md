# Generics in Liva - Quick Start Guide

## Introduction

Generics allow you to write type-safe, reusable code that works with multiple types. Instead of writing separate `IntBox`, `StringBox`, and `BoolBox` classes, you can write one `Box<T>` that works with any type.

## Basic Generic Functions

The simplest generic is a function with a type parameter:

```liva
// Generic identity function
identity<T>(value: T): T => value

main() {
    let num = identity(42)        // Works with int
    let text = identity("hello")  // Works with string
    let flag = identity(true)     // Works with bool
    
    print(num)   // 42
    print(text)  // hello
    print(flag)  // true
}
```

**Key points:**
- `<T>` declares a type parameter
- `T` can be used in parameter types and return type
- Liva generates specialized code for each type used

## Generic Classes

Create reusable data structures:

```liva
// A box that holds any type of value
Box<T> {
    value: T
    
    constructor(value: T) {
        this.value = value
    }
}

main() {
    let intBox = Box(42)
    let strBox = Box("Liva")
    let boolBox = Box(true)
    
    print(intBox.value)   // 42
    print(strBox.value)   // Liva
    print(boolBox.value)  // true
}
```

## Multiple Type Parameters

Use multiple type parameters for more complex structures:

```liva
// A pair holding two values of different types
Pair<T, U> {
    first: T
    second: U
    
    constructor(first: T, second: U) {
        this.first = first
        this.second = second
    }
}

main() {
    let coords = Pair(10, 20)           // Pair<int, int>
    let nameAge = Pair("Alice", 25)     // Pair<string, int>
    let mixed = Pair(true, 3.14)        // Pair<bool, float>
    
    print(coords.first)    // 10
    print(nameAge.second)  // 25
    print(mixed.first)     // true
}
```

## Array Type Annotations

Specify the type of array elements:

```liva
// Function that takes an array of integers
sum(numbers: [int]): int {
    let total = 0
    for num in numbers {
        total = total + num
    }
    return total
}

// Function that takes an array of strings
join(words: [string], separator: string): string {
    if words.length == 0 { return "" }
    
    let result = words[0]
    let i = 1
    while i < words.length {
        result = result + separator + words[i]
        i = i + 1
    }
    return result
}

main() {
    let nums = [1, 2, 3, 4, 5]
    print(sum(nums))  // 15
    
    let words = ["Hello", "from", "Liva"]
    print(join(words, " "))  // Hello from Liva
}
```

## Optional Values with Option<T>

Handle optional values safely:

```liva
Option<T> {
    value: T
    hasValue: bool
    
    constructor(value: T, hasValue: bool) {
        this.value = value
        this.hasValue = hasValue
    }
    
    isSome(): bool {
        return this.hasValue
    }
    
    isNone(): bool {
        return !this.hasValue
    }
}

// Helper functions
some<T>(value: T): Option<T> {
    return Option(value, true)
}

none<T>(defaultValue: T): Option<T> {
    return Option(defaultValue, false)
}

// Example: Find user by ID
findUser(id: int): Option<string> {
    if id == 1 {
        return some("Alice")
    }
    return none("")
}

main() {
    let user = findUser(1)
    if user.isSome() {
        print($"Found: {user.value}")
    } else {
        print("User not found")
    }
}
```

## Error Handling with Result<T, E>

Handle operations that can fail:

```liva
Result<T, E> {
    value: T
    error: E
    isOk: bool
    
    constructor(value: T, error: E, isOk: bool) {
        this.value = value
        this.error = error
        this.isOk = isOk
    }
    
    isSuccess(): bool {
        return this.isOk
    }
    
    isError(): bool {
        return !this.isOk
    }
}

// Helper functions
ok<T, E>(value: T, defaultError: E): Result<T, E> {
    return Result(value, defaultError, true)
}

err<T, E>(defaultValue: T, error: E): Result<T, E> {
    return Result(defaultValue, error, false)
}

// Example: Safe division
divide(a: int, b: int): Result<int, string> {
    if b == 0 {
        return err(0, "Division by zero")
    }
    return ok(a / b, "")
}

main() {
    let result = divide(10, 2)
    if result.isSuccess() {
        print($"Result: {result.value}")
    } else {
        print($"Error: {result.error}")
    }
    
    let failed = divide(10, 0)
    if failed.isError() {
        print($"Error: {failed.error}")
    }
}
```

## Best Practices

### ✅ Do:

1. **Use descriptive type parameter names:**
   ```liva
   Map<Key, Value> { ... }  // Clear
   ```

2. **Access fields directly when needed:**
   ```liva
   let value = myBox.value  // Works
   ```

3. **Keep generic classes simple:**
   ```liva
   Container<T> {
       item: T
       constructor(item: T) { this.item = item }
   }
   ```

### ❌ Don't:

1. **Don't try to return T by value from methods:**
   ```liva
   // This won't work (ownership issue)
   getValue(): T {
       return this.value  // Error!
   }
   
   // Use field access instead
   let value = container.value  // Works
   ```

2. **Don't use confusing single letters:**
   ```liva
   // Unclear
   DoSomething<A, B, C> { ... }
   
   // Better
   Cache<Key, Value, Timestamp> { ... }
   ```

## Common Patterns

### Stack<T>

```liva
Stack<T> {
    items: [T]
    
    constructor() {
        this.items = []
    }
    
    push(item: T) {
        this.items.push(item)
    }
    
    isEmpty(): bool {
        return this.items.length == 0
    }
}
```

### Wrapper<T>

```liva
Wrapper<T> {
    inner: T
    
    constructor(inner: T) {
        this.inner = inner
    }
}
```

## Known Limitations (v0.9.0)

1. **No type inference yet** - Must specify type parameters explicitly
2. **No constraint checking** - `T: Clone` syntax is planned for v0.9.1
3. **Methods can't return T by value** - Use field access instead
4. **VSCode LSP shows false errors** - Compiler works fine

## What's Next

Features coming in v0.9.1:
- Type inference for generic calls
- Constraint checking (`T: Clone`, `T: Display`)
- Improved method signatures
- More stdlib generic types

## Examples

See working examples in the `examples/` directory:
- `test_array_generic.liva` - Generic functions
- `test_generic_class.liva` - Single type parameter
- `test_generic_methods.liva` - Multiple type parameters
- `test_option_generic.liva` - Option<T> pattern
- `test_result_generic.liva` - Result<T,E> pattern
- `test_type_param_validation.liva` - Type validation

## Summary

Generics in Liva v0.9.0 provide:
- ✅ Type-safe generic functions
- ✅ Generic classes with single or multiple parameters
- ✅ Array type annotations
- ✅ Option<T> and Result<T,E> patterns
- ✅ Compile-time specialization (monomorphization)

Start using generics today to write more reusable, type-safe code!
