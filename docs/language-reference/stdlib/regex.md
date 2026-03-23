# Regex — Regular Expressions

> **Version:** 1.6.0  
> **Status:** ✅ Complete (5/5 functions)  
> **Crate:** `regex = "1"` (auto-injected when `Regex.*` is used)

The `Regex` module provides regular expression matching, searching, replacing, and splitting. The `regex` crate is automatically added to `Cargo.toml` when any `Regex.*` function is used — no `use rust` needed.

---

## 📋 Table of Contents

- [Regex.test()](#regextest)
- [Regex.match()](#regexmatch)
- [Regex.findAll()](#regexfindall)
- [Regex.replace()](#regexreplace)
- [Regex.split()](#regexsplit)

---

## `Regex.test()`

Tests whether a pattern matches anywhere in the text.

**Signature:**
```liva
Regex.test(pattern: string, text: string): bool
```

**Parameters:**
- `pattern`: Regular expression pattern (Rust regex syntax)
- `text`: Text to search

**Returns:**
- `true` if the pattern matches anywhere in the text
- `false` if no match or invalid pattern

**Example:**
```liva
let isEmail = Regex.test("^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$", "user@example.com")
print($"Valid email: {isEmail}")  // Valid email: true

let hasDigit = Regex.test("\\d", "hello123")
print($"Has digit: {hasDigit}")  // Has digit: true
```

**Rust Codegen:**
```rust
regex::Regex::new(&pattern).map(|re| re.is_match(&text)).unwrap_or(false)
```

**Notes:**
- Returns `false` (not error) for invalid patterns — use `Regex.match()` if you need error info
- Does not use error binding

---

## `Regex.match()`

Finds the first match of a pattern in the text.

**Signature:**
```liva
Regex.match(pattern: string, text: string): (string?, Error?)
```

**Parameters:**
- `pattern`: Regular expression pattern
- `text`: Text to search

**Returns:**
- Success with match: `(Some(matched_text), None)`
- Success with no match: `(None, None)` — empty string and no error
- Invalid pattern: `(None, Some(Error))`

**Example:**
```liva
let found, err = Regex.match("\\d+", "Order #42 has 3 items")
if err == "" {
    print($"First match: {found}")  // First match: 42
}

// No match case
let noMatch, err2 = Regex.match("\\d+", "no numbers here")
if noMatch == "" {
    print("No match found")
}
```

**Rust Codegen:**
```rust
match regex::Regex::new(&pattern) {
    Ok(re) => match re.find(&text) {
        Some(m) => (Some(m.as_str().to_string()), String::new()),
        None => (None, String::new())
    },
    Err(e) => (None, format!("Regex error: {}", e))
}
```

**Notes:**
- Uses error binding pattern (like `File.read`)
- Returns the **full match text**, not capture groups

---

## `Regex.findAll()`

Finds all non-overlapping matches of a pattern in the text.

**Signature:**
```liva
Regex.findAll(pattern: string, text: string): [string]
```

**Parameters:**
- `pattern`: Regular expression pattern
- `text`: Text to search

**Returns:**
- Array of matched strings
- Empty array `[]` if no matches or invalid pattern

**Example:**
```liva
let numbers = Regex.findAll("\\d+", "a1b22c333")
print(numbers)  // ["1", "22", "333"]

let words = Regex.findAll("[A-Z][a-z]+", "HelloWorld FooBar")
print(words)    // ["Hello", "World", "Foo", "Bar"]
```

**Rust Codegen:**
```rust
regex::Regex::new(&pattern)
    .map(|re| re.find_iter(&text).map(|m| m.as_str().to_string()).collect())
    .unwrap_or_default()
```

**Notes:**
- Returns empty array (not error) for invalid patterns
- No error binding — always succeeds
- Matches are non-overlapping

---

## `Regex.replace()`

Replaces all occurrences of a pattern with a replacement string.

**Signature:**
```liva
Regex.replace(pattern: string, text: string, replacement: string): string
```

**Parameters:**
- `pattern`: Regular expression pattern
- `text`: Text to search in
- `replacement`: Replacement string (supports `$1`, `$2` backreferences)

**Returns:**
- New string with all matches replaced
- Original text if pattern is invalid

**Example:**
```liva
let cleaned = Regex.replace("\\s+", "  hello   world  ", " ")
print(cleaned)  // " hello world "

let censored = Regex.replace("\\d{4}", "Card: 1234-5678", "****")
print(censored)  // "Card: ****-****"

// Backreferences
let swapped = Regex.replace("(\\w+) (\\w+)", "Hello World", "$2 $1")
print(swapped)  // "World Hello"
```

**Rust Codegen:**
```rust
regex::Regex::new(&pattern)
    .map(|re| re.replace_all(&text, replacement).to_string())
    .unwrap_or_else(|_| text.to_string())
```

**Notes:**
- Replaces **all** occurrences (like JavaScript's `replaceAll`)
- Returns original text (not error) for invalid patterns
- Supports `$1`, `$2`, etc. for capture group backreferences

---

## `Regex.split()`

Splits text by a regex pattern.

**Signature:**
```liva
Regex.split(pattern: string, text: string): [string]
```

**Parameters:**
- `pattern`: Regular expression pattern to split on
- `text`: Text to split

**Returns:**
- Array of strings from splitting
- Array with original text as sole element if pattern is invalid

**Example:**
```liva
let parts = Regex.split("[,;\\s]+", "a, b; c  d")
print(parts)  // ["a", "b", "c", "d"]

let words = Regex.split("\\s+", "hello world  foo")
print(words)  // ["hello", "world", "foo"]
```

**Rust Codegen:**
```rust
regex::Regex::new(&pattern)
    .map(|re| re.split(&text).map(|s| s.to_string()).collect())
    .unwrap_or_else(|_| vec![text.to_string()])
```

**Notes:**
- Similar to `String.split()` but with regex patterns
- Returns single-element array with original text on invalid pattern

---

## Regex Pattern Syntax

Liva uses Rust's regex syntax (similar to PCRE without lookahead/lookbehind):

| Pattern | Description |
|---------|-------------|
| `.` | Any character except newline |
| `\d` | Digit `[0-9]` |
| `\w` | Word character `[a-zA-Z0-9_]` |
| `\s` | Whitespace |
| `\D`, `\W`, `\S` | Negated versions |
| `[abc]` | Character class |
| `[^abc]` | Negated character class |
| `^`, `$` | Start/end of string |
| `*`, `+`, `?` | Quantifiers (0+, 1+, 0-1) |
| `{n}`, `{n,m}` | Exact/range repetition |
| `(...)` | Capture group |
| `(?:...)` | Non-capturing group |
| `a\|b` | Alternation |

**Important:** In Liva string literals, backslashes need double escaping: `"\\d+"` for the regex `\d+`.

---

## Common Patterns

### Email Validation
```liva
let isValid = Regex.test("^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$", email)
```

### Extract Numbers
```liva
let nums = Regex.findAll("\\d+", text)
```

### Clean Whitespace
```liva
let clean = Regex.replace("\\s+", text, " ")
```

### Parse CSV-like Lines
```liva
let fields = Regex.split(",\\s*", "name, age, city")
```

### Validate Phone Number
```liva
let isPhone = Regex.test("^\\+?\\d{1,3}[-.\\s]?\\d{3,14}$", phone)
```

---

## Error Handling

Regex functions handle errors gracefully without crashing:

| Function | Invalid pattern behavior |
|----------|------------------------|
| `Regex.test()` | Returns `false` |
| `Regex.match()` | Returns `("", "Regex error: ...")` |
| `Regex.findAll()` | Returns `[]` |
| `Regex.replace()` | Returns original text unchanged |
| `Regex.split()` | Returns `[original_text]` |

Only `Regex.match()` uses error binding. All other functions degrade gracefully.

---

## See Also

- [String Methods](./strings.md) — `contains`, `replace`, `split` for literal patterns
- [File I/O](../file-io.md) — Reading text files to process with Regex
- [Standard Library Overview](./README.md)
