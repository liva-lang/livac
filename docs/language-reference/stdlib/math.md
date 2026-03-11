# Math Functions

> **Status:** ✅ Complete (14 functions/constants)  
> **Version:** v1.4.0

Mathematical operations and constants for Liva.

---

## 📋 Table of Contents

- [Constants](#constants)
- [Core Operations](#core-operations)
- [Rounding](#rounding)
- [Comparison](#comparison)
- [Random](#random)

---

## Constants

```liva
Math.PI              // 3.141592653589793
Math.E               // 2.718281828459045
```

---

## Core Operations

### `Math.sqrt(n)`

Square root.

```liva
let result = Math.sqrt(16.0)   // 4.0
let root = Math.sqrt(2.0)     // 1.4142135623730951
```

### `Math.pow(base, exp)`

Power/exponentiation.

```liva
let squared = Math.pow(5.0, 2.0)  // 25.0
let cubed = Math.pow(2.0, 3.0)    // 8.0
```

### `Math.abs(n)`

Absolute value.

```liva
let pos = Math.abs(-10.5)   // 10.5
let same = Math.abs(5.0)    // 5.0
```

### `Math.log(x)` *(v1.4.0)*

Natural logarithm (ln).

```liva
let ln = Math.log(2.718)     // ~1.0
let ln2 = Math.log(1.0)     // 0.0
```

**Rust Codegen:**
```rust
(x as f64).ln()
```

---

## Rounding

### `Math.floor(n)`

Round down to the nearest integer.

```liva
let result = Math.floor(3.7)   // 3
let neg = Math.floor(-1.2)    // -2
```

### `Math.ceil(n)`

Round up to the nearest integer.

```liva
let result = Math.ceil(3.2)   // 4
let neg = Math.ceil(-1.8)    // -1
```

### `Math.round(n)`

Round to the nearest integer.

```liva
let result = Math.round(3.5)   // 4
let down = Math.round(3.4)    // 3
```

---

## Comparison

### `Math.min(a, b)`

Return the smaller of two numbers.

```liva
let smaller = Math.min(10.5, 20.3)  // 10.5
```

### `Math.max(a, b)`

Return the larger of two numbers.

```liva
let larger = Math.max(10.5, 20.3)  // 20.3
```

### `Math.clamp(val, min, max)` *(v1.4.0)*

Clamp a value to a range.

```liva
let clamped = Math.clamp(15, 0, 10)   // 10
let inRange = Math.clamp(5, 0, 10)    // 5
let tooLow = Math.clamp(-5, 0, 10)    // 0
```

**Rust Codegen:**
```rust
val.max(min).min(max)
```

### `Math.sign(val)` *(v1.4.0)*

Return the sign of a number: -1, 0, or 1.

```liva
let neg = Math.sign(-42)    // -1
let zero = Math.sign(0)     // 0
let pos = Math.sign(100)    // 1
```

**Rust Codegen:**
```rust
{
    let v = val;
    if v > 0 { 1 } else if v < 0 { -1 } else { 0 }
}
```

---

## Random

### `Math.random()`

Generate a random float between 0.0 (inclusive) and 1.0 (exclusive).

```liva
let rand = Math.random()
print(rand)  // e.g., 0.7234...
```

---

## 📝 See Also

- [Array Methods](./arrays.md)
- [String Methods](./strings.md)
- [Standard Library Overview](./README.md)
