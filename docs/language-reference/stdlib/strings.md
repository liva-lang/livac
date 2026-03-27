# String Methods

> **30 methods** | v1.4.0

---

## Manipulation

### split(delimiter: string) => [string]
  "a,b,c".split(",")              // ["a", "b", "c"]
  "hello".split(",")              // ["hello"]
  "a::b::c".split("::")           // ["a", "b", "c"]

### replace(search: string, replacement: string) => string
  "hello world".replace("world", "Liva")  // "hello Liva"
  "la la la".replace("la", "ha")          // "ha ha ha"
  — Replaces ALL occurrences (not just first)

### replaceAll(search: string, replacement: string) => string
  "hello world hello".replaceAll("hello", "hi")  // "hi world hi"
  — Alias for replace(). Both replace all occurrences.

---

## Transformation

### toUpperCase() => string
  "hello".toUpperCase()            // "HELLO"
  "Hello World".toUpperCase()      // "HELLO WORLD"

### toLowerCase() => string
  "HELLO".toLowerCase()            // "hello"
  "HeLLo".toLowerCase()            // "hello"

### trim() => string
  "  hello  ".trim()               // "hello"
  "  hello  world  ".trim()        // "hello  world"
  — Removes spaces, tabs, newlines. Internal whitespace preserved.

### trimStart() => string
  "  hello  ".trimStart()          // "hello  "
  "hello".trimStart()              // "hello"

### trimEnd() => string
  "  hello  ".trimEnd()            // "  hello"
  "hello".trimEnd()                // "hello"

### capitalize() => string
  "hello world".capitalize()       // "Hello world"
  "Hello".capitalize()             // "Hello"
  "".capitalize()                  // ""
  — Only capitalizes the first character

### reverse() => string
  "hello".reverse()                // "olleh"
  "racecar".reverse()              // "racecar"

### truncate(maxLength: int) => string
  "Hello, World!".truncate(5)      // "Hello"
  "Hi".truncate(10)                // "Hi"
  — UTF-8 safe (uses chars, not bytes)

### padStart(targetLength: int, padChar?: string) => string
  "5".padStart(3, "0")            // "005"
  "hi".padStart(5)                // "   hi"  (default: space)
  "hello".padStart(3, "x")        // "hello"  (already long enough)

### padEnd(targetLength: int, padChar?: string) => string
  "hi".padEnd(5, ".")             // "hi..."
  "hi".padEnd(5)                  // "hi   "  (default: space)

### repeat(count: int) => string
  "ha".repeat(3)                   // "hahaha"
  "-".repeat(20)                   // "--------------------"
  "x".repeat(0)                    // ""

### removePrefix(prefix: string) => string
  "prefix_value".removePrefix("prefix_")  // "value"
  "hello".removePrefix("xyz")             // "hello"  (not found → original)

### removeSuffix(suffix: string) => string
  "document.txt".removeSuffix(".txt")     // "document"
  "hello".removeSuffix(".pdf")            // "hello"  (not found → original)

---

## Conversion

### toInt() => int
  "42".toInt()                             // 42
  "-7".toInt()                             // -7
  "abc".toInt()                            // 0  (invalid → 0)
  "3.14".toInt()                           // 0  (float string → 0)
  — Returns 0 for non-parseable strings. For error handling, use `parseInt()`.

### toFloat() => float
  "3.14".toFloat()                         // 3.14
  "-0.5".toFloat()                         // -0.5
  "42".toFloat()                           // 42.0
  "abc".toFloat()                          // 0.0  (invalid → 0.0)
  — Returns 0.0 for non-parseable strings. For error handling, use `parseFloat()`.

---

## Query

### startsWith(prefix: string) => bool
  "document.pdf".startsWith("document")   // true
  "https://x.com".startsWith("https://")  // true
  "Hello".startsWith("hello")             // false  (case-sensitive)

### endsWith(suffix: string) => bool
  "document.pdf".endsWith(".pdf")          // true
  "document.pdf".endsWith(".doc")          // false

### contains(substring: string) => bool
  "Hello, World!".contains("World")       // true
  "Hello, World!".contains("world")       // false  (case-sensitive)

### indexOf(substring: string) => int
  "hello world".indexOf("world")           // 6
  "hello world".indexOf("cat")            // -1  (not found)
  "hello".indexOf("H")                    // -1  (case-sensitive)
  — Returns index of first occurrence, or -1

### lastIndexOf(substring: string) => int
  "hello world hello".lastIndexOf("hello") // 12
  "hello".lastIndexOf("xyz")              // -1
  — Returns index of last occurrence, or -1

### isBlank() => bool
  "   ".isBlank()                          // true
  "".isBlank()                             // true
  "hello".isBlank()                        // false
  " a ".isBlank()                          // false
  — True for empty or whitespace-only

### isEmpty() => bool
  "".isEmpty()                             // true
  "   ".isEmpty()                          // false  (spaces count!)
  "hello".isEmpty()                        // false
  — Only checks zero length (use isBlank for whitespace)

### countMatches(substring: string) => int
  "banana".countMatches("an")              // 2
  "a,b,c,d".countMatches(",")             // 3
  "hello".countMatches("xyz")             // 0
  — Non-overlapping matches, case-sensitive

---

## Access

### substring(start: int, end: int) => string
  "Hello World".substring(0, 5)            // "Hello"
  "Hello World".substring(6, 11)           // "World"
  — start inclusive, end exclusive, zero-indexed

### charAt(index: int) => char
  "Hello".charAt(0)                        // 'H'
  "Hello".charAt(4)                        // 'o'
  "Hello".charAt(100)                      // ' '  (out of bounds → space)
  — UTF-8 safe

### slice(start: int, end?: int) => string
  "Hello, World!".slice(0, 5)              // "Hello"
  "Hello, World!".slice(7)                 // "World!"  (no end → to end)
  — Also available on arrays

### chars() => [string]
  "hello".chars()                          // ["h", "e", "l", "l", "o"]
  — Each character becomes a single-character string
  — Useful for character-level filtering/mapping
