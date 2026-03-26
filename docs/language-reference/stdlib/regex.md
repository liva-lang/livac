# Regex — Regular Expressions

> `regex = "1"` auto-injected when `Regex.*` is used.

## Functions

### `Regex.test(pattern: string, text: string)` → `bool`

Tests whether pattern matches anywhere in text. Returns `false` on invalid pattern.

```liva
let isEmail = Regex.test("^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$", "user@example.com")
let hasDigit = Regex.test("\\d", "hello123")
```

### `Regex.match(pattern: string, text: string)` → `(string, string)`

Finds first match. Uses error binding.

- Match found: `("matched_text", "")`
- No match: `("", "")`
- Invalid pattern: `("", "Regex error: ...")`

```liva
let found, err = Regex.match("\\d+", "Order #42 has 3 items")
if !err {
    print($"First match: {found}")  // 42
}
```

### `Regex.findAll(pattern: string, text: string)` → `[string]`

All non-overlapping matches. Returns `[]` on invalid pattern.

```liva
let numbers = Regex.findAll("\\d+", "a1b22c333")  // ["1", "22", "333"]
let words = Regex.findAll("[A-Z][a-z]+", "HelloWorld FooBar")  // ["Hello", "World", "Foo", "Bar"]
```

### `Regex.replace(pattern: string, text: string, replacement: string)` → `string`

Replaces **all** occurrences. Supports `$1`, `$2` backreferences. Returns original text on invalid pattern.

```liva
let cleaned = Regex.replace("\\s+", "  hello   world  ", " ")
let swapped = Regex.replace("(\\w+) (\\w+)", "Hello World", "$2 $1")
```

### `Regex.split(pattern: string, text: string)` → `[string]`

Splits text by regex. Returns `[original_text]` on invalid pattern.

```liva
let parts = Regex.split("[,;\\s]+", "a, b; c  d")  // ["a", "b", "c", "d"]
```

## Pattern Syntax

Rust regex syntax (like PCRE without lookahead/lookbehind):

| Pattern | Description |
|---------|-------------|
| `.` | Any character except newline |
| `\d`, `\w`, `\s` | Digit, word char, whitespace |
| `\D`, `\W`, `\S` | Negated versions |
| `[abc]` / `[^abc]` | Character class / negated |
| `^`, `$` | Start/end of string |
| `*`, `+`, `?` | Quantifiers (0+, 1+, 0-1) |
| `{n}`, `{n,m}` | Exact/range repetition |
| `(...)` / `(?:...)` | Capture / non-capturing group |
| `a\|b` | Alternation |

**Escaping:** In Liva strings, double-escape backslashes: `"\\d+"` for regex `\d+`.

## Common Patterns

```liva
Regex.test("^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$", email)   // Email
Regex.findAll("\\d+", text)                                    // Extract numbers
Regex.replace("\\s+", text, " ")                               // Clean whitespace
Regex.split(",\\s*", "name, age, city")                        // Parse CSV-like
Regex.test("^\\+?\\d{1,3}[-.\\s]?\\d{3,14}$", phone)         // Phone number
```

## Error Handling

| Function | Invalid pattern behavior |
|----------|------------------------|
| `Regex.test()` | Returns `false` |
| `Regex.match()` | Returns `("", "Regex error: ...")` |
| `Regex.findAll()` | Returns `[]` |
| `Regex.replace()` | Returns original text |
| `Regex.split()` | Returns `[original_text]` |

Only `Regex.match()` uses error binding. All others degrade gracefully.
