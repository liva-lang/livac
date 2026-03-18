# Error Troubleshooting Guide

Quick reference for fixing common Liva compiler errors.

## Quick Index

- [Parse Errors (E2xxx)](#parse-errors-e2xxx)
- [Module Errors (E4xxx)](#module-errors-e4xxx)
- [Concurrency Errors (E6xxx)](#concurrency-errors-e6xxx)
- [Error Handling (E7xxx)](#error-handling-e7xxx)

---

## Parse Errors (E2xxx)

### E2000: Parse Error

**Common Causes:**
- Missing `=` in assignments
- Unclosed parentheses or braces
- Missing commas in lists
- Typo in keywords

**Example Error:**
```
● E2000: Parse Error [Parser]
────────────────────────────────────────────────────────────
  → test.liva:5:17

     3 │ main() {
     4 │   let x = 5
     5 │
       │ let resultado x + y
       │               ^
     6 │   print(x)
     7 │ }

  ⓘ Expected Assign

  💡 Check for missing semicolons, parentheses, or keywords

  📚 Learn more: https://liva-lang.org/docs/errors/parser#e2000
```

**Solutions:**

1. **Missing `=` in assignment:**
   ```liva
   // ❌ Wrong
   let resultado x + y
   
   // ✅ Correct
   let resultado = x + y
   ```

2. **Unclosed braces:**
   ```liva
   // ❌ Wrong
   if x > 5 {
     print(x)
   // Missing }
   
   // ✅ Correct
   if x > 5 {
     print(x)
   }
   ```

3. **Missing comma:**
   ```liva
   // ❌ Wrong
   let arr = [1, 2 3, 4]
   
   // ✅ Correct
   let arr = [1, 2, 3, 4]
   ```

---

## Module Errors (E4xxx)

### E4004: Module Not Found

**Common Causes:**
- Wrong file path
- Missing `.liva` extension in filename
- File doesn't exist
- Typo in path

**Example Error:**
```
● E4004: Module Not Found [Module System]
────────────────────────────────────────────────────────────

  ⓘ Module file './maths.liva' not found

  💡 Make sure the file exists and the path is correct

  📚 Learn more: https://liva-lang.org/docs/errors/modules#e4004
```

**Solutions:**

1. **Check file exists:**
   ```bash
   ls -la math.liva  # Verify file is there
   ```

2. **Use correct relative path:**
   ```liva
   // ❌ Wrong (missing ./)
   import { add } from 'math'
   
   // ✅ Correct
   import { add } from './math'
   ```

3. **Check filename spelling:**
   ```liva
   // ❌ Wrong (typo: maths vs math)
   import { add } from './maths'
   
   // ✅ Correct
   import { add } from './math'
   ```

### E4006: Symbol Not Found

**Common Causes:**
- Typo in symbol name
- Symbol not exported from module
- Wrong module

**Example Error:**
```
● E4006: Imported symbol not found [Module System]
────────────────────────────────────────────────────────────

  ⓘ Symbol 'ad' not found in module './math.liva'

💡 Did you mean 'add'?

  💡 Check the module's exports or look for typos

  📚 Learn more: https://liva-lang.org/docs/errors/modules#e4006
```

**Solutions:**

1. **Use "Did you mean?" suggestion:**
   ```liva
   // ❌ Wrong
   import { ad } from './math'
   
   // ✅ Correct (use suggestion)
   import { add } from './math'
   ```

2. **Check module exports:**
   ```liva
   // In math.liva
   add(a, b) => a + b
   subtract(a, b) => a - b
   
   // ❌ Wrong - multiply is not defined
   import { multiply } from './math'
   
   // ✅ Correct - only import what exists
   import { add, subtract } from './math'
   ```

### E4007: Invalid Import Syntax

**Common Causes:**
- Missing curly braces
- Wrong import format
- Missing `from` keyword

**Solutions:**

```liva
// ❌ Wrong - missing braces
import add from './math'

// ❌ Wrong - missing 'from'
import { add } './math'

// ✅ Correct
import { add } from './math'

// ✅ Correct - multiple imports
import { add, subtract, multiply } from './math'
```

---

## Concurrency Errors (E6xxx)

### E0603: Not Awaitable

**Common Causes:**
- Trying to await `par` expressions
- Awaiting literals
- Awaiting non-async calls

**Example Error:**
```
● E0603: Not Awaitable [Concurrency]
────────────────────────────────────────────────────────────

  ⓘ par calls complete eagerly and cannot be awaited

  💡 Only async and task async expressions can be awaited

  📝 Example:
     // Correct:
     let result = await asyncFunc()
     
     // Incorrect:
     let result = await parFunc()

  📚 Learn more: https://liva-lang.org/docs/errors/concurrency#e0603
```

**Solutions:**

1. **Use async instead of par:**
   ```liva
   // ❌ Wrong - par completes eagerly
   let result = await par heavyWork()
   
   // ✅ Correct - async can be awaited
   let result = await async heavyWork()
   ```

2. **Don't await par, just use it:**
   ```liva
   // ❌ Wrong
   let result = await par compute(100)
   
   // ✅ Correct - par completes immediately
   let result = par compute(100)
   print(result)  // Already computed
   ```

### E0604: Await Multiple Times

**Common Causes:**
- Awaiting same expression twice
- Awaiting same handle multiple times

**Solutions:**

```liva
// ❌ Wrong - await twice
let handle = task async fetchData()
let result1 = await handle
let result2 = await handle  // Error!

// ✅ Correct - await once, store result
let handle = task async fetchData()
let result = await handle
let result2 = result  // Use stored value
```

### E0605: Await in Parallel Loop

**Common Causes:**
- Using `await` inside `for par` or `for parvec`
- Confusion between parallel and async iteration

**Example Error:**
```
● E0605: Await in Parallel Loop [Concurrency]
────────────────────────────────────────────────────────────

  ⓘ await is not allowed inside for par or for parvec loops

  💡 Parallel loops execute synchronously. Use 'for async'

  📝 Example:
     // Correct:
     for async item in items {
       let data = await fetchData(item)
     }
     
     // Incorrect:
     for par item in items {
       await fetchData(item)
     }

  📚 Learn more: https://liva-lang.org/docs/errors/concurrency#e0605
```

**Solutions:**

```liva
// ❌ Wrong - await in parallel loop
for par item in urls {
  let data = await fetch(item)
  process(data)
}

// ✅ Correct - use async loop
for async item in urls {
  let data = await fetch(item)
  process(data)
}

// ✅ Or use par without await
for par item in numbers {
  let result = compute(item)  // CPU-bound, no await
  results.push(result)
}
```

---

## Error Handling (E7xxx)

### E0701: Fallible Without Binding

**Common Causes:**
- Calling function with `fail` without error binding
- Forgetting to handle errors
- Not aware function is fallible

**Example Error:**
```
● E0701: Fallible function must be called with error binding [Semantic]
────────────────────────────────────────────────────────────
  → test.liva:8:16

     6 │ divide(a, b) => b == 0 ? fail "Division by zero" : a / b
     7 │ 
     8 │
       │ let result = divide(10, 0)
       │              ^^^^^^
     9 │   
    10 │ print(result)

  ⓘ Function 'divide' can fail but is not being called with error binding

  💡 Change to: let result, err = divide(...)

  📝 Example:
     // Correct:
     let result, err = divide(10, 2)
     if err == "" {
       print(result)
     }
     
     // Incorrect:
     divide(10, 2)

  📚 Learn more: https://liva-lang.org/docs/errors/semantic#e0701
```

**Solutions:**

1. **Use error binding:**
   ```liva
   // ❌ Wrong - no error handling
   let result = divide(10, 0)
   
   // ✅ Correct - handle error
   let result, err = divide(10, 0)
   if err != "" {
     print($"Error: {err}")
   } else {
     print(result)
   }
   ```

2. **Ignore error explicitly:**
   ```liva
   // If you're sure it won't fail
   let result, _ = divide(10, 2)
   ```

3. **Check error first:**
   ```liva
   let result, err = divide(10, 0)
   if err == "" {
     // Safe to use result
     print(result)
   }
   ```

### E0702-E0706: Loop Options

**Common Issues:**
- Invalid chunk size
- Invalid thread count
- SIMD without vectorization

**Solutions:**

```liva
// ❌ E0702 - chunk must be positive
for par x in data with chunk 0 { }

// ✅ Correct
for par x in data with chunk 100 { }

// ❌ E0704 - threads must be positive
for par x in data with threads -1 { }

// ✅ Correct
for par x in data with threads 4 { }

// ❌ E0705 - simdWidth needs vec/parvec
for par x in data with simdWidth 8 { }

// ✅ Correct
for parvec x in data with simdWidth 8 { }
```

---

## General Debugging Tips

### 1. Read the Context

The error shows 2 lines before and after. Use this context:

```
     5 │ let userName = "Alice"
     6 │ let userAge = 25
     7 │
       │ usrName = "Bob"  ← Error here
       │ ^^^^^^^
     8 │   
     9 │ print(userName)
```

Looking at line 5, you can see `userName` was declared, so `usrName` on line 7 is likely a typo.

### 2. Check "Did You Mean?" Suggestions

These are usually correct:

```
💡 Did you mean 'userName'?
```

95% of the time, this is the fix you need.

### 3. Use the Examples

Copy-paste the correct pattern from examples:

```
📝 Example:
   // Correct:
   let result, err = divide(10, 2)  ← Use this pattern
```

### 4. Follow Documentation Links

For complex errors, click the link:

```
📚 Learn more: https://liva-lang.org/docs/errors/semantic#e0701
```

### 5. Fix One Error at a Time

Don't try to fix all errors at once. Fix the first one, recompile, then move to the next.

---

## Getting Help

If you're stuck:

1. **Read the full error message** - It contains the answer 90% of the time
2. **Check the examples** - Shows exactly what to do
3. **Search error code** - Google "Liva E0701" or check docs
4. **Ask on Discord** - Community can help: https://discord.gg/liva
5. **File an issue** - If error message is unclear: https://github.com/liva-lang/liva/issues

---

## Contributing

Help improve these error messages!

- Found a confusing error? Report it!
- Have a better hint? Submit a PR!
- Missing example? Add one!

See [ERROR_HANDLING_GUIDE.md](ERROR_HANDLING_GUIDE.md#contributing) for details.
