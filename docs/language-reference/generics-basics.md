# Generics

Generics enable parametric polymorphism — write code that works with multiple types while maintaining type safety.

---

## Generic Functions

Type parameters in angle brackets before the parameter list:

```liva
identity<T>(value: T): T {
    return value
}

// One-liner
identity<T>(value: T): T => value

// Multiple type parameters
process<T, U>(first: T, second: U): T {
    return first
}
```

Type is **inferred** from arguments — explicit type args rarely needed:

```liva
let x = identity(42)        // T = int
let y = identity("hello")   // T = string
```

Explicit when needed:

```liva
let result = identity<string>("test")
```

---

## Generic Classes

Type parameters after class name — NO `class` keyword:

```liva
Box<T> {
    value: T

    constructor(value: T) {
        this.value = value
    }

    get(): T {
        return this.value
    }
}

let intBox = Box(42)        // T = int
let strBox = Box("hello")   // T = string
```

### Multiple Type Parameters

```liva
Pair<K, V> {
    key: K
    value: V

    constructor(key: K, value: V) {
        this.key = key
        this.value = value
    }
}
```

### Generic Method in Non-Generic Class

```liva
Container {
    wrap<T>(value: T): T {
        return value
    }
}
```

---

## Type Constraints

### Trait Aliases (Recommended)

| Alias | Expands To | Use Case |
|-------|-----------|----------|
| `Numeric` | `Add + Sub + Mul + Div + Rem + Neg` | Arithmetic |
| `Comparable` | `Ord + Eq` | Comparisons |
| `Number` | `Numeric + Comparable` | Complete number ops |
| `Printable` | `Display + Debug` | Formatting |

```liva
sum<T: Numeric>(a: T, b: T): T => a + b

max<T: Comparable>(a: T, b: T): T {
    if a > b { return a }
    return b
}

clamp<T: Number>(value: T, minVal: T, maxVal: T): T {
    if value < minVal { return minVal }
    if value > maxVal { return maxVal }
    return value
}
```

### Granular Traits (Fine Control)

Individual traits: `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`, `Eq`, `Ord`, `Clone`, `Copy`, `Display`, `Debug`, `Not`.

```liva
addOnly<T: Add>(a: T, b: T): T => a + b
lessThan<T: Ord>(a: T, b: T): bool => a < b
```

### Mixing Aliases + Granular

```liva
formatAndCompare<T: Comparable + Display>(a: T, b: T): string {
    if a == b { return $"Equal: {a}" }
    if a > b { return $"{a} > {b}" }
    return $"{a} < {b}"
}

// Multiple constraints
process<T: Comparable, U>(first: T, second: U): T {
    return first
}
```

### Class with Constraint

```liva
ComparableBox<T: Comparable> {
    value: T

    constructor(value: T) {
        this.value = value
    }

    isGreater(other: T): bool {
        return this.value > other
    }
}
```

---

## Type Inference

| Context | Example | Inference |
|---------|---------|-----------|
| From arguments | `identity(42)` | `T = int` |
| From variable type | `let b: Box<int> = Box(42)` | `T = int` |
| From return type | `let b: Box<string> = makeBox("hi")` | `T = string` |
| Explicit | `Array<int>()` | Direct specification |

---

## Array Syntax

Arrays use `[T]`, NOT `T[]`:

```liva
type IntArray = [int]
type Matrix = [[int]]
```

---

## Notes

- `where` clauses are planned but not yet implemented.
- Generic interfaces follow the same `Name<T> { signatures }` pattern.
- No `Option<T>` or `Result<T,E>` as generic classes — use `type?` and `fail`/`or fail` instead.
