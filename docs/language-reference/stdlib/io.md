# I/O Functions

> **Status:** 🚧 Planned  
> **Version:** v0.7.0 (upcoming)

Input/output functions for Liva.

---

## 📋 Planned Functions

### Console I/O
- `print(value: any) => void` - Print to stdout (already implemented)
- `readLine() => string` - Read line from stdin

### File I/O (Future)
- `readFile(path: string) => string` - Read file contents
- `writeFile(path: string, content: string) => void` - Write to file

---

## 🔮 Future Examples

```liva
// Print (already works)
print("Hello, World!")

// Read input
let name = readLine()
print($"Hello, {name}!")

// File I/O (future)
let content = readFile("data.txt")
print(content)

writeFile("output.txt", "Hello from Liva!")
```

---

## 📝 See Also

- [Standard Library Overview](./README.md)
