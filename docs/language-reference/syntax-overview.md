# 📝 Syntax Overview

Quick reference of Liva language syntax. Each section links to its full dedicated doc.

## Comments

```liva
// Single-line comment

/*
  Multi-line comment
  Can span multiple lines
*/
```

## Variables and Constants

```liva
let x = 10                     // Mutable variable
let name: string = "Alice"     // With type annotation
x = 20                         // Reassignment

const PI = 3.1416              // Immutable constant
const MAX_USERS = 100

let [a, b] = [1, 2]           // Destructuring
let {id, name} = user          // Object destructuring
let [head, ...tail] = [1, 2, 3]
```

> See [variables.md](variables.md) for full reference.

## Functions

```liva
// One-liner
add(a: number, b: number): number => a + b

// Block function
greet(name: string) {
  print($"Hello, {name}!")
}

// Parameter destructuring
func([x, y]: [int]) { }
func({id, name}: User) { }
```

> See [functions-basics.md](functions-basics.md) and [functions-advanced.md](functions-advanced.md) for full reference.

## Classes

```liva
Person {
  name: string
  age: number

  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }

  greet() { print($"Hi, I'm {this.name}") }
}

let p = Person("Alice", 30)
```

Visibility: `name` (public), `_name` (private).

> See [classes-basics.md](classes-basics.md), [classes-data.md](classes-data.md), and [classes-interfaces.md](classes-interfaces.md) for full reference.

## Control Flow

### If Statements

```liva
if condition {
  // code
} else if condition2 {
  // code
} else {
  // code
}

// Logical operators
if age >= 18 and hasId { print("Can enter") }
if isAdmin or isModerator { print("Has access") }
```

### For Loops

```liva
for i in 0..10 { print(i) }           // Range (exclusive end)
for i in 0..=10 { print(i) }          // Range (inclusive end)

for item in items { print(item) }      // Array iteration
```

### While Loops

```liva
let counter = 0
while counter < 10 {
  print(counter)
  counter = counter + 1
  if counter == 5 break                // Loop control
}
```

### Switch Statements

```liva
switch status {
  case "active": print("Active")
  case "inactive": print("Inactive")
  default: print("Unknown")
}
```

> See [control-flow.md](control-flow.md) for full reference.

## Operators

```liva
// Arithmetic
a + b    a - b    a * b    a / b    a % b

// Comparison
a == b   a != b   a < b    a <= b   a > b    a >= b

// Logical (word operators preferred)
a and b    a or b    not a
a && b     a || b    !a

// Assignment
x = 10

// Method reference (v1.1.0)
names.map(fmt::format)       // binds instance method as callback
```

> See [operators.md](operators.md) for full reference.

## String Templates

```liva
let name = "Alice"
let msg = $"Hello, {name}!"
let math = $"2 + 2 = {2 + 2}"
```

> See [string-templates.md](string-templates.md) for full reference.

## Arrays and Objects

```liva
// Arrays
let numbers = [1, 2, 3, 4, 5]
let first = numbers[0]
let doubled = numbers.map(x => x * 2)
let evens = numbers.filter(x => x % 2 == 0)
numbers.forEach(print)                    // point-free (v1.1.0)

// Objects
let person = { name: "Alice", age: 30 }
print(person.name)

// Array of objects
let users = [
  { name: "Alice", age: 30 },
  { name: "Bob", age: 25 }
]
```

> See [collections.md](collections.md) for full reference.

## Concurrency

```liva
// Async (I/O-bound)
let data = async fetchFromAPI()
let task = task async fetchUser(123)
let user = await task
fire async logEvent("started")

// Parallel (CPU-bound)
let result = par heavyComputation()
fire par backgroundCleanup()

// Data-parallel loop
for par item in items with chunk 2 threads 4 {
  process(item)
}
```

> See [concurrency.md](concurrency.md) for full reference.

## Error Handling

```liva
// Fail keyword
divide(a, b) => b == 0 ? fail "Division by zero" : a / b

// Error binding
let result, err = divide(10, 2)
if err != "" { print($"Error: {err}") }

// Ignore error
let result, _ = divide(10, 2)

// Or fail (propagation)
let data = fetchData() or fail
```

> See [error-handling.md](error-handling.md) for full reference.

## Types

```liva
// Liva types
let count: number = 100
let name: string = "Alice"
let active: bool = true

// Rust numeric types
let a: i32 = 42
let b: u64 = 100
let c: f64 = 3.14
let d: usize = 10
```

> See [types.md](types.md), [types-primitives.md](types-primitives.md), and [types-advanced.md](types-advanced.md) for full reference.

## Enums

```liva
enum Color { Red, Green, Blue }
enum Shape {
  Circle(radius: float)
  Rectangle(width: float, height: float)
}

let s = Shape.Circle(5.0)
match s {
  Shape.Circle(r) => print($"Circle r={r}")
  Shape.Rectangle(w, h) => print($"Rect {w}x{h}")
}
```

> See [enums.md](enums.md) and [pattern-matching.md](pattern-matching.md) for full reference.

## Generics

```liva
first<T>(items: [T]): T => items[0]

Stack<T> {
  items: [T]
  push(item: T) { this.items.push(item) }
  pop(): T { return this.items.pop() }
}
```

> See [generics-basics.md](generics-basics.md) and [generics-advanced.md](generics-advanced.md) for full reference.

## Literals

```liva
// Numbers
let decimal = 42
let hex = 0xFF
let octal = 0o77
let binary = 0b1010
let float = 3.14
let scientific = 1.23e-4

// Strings
let simple = "Hello"
let template = $"Hello, {name}"

// Booleans
let yes = true
let no = false

// Collections
let arr = [1, 2, 3]
let obj = { name: "Alice", age: 30 }
```

## Keywords

```
let       const     if        else      for       in
while     switch    case      default   return    break
continue  async     par       task      fire      await
fail      and       or        not       true      false
this      constructor         enum      match     data
describe  test      expect    use       import    export
```

## See Also

- **[Variables](variables.md)** — Variable declarations and destructuring
- **[Types](types.md)** — Type system details
- **[Functions](functions-basics.md)** — Function reference
- **[Classes](classes-basics.md)** — Class reference
- **[Control Flow](control-flow.md)** — Control structures
- **[Operators](operators.md)** — All operators
- **[Collections](collections.md)** — Arrays and objects
- **[Concurrency](concurrency.md)** — Async and parallel
- **[Error Handling](error-handling.md)** — Fallibility system
- **[Enums](enums.md)** — Enum types and pattern matching
- **[Generics](generics-basics.md)** — Generic types and functions
- **[String Templates](string-templates.md)** — String interpolation

---

**Next:** [Variables](variables.md)
