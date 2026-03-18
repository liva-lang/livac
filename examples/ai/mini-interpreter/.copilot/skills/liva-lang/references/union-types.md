# Union Types in Liva

## Overview

Union types allow a value to be one of several types. This is useful for representing values that can have multiple forms, error handling, optional values, and more.

## Syntax

### Basic Union Types

```liva
// Simple union
type StringOrInt = string | int

// Union with multiple types
type Value = int | float | string | bool

// Inline union type annotation
let x: int | string = 42
let y: int | string = "hello"
```

### Generic Union Types

```liva
// Generic Result type
type Result<T> = T | Error

// Generic Option type  
type Option<T> = T | null

// Multiple type parameters
type Either<L, R> = L | R
```

### Nested Unions

```liva
// Unions are flattened: A | (B | C) becomes A | B | C
type ABC = int | (string | bool)  // Same as: int | string | bool
```

## Semantics

### Type Checking

A value of union type `A | B` can be:
- Any value of type `A`
- Any value of type `B`

```liva
let x: int | string = 42        // ✅ OK: 42 is an int
let y: int | string = "hello"   // ✅ OK: "hello" is a string
let z: int | string = true      // ❌ Error: bool is not in the union
```

### Assignment

You can assign any member type to a union:

```liva
type NumOrStr = int | string

let value: NumOrStr = 10      // ✅ Assign int
value = "text"                // ✅ Assign string
value = true                  // ❌ Error: bool not in union
```

### Function Parameters and Returns

```liva
// Parameter with union type
fn process(value: int | string) {
    // ... handle both cases
}

// Return union type
fn parseNumber(s: string): int | Error {
    // ... return int on success, Error on failure
}
```

## Pattern Matching with Unions

Union types work seamlessly with Liva's pattern matching:

```liva
type Result<T> = T | Error

fn divide(a: int, b: int): Result<int> {
    if b == 0 {
        return Error("Division by zero")
    }
    return a / b
}

main() {
    let result = divide(10, 2)
    
    switch result {
        case Error(msg) => print("Error: " + msg)
        case value => print("Result: " + value)
    }
}
```

### Type Narrowing

When you check the type in a conditional, Liva narrows the type:

```liva
fn handleValue(x: int | string) {
    if x is int {
        // Here, x is narrowed to int
        print(x + 10)
    } else {
        // Here, x is narrowed to string
        print(x.toUpperCase())
    }
}
```

### Pattern Matching with Switch (v0.11.3)

The most powerful way to work with unions is through pattern matching with `switch`:

```liva
fn processValue(x: int | string | bool) {
    switch x {
        case n: int => print("Number: {}", n * 2)
        case s: string => print("String: {}", s.length)
        case b: bool => print("Boolean: {}", b)
    }
}
```

#### Type Pattern Syntax

Pattern arms use the syntax `case variable: type => expression`:

```liva
let value: int | string = 42

let result = switch value {
    case num: int => num * 2        // Binds to 'num' as int
    case text: string => text.length // Binds to 'text' as string
}
```

#### Automatic Type Narrowing

Inside each match arm, the variable is automatically narrowed to the matched type:

```liva
fn describe(x: int | string | bool) -> string {
    return switch x {
        case n: int => "Number with value " + n.toString()
        case s: string => "String with length " + s.length.toString()
        case b: bool => "Boolean: " + (b ? "true" : "false")
    }
}
```

#### Exhaustiveness Checking

The compiler ensures all union variants are handled:

```liva
let x: int | string = 42

// ✅ OK - all variants covered
switch x {
    case n: int => print(n)
    case s: string => print(s)
}

// ❌ Error: Non-exhaustive pattern - missing string case
switch x {
    case n: int => print(n)
}
```

#### Wildcard Pattern

Use `_` to match any remaining variants:

```liva
let value: int | string | bool | float = 3.14

switch value {
    case n: int => print("Integer: {}", n)
    case s: string => print("String: {}", s)
    case _ => print("Other type")  // Matches bool and float
}
```

#### Nested Unions

Pattern matching works with nested union types:

```liva
type Value = int | string | (int, string)

fn process(v: Value) {
    switch v {
        case n: int => print("Number: {}", n)
        case s: string => print("String: {}", s)
        case (num, text): (int, string) => {
            print("Tuple: ({}, {})", num, text)
        }
    }
}
```

#### Multiple Type Unions

Handle unions with three or more types:

```liva
type Token = int | string | bool | float

fn tokenValue(t: Token) -> string {
    return switch t {
        case n: int => "int:" + n.toString()
        case s: string => "str:" + s
        case b: bool => "bool:" + (b ? "true" : "false")
        case f: float => "float:" + f.toString()
    }
}
```

#### Code Generation

Pattern matching on unions generates efficient Rust match expressions:

```liva
// Liva code
let x: int | string = 42
switch x {
    case n: int => print(n * 2)
    case s: string => print(s.length)
}
```

```rust
// Generated Rust code
let x: Union_i32_String = Union_i32_String::Int(42);
match x {
    Union_i32_String::Int(n) => println!("{}", n * 2),
    Union_i32_String::Str(s) => println!("{}", s.len()),
}
```

## Common Patterns

### Optional Values

```liva
type Option<T> = T | null

fn findUser(id: int): Option<User> {
    // ... search for user
    if found {
        return user
    }
    return null
}

let user = findUser(42)
if user != null {
    print(user.name)
}
```

### Result/Error Handling

```liva
type Result<T> = T | Error

fn readFile(path: string): Result<string> {
    // ... read file
    if error {
        return Error("File not found")
    }
    return content
}

let content = readFile("data.txt")
switch content {
    case Error(msg) => print("Failed: " + msg)
    case text => print("Content: " + text)
}
```

### Discriminated Unions with Tuples

```liva
// Use tuples with tags for discriminated unions
type Shape = ("circle", float)      // (tag, radius)
           | ("rectangle", float, float)  // (tag, width, height)
           | ("square", float)      // (tag, side)

fn area(shape: Shape): float {
    switch shape {
        case ("circle", r) => 3.14 * r * r
        case ("rectangle", w, h) => w * h
        case ("square", s) => s * s
    }
}
```

## Implementation Details

### Rust Codegen

Union types are compiled to Rust enums:

```liva
// Liva code
type Result<T> = T | Error
```

```rust
// Generated Rust code
enum Result<T> {
    Value(T),
    Error(liva_rt::Error),
}
```

For simple unions without generic parameters:

```liva
// Liva code
type IntOrString = int | string
```

```rust
// Generated Rust code
enum IntOrString {
    Int(i32),
    String(String),
}
```

### Null Unions

`T | null` is a special case that maps to Rust's `Option<T>`:

```liva
// Liva code
let x: int | null = null
```

```rust
// Generated Rust code
let x: Option<i32> = None;
```

### Error Unions

`T | Error` maps to Rust's `Result<T, liva_rt::Error>`:

```liva
// Liva code
fn divide(a: int, b: int): int | Error { ... }
```

```rust
// Generated Rust code
fn divide(a: i32, b: i32) -> Result<i32, liva_rt::Error> { ... }
```

## Type Operations

### Union of Unions

Unions are automatically flattened:

```liva
type A = int | string
type B = bool | float
type C = A | B  // Same as: int | string | bool | float
```

### Duplicate Types

Duplicate types in a union are removed:

```liva
type X = int | int | string  // Same as: int | string
```

### Order Independence

Union type order doesn't matter for type checking:

```liva
type A = int | string
type B = string | int
// A and B are equivalent
```

## Restrictions

### Cannot Mix Value and Type

```liva
type Bad = int | 42  // ❌ Error: 42 is a value, not a type
```

### Circular Unions

Circular union definitions are not allowed:

```liva
type A = B | int
type B = A | string  // ❌ Error: Circular union reference
```

### Union Member Access

You cannot directly access members on a union without narrowing:

```liva
let x: User | null = getUser()
print(x.name)  // ❌ Error: Cannot access .name on union type

if x != null {
    print(x.name)  // ✅ OK: x is narrowed to User
}
```

## Best Practices

### 1. Use Semantic Names

```liva
// ✅ Good: Descriptive name
type Result<T> = T | Error

// ❌ Bad: Generic name
type OrError<T> = T | Error
```

### 2. Prefer Result for Errors

```liva
// ✅ Good: Clear error handling
fn parse(s: string): int | Error { ... }

// ❌ Avoid: Unclear what -1 means
fn parse(s: string): int { 
    // returns -1 on error
}
```

### 3. Use Pattern Matching

```liva
// ✅ Good: Exhaustive pattern matching
switch value {
    case Error(msg) => handleError(msg)
    case n => useValue(n)
}

// ❌ Avoid: Manual type checking
if value is Error {
    handleError(value.message)
} else {
    useValue(value)
}
```

### 4. Keep Unions Simple

```liva
// ✅ Good: 2-3 types
type Value = int | string | null

// ⚠️ Avoid: Too many types (consider enum instead)
type Value = int | string | float | bool | null | Error
```

## Comparison with Other Languages

### TypeScript

```typescript
// TypeScript
type StringOrNumber = string | number;
let x: StringOrNumber = "hello";
x = 42;
```

```liva
// Liva (same concept)
type StringOrNumber = string | int
let x: StringOrNumber = "hello"
x = 42
```

### Rust

```rust
// Rust (enum)
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

```liva
// Liva (union type)
type Result<T> = T | Error
```

### Haskell

```haskell
-- Haskell (sum type)
data Either a b = Left a | Right b
```

```liva
// Liva (generic union)
type Either<A, B> = A | B
```

## Advanced Examples

### State Machine

```liva
type State = ("idle", null)
           | ("loading", string)  // loading with URL
           | ("success", Data)
           | ("error", string)    // error with message

fn handleState(state: State) {
    switch state {
        case ("idle", _) => print("Waiting...")
        case ("loading", url) => print("Loading: " + url)
        case ("success", data) => processData(data)
        case ("error", msg) => print("Error: " + msg)
    }
}
```

### Parser Result

```liva
type ParseResult<T> = ("ok", T, int)        // (value, remaining_input_pos)
                    | ("error", string, int)  // (message, error_pos)

fn parseNumber(s: string): ParseResult<int> {
    // ... parsing logic
}

let result = parseNumber("42abc")
switch result {
    case ("ok", value, pos) => {
        print("Parsed: " + value)
        print("Stopped at: " + pos)
    }
    case ("error", msg, pos) => {
        print("Parse error at " + pos + ": " + msg)
    }
}
```

### Event System

```liva
type Event = ("click", int, int)      // x, y coordinates
           | ("keypress", string)      // key
           | ("resize", int, int)      // width, height
           | ("custom", string, any)   // name, data

fn handleEvent(event: Event) {
    switch event {
        case ("click", x, y) => handleClick(x, y)
        case ("keypress", key) => handleKey(key)
        case ("resize", w, h) => handleResize(w, h)
        case ("custom", name, data) => handleCustom(name, data)
    }
}
```

## Error Codes

- **E0801**: Union type member not found
- **E0802**: Cannot access member on union type without narrowing
- **E0803**: Circular union definition detected
- **E0804**: Type mismatch in union assignment

## Summary

Union types in Liva provide:
- ✅ Type-safe way to represent "one of several types"
- ✅ Seamless integration with pattern matching
- ✅ Type narrowing in conditionals
- ✅ Clean error handling with `T | Error`
- ✅ Optional values with `T | null`
- ✅ Efficient Rust codegen (enums)
- ✅ Zero runtime overhead

Union types complement tuples and type aliases to create a powerful, modern type system that's both safe and ergonomic.
