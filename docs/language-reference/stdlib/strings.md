# String Methods

> **Status:** ✅ Complete (28 methods)  
> **Version:** v1.4.0

String methods in Liva provide operations for string manipulation, transformation, and queries.

---

## 📋 Table of Contents

- [Manipulation Methods](#manipulation-methods)
  - [split()](#split)
  - [replace()](#replace)
  - [replaceAll()](#replaceall)
- [Transformation Methods](#transformation-methods)
  - [toUpperCase()](#touppercase)
  - [toLowerCase()](#tolowercase)
  - [trim()](#trim)
  - [trimStart()](#trimstart)
  - [trimEnd()](#trimend)
  - [capitalize()](#capitalize)
  - [reverse()](#reverse)
  - [truncate()](#truncate)
  - [padStart()](#padstart)
  - [padEnd()](#padend)
  - [repeat()](#repeat)
  - [removePrefix()](#removeprefix)
  - [removeSuffix()](#removesuffix)
- [Query Methods](#query-methods)
  - [startsWith()](#startswith)
  - [endsWith()](#endswith)
  - [contains()](#contains)
  - [indexOf()](#indexof)
  - [lastIndexOf()](#lastindexof)
  - [isBlank()](#isblank)
  - [isEmpty()](#isempty)
  - [countMatches()](#countmatches)
- [Access Methods](#access-methods)
  - [substring()](#substring)
  - [charAt()](#charat)
  - [slice()](#slice)
  - [chars()](#chars)

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

### `replaceAll()` *(v1.4.0)*

Replace all occurrences of a substring (alias for `replace()` for JavaScript compatibility).

**Signature:**
```liva
replaceAll(search: string, replacement: string) => string
```

**Examples:**
```liva
let text = "hello world hello"
let result = text.replaceAll("hello", "hi")
print(result)  // "hi world hi"
```

**Rust Codegen:**
```rust
text.replace("hello", "hi")
```

**Notes:**
- Functionally identical to `replace()` — both replace all occurrences
- Provided for familiarity with JavaScript's `replaceAll()`

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

### `capitalize()` *(v1.4.0)*

Capitalize the first character of the string.

**Signature:**
```liva
capitalize() => string
```

**Examples:**
```liva
let text = "hello world"
let result = text.capitalize()
print(result)  // "Hello world"

let already = "Hello"
print(already.capitalize())  // "Hello"

let empty = ""
print(empty.capitalize())  // ""
```

**Rust Codegen:**
```rust
{
    let s = text.to_string();
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
```

**Notes:**
- Only capitalizes the **first** character
- Does not modify the rest of the string
- Returns empty string for empty input

---

### `reverse()` *(v1.4.0)*

Reverse the characters in the string.

**Signature:**
```liva
reverse() => string
```

**Examples:**
```liva
let text = "hello"
let result = text.reverse()
print(result)  // "olleh"

let palindrome = "racecar"
print(palindrome == palindrome.reverse())  // true
```

**Rust Codegen:**
```rust
text.chars().rev().collect::<String>()
```

**Notes:**
- Reverses by Unicode characters, not bytes
- Returns new string; original unchanged

---

### `truncate()` *(v1.4.0)*

Truncate the string to a maximum number of characters.

**Signature:**
```liva
truncate(maxLength: int) => string
```

**Examples:**
```liva
let text = "Hello, World!"
let short = text.truncate(5)
print(short)  // "Hello"

// Already shorter than max
let ok = "Hi".truncate(10)
print(ok)  // "Hi"
```

**Rust Codegen:**
```rust
text.chars().take(max_length as usize).collect::<String>()
```

**Notes:**
- UTF-8 safe (uses `.chars().take()`, not byte slicing)
- If string is shorter than maxLength, returns original unchanged
- Returns new string; original unchanged

---

### `padStart()` *(v1.4.0)*

Pad the start of the string to reach a target length.

**Signature:**
```liva
padStart(targetLength: int, padChar?: string) => string
```

**Examples:**
```liva
let num = "5"
let padded = num.padStart(3, "0")
print(padded)  // "005"

// Default pad with spaces
let text = "hi"
let result = text.padStart(5)
print(result)  // "   hi"

// Already long enough
let long = "hello"
print(long.padStart(3, "x"))  // "hello"
```

**Rust Codegen:**
```rust
{
    let s = num.to_string();
    let target = 3 as usize;
    if s.len() >= target {
        s
    } else {
        format!("{}{}", "0".repeat(target - s.len()), s)
    }
}
```

**Notes:**
- If string already meets or exceeds target length, returns original
- Default pad character is space `" "` when not specified
- Only first character of padChar is used

---

### `padEnd()` *(v1.4.0)*

Pad the end of the string to reach a target length.

**Signature:**
```liva
padEnd(targetLength: int, padChar?: string) => string
```

**Examples:**
```liva
let text = "hi"
let padded = text.padEnd(5, ".")
print(padded)  // "hi..."

// Default pad with spaces
let result = "hi".padEnd(5)
print(result)  // "hi   "
```

**Rust Codegen:**
```rust
{
    let s = text.to_string();
    let target = 5 as usize;
    if s.len() >= target {
        s
    } else {
        format!("{}{}", s, ".".repeat(target - s.len()))
    }
}
```

**Notes:**
- If string already meets or exceeds target length, returns original
- Default pad character is space `" "` when not specified
- Only first character of padChar is used

---

### `repeat()` *(v1.4.0)*

Repeat the string a given number of times.

**Signature:**
```liva
repeat(count: int) => string
```

**Examples:**
```liva
let text = "ha"
let laugh = text.repeat(3)
print(laugh)  // "hahaha"

let separator = "-".repeat(20)
print(separator)  // "--------------------"
```

**Rust Codegen:**
```rust
text.repeat(3 as usize)
```

**Notes:**
- `count` of 0 returns empty string
- Returns new string; original unchanged

---

### `removePrefix()` *(v1.4.0)*

Remove a prefix from the string if present.

**Signature:**
```liva
removePrefix(prefix: string) => string
```

**Examples:**
```liva
let path = "prefix_value"
let result = path.removePrefix("prefix_")
print(result)  // "value"

// Prefix not present — returns original
let text = "hello"
print(text.removePrefix("xyz"))  // "hello"
```

**Rust Codegen:**
```rust
match path.strip_prefix("prefix_") {
    Some(s) => s.to_string(),
    None => path.to_string(),
}
```

**Notes:**
- Returns original string if prefix not found
- Only removes prefix once (not recursive)

---

### `removeSuffix()` *(v1.4.0)*

Remove a suffix from the string if present.

**Signature:**
```liva
removeSuffix(suffix: string) => string
```

**Examples:**
```liva
let filename = "document.txt"
let name = filename.removeSuffix(".txt")
print(name)  // "document"

// Suffix not present — returns original
let text = "hello"
print(text.removeSuffix(".pdf"))  // "hello"
```

**Rust Codegen:**
```rust
match filename.strip_suffix(".txt") {
    Some(s) => s.to_string(),
    None => filename.to_string(),
}
```

**Notes:**
- Returns original string if suffix not found
- Only removes suffix once (not recursive)

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

### `lastIndexOf()` *(v1.4.0)*

Find the position of the **last** occurrence of a substring.

**Signature:**
```liva
lastIndexOf(substring: string) => i32
```

**Examples:**
```liva
let text = "hello world hello"
let idx = text.lastIndexOf("hello")
print(idx)  // 12

let notFound = text.lastIndexOf("xyz")
print(notFound)  // -1
```

**Rust Codegen:**
```rust
text.rfind("hello").map(|i| i as i32).unwrap_or(-1)
```

**Notes:**
- Returns `-1` if substring not found
- Returns index of **last** occurrence (searches from end)
- Case-sensitive matching

---

### `isBlank()` *(v1.4.0)*

Check if the string is empty or contains only whitespace.

**Signature:**
```liva
isBlank() => bool
```

**Examples:**
```liva
let blank = "   "
print(blank.isBlank())  // true

let empty = ""
print(empty.isBlank())  // true

let text = "hello"
print(text.isBlank())  // false

let mixed = " a "
print(mixed.isBlank())  // false
```

**Rust Codegen:**
```rust
text.trim().is_empty()
```

**Notes:**
- Returns `true` for empty strings AND whitespace-only strings
- Considers spaces, tabs, newlines as whitespace
- Different from `isEmpty()` which only checks for zero length

---

### `isEmpty()` *(v1.4.0)*

Check if the string has zero length.

**Signature:**
```liva
isEmpty() => bool
```

**Examples:**
```liva
let empty = ""
print(empty.isEmpty())  // true

let spaces = "   "
print(spaces.isEmpty())  // false (spaces count!)

let text = "hello"
print(text.isEmpty())  // false
```

**Rust Codegen:**
```rust
text.is_empty()
```

**Notes:**
- Only checks for zero length
- Whitespace-only strings are NOT empty (use `isBlank()` for that)
- Also works on arrays: `[].isEmpty()` → `true`

---

### `countMatches()` *(v1.4.0)*

Count how many times a substring appears in the string.

**Signature:**
```liva
countMatches(substring: string) => int
```

**Examples:**
```liva
let text = "banana"
let count = text.countMatches("an")
print(count)  // 2

let csv = "a,b,c,d"
let commas = csv.countMatches(",")
print(commas)  // 3

let none = "hello".countMatches("xyz")
print(none)  // 0
```

**Rust Codegen:**
```rust
text.matches("an").count() as i32
```

**Notes:**
- Non-overlapping matches only
- Case-sensitive matching
- Returns 0 if substring not found

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
- For substrings, use `substring()` or `slice()`

---

### `slice()` *(v1.4.0)*

Extract a substring by start and optional end index.

**Signature:**
```liva
slice(start: int, end?: int) => string
```

**Examples:**
```liva
let text = "Hello, World!"

// With start and end
let hello = text.slice(0, 5)
print(hello)  // "Hello"

// With start only (to end of string)
let world = text.slice(7)
print(world)  // "World!"
```

**Rust Codegen:**
```rust
// With end
text[0..5].to_string()

// Without end
text[7..].to_string()
```

**Notes:**
- `start` is inclusive, `end` is exclusive
- Without `end`, slices to the end of the string
- Similar to `substring()` but with optional end parameter
- Also available on arrays: `[1,2,3].slice(0,2)` → `[1,2]`

---

### `chars()` *(v1.4.0)*

Split the string into an array of individual characters.

**Signature:**
```liva
chars() => [string]
```

**Examples:**
```liva
let text = "hello"
let characters = text.chars()
print(characters)  // ["h", "e", "l", "l", "o"]

// Useful for character-level processing
let vowels = "hello".chars().filter(c => c == "a" or c == "e" or c == "i" or c == "o" or c == "u")
print(vowels)  // ["e", "o"]
```

**Rust Codegen:**
```rust
text.chars().map(|c| c.to_string()).collect::<Vec<String>>()
```

**Notes:**
- Each character becomes a single-character string (not `char`)
- Unicode-aware (handles multi-byte characters correctly)
- Useful for character-level filtering or mapping

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

##  See Also

- [Array Methods](./arrays.md)
- [Standard Library Overview](./README.md)
- [String Templates](../string-templates.md)
- [Language Reference](../README.md)
