# Collections — Additional Reference

> SKILL.md covers array/map/set creation, basic CRUD, and iteration.
> This file covers **type annotations, complete method tables, and advanced patterns only**.

## Map Type Annotations

```liva
// Empty map requires type annotation
let empty: Map<string, int> = Map {}

// As function parameter
processConfig(config: Map<string, string>) {
    for key, value in config {
        print($"{key}: {value}")
    }
}

// As return type
getDefaults(): Map<string, int> {
    return Map { "timeout": 30, "retries": 3 }
}
```

## Map Method Table

| Method | Signature | Returns |
|--------|-----------|---------|
| `map.get(key) or default` | Get value with fallback | `V` |
| `map.set(key, value)` | Insert or update | `void` |
| `map.has(key)` | Check key existence | `bool` |
| `map.delete(key)` | Remove by key | `void` |
| `map.keys()` | All keys as array | `[K]` |
| `map.values()` | All values as array | `[V]` |
| `map.entries()` | All pairs as tuples | `[(K, V)]` |
| `map.clear()` | Remove all entries | `void` |
| `map.forEach((k, v) => { })` | Iterate with callback | `void` |
| `map.length` | Number of entries (property) | `int` |

> **Always use `or default` with `map.get()`** — the key may not exist.

## Map Iteration

```liva
// Destructured key-value (order may vary)
for subject, score in scores {
    print($"{subject}: {score}")
}

// Keys only
for key in config.keys() {
    print(key)
}

// forEach with lambda
ages.forEach((name, age) => {
    print($"{name} is {age} years old")
})
```

## Set Type Annotations

```liva
// Empty set requires type annotation
let empty: Set<string> = Set {}

// With values (type inferred from first element)
let colors = Set { "red", "green", "blue" }
let primes = Set { 2, 3, 5, 7, 11 }
```

## Set Method Table

| Method | Returns | Description |
|--------|---------|-------------|
| `set.add(value)` | `void` | Add element |
| `set.has(value)` | `bool` | Check membership |
| `set.delete(value)` | `void` | Remove element |
| `set.values()` | `[T]` | All values as array |
| `set.union(other)` | `Set<T>` | Elements in either set |
| `set.intersection(other)` | `Set<T>` | Elements in both sets |
| `set.difference(other)` | `Set<T>` | Elements in this but not other |
| `set.clear()` | `void` | Remove all elements |
| `set.forEach(fn)` | `void` | Iterate with callback |
| `set.length` | `int` | Number of elements (property) |

## Set Iteration

```liva
for n in numbers {
    print(n)
}

numbers.forEach((n) => {
    print($"Number: {n}")
})
```

## Object Literals

```liva
let user = {
    name: "Alice",
    age: 25,
    email: "alice@example.com"
}

// Property access
let name = user.name

// Dynamic property access
let key = "age"
let value = user[key]
```

## Nested Collections

```liva
// Array of arrays (matrix)
let matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]
let element = matrix[1][2]  // 6

// Object with arrays
let team = {
    name: "Engineering",
    members: ["Alice", "Bob", "Charlie"]
}
let first = team.members[0]  // "Alice"

// Nested objects
let person = {
    name: "Alice",
    address: {
        city: "Boston",
        zip: "02101"
    }
}
let city = person.address.city  // "Boston"
```

## Struct Literals (From Classes)

```liva
Person {
    name: string
    age: number
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
}

// Two creation styles:
let alice = Person { name: "Alice", age: 25 }   // Struct literal
let bob = Person("Bob", 30)                       // Constructor call
```

## Collection Type Annotation Summary

```liva
let nums: [number] = [1, 2, 3]             // Typed array
let ages: Map<string, int> = Map {}         // Typed empty map
let tags: Set<string> = Set {}              // Typed empty set
let matrix: [[number]] = [[1, 2], [3, 4]]  // Nested array type
let maybe: [string?] = ["a", null, "b"]    // Array of optionals

// Typed array of objects
let users: [{ name: string, age: number }] = [
    { name: "Alice", age: 25 },
    { name: "Bob", age: 30 }
]
```

## Iteration Patterns Summary

```liva
// Array
for item in items { print(item) }
for i, item in items { print($"{i}: {item}") }  // With index

// Map (destructured key-value)
for key, value in ages { print($"{key}: {value}") }

// Set
for color in colors { print(color) }

// Range
for i in 0..items.length { print(items[i]) }

// One-liner / point-free
for item in items => print
for item in items => process
```
