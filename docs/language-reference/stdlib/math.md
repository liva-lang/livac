# Math Functions

Mathematical operations and constants for Liva.

## Constants

```liva
Math.PI              // 3.141592653589793
Math.E               // 2.718281828459045
```

## Core Operations

### `Math.sqrt(n)` → `float`

```liva
let result = Math.sqrt(16.0)   // 4.0
```

### `Math.pow(base, exp)` → `float`

```liva
let squared = Math.pow(5.0, 2.0)  // 25.0
```

### `Math.abs(n)` → `float`

```liva
let pos = Math.abs(-10.5)   // 10.5
```

### `Math.log(x)` → `float`

Natural logarithm (ln).

```liva
let ln = Math.log(2.718)     // ~1.0
```

## Rounding

### `Math.floor(n)` → `int`

```liva
let result = Math.floor(3.7)   // 3
```

### `Math.ceil(n)` → `int`

```liva
let result = Math.ceil(3.2)   // 4
```

### `Math.round(n)` → `int`

```liva
let result = Math.round(3.5)   // 4
```

## Comparison

### `Math.min(a, b)` → `float`

```liva
let smaller = Math.min(10.5, 20.3)  // 10.5
```

### `Math.max(a, b)` → `float`

```liva
let larger = Math.max(10.5, 20.3)  // 20.3
```

### `Math.clamp(val, min, max)` → number

Clamp a value to a range.

```liva
let clamped = Math.clamp(15, 0, 10)   // 10
```

### `Math.sign(val)` → `int`

Return the sign: -1, 0, or 1.

```liva
let neg = Math.sign(-42)    // -1
```

## Random

### `Math.random()` → `float`

Random float in [0.0, 1.0).

```liva
let rand = Math.random()
```
