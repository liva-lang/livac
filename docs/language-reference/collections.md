# Collections

Complete reference for arrays, maps, objects, and data structures in Liva.

## Table of Contents
- [Arrays](#arrays)
- [Maps (Dictionaries)](#maps-dictionaries)
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

## Maps (Dictionaries)

*Added in v1.3.0*

Maps are key-value collections backed by `HashMap` with O(1) lookup, insertion, and deletion.

### Map Literals

```liva
// Empty map (requires type annotation)
let empty: Map<string, int> = Map {}

// Map with entries
let ages = Map {
  "Alice": 30,
  "Bob": 25,
  "Carlos": 35
}
```

### Type Annotations

```liva
let scores: Map<string, number> = Map {}
let config: Map<string, string> = Map { "host": "localhost" }

// As function return type
getDefaults(): Map<string, int> {
  return Map { "timeout": 30, "retries": 3 }
}

// As function parameter
processConfig(config: Map<string, string>) {
  // ...
}
```

### Getting Values

`map.get(key)` returns an optional value — use `or` to provide a default:

```liva
let ages = Map { "Alice": 30, "Bob": 25 }

let age = ages.get("Alice") or 0       // 30
let missing = ages.get("Unknown") or -1 // -1 (key not found)
```

> **Note:** Always use `or default` with `map.get()` since the key may not exist.

### Setting Values

```liva
let users = Map { "alice": "Alice Smith" }

// Insert new entry
users.set("bob", "Bob Jones")

// Update existing entry
users.set("alice", "Alice Johnson")
```

### Checking Key Existence

```liva
let ages = Map { "Alice": 30, "Bob": 25 }

let hasAlice = ages.has("Alice")    // true
let hasEve = ages.has("Eve")        // false

if ages.has("Alice") {
  print("Alice is in the map")
}
```

### Deleting Entries

```liva
let ages = Map { "Alice": 30, "Bob": 25, "Carlos": 35 }

ages.delete("Bob")
// ages now has: Alice: 30, Carlos: 35
```

### Size

```liva
let ages = Map { "Alice": 30, "Bob": 25 }
let count = ages.length  // 2
```

### Clearing All Entries

```liva
let cache = Map { "key1": "val1", "key2": "val2" }
cache.clear()
// cache is now empty, cache.length == 0
```

### Extracting Keys, Values, and Entries

```liva
let config = Map { "host": "localhost", "port": "8080" }

let allKeys = config.keys()       // [string] — ["host", "port"]
let allValues = config.values()   // [string] — ["localhost", "8080"]
let allPairs = config.entries()   // [(string, string)] — tuples
```

### Iterating Maps

#### for key, value in map

Destructured iteration gives you both the key and value:

```liva
let scores = Map { "math": 95, "english": 88, "science": 92 }

for subject, score in scores {
  print($"{subject}: {score}")
}
// Output (order may vary):
// math: 95
// english: 88
// science: 92
```

#### forEach with Lambda

```liva
let ages = Map { "Alice": 30, "Bob": 25 }

ages.forEach((name, age) => {
  print($"{name} is {age} years old")
})
```

#### Iterating Keys Only

```liva
let config = Map { "host": "localhost", "port": "8080" }

for key in config.keys() {
  print(key)
}
```

### Map Methods Summary

| Method | Signature | Description | Returns |
|--------|-----------|-------------|---------|
| `get` | `map.get(key) or default` | Get value by key with fallback | `V` |
| `set` | `map.set(key, value)` | Insert or update entry | `void` |
| `has` | `map.has(key)` | Check if key exists | `bool` |
| `delete` | `map.delete(key)` | Remove entry by key | `void` |
| `keys` | `map.keys()` | Get all keys as array | `[K]` |
| `values` | `map.values()` | Get all values as array | `[V]` |
| `entries` | `map.entries()` | Get all pairs as tuples | `[(K, V)]` |
| `clear` | `map.clear()` | Remove all entries | `void` |
| `forEach` | `map.forEach((k, v) => { })` | Iterate with callback | `void` |
| `length` | `map.length` | Number of entries | `int` |

### Compiled Output

Liva maps compile to Rust's `std::collections::HashMap<K, V>`:

| Liva | Rust |
|------|------|
| `Map { "a": 1 }` | `HashMap::from([("a".to_string(), 1)])` |
| `map.get("k") or 0` | `map.get(&"k".to_string()).cloned().unwrap_or(0)` |
| `map.set("k", v)` | `map.insert("k".to_string(), v)` |
| `map.has("k")` | `map.contains_key(&"k".to_string())` |
| `for k, v in map` | `for (k, v) in map.iter()` |

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

// One-liner with => (v1.1.0)
for num in numbers => print(num)

// Point-free (v1.1.0)
for num in numbers => print
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

### Maps

```liva
// Create
let ages = Map { "Alice": 30, "Bob": 25 }

// CRUD
ages.set("Carlos", 35)
let age = ages.get("Alice") or 0
ages.delete("Bob")

// Iterate
for key, value in ages { }
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

// Map
let ages = Map { "Alice": 30, "Bob": 25 }
ages.set("Carlos", 35)
let age = ages.get("Alice") or 0
ages.delete("Bob")

for name, age in ages {
  print($"{name}: {age}")
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
