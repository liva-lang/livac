# Math Functions

> **Status:** 🚧 Planned  
> **Version:** v0.7.0 (upcoming)

Mathematical operations and constants for Liva.

---

## 📋 Planned Functions

### Core Operations
- `Math.sqrt(n: f64) => f64` - Square root
- `Math.pow(base: f64, exp: f64) => f64` - Power/exponentiation
- `Math.abs(n: f64) => f64` - Absolute value

### Rounding
- `Math.floor(n: f64) => i32` - Round down
- `Math.ceil(n: f64) => i32` - Round up
- `Math.round(n: f64) => i32` - Round to nearest

### Comparison
- `Math.min(a: f64, b: f64) => f64` - Minimum of two numbers
- `Math.max(a: f64, b: f64) => f64` - Maximum of two numbers

### Random
- `Math.random() => f64` - Random number [0.0, 1.0)

### Constants
- `Math.PI` - π (3.14159...)
- `Math.E` - e (2.71828...)

---

## 🔮 Future Examples

```liva
// Square root
let sqrt = Math.sqrt(16.0)
print(sqrt)  // 4.0

// Power
let squared = Math.pow(5.0, 2.0)
print(squared)  // 25.0

// Rounding
let rounded = Math.round(3.7)
print(rounded)  // 4

// Random
let rand = Math.random()
print(rand)  // Random number between 0.0 and 1.0

// Constants
let area = Math.PI * radius * radius
```

---

## 📝 See Also

- [Array Methods](./arrays.md)
- [String Methods](./strings.md)
- [Standard Library Overview](./README.md)
