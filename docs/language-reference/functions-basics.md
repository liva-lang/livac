# Functions: Basics

> Arrow/block syntax, default params, tuple returns, and point-free refs are in SKILL.md. This file covers parameter destructuring, typed parameters, rest patterns, and return type details.

## Parameter Destructuring

### Array Destructuring in Parameters

```liva
printPair([first, second]: [int]): int {
    print("First:", first)
    return first + second
}

// In lambdas — works with map, filter, reduce, forEach, find, some, every
let points = [[1, 2], [3, 4], [5, 6]]
let sums = points.map(([a, b]) => a + b)           // [3, 7, 11]
let filtered = points.filter(([x, y]) => x > 2)    // [[3, 4], [5, 6]]
let total = points.reduce((acc, [x, y]) => acc + x + y, 0)  // 21
```

### Object Destructuring in Parameters

```liva
User { id: int; name: string; email: string }

printUser({id, name}: User) {
    print($"User #{id}: {name}")
}

// In lambdas
users.forEach(({id, name}) => print($"#{id}: {name}"))
users.map(({name}) => name)
```

### Field Renaming

```liva
Person { firstName: string; lastName: string }

greet({firstName: first, lastName: last}: Person) {
    print($"Hello, {first} {last}!")
}
```

### Rest Patterns

```liva
processList([head, ...tail]: [int]) {
    print("First:", head)       // 10
    print("Remaining:", tail)   // [20, 30, 40]
}
```

### Multiple Destructured Parameters

```liva
addPairs([a, b]: [int], [c, d]: [int]): int => a + b + c + d
```

### Parallel Execution with Destructuring

```liva
let data = [[1, 2], [3, 4], [5, 6]]
data.parvec().forEach(([x, y]) => print($"({x}, {y})"))
```

## Typed Parameters

```liva
// Explicit types
calculateTax(amount: number, rate: float): float => amount * rate

// Type annotations recommended for public APIs
formatUser({id, name}: User): string => $"User {id}: {name}"

// Without types — compiler infers from usage
sum([a, b]) => a + b
```

## Return Types

### Inference Rules

- Arrow functions: return type inferred from expression
- Block functions: inferred from `return` statements
- Void: no `return` statement = void return
- **Tuple returns require explicit type annotation** — inference defaults to f64

### Optional Returns

```liva
findUser(id: number): string? {
    if id == 1 { return "Alice" }
    return null
}
```

### Tuple Returns

```liva
// Explicit type REQUIRED
getPoint(): (int, int) => (10, 20)
getUserInfo(): (string, int, bool) => ("Alice", 30, true)

// Access via .0, .1, .2
let p = getPoint()
let x = p.0
let y = p.1

// Destructuring in let NOT yet supported — use positional access
// Pattern matching on tuples works in switch:
let msg = switch getStatus() {
    (200, text) => $"OK: {text}",
    (404, _) => "Not Found",
    (code, text) => $"Status {code}: {text}"
}
```

### Array Returns

```liva
getNumbers(): [number] => [1, 2, 3, 4, 5]
```
