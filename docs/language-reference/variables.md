# Variables — Additional Reference

> SKILL.md covers let/const, types, and basic destructuring.
> This file covers **scoping rules, advanced destructuring, and edge cases only**.

## Scoping Rules

### Block Scope

Variables are block-scoped (like JavaScript `let`):

```liva
main() {
    let x = 10

    if true {
        let y = 20
        print(x)  // ✅ Accessible: 10
        print(y)  // ✅ Accessible: 20
    }

    print(x)  // ✅ Accessible: 10
    print(y)  // ❌ Compile error: y not in scope
}
```

### Function Scope

```liva
calculate() {
    let result = 42
    return result
}

main() {
    print(result)  // ❌ Compile error: result not defined
}
```

### Loop Scope

Loop variables and inner bindings are scoped to the loop body:

```liva
main() {
    for i in 1..5 {
        let squared = i * i
    }
    print(i)       // ❌ i not in scope
    print(squared) // ❌ squared not in scope
}
```

### Shadowing

Inner scopes can shadow outer variables without affecting them:

```liva
main() {
    let x = 10
    if true {
        let x = 20   // Shadows outer x
        print(x)     // 20
    }
    print(x)         // 10 (original unchanged)
}
```

## Advanced Destructuring

### Skip Elements

```liva
let [first, , third] = [1, 2, 3]
print($"{first}, {third}")  // 1, 3
```

### Rest Patterns

```liva
// Array rest — captures remaining elements
let [head, ...tail] = [1, 2, 3, 4, 5]
print(head)  // 1
print(tail)  // [2, 3, 4, 5]

// Object rest — captures remaining fields
let {id, ...rest} = user
print(rest)  // remaining fields as object
```

### Object Destructuring with Renaming

```liva
let {name: userName, email: userEmail} = user
print($"{userName} <{userEmail}>")
```

### Nested Destructuring

```liva
// Nested arrays
let [[a, b], [c, d]] = [[1, 2], [3, 4]]
print($"{a}, {b}, {c}, {d}")  // 1, 2, 3, 4

// Nested objects
let {address: {city, country}} = user
print($"{city}, {country}")
```

### Type Annotations with Destructuring

```liva
let [x, y]: [int] = [10, 20]
let {id, name}: User = getUser(1)
let [first, ...rest]: [string] = ["a", "b", "c"]
```

## Error Binding Details

### With Async/Parallel

```liva
// Async — lazy await on first use
let asyncResult, asyncErr = async fetchData(url)
print(asyncResult)  // Implicitly awaits here

// Parallel — lazy join on first use
let parResult, parErr = par heavyComputation(100)
print(parResult)    // Implicitly joins here
```

### Ignoring Errors

```liva
let value, _ = divide(10, 2)  // Ignore error with _
print(value)
```

### Non-Fallible Functions

Error binding on non-fallible functions always gives empty `err`:

```liva
multiply(a: number, b: number) => a * b

let result, err = multiply(5, 3)  // err will be ""
print(result)  // 15
```

## Mutability

All `let` variables are mutable. Use `const` for immutability:

```liva
let counter = 0
counter += 1            // ✅ Compound assignment

const PI = 3.14159
PI = 3.0                // ❌ Compile error: cannot reassign const
```

| Feature | `let` | `const` |
|---------|-------|---------|
| Mutability | Mutable | Immutable |
| Reassignment | ✅ Allowed | ❌ Forbidden |
| Type Annotation | Optional | Optional |
| Scoping | Block-scoped | Block-scoped |

## Initialization

All variables must be initialized at declaration:

```liva
let x = 10         // ✅
let y: number      // ❌ Compile error: uninitialized variable
```
