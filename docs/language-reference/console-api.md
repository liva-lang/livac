# Console API

> SKILL.md covers: `print()`, `console.log()`, `console.error()`, `console.warn()`, `console.input()`.
> This file: `console.success`, formatting behavior differences, color output details.

## `console.success(...)`

Prints to stdout in **green** (ANSI). Variadic like the other console methods.

```liva
console.success("User created successfully!")
console.success("All tests passed:", testCount, "tests")
```

## Formatting Behavior Differences

| Function | Stream | Color | Formatting | Separator |
|----------|--------|-------|-----------|-----------|
| `print(x)` | stdout | none | Display (`{}`) | — (single arg) |
| `console.log(a, b, ...)` | stdout | none | Debug (`{:?}`) | space |
| `console.error(a, b, ...)` | **stderr** | **red** | Display (`{}`) | space |
| `console.warn(a, b, ...)` | **stderr** | **yellow** | Display (`{}`) | space |
| `console.success(a, b, ...)` | stdout | **green** | Display (`{}`) | space |

Key differences:
- `print()` takes a single argument, uses Display formatting
- `console.log()` uses **Debug** formatting (`{:?}`) — shows quotes around strings, array brackets
- `console.error/warn` go to **stderr** — not captured by stdout redirection
- All `console.*` methods accept **variadic arguments** separated by spaces
- Colors use ANSI escape codes, auto-reset after message

## `console.input()` Details

```liva
let name = console.input("Name: ")     // With prompt
let line = console.input()              // No prompt (silent read)
```

- Returns `string` — always trimmed (strips `\n`, spaces, tabs)
- Blocking — waits for Enter
- Prompt flushes stdout automatically
- For numeric input, pipe through `parseInt()` / `parseFloat()`:

```liva
let age, err = parseInt(console.input("Age: "))
if err { console.error("Invalid number") }
```
