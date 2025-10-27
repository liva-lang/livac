# Type Aliases in Liva

**Version:** v0.11.1  
**Status:** Implemented  
**Date:** 2025-01-28

---

## Overview

Type aliases allow you to create alternative names for existing types, making code more readable and maintainable. They are purely compile-time constructs with zero runtime overhead.

---

## Syntax

### Basic Type Alias

```liva
type AliasName = ExistingType
```

**Example:**
```liva
type UserId = int
type Username = string
type Coordinate = float
```

### Generic Type Alias

```liva
type AliasName<T> = GenericType<T>
type AliasName<T, U> = ComplexType<T, U>
```

**Example:**
```liva
type Box<T> = (T,)
type Pair<T, U> = (T, U)
type Triple<A, B, C> = (A, B, C)
```

---

## Use Cases

### 1. Simplify Complex Types

**Before:**
```liva
processCoordinate(point: (float, float)): float {
    // ...
}

calculateDistance(p1: (float, float), p2: (float, float)): float {
    // ...
}
```

**After:**
```liva
type Point2D = (float, float)

processCoordinate(point: Point2D): float {
    // ...
}

calculateDistance(p1: Point2D, p2: Point2D): float {
    // ...
}
```

### 2. Domain Modeling

```liva
// Make intent explicit
type UserId = int
type PostId = int
type CommentId = int

// Instead of generic 'int' everywhere
getUser(id: UserId): User { ... }
getPost(id: PostId): Post { ... }
getComment(id: CommentId): Comment { ... }
```

### 3. RGB Colors

```liva
type RGB = (u8, u8, u8)
type Color = RGB

let red: Color = (255, 0, 0)
let green: Color = (0, 255, 0)
let blue: Color = (0, 0, 255)

mixColors(c1: Color, c2: Color): Color {
    let (r1, g1, b1) = c1
    let (r2, g2, b2) = c2
    return ((r1 + r2) / 2, (g1 + g2) / 2, (b1 + b2) / 2)
}
```

### 4. Result/Error Patterns

```liva
type Result<T> = (T, string)  // (value, error)
type Option<T> = T?           // Optional value

parseNumber(input: string): Result<int> {
    let num, err = parseInt(input)
    return (num, err)
}
```

### 5. Coordinate Systems

```liva
type Point2D = (float, float)
type Point3D = (float, float, float)
type Vector2D = Point2D
type Vector3D = Point3D

addVectors(v1: Vector2D, v2: Vector2D): Vector2D {
    return (v1.0 + v2.0, v1.1 + v2.1)
}
```

---

## Generic Type Aliases

### Single Type Parameter

```liva
type Container<T> = (T, int)  // Value with count

let numbers: Container<int> = (42, 1)
let words: Container<string> = ("hello", 5)
```

### Multiple Type Parameters

```liva
type KeyValue<K, V> = (K, V)
type Map<K, V> = [KeyValue<K, V>]

let config: Map<string, int> = [
    ("timeout", 30),
    ("retries", 3),
    ("port", 8080)
]
```

### Nested Generic Aliases

```liva
type Pair<T, U> = (T, U)
type DoublePair<T> = Pair<T, T>

let coords: DoublePair<int> = (10, 20)
```

---

## Type Alias Resolution

Type aliases are expanded during type checking:

```liva
type UserId = int

getUserName(id: UserId): string { ... }

// Compiler sees:
getUserName(id: int): string { ... }
```

### No Runtime Overhead

Type aliases are compile-time only - they don't exist in the generated Rust code:

```liva
// Liva code
type Point = (int, int)
let p: Point = (10, 20)

// Generated Rust (alias expanded)
let p: (i32, i32) = (10, 20);
```

---

## Restrictions & Rules

### 1. No Circular References

```liva
// ❌ Error: Circular type alias
type A = B
type B = A

// ❌ Error: Self-referencing
type Tree = (int, Tree)
```

**Workaround:** Use classes for recursive types:
```liva
// ✅ OK: Classes can be recursive
Node {
    value: int
    left: Node?
    right: Node?
}
```

### 2. No Type Parameters Without Definition

```liva
// ❌ Error: T not defined
type Container = (T, int)

// ✅ OK: T defined as generic parameter
type Container<T> = (T, int)
```

### 3. Aliases Don't Create New Types

Type aliases are **transparent** - they don't create distinct types:

```liva
type UserId = int
type PostId = int

let uid: UserId = 42
let pid: PostId = uid  // ✅ OK: Both are 'int'
```

**Note:** Liva doesn't have "newtype" pattern (like Rust's tuple structs). Use classes if you need type distinction:

```liva
// If you need distinct types:
UserId {
    value: int
    constructor(value: int) { this.value = value }
}

PostId {
    value: int
    constructor(value: int) { this.value = value }
}

let uid = UserId(42)
let pid: PostId = uid  // ❌ Error: Type mismatch
```

---

## Scope & Visibility

### Module-Level Aliases

```liva
// math.liva
type Point = (float, float)
type Vector = Point

// Exported automatically (no _ prefix)
```

### Private Aliases

```liva
// Private type alias (not exported)
type _InternalBuffer = [u8]

_processBuffer(buf: _InternalBuffer) {
    // ...
}
```

---

## Best Practices

### ✅ Do

1. **Use for clarity:**
   ```liva
   type Timestamp = int
   type Duration = int
   ```

2. **Document complex types:**
   ```liva
   type UserPreferences = (bool, int, string)  // (notifications, theme, language)
   ```

3. **Create semantic names:**
   ```liva
   type Latitude = float
   type Longitude = float
   type Coordinates = (Latitude, Longitude)
   ```

4. **Use for API consistency:**
   ```liva
   type ApiResponse<T> = (T, string)  // (data, error)
   ```

### ❌ Don't

1. **Don't overuse:**
   ```liva
   // ❌ Too many layers
   type A = int
   type B = A
   type C = B
   
   // ✅ Keep it simple
   type Coordinate = int
   ```

2. **Don't hide important information:**
   ```liva
   // ❌ Unclear what T is
   type Thing<T> = (T, T)
   
   // ✅ Descriptive name
   type Range<T> = (T, T)
   ```

3. **Don't use for single-use types:**
   ```liva
   // ❌ Used only once
   type TempData = (int, string)
   processTempData(data: TempData) { ... }
   
   // ✅ Inline it
   processTempData(data: (int, string)) { ... }
   ```

---

## Examples

### Coordinate System

```liva
type Point2D = (float, float)
type Point3D = (float, float, float)

distance2D(p1: Point2D, p2: Point2D): float {
    let dx = p2.0 - p1.0
    let dy = p2.1 - p1.1
    return Math.sqrt(dx * dx + dy * dy)
}

distance3D(p1: Point3D, p2: Point3D): float {
    let dx = p2.0 - p1.0
    let dy = p2.1 - p1.1
    let dz = p2.2 - p1.2
    return Math.sqrt(dx * dx + dy * dy + dz * dz)
}
```

### Color System

```liva
type RGB = (u8, u8, u8)
type RGBA = (u8, u8, u8, u8)

toRGBA(color: RGB): RGBA {
    return (color.0, color.1, color.2, 255)
}

mixColors(c1: RGB, c2: RGB, ratio: float): RGB {
    let r = (c1.0 * ratio + c2.0 * (1.0 - ratio)) as u8
    let g = (c1.1 * ratio + c2.1 * (1.0 - ratio)) as u8
    let b = (c1.2 * ratio + c2.2 * (1.0 - ratio)) as u8
    return (r, g, b)
}
```

### API Response Types

```liva
type ApiResult<T> = (T?, string)  // (data, error)

User {
    id: int
    name: string
    email: string
}

fetchUser(id: int): ApiResult<User> {
    let response, err = async HTTP.get($"https://api.example.com/users/{id}")
    
    if err != "" {
        return (null, err)
    }
    
    let user: User, jsonErr = JSON.parse(response.body)
    if jsonErr != "" {
        return (null, jsonErr)
    }
    
    return (user, "")
}

main() {
    let user, error = fetchUser(1)
    
    if error != "" {
        console.error($"Error: {error}")
    } else {
        print($"User: {user.name}")
    }
}
```

---

## Comparison with Other Languages

### TypeScript

```typescript
// TypeScript
type Point = [number, number];
type UserId = number;
```

```liva
// Liva
type Point = (float, float)
type UserId = int
```

### Rust

```rust
// Rust
type Point = (f64, f64);
type Result<T, E> = std::result::Result<T, E>;
```

```liva
// Liva
type Point = (float, float)
type Result<T, E> = (T?, E)
```

### Haskell

```haskell
-- Haskell
type Point = (Double, Double)
type UserId = Int
```

```liva
// Liva
type Point = (float, float)
type UserId = int
```

---

## Implementation Details

### AST Representation

```rust
pub struct TypeAlias {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub target_type: TypeRef,
    pub span: Option<Span>,
}
```

### Type Resolution

1. **Registration:** Type aliases registered in type registry during semantic analysis
2. **Expansion:** Aliases expanded to target types during type checking
3. **Generic Substitution:** Type parameters substituted with actual types
4. **Validation:** Circular references detected and rejected

### Generated Code

Type aliases don't generate Rust type aliases - they're expanded inline:

```liva
// Liva
type Point = (int, int)
let p: Point = (10, 20)
```

```rust
// Generated Rust (no type alias)
let p: (i32, i32) = (10, 20);
```

**Rationale:** 
- Simpler codegen (no need to track Rust alias scope)
- Matches Liva's "zero overhead" philosophy
- Aliases are for developer convenience, not runtime semantics

---

## Future Enhancements (v0.12.0+)

### 1. Type Alias Exports (v0.12.0)
```liva
// Explicit export control
export type PublicPoint = (int, int)
type PrivatePoint = (float, float)
```

### 2. Associated Type Aliases (v0.12.1)
```liva
interface Container<T> {
    type Item = T
    getItem(): Item
}
```

### 3. Conditional Type Aliases (v0.13.0)
```liva
type NumberOrString<T> = if T == int { int } else { string }
```

---

## Summary

**Type aliases provide:**
- ✅ Improved code readability
- ✅ Better documentation through names
- ✅ Type-safe domain modeling
- ✅ Zero runtime overhead
- ✅ Generic type support
- ✅ Simple, transparent semantics

**When to use:**
- Simplifying complex tuple types
- Creating semantic domain types
- Building consistent APIs
- Documenting type intentions

**When NOT to use:**
- Single-use types (inline instead)
- Creating distinct types (use classes)
- Hiding important type information

---

**Next:** [Union Types](union-types.md) | [Advanced Types Guide](../guides/advanced-types.md)
