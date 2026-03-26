# Type Aliases

Create alternative names for types — purely compile-time, zero runtime overhead.

## Syntax

```liva
type UserId = int
type Point2D = (float, float)
type Pair<T, U> = (T, U)
type Triple<A, B, C> = (A, B, C)
```

## Generic Aliases

```liva
// Single parameter
type Container<T> = (T, int)
let numbers: Container<int> = (42, 1)
let words: Container<string> = ("hello", 5)

// Multiple parameters
type KeyValue<K, V> = (K, V)

// Nested generic
type DoublePair<T> = Pair<T, T>
let coords: DoublePair<int> = (10, 20)
```

## Restrictions

1. **No circular references:**
   ```liva
   type A = B                   // ❌ Error
   type B = A                   // ❌ Error
   type Tree = (int, Tree)      // ❌ Self-referencing
   ```
   Use classes for recursive types: `Node { value: int; left: Node?; right: Node? }`

2. **Generic params must be declared:**
   ```liva
   type Container = (T, int)      // ❌ T not defined
   type Container<T> = (T, int)   // ✅
   ```

3. **Aliases are transparent (not new types):**
   ```liva
   type UserId = int
   type PostId = int
   let uid: UserId = 42
   let pid: PostId = uid   // ✅ Both are int — no type distinction
   ```
   For distinct types, use classes instead.

## Visibility

```liva
type Point = (float, float)      // Public (exported)
type _InternalBuf = [u8]         // Private (_ prefix)
```

## Practical Examples

```liva
// Domain types for clarity
type Latitude = float
type Longitude = float
type Coordinates = (Latitude, Longitude)

// Simplify repeated complex types
type Point3D = (float, float, float)

distance3D(p1: Point3D, p2: Point3D): float {
    let dx = p2.0 - p1.0
    let dy = p2.1 - p1.1
    let dz = p2.2 - p1.2
    return Math.sqrt(dx * dx + dy * dy + dz * dz)
}

// API response pattern
type ApiResult<T> = (T?, string)

fetchUser(id: int): ApiResult<User> {
    let resp, err = async HTTP.get($"https://api.example.com/users/{id}")
    if err { return (null, $"{err}") }
    let user, jsonErr = JSON.parse(resp.body)
    if jsonErr { return (null, $"{jsonErr}") }
    return (user, "")
}
```

## When to Use / Not Use

- **Use:** simplify complex tuple types, domain modeling, consistent API types
- **Don't use:** single-use types (inline instead), creating distinct types (use classes), deep alias chains (`A = B = C`)
