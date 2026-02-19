# Error Handling Guide

This guide explains how Liva's enhanced error system helps you write better code faster.

## Overview

Liva provides developer-friendly error messages with:

- 🎯 **Precise Location**: Exact line and column numbers with context
- 💡 **Smart Suggestions**: "Did you mean?" for typos
- 📝 **Code Examples**: Shows correct vs incorrect usage
- 📚 **Documentation Links**: Direct links to relevant docs
- 🏷️ **Error Categories**: Organized by type (Parser, Semantic, etc.)

---

## Error Message Anatomy

```
● E0701: Fallible function must be called with error binding [Semantic]
────────────────────────────────────────────────────────────
  → examples/test.liva:7:16

     5 │ divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b
     6 │ 
     7 │
       │ let result = divide(10, 2)
       │              ^^^^^^
     8 │   
     9 │ print(result)
       │

  ⓘ Function 'divide' can fail but is not being called with error binding.

  💡 Use error binding: let result, err = fallibleFunc(...)

  📝 Example:
     // Correct:
     let result, err = divide(10, 2)
     if err == "" {
       print(result)
     }
     
     // Incorrect:
     divide(10, 2)

  📚 Learn more: https://liva-lang.org/docs/errors/semantic#e0701
────────────────────────────────────────────────────────────
```

### Components

1. **Header** (`● E0701: ...`)
   - Error code (E0701)
   - Error title
   - Category in brackets [Semantic]

2. **Location** (`→ examples/test.liva:7:16`)
   - File path
   - Line number
   - Column number

3. **Context** (Lines 5-9)
   - 2 lines before error
   - Error line with precise underline
   - 2 lines after error

4. **Message** (`ⓘ Function 'divide'...`)
   - Clear explanation of what went wrong

5. **Hint** (`💡 Use error binding...`)
   - Actionable advice on how to fix

6. **Example** (`📝 Example:`)
   - Code showing correct and incorrect patterns

7. **Documentation** (`📚 Learn more:`)
   - Link to detailed documentation

---

## Error Categories

Liva organizes errors into 8 categories:

### E0xxx: Semantic Errors
Issues with program logic and structure.

**Example:**
```liva
interface Drawable {
  draw()
}

class Circle : Drawable {
  // E0001: Missing draw() method
}
```

### E1xxx: Lexer Errors
Invalid characters or tokens.

**Example:**
```liva
let x = 5@  // E1000: Invalid character '@'
```

### E2xxx: Parser Errors
Syntax errors.

**Example:**
```liva
let x 5  // E2000: Missing '=' in assignment
```

### E3xxx: Code Generation Errors
Issues during Rust code generation.

### E4xxx: Module System Errors
Import and module resolution issues.

**Example:**
```liva
import { ad } from './math'  // E4006: Symbol not found
// 💡 Did you mean 'add'?
```

### E5xxx: Type System Errors
Type mismatches and incompatibilities.

### E6xxx: Concurrency Errors
Async and parallel execution issues.

**Example:**
```liva
for par item in items {
  await fetchData(item)  // E0605: Cannot await in parallel loop
}
```

### E7xxx: Error Handling Errors
Fallibility and error binding issues.

**Example:**
```liva
divide(10, 0)  // E0701: Must use error binding
```

---

## "Did You Mean?" Suggestions

Liva uses Levenshtein distance to suggest corrections for typos.

### Variable Names

```liva
let userName = "Alice"
usrName = "Bob"  // Did you mean 'userName'?
```

### Module Imports

```liva
import { ad } from './math'  // Did you mean 'add'?
```

### Type Names

```liva
class Cricle : Drawable  // Did you mean 'Circle'?
```

### How It Works

- Maximum edit distance: 2 characters
- Suggests most similar valid identifier
- Only suggests if similarity > 50%

---

## Smart Hints

Every error code has contextual hints:

### Fallible Functions (E0701)

```liva
// ❌ Error
divide(10, 2)

// ✅ Correct
let result, err = divide(10, 2)
if err == "" {
  print(result)
} else {
  print($"Error: {err}")
}
```

**Hint:** Use error binding: `let result, err = fallibleFunc(...)`

### Import Errors (E4006)

```liva
// ❌ Error
import { multiply } from './math'  // Symbol not found

// ✅ Correct
import { add, subtract } from './math'
```

**Hint:** Check the module's exports or look for typos in the symbol name

### Await Errors (E0603)

```liva
// ❌ Error
let result = await parFunc()  // par completes eagerly

// ✅ Correct
let result = await asyncFunc()
```

**Hint:** Only async and task async expressions can be awaited

---

## Code Examples

Many errors include code examples showing correct usage:

```
📝 Example:
   // Correct:
   let result, err = divide(10, 2)
   if err == "" {
     print(result)
   }
   
   // Incorrect:
   divide(10, 2)
```

---

## Documentation Links

Every error includes a link to detailed documentation:

```
📚 Learn more: https://liva-lang.org/docs/errors/semantic#e0701
```

Link format: `https://liva-lang.org/docs/errors/{category}#{error_code}`

Categories:
- `semantic` - E0xxx errors
- `lexer` - E1xxx errors
- `parser` - E2xxx errors
- `codegen` - E3xxx errors
- `modules` - E4xxx errors
- `types` - E5xxx errors
- `concurrency` - E6xxx errors
- `error-handling` - E7xxx errors

---

## Common Error Patterns

### 1. Missing Error Binding

**Problem:** Calling a fallible function without handling errors

```liva
// ❌ Error E0701
divide(10, 0)
```

**Solution:**

```liva
// ✅ Handle the error
let result, err = divide(10, 0)
if err != "" {
  print($"Error: {err}")
} else {
  print(result)
}

// ✅ Or ignore it explicitly
let result, _ = divide(10, 0)
```

### 2. Module Import Typos

**Problem:** Importing non-existent symbols

```liva
// ❌ Error E4006
import { ad } from './math'
```

**Solution:**

```liva
// ✅ Use correct symbol name
import { add } from './math'
```

### 3. Await in Parallel Context

**Problem:** Using await inside parallel loops

```liva
// ❌ Error E0605
for par item in items {
  let data = await fetch(item)
}
```

**Solution:**

```liva
// ✅ Use async loop instead
for async item in items {
  let data = await fetch(item)
}
```

### 4. Duplicate Execution Modifiers

**Problem:** Using multiple conflicting modifiers

```liva
// ❌ Error E0602
async async fetchData()
```

**Solution:**

```liva
// ✅ Use async at call site, not declaration
let result = async fetchData()
```

---

## Best Practices

### 1. Read the Full Error Message

Don't just look at the error code. The message contains:
- What went wrong
- Where it went wrong
- How to fix it
- Examples of correct usage

### 2. Use "Did You Mean?" Suggestions

If Liva suggests an alternative, it's usually right:

```liva
usrName = "Bob"
// Did you mean 'userName'? ← Usually correct!
```

### 3. Follow the Examples

Code examples show the exact pattern you should use:

```liva
📝 Example:
   // Correct:
   let result, err = divide(10, 2)  ← Copy this pattern
```

### 4. Check Documentation Links

For complex errors, click the documentation link for:
- Detailed explanations
- More examples
- Advanced usage patterns

### 5. Fix Errors Top-to-Bottom

Start with the first error. Later errors might be caused by earlier ones.

---

## Troubleshooting

### "Symbol not found" but it exists

**Possible causes:**
1. Typo in symbol name
2. Symbol not exported from module
3. Wrong module path

**Solutions:**
1. Check spelling (use "Did you mean?" suggestion)
2. Add `export` to the symbol in the module
3. Verify file path is correct

### "Cannot await" errors

**Possible causes:**
1. Trying to await a `par` call (they complete eagerly)
2. Trying to await in a parallel loop
3. Awaiting a non-async expression

**Solutions:**
1. Use `async` instead of `par` if you need to await
2. Use `for async` instead of `for par`
3. Only await async/task async expressions

### Parse errors without clear cause

**Common causes:**
1. Missing semicolons
2. Unclosed parentheses/braces
3. Typo in keyword
4. Extra comma or operator

**Solutions:**
1. Check the underlined token and surrounding code
2. Count opening/closing braces and parentheses
3. Verify all keywords are spelled correctly

---

## IDE Integration

Liva errors are designed for IDE integration:

### JSON Format

Errors can be exported as JSON for tool integration:

```json
{
  "location": {
    "file": "test.liva",
    "line": 7,
    "column": 16,
    "source_line": "  let result = divide(10, 2)",
    "length": 6
  },
  "code": "E0701",
  "title": "Fallible function must be called with error binding",
  "message": "Function 'divide' can fail...",
  "help": "Use error binding: let result, err = divide(...)"
}
```

### VS Code Extension

The Liva VS Code extension shows:
- Real-time error highlighting
- Inline error messages
- Quick fix suggestions
- Hover documentation

---

## Contributing

### Adding New Error Codes

1. Choose appropriate category (E0xxx-E7xxx)
2. Add constant to `src/error_codes.rs`
3. Add hint to `src/hints.rs`
4. Add example if applicable
5. Document in `docs/ERROR_CODES.md`
6. Add test cases

### Adding Hints

```rust
// In src/hints.rs

pub fn get_hint(error_code: &str) -> Option<&'static str> {
    match error_code {
        "E1234" => Some("Your helpful hint here"),
        _ => None,
    }
}
```

### Adding Examples

```rust
pub fn get_example(error_code: &str) -> Option<&'static str> {
    match error_code {
        "E1234" => Some(
            "// Correct:\ncorrect_code()\n\n// Incorrect:\nwrong_code()"
        ),
        _ => None,
    }
}
```

---

## References

- [Error Codes Reference](ERROR_CODES.md) - Complete list of all error codes
- [Language Reference](../language-reference/) - Liva language documentation
- [Error Handling](../language-reference/error-handling.md) - Fallibility system
- [VS Code Extension](../../vscode-extension/) - IDE integration

---

## Feedback

Found an error message confusing? Let us know!

- GitHub Issues: https://github.com/liva-lang/liva/issues
- Discord: https://discord.gg/liva
- Email: support@liva-lang.org

We're always improving error messages based on user feedback.
