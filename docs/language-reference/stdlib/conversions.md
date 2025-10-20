# Type Conversions

> **Status:** 🚧 Planned  
> **Version:** v0.7.0 (upcoming)

Type conversion functions for Liva.

---

## 📋 Planned Functions

### String Conversions
- `parseInt(s: string) => i32` - Parse string to integer
- `parseFloat(s: string) => f64` - Parse string to float
- `toString(n: i32|f64) => string` - Convert number to string

---

## 🔮 Future Examples

```liva
// Parse integer
let num = parseInt("42")
print(num)  // 42

// Parse float
let pi = parseFloat("3.14159")
print(pi)  // 3.14159

// To string
let str = toString(123)
print(str)  // "123"
```

---

## 📝 See Also

- [Math Functions](./math.md)
- [Standard Library Overview](./README.md)
