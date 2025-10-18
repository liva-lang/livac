# Collections

Complete reference for arrays, objects, and data structures in Liva.

## Table of Contents
- [Arrays](#arrays)
- [Object Literals](#object-literals)
- [Struct Literals](#struct-literals)
- [Iteration](#iteration)
- [Best Practices](#best-practices)

---

## Arrays

### Array Literals

```liva
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]
let mixed = [1, "two", 3.0, true]  // Mixed types
```

### Type Annotations

```liva
let numbers: [number] = [1, 2, 3]
let names: [string] = ["Alice", "Bob"]
```

### Accessing Elements

```liva
let numbers = [10, 20, 30, 40]

let first = numbers[0]   // 10
let second = numbers[1]  // 20
let last = numbers[3]    // 40
```

### Array Properties

```liva
let items = [1, 2, 3, 4, 5]

let count = items.length  // 5
```

### Nested Arrays

```liva
let matrix = [
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9]
]

let element = matrix[1][2]  // 6
```

### Array of Objects

```liva
let users = [
  { name: "Alice", age: 25 },
  { name: "Bob", age: 30 },
  { name: "Charlie", age: 35 }
]

let firstUser = users[0]
let firstName = users[0].name  // "Alice"
```

---

## Object Literals

### Basic Object

```liva
let user = {
  name: "Alice",
  age: 25,
  email: "alice@example.com"
}
```

### Accessing Properties

```liva
let name = user.name      // "Alice"
let age = user.age        // 25
let email = user.email    // "alice@example.com"
```

### Nested Objects

```liva
let person = {
  name: "Alice",
  address: {
    street: "123 Main St",
    city: "Boston",
    zip: "02101"
  }
}

let city = person.address.city  // "Boston"
```

### Object with Arrays

```liva
let team = {
  name: "Engineering",
  members: ["Alice", "Bob", "Charlie"],
  size: 3
}

let firstMember = team.members[0]  // "Alice"
```

### Dynamic Properties

```liva
let key = "name"
let value = user[key]  // Access property dynamically
```

---

## Struct Literals

### Using Class Types

```liva
Person {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
}

// Struct literal syntax
let alice = Person {
  name: "Alice",
  age: 25
}

// Constructor syntax
let bob = Person("Bob", 30)
```

### Struct with Nested Objects

```liva
let config = Config {
  database: {
    host: "localhost",
    port: 5432
  },
  cache: {
    enabled: true,
    ttl: 300
  }
}
```

---

## Iteration

### Iterating Arrays

```liva
let numbers = [1, 2, 3, 4, 5]

for num in numbers {
  print(num)
}
```

### Iterating with Index

```liva
let names = ["Alice", "Bob", "Charlie"]

for i in 0..names.length {
  print($"{i}: {names[i]}")
}
// Output:
// 0: Alice
// 1: Bob
// 2: Charlie
```

### Iterating Objects in Arrays

```liva
let users = [
  { name: "Alice", age: 25 },
  { name: "Bob", age: 30 }
]

for user in users {
  print($"{user.name} is {user.age} years old")
}
```

### Parallel Iteration

```liva
let numbers = [1, 2, 3, 4, 5, 6, 7, 8]

for par num in numbers with threads 4 {
  let result = heavyComputation(num)
  print(result)
}
```

### Vectorized Iteration

```liva
let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]

for vec value in values with simdWidth 4 {
  let result = mathOperation(value)
  print(result)
}
```

---

## Best Practices

### Array Naming

```liva
// ✅ Good: Plural nouns
let users = [...]
let products = [...]
let errors = [...]

// ❌ Bad: Singular or unclear
let user = [...]
let data = [...]
```

### Type Annotations for Complex Arrays

```liva
// ✅ Good: Explicit type for clarity
let users: [{ name: string, age: number }] = [
  { name: "Alice", age: 25 },
  { name: "Bob", age: 30 }
]

// ⚠️ Acceptable: Inference for simple types
let numbers = [1, 2, 3]
```

### Avoid Mixed-Type Arrays

```liva
// ❌ Bad: Mixed types
let mixed = [1, "two", 3.0, true]

// ✅ Good: Consistent types
let numbers = [1, 2, 3]
let strings = ["one", "two", "three"]
```

### Use Descriptive Object Keys

```liva
// ✅ Good: Clear property names
let user = {
  firstName: "Alice",
  lastName: "Smith",
  emailAddress: "alice@example.com"
}

// ❌ Bad: Cryptic abbreviations
let user = {
  fn: "Alice",
  ln: "Smith",
  em: "alice@example.com"
}
```

### Extract Complex Nested Structures

```liva
// ⚠️ Acceptable but hard to read
let config = {
  db: { host: "localhost", port: 5432, creds: { user: "admin", pass: "secret" } }
}

// ✅ Better: Extract to separate objects
let dbCreds = { user: "admin", pass: "secret" }
let dbConfig = { host: "localhost", port: 5432, creds: dbCreds }
let config = { db: dbConfig }
```

### Use Parallel Iteration for CPU-Bound Work

```liva
// ✅ Good: Parallel for CPU-intensive tasks
for par item in largeDataset with threads 8 {
  complexComputation(item)
}

// ✅ Good: Sequential for I/O or order-dependent work
for item in files {
  writeToFile(item)
}
```

---

## Summary

### Arrays

```liva
// Literal
let arr = [1, 2, 3]

// Access
let first = arr[0]

// Iterate
for item in arr { }
```

### Objects

```liva
// Literal
let obj = { name: "Alice", age: 25 }

// Access
let name = obj.name

// Dynamic access
let key = "age"
let value = obj[key]
```

### Struct Literals

```liva
// Using class
let person = Person {
  name: "Alice",
  age: 25
}
```

### Iteration

```liva
// Sequential
for item in items { }

// Parallel
for par item in items with threads 4 { }

// Vectorized
for vec value in values with simdWidth 4 { }
```

### Quick Reference

```liva
// Array
let numbers = [1, 2, 3, 4, 5]
let first = numbers[0]
let length = numbers.length

for num in numbers {
  print(num)
}

// Object
let user = {
  name: "Alice",
  age: 25,
  email: "alice@example.com"
}

let name = user.name
let age = user["age"]

// Array of objects
let users = [
  { name: "Alice", age: 25 },
  { name: "Bob", age: 30 }
]

for user in users {
  print($"{user.name}: {user.age}")
}

// Parallel iteration
for par item in items with threads 4 {
  processItem(item)
}
```

---

**Next**: [Visibility →](visibility.md)

**See Also**:
- [Variables](variables.md)
- [Control Flow](control-flow.md) - For loops
- [Concurrency](concurrency.md) - Data-parallel iteration
