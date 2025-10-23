# Trait Aliases Guide for Liva Generics

**Version:** 0.9.2  
**Last Updated:** October 23, 2025

---

## Overview

Liva offers **two approaches** to generic constraints:
1. **Trait Aliases** - Simple, intuitive, recommended for most cases
2. **Granular Traits** - Precise control when you need it

You can use either approach or **mix both** for maximum flexibility.

---

## Trait Aliases (Recommended)

### Built-in Aliases

| Alias | Expands To | Use Case | Example |
|-------|-----------|----------|---------|
| `Numeric` | `Add + Sub + Mul + Div + Rem + Neg` | Any arithmetic | `sum<T: Numeric>(a, b)` |
| `Comparable` | `Ord + Eq` | Comparisons, sorting | `max<T: Comparable>(a, b)` |
| `Number` | `Numeric + Comparable` | Full number operations | `clamp<T: Number>(val, min, max)` |
| `Printable` | `Display + Debug` | Formatting, debugging | `show<T: Printable>(value)` |

### When to Use Each Alias

#### Numeric

Use when you need **any combination** of arithmetic operations:

```liva
// General arithmetic functions
sum<T: Numeric>(a: T, b: T): T => a + b
subtract<T: Numeric>(a: T, b: T): T => a - b
multiply<T: Numeric>(a: T, b: T): T => a * b
divide<T: Numeric>(a: T, b: T): T => a / b
negate<T: Numeric>(value: T): T => -value

// Complex arithmetic
average<T: Numeric>(a: T, b: T, divisor: T): T {
    let sum_val = a + b
    return sum_val / divisor
}
```

**Works with:** `int`, `float`, custom numeric types

#### Comparable

Use when you need **ordering or equality**:

```liva
// Finding extremes
max<T: Comparable>(a: T, b: T): T {
    if a > b { return a }
    return b
}

min<T: Comparable>(a: T, b: T): T {
    if a < b { return a }
    return b
}

// Equality checking
equals<T: Comparable>(a: T, b: T): bool => a == b

notEquals<T: Comparable>(a: T, b: T): bool => a != b

// Sorting helpers
isSorted<T: Comparable>(a: T, b: T, c: T): bool {
    if a <= b {
        if b <= c {
            return true
        }
    }
    return false
}
```

**Works with:** `int`, `float`, `string`, `bool`, custom ordered types

#### Number

Use when you need **both arithmetic AND comparison**:

```liva
// Clamping (uses comparison + returns value)
clamp<T: Number>(value: T, min_val: T, max_val: T): T {
    if value < min_val { return min_val }
    if value > max_val { return max_val }
    return value
}

// Range calculations
inRange<T: Number>(value: T, min_val: T, max_val: T): bool {
    if value >= min_val {
        if value <= max_val {
            return true
        }
    }
    return false
}

// Distance (comparison + subtraction)
distance<T: Number>(a: T, b: T): T {
    if a > b { return a - b }
    return b - a
}

// Statistical functions
median<T: Number>(a: T, b: T, c: T): T {
    // Uses both comparison and returns numeric value
    if a > b {
        if b > c { return b }
        if a > c { return c }
        return a
    }
    if a > c { return a }
    if b > c { return c }
    return b
}
```

**Works with:** `int`, `float`, custom numeric types with ordering

#### Printable

Use when you need **formatting for output**:

```liva
// Simple display
show<T: Printable>(value: T) {
    console.log(value)
}

// Formatted output
logWithLabel<T: Printable>(label: string, value: T) {
    console.log($"{label}: {value}")
}

// Debug output
debug<T: Printable>(name: string, value: T) {
    console.log($"[DEBUG] {name} = {value}")
}
```

**Works with:** Any type implementing Display/Debug

---

## Granular Traits (For Precise Control)

### Individual Traits

**Arithmetic:**
- `Add` - Addition only (`+`)
- `Sub` - Subtraction only (`-`)
- `Mul` - Multiplication only (`*`)
- `Div` - Division only (`/`)
- `Rem` - Remainder only (`%`)
- `Neg` - Unary negation (`-value`)

**Comparison:**
- `Eq` - Equality only (`==`, `!=`)
- `Ord` - Ordering only (`<`, `>`, `<=`, `>=`)

**Utilities:**
- `Display` - User-facing formatting
- `Debug` - Debug formatting
- `Clone` - Deep copying
- `Copy` - Bitwise copying
- `Not` - Boolean negation

### When to Use Granular Traits

Use granular traits when you need **precise control** or want to be **more restrictive**:

```liva
// Only allow addition (not subtraction, multiplication, etc.)
addOnly<T: Add>(a: T, b: T): T => a + b

// Only allow comparison (not arithmetic)
isLessThan<T: Ord>(a: T, b: T): bool => a < b

// Only equality (not ordering)
areEqual<T: Eq>(a: T, b: T): bool => a == b

// Specific combination (Add + Ord)
addIfGreater<T: Add + Ord>(a: T, b: T, threshold: T): T {
    if a > threshold {
        return a + b
    }
    return a
}
```

---

## Mixing Approaches

You can **combine trait aliases with granular traits**:

```liva
// Comparable (alias) + Display (granular)
formatComparison<T: Comparable + Display>(a: T, b: T): string {
    if a == b { return $"Equal: {a}" }
    if a > b { return $"{a} > {b}" }
    return $"{a} < {b}"
}

// Numeric (alias) + Printable (alias)
debugArithmetic<T: Numeric + Printable>(a: T, b: T): T {
    console.log($"Computing {a} + {b}")
    let result = a + b
    console.log($"Result: {result}")
    return result
}

// Number (alias) + Display (granular)
formatNumber<T: Number + Display>(value: T, min: T, max: T): string {
    let clamped = if value < min { min } 
                  else if value > max { max }
                  else { value }
    return $"Value: {clamped}"
}

// Multiple granular + alias
complexOperation<T: Add + Mul + Comparable>(a: T, b: T, c: T): T {
    let sum_val = a + b
    let product = sum_val * c
    if product > a { return product }
    return a
}
```

---

## Decision Tree

```
Need generic constraints?
  │
  ├─ Arithmetic operations only?
  │  └─ Use Numeric
  │
  ├─ Comparisons only?
  │  └─ Use Comparable
  │
  ├─ Both arithmetic AND comparisons?
  │  └─ Use Number
  │
  ├─ Need to format/print?
  │  └─ Add Printable
  │
  ├─ Need VERY specific operations?
  │  └─ Use granular traits (Add, Ord, etc.)
  │
  └─ Complex requirements?
     └─ Mix aliases + granular traits
```

---

## Best Practices

### ✅ Do

**1. Start with aliases** (simpler, more intuitive):
```liva
// Good - uses alias
average<T: Numeric>(a: T, b: T, divisor: T): T { ... }
```

**2. Use granular when you need precision**:
```liva
// Good - only needs addition
increment<T: Add>(value: T, amount: T): T => value + amount
```

**3. Mix when appropriate**:
```liva
// Good - Number for math, Display for output
formatCalculation<T: Number + Display>(a: T, b: T): string {
    let sum_val = a + b
    return $"Sum: {sum_val}"
}
```

**4. Document why you chose granular over alias**:
```liva
// Only allow addition to enforce API constraints
// (Numeric would allow subtraction which breaks invariants)
addOnly<T: Add>(a: T, b: T): T => a + b
```

### ❌ Don't

**1. Don't use granular when alias suffices**:
```liva
// Bad - overly specific
badSum<T: Add + Sub + Mul + Div + Rem + Neg>(a: T, b: T): T => a + b

// Good - uses alias
sum<T: Numeric>(a: T, b: T): T => a + b
```

**2. Don't over-constrain**:
```liva
// Bad - only needs Add, not full Numeric
increment<T: Numeric>(value: T, step: T): T => value + step

// Good - precisely what's needed
increment<T: Add>(value: T, step: T): T => value + step
```

**3. Don't use Number when Numeric/Comparable suffices**:
```liva
// Bad - doesn't use comparison
badSum<T: Number>(a: T, b: T): T => a + b

// Good - only needs arithmetic
sum<T: Numeric>(a: T, b: T): T => a + b
```

---

## Examples by Use Case

### Mathematical Operations

```liva
// Basic arithmetic
sum<T: Numeric>(a: T, b: T): T => a + b
product<T: Numeric>(a: T, b: T): T => a * b

// With comparison
clamp<T: Number>(val: T, min: T, max: T): T { ... }
abs<T: Number>(val: T, zero: T): T {
    if val < zero { return -val }
    return val
}
```

### Data Structures

```liva
// Generic container with comparison
BinarySearchTree<T: Comparable> {
    value: T
    left: BinarySearchTree<T>?
    right: BinarySearchTree<T>?
    
    insert(val: T) {
        if val < this.value {
            // insert left
        } else {
            // insert right
        }
    }
}

// Numeric accumulator
Accumulator<T: Numeric> {
    total: T
    
    add(value: T) {
        this.total = this.total + value
    }
}
```

### Formatting and Display

```liva
// Format any printable value
prettyPrint<T: Printable>(values: [T]) {
    for value in values {
        console.log($"  - {value}")
    }
}

// Format numbers with comparison
formatRange<T: Number + Display>(min: T, max: T): string {
    return $"Range: [{min}, {max}]"
}
```

---

## Common Patterns

### 1. Factory Functions

```liva
// Create with default using Number
createWithDefault<T: Number>(value: T, min: T): T {
    if value < min { return min }
    return value
}
```

### 2. Transformation Functions

```liva
// Transform with arithmetic
scale<T: Numeric>(value: T, factor: T): T => value * factor

// Transform with bounds
clampTransform<T: Number>(value: T, min: T, max: T, scale: T): T {
    let clamped = if value < min { min }
                  else if value > max { max }
                  else { value }
    return clamped * scale
}
```

### 3. Validation Functions

```liva
// Validate range using Comparable
isValid<T: Comparable>(value: T, min: T, max: T): bool {
    if value < min { return false }
    if value > max { return false }
    return true
}
```

---

## Summary

| Scenario | Recommended Approach | Example |
|----------|---------------------|---------|
| General arithmetic | `Numeric` | `sum<T: Numeric>` |
| General comparison | `Comparable` | `max<T: Comparable>` |
| Arithmetic + comparison | `Number` | `clamp<T: Number>` |
| Only addition | `Add` | `increment<T: Add>` |
| Only ordering | `Ord` | `isGreater<T: Ord>` |
| Display + comparison | `Comparable + Display` | `formatCompare<T: Comparable + Display>` |
| All operations + formatting | `Number + Printable` | `debugCalc<T: Number + Printable>` |

**Philosophy:** Start simple with aliases, get granular when you need precision. Mix both for complex requirements.

---

**Remember:** Liva gives you the **best of both worlds** - simplicity for common cases, precision for advanced needs.
