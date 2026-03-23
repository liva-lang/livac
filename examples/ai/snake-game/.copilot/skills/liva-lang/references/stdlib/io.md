# Console / IO Functions

> **6 functions** | v0.7.0  
> Hybrid approach: `print()` for simplicity + `console.*` for power

---

## Design: print() vs console.*

| | `print()` | `console.log()` | `console.error()` / `console.warn()` |
|--|-----------|-----------------|--------------------------------------|
| **Format** | Display `{}` | Debug `{:?}` | Display `{}` |
| **Strings** | Hello | "Hello" (with quotes) | Hello |
| **Stream** | stdout | stdout | stderr |
| **Use case** | User output | Dev/debugging | Error/warning messages |

---

## Output

### print(...args) => void
  print("Hello, World!")                     // Hello, World!
  print($"Hello, {name}!")                   // Hello, Alice!
  print($"X = {x}, Y = {y}")                // X = 42, Y = 3.14
  — Display format ({}). Clean, user-facing output.
  — Generates: println!("{}", ...)

### console.log(...args) => void
  console.log("Hello!")                      // "Hello!"  (with quotes)
  console.log([1, 2, 3])                     // [1, 2, 3]
  console.log({name: "Alice", age: 25})      // { name: "Alice", age: 25 }
  — Debug format ({:?}). Shows internal structure.
  — Generates: println!("{:?}", ...)

### console.error(...args) => void
  console.error("File not found!")           // → stderr: File not found!
  console.error($"Error {code}: Not found")  // → stderr: Error 404: Not found
  — Output goes to stderr (separate from stdout)
  — Generates: eprintln!("{:?}", ...)

### console.warn(...args) => void
  console.warn("Deprecated API")             // → stderr: Warning: Deprecated API
  console.warn($"Low disk: {pct}%")          // → stderr: Warning: Low disk: 85%
  — Adds "Warning: " prefix automatically
  — Output goes to stderr
  — Generates: eprintln!("Warning: {:?}", ...)

---

## Input

### console.readLine() => string
  console.log("What is your name?")
  let name = console.readLine()              // blocks, reads line from stdin
  — Returns trimmed string
  — Blocks until user presses Enter

### console.prompt(message: string) => string
  let name = console.prompt("Enter name: ") // displays prompt, reads input
  let age = console.prompt("Age: ")          // same line input
  — Combines output + input in one call
  — Equivalent to: print(message) + console.readLine()

---

## Common Patterns

```liva
// Interactive input
let name = console.prompt("Name: ")
let ageStr = console.prompt("Age: ")

// Error handling with proper streams
let result, err = doSomething()
if err {
    console.error($"Failed: {err}")    // → stderr
    return
}
print($"Success: {result}")            // → stdout

// Debug vs production output
print("Processing data...")             // user sees this
console.log(data)                       // developer sees structure
```
