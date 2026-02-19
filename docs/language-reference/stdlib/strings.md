# String Methods

> **Status:** ✅ Complete (12/12 methods)  
> **Version:** v1.3.0

String methods in Liva provide operations for string manipulation, transformation, and queries.

---

## 📋 Table of Contents

- [Manipulation Methods](#manipulation-methods)
  - [split()](#split)
  - [replace()](#replace)
- [Transformation Methods](#transformation-methods)
  - [toUpperCase()](#touppercase)
  - [toLowerCase()](#tolowercase)
  - [trim()](#trim)
  - [trimStart()](#trimstart)
  - [trimEnd()](#trimend)
- [Query Methods](#query-methods)
  - [startsWith()](#startswith)
  - [endsWith()](#endswith)
  - [contains()](#contains)
  - [indexOf()](#indexof)
- [Access Methods](#access-methods)
  - [substring()](#substring)
  - [charAt()](#charat)

---

## Manipulation Methods

### `split()`

Split a string into an array of substrings using a delimiter.

**Signature:**
```liva
split(delimiter: string) => Array<string>
```

**Examples:**
```liva
// Split by comma
let csv = "apple,banana,orange"
let fruits = csv.split(",")
print(fruits)  // ["apple", "banana", "orange"]

// Split by space
let sentence = "Hello World from Liva"
let words = sentence.split(" ")
print(words)  // ["Hello", "World", "from", "Liva"]

// Split by newline
let text = "line1\nline2\nline3"
let lines = text.split("\n")
print(lines)  // ["line1", "line2", "line3"]
```

**Rust Codegen:**
```rust
csv.split(",").map(|s| s.to_string()).collect::<Vec<String>>()
```

**Notes:**
- Returns `Vec<String>` for Liva array compatibility
- Empty strings are included in result
- If delimiter not found, returns array with original string
- Delimiter is consumed (not included in results)

---

### `replace()`

Replace all occurrences of a substring with another string.

**Signature:**
```liva
replace(search: string, replacement: string) => string
```

**Examples:**
```liva
// Replace word
let text = "hello world"
let newText = text.replace("world", "Liva")
print(newText)  // "hello Liva"

// Replace character
let name = "John Doe"
let formatted = name.replace(" ", "_")
print(formatted)  // "John_Doe"

// Replace all occurrences
let repeated = "la la la"
let changed = repeated.replace("la", "ha")
print(changed)  // "ha ha ha"
```

**Rust Codegen:**
```rust
text.replace("world", "Liva")
```

**Notes:**
- Replaces **all** occurrences (not just first)
- Case-sensitive matching
- Returns new string; original unchanged
- If search string not found, returns original

---

## Transformation Methods

### `toUpperCase()`

Convert all characters to uppercase.

**Signature:**
```liva
toUpperCase() => string
```

**Examples:**
```liva
// Simple conversion
let text = "hello"
let upper = text.toUpperCase()
print(upper)  // "HELLO"

// Mixed case
let mixed = "Hello World"
let allCaps = mixed.toUpperCase()
print(allCaps)  // "HELLO WORLD"

// Already uppercase
let caps = "LIVA"
let same = caps.toUpperCase()
print(same)  // "LIVA"
```

**Rust Codegen:**
```rust
text.to_uppercase()
```

**Notes:**
- Unicode-aware (handles non-ASCII correctly)
- Returns new string; original unchanged
- Works with all Unicode characters

---

### `toLowerCase()`

Convert all characters to lowercase.

**Signature:**
```liva
toLowerCase() => string
```

**Examples:**
```liva
// Simple conversion
let text = "HELLO WORLD"
let lower = text.toLowerCase()
print(lower)  // "hello world"

// Mixed case
let mixed = "HeLLo WoRLd"
let normalized = mixed.toLowerCase()
print(normalized)  // "hello world"

// Already lowercase
let lower = "liva"
let same = lower.toLowerCase()
print(same)  // "liva"
```

**Rust Codegen:**
```rust
text.to_lowercase()
```

**Notes:**
- Unicode-aware (handles non-ASCII correctly)
- Returns new string; original unchanged
- Useful for case-insensitive comparisons

---

### `trim()`

Remove leading and trailing whitespace.

**Signature:**
```liva
trim() => string
```

**Examples:**
```liva
// Both sides
let text = "   hello   "
let trimmed = text.trim()
print(trimmed)  // "hello"

// Leading only
let leading = "   hello"
let result = leading.trim()
print(result)  // "hello"

// Trailing only
let trailing = "hello   "
let result = trailing.trim()
print(result)  // "hello"

// Internal spaces preserved
let internal = "  hello  world  "
let result = internal.trim()
print(result)  // "hello  world"
```

**Rust Codegen:**
```rust
text.trim()
```

**Notes:**
- Removes spaces, tabs, newlines, and other Unicode whitespace
- Internal whitespace is preserved
- Returns new string; original unchanged

---

### `trimStart()`

Remove leading whitespace only.

**Signature:**
```liva
trimStart() => string
```

**Examples:**
```liva
// Leading spaces
let text = "   hello"
let trimmed = text.trimStart()
print(trimmed)  // "hello"

// Trailing preserved
let both = "   hello   "
let result = both.trimStart()
print(result)  // "hello   "

// No leading whitespace
let clean = "hello"
let same = clean.trimStart()
print(same)  // "hello"
```

**Rust Codegen:**
```rust
text.trim_start()
```

**Notes:**
- Also known as `trimLeft()` in some languages
- Only removes from beginning
- Returns new string; original unchanged

---

### `trimEnd()`

Remove trailing whitespace only.

**Signature:**
```liva
trimEnd() => string
```

**Examples:**
```liva
// Trailing spaces
let text = "hello   "
let trimmed = text.trimEnd()
print(trimmed)  // "hello"

// Leading preserved
let both = "   hello   "
let result = both.trimEnd()
print(result)  // "   hello"

// No trailing whitespace
let clean = "hello"
let same = clean.trimEnd()
print(same)  // "hello"
```

**Rust Codegen:**
```rust
text.trim_end()
```

**Notes:**
- Also known as `trimRight()` in some languages
- Only removes from end
- Returns new string; original unchanged

---

## Query Methods

### `startsWith()`

Check if string starts with a given prefix.

**Signature:**
```liva
startsWith(prefix: string) => bool
```

**Examples:**
```liva
// File extension check
let filename = "document.pdf"
let isPdf = filename.startsWith("document")
print(isPdf)  // true

// URL protocol check
let url = "https://example.com"
let isHttps = url.startsWith("https://")
print(isHttps)  // true

// Case-sensitive
let text = "Hello"
let starts = text.startsWith("hello")
print(starts)  // false
```

**Rust Codegen:**
```rust
filename.starts_with("document")
```

**Notes:**
- Case-sensitive matching
- Returns boolean (`true` or `false`)
- Empty prefix always returns `true`

---

### `endsWith()`

Check if string ends with a given suffix.

**Signature:**
```liva
endsWith(suffix: string) => bool
```

**Examples:**
```liva
// File extension check
let filename = "document.pdf"
let isPdf = filename.endsWith(".pdf")
print(isPdf)  // true

let isDoc = filename.endsWith(".doc")
print(isDoc)  // false

// Domain check
let email = "user@example.com"
let isExampleDomain = email.endsWith("@example.com")
print(isExampleDomain)  // true

// Case-sensitive
let text = "Hello"
let ends = text.endsWith("LO")
print(ends)  // false
```

**Rust Codegen:**
```rust
filename.ends_with(".pdf")
```

**Notes:**
- Case-sensitive matching
- Returns boolean (`true` or `false`)
- Empty suffix always returns `true`

---

### `contains()`

Check if a string contains a given substring.

**Signature:**
```liva
contains(substring: string) => bool
```

**Examples:**
```liva
let text = "Hello, World!"

// Check if contains
let hasWorld = text.contains("World")
print(hasWorld)  // true

let hasFoo = text.contains("Foo")
print(hasFoo)  // false

// Case-sensitive
let hasHello = text.contains("hello")
print(hasHello)  // false

// Search in file content
let content, err = File.read("data.txt")
if !err {
    if content.contains("ERROR") {
        print("Found errors in log")
    }
}
```

**Rust Codegen:**
```rust
text.contains("World")
```

**Notes:**
- Case-sensitive matching
- Returns boolean (`true` or `false`)
- Empty substring always returns `true`
- Similar to `indexOf()` but returns bool instead of position

---

### `indexOf()`

Find the position of a substring within the string.

**Signature:**
```liva
indexOf(substring: string) => i32
```

**Examples:**
```liva
let text = "The quick brown fox jumps over the lazy dog"

// Find "quick"
let idx1 = text.indexOf("quick")
print(idx1)  // 4

// Find "fox"
let idx2 = text.indexOf("fox")
print(idx2)  // 16

// Find "the" (lowercase)
let idx3 = text.indexOf("the")
print(idx3)  // 31 (second occurrence)

// Not found
let idx4 = text.indexOf("cat")
print(idx4)  // -1

// First character
let greeting = "Hello"
let h = greeting.indexOf("H")
print(h)  // 0
```

**Rust Codegen:**
```rust
text.find("quick").map(|i| i as i32).unwrap_or(-1)
```

**Notes:**
- Returns `-1` if substring not found (JavaScript convention)
- Returns index of **first** occurrence
- Case-sensitive matching
- Zero-indexed (first character is 0)
- Different from [array indexOf](./arrays.md#indexof) which searches for values

---

## Access Methods

### `substring()`

Extract a substring from start index to end index.

**Signature:**
```liva
substring(start: i32, end: i32) => string
```

**Examples:**
```liva
let text = "Hello World"

// Extract "Hello"
let greeting = text.substring(0, 5)
print(greeting)  // "Hello"

// Extract "World"
let word = text.substring(6, 11)
print(word)  // "World"

// From middle
let middle = text.substring(3, 8)
print(middle)  // "lo Wo"

// Single character
let char = text.substring(0, 1)
print(char)  // "H"
```

**Rust Codegen:**
```rust
text[start as usize..end as usize].to_string()
```

**Notes:**
- Uses slice syntax internally
- `start` is inclusive, `end` is exclusive
- Zero-indexed
- Panics if indices out of bounds
- For single character access, use `charAt()`

---

### `charAt()`

Get the character at a specific index.

**Signature:**
```liva
charAt(index: i32) => char
```

**Examples:**
```liva
let text = "Hello World"

// First character
let first = text.charAt(0)
print(first)  // 'H'

// Last character
let last = text.charAt(10)
print(last)  // 'd'

// Middle character
let space = text.charAt(5)
print(space)  // ' '

// Out of bounds returns space
let invalid = text.charAt(100)
print(invalid)  // ' '
```

**Rust Codegen:**
```rust
text.chars().nth(index as usize).unwrap_or(' ')
```

**Notes:**
- Returns single `char` (not string)
- UTF-8 safe (handles multi-byte characters correctly)
- Out of bounds returns space `' '`
- Zero-indexed
- For substrings, use `substring()`

---

## 🎯 Method Chaining

String methods can be chained together:

```liva
let input = "  Hello, World!  "

// Chain multiple operations
let result = input
  .trim()              // "Hello, World!"
  .toLowerCase()       // "hello, world!"
  .replace(", ", "_")  // "hello_world!"

print(result)  // "hello_world!"

// Split and process
let csv = "apple,banana,orange"
let processed = csv
  .toUpperCase()       // "APPLE,BANANA,ORANGE"
  .split(",")          // ["APPLE", "BANANA", "ORANGE"]

print(processed)  // ["APPLE", "BANANA", "ORANGE"]
```

---

## 🚀 Performance Notes

### Direct Mapping
String methods map directly to Rust standard library:
- No iterator overhead
- Minimal allocations
- Compiler optimizations apply

### Memory
- Methods returning strings create new allocations
- Original strings are never modified (immutable)
- Query methods (`startsWith`, `endsWith`, `indexOf`) don't allocate

### UTF-8 Safety
- All methods handle Unicode correctly
- `charAt()` uses `.chars().nth()` for multi-byte safety
- `substring()` uses byte slicing (assumes valid UTF-8)

---

## 📝 Common Patterns

### Case-Insensitive Comparison
```liva
let str1 = "Hello"
let str2 = "HELLO"
let equal = str1.toLowerCase() == str2.toLowerCase()
print(equal)  // true
```

### Parsing CSV
```liva
let csv = "name,age,city"
let headers = csv.split(",")
// ["name", "age", "city"]
```

### Trimming User Input
```liva
let input = "  john@example.com  "
let email = input.trim().toLowerCase()
// "john@example.com"
```

### File Extension Check
```liva
let filename = "document.pdf"
if filename.endsWith(".pdf") {
  print("PDF file")
}
```

### String Sanitization
```liva
let unsafe = "Hello <script>alert('xss')</script>"
let safe = unsafe
  .replace("<", "&lt;")
  .replace(">", "&gt;")
// "Hello &lt;script&gt;alert('xss')&lt;/script&gt;"
```

---

## 🔮 Future Enhancements

Planned string methods for future versions:

```liva
// String interpolation (already supported via templates)
let name = "Liva"
let greeting = $"Hello, {name}!"

// Repeat (future)
let repeated = "ha".repeat(3)  // "hahaha"

// Pad (future)
let padded = "5".padStart(3, "0")  // "005"

// Join on string arrays (future)
let joined = ["a", "b", "c"].join(",")  // "a,b,c"
```

---

## 📝 See Also

- [Array Methods](./arrays.md)
- [Standard Library Overview](./README.md)
- [String Templates](../string-templates.md)
- [Language Reference](../README.md)
