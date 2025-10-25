# JSON API Reference

**Version:** v0.10.0  
**Status:** Stable  
**New in v0.10.0:** Type-safe JSON parsing with type hints and custom classes ✨

---

## Overview

The JSON module provides functions to parse JSON strings and serialize Liva values to JSON format. Starting with v0.10.0, JSON parsing supports **type hints** for direct deserialization into typed values and custom classes, eliminating the need for manual type conversions.

---

## Quick Start (v0.10.0)

### Type-Safe Parsing with Type Hints

```liva
// OLD way (v0.9.x) - verbose with .unwrap()
let data = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n.as_i32().unwrap() * 2)

// NEW way (v0.10.0) - clean and type-safe! ✨
let data: [i32], err = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n * 2)  // No .unwrap() needed!
```

### Parsing into Custom Classes

```liva
User {
    id: u64
    name: String
    age: i32
}

let user: User, err = JSON.parse("{\"id\": 1, \"name\": \"Alice\", \"age\": 30}")
// Returns User directly, not a generic JsonValue!
```

---

## Functions

### JSON.parse() - Basic Usage (v0.9.x)

### JSON.parse() - Basic Usage (v0.9.x)

**Signature (Legacy):**
```liva
JSON.parse(json: string): (JsonValue?, String?)
```

**Signature (v0.10.0 - Type-Safe):**
```liva
JSON.parse(json: string): (T, String)  // With type hint: let data: T, err = ...
```

**Description:**  
Parses a JSON string and returns the parsed value. In v0.10.0+, supports type hints for direct deserialization into typed values.

**Parameters:**
- `json` (string): The JSON string to parse

**Returns (v0.10.0 with type hint):**
- Tuple `(value, error)`:
  - `value`: The parsed value of type T (defaults to empty/zero on error)
  - `error`: Empty string `""` on success, error message on failure

**Returns (Legacy without type hint):**
- Tuple `(value?, error?)`:
  - `value`: The parsed JsonValue (Some) on success, None on error
  - `error`: None on success, Some(Error) on failure

**Example (v0.10.0 - Type-Safe):**
```liva
main() {
    // Parse primitives
    let num: i32, err = JSON.parse("42")
    let text: String, err2 = JSON.parse("\"hello\"")
    let flag: bool, err3 = JSON.parse("true")
    
    // Parse arrays
    let numbers: [i32], err4 = JSON.parse("[1, 2, 3]")
    let floats: [f64], err5 = JSON.parse("[1.5, 2.7, 3.9]")
    
    // Parse custom classes
    let user: User, err6 = JSON.parse("{\"id\": 1, \"name\": \"Alice\", \"age\": 30}")
    let users: [User], err7 = JSON.parse("[{...}, {...}]")
    
    if err == "" {
        print($"Parsed number: {num}")
    }
}
```

**Example (Legacy v0.9.x):**
```liva
main() {
    const jsonStr = "{\"name\": \"Alice\", \"age\": 30}"
    let data, err = JSON.parse(jsonStr)
    
    if err {
        print("Parse error!")
    } else {
        print("Parsed successfully!")
    }
}
```

**Supported JSON Types:**
- `null` → `none`
- `true`/`false` → `bool`
- Numbers → `int` or `float`
- Strings → `string`
- Arrays → `array<any>`
- Objects → `object`

**Error Cases:**
- Invalid JSON syntax
- Unexpected end of input
- Invalid escape sequences
- Malformed numbers
- Type mismatch (v0.10.0): JSON doesn't match expected type

---

## Type-Safe Parsing (v0.10.0+) 🎉

### Overview

Starting with v0.10.0, Liva supports **type hints** on variable declarations with `JSON.parse()` for automatic, type-safe deserialization. This eliminates verbose `.as_i32().unwrap()` calls and provides compile-time type checking.

### Supported Types

#### Primitive Types
All Rust integer and float types are supported:

| Liva Type | Rust Type | JSON Example |
|-----------|-----------|--------------|
| `int` | `i32` | `42` |
| `i8` | `i8` | `127` |
| `i16` | `i16` | `32767` |
| `i32` | `i32` | `2147483647` |
| `i64` | `i64` | `9223372036854775807` |
| `i128` | `i128` | Large numbers |
| `u8` | `u8` | `255` |
| `u16` | `u16` | `65535` |
| `u32` | `u32` | `4294967295` |
| `u64` | `u64` | Large unsigned |
| `u128` | `u128` | Very large unsigned |
| `isize` | `isize` | Platform-dependent |
| `usize` | `usize` | Platform-dependent |
| `float` | `f64` | `3.14159` |
| `f32` | `f32` | `3.14` |
| `f64` | `f64` | `3.14159265359` |
| `bool` | `bool` | `true` / `false` |
| `string`, `String` | `String` | `"hello"` |

#### Arrays
Arrays of any supported type:
```liva
let numbers: [i32], err = JSON.parse("[1, 2, 3]")
let floats: [f64], err = JSON.parse("[1.5, 2.7]")
let texts: [String], err = JSON.parse("[\"a\", \"b\"]")
```

#### Custom Classes (v0.10.0 Phase 2)
Parse directly into user-defined classes:

```liva
User {
    id: u64
    name: String
    age: i32
}

let user: User, err = JSON.parse("{\"id\": 1, \"name\": \"Alice\", \"age\": 30}")
let users: [User], err = JSON.parse("[{\"id\": 1, ...}, {\"id\": 2, ...}]")
```

Classes used with `JSON.parse()` automatically get `serde::Deserialize` derives generated in the Rust output.

### Syntax

**With Error Binding (Recommended):**
```liva
let value: Type, err = JSON.parse(jsonString)

if err == "" {
    // Success - use value
} else {
    // Handle error
    print($"Error: {err}")
}
```

**Without Error Binding (Panics on Error):**
```liva
let value: Type = JSON.parse(jsonString)
// Will panic with "JSON parse failed" if parsing fails
```

### Examples

#### Example 1: Parse Primitive Types
```liva
main() {
    // Integer
    let count: i32, err = JSON.parse("42")
    print($"Count: {count}")
    
    // Float
    let price: f64, err2 = JSON.parse("19.99")
    print($"Price: {price}")
    
    // Boolean
    let active: bool, err3 = JSON.parse("true")
    print($"Active: {active}")
    
    // String
    let message: String, err4 = JSON.parse("\"Hello, World!\"")
    print($"Message: {message}")
}
```

#### Example 2: Parse Arrays
```liva
main() {
    let numbers: [i32], err = JSON.parse("[1, 2, 3, 4, 5]")
    
    if err == "" {
        // Process without .unwrap()!
        let doubled = numbers.map(n => n * 2)
        let sum = numbers.map(n => n).sum()
        
        print($"Doubled: {doubled}")
    }
}
```

#### Example 3: Parse Custom Classes
```liva
Post {
    id: u64
    title: String
    body: String
    userId: u32
}

main() {
    let postJson = "{\"id\": 1, \"title\": \"Hello\", \"body\": \"Content\", \"userId\": 123}"
    let post: Post, err = JSON.parse(postJson)
    
    if err == "" {
        print($"Post: {post.title} by user {post.userId}")
    }
}
```

#### Example 4: Parse Array of Classes
```liva
User {
    name: String
    age: i32
}

main() {
    let usersJson = "[{\"name\": \"Alice\", \"age\": 30}, {\"name\": \"Bob\", \"age\": 25}]"
    let users: [User], err = JSON.parse(usersJson)
    
    if err == "" {
        print($"Loaded {users.len()} users")
        // Process users...
    }
}
```

#### Example 5: Parallel Processing with Typed JSON
```liva
main() {
    let data: [i32], err = JSON.parse("[1, 2, 3, 4, 5, 6, 7, 8]")
    
    if err == "" {
        // Parallel map - no .unwrap() needed!
        let results = data.parvec().map(n => n * n)
        print($"Squared: {results}")
    }
}
```

### Error Handling

Type-safe parsing returns a tuple with the value and error string:

```liva
let value: Type, err = JSON.parse(json)

if err == "" {
    // Success - value is populated
} else {
    // Error - err contains message, value is default (0, "", Vec::new(), etc.)
    print($"Parse failed: {err}")
}
```

**Default Values on Error:**
- Integers: `0`
- Floats: `0.0`
- Booleans: `false`
- Strings: `""`
- Arrays: `[]` (empty)
- Classes: Default-initialized (all fields default)

### Implementation Details

- Uses `serde_json::from_str::<T>()` for deserialization
- Type validation happens at compile-time (semantic analysis)
- Classes automatically get `#[derive(Serialize, Deserialize)]` when used with JSON
- Zero runtime overhead compared to manual serde usage
- Error messages include detailed serde_json error information

---

### JSON.stringify()

**Signature:**
```liva
JSON.stringify(value: any): (string?, Error?)
```

**Description:**  
Converts a Liva value to a JSON string.

**Parameters:**
- `value` (any): The value to serialize

**Returns:**
- Tuple `(json?, error?)`:
  - `json`: The JSON string (Some) on success, None on error
  - `error`: None on success, Some(Error) on failure

**Example:**
```liva
main() {
    const obj = {name: "Bob", age: 25}
    let json, err = JSON.stringify(obj)
    
    if err {
        print("Stringify error!")
    } else {
        print("JSON created!")
    }
}
```

**Supported Types:**
- `none` → `null`
- `bool` → `true`/`false`
- `int` → number
- `float` → number
- `string` → string
- Arrays → JSON array
- Objects → JSON object

**Unsupported Types (will error):**
- Functions
- Closures
- Tasks
- Circular references

---

## Error Handling

All JSON functions use the error binding pattern:

```liva
let result, err = JSON.parse(str)

if err {
    // Handle error
    print("Error: ${err}")
} else {
    // Use result
    print("Success!")
}
```

**Common Errors:**
- `"JSON parse error: ..."` - Invalid JSON syntax
- `"JSON stringify error: ..."` - Unsupported type or circular reference

---

## Type Mapping

### JSON → Liva

| JSON Type | Liva Type | Example |
|-----------|-----------|---------|
| `null` | `none` | `null` → `none` |
| `true`/`false` | `bool` | `true` → `true` |
| number (int) | `int` | `42` → `42` |
| number (float) | `float` | `3.14` → `3.14` |
| string | `string` | `"hello"` → `"hello"` |
| array | `array<any>` | `[1, 2, 3]` → `[1, 2, 3]` |
| object | `object` | `{"a": 1}` → `{a: 1}` |

### Liva → JSON

| Liva Type | JSON Type | Example |
|-----------|-----------|---------|
| `none` | `null` | `none` → `null` |
| `bool` | boolean | `true` → `true` |
| `int` | number | `42` → `42` |
| `float` | number | `3.14` → `3.14` |
| `string` | string | `"hello"` → `"hello"` |
| `array` | array | `[1, 2]` → `[1, 2]` |
| `object` | object | `{a: 1}` → `{"a": 1}` |

---

## Complete Examples

### Example 1: Parse JSON API Response

```liva
main() {
    // Simulate API response
    const apiResponse = "{\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}"
    
    let data, err = JSON.parse(apiResponse)
    
    if err {
        print("Failed to parse API response")
        fail err
    }
    
    print("API data received!")
}
```

### Example 2: Create JSON Request Body

```liva
main() {
    const requestBody = {
        method: "POST",
        userId: 123,
        active: true
    }
    
    let json, err = JSON.stringify(requestBody)
    
    if err {
        print("Failed to create request")
    } else {
        print("Request body ready!")
    }
}
```

### Example 3: Handle Parse Errors

```liva
main() {
    const invalidJson = "{broken json}"
    let data, err = JSON.parse(invalidJson)
    
    if err {
        print("Parse failed (expected)")
        // Continue execution
    }
    
    // Try again with valid JSON
    const validJson = "{\"status\": \"ok\"}"
    let data2, err2 = JSON.parse(validJson)
    
    if err2 {
        print("Unexpected error!")
    } else {
        print("Second parse succeeded!")
    }
}
```

### Example 4: Round-trip Conversion

```liva
main() {
    // Original data
    const original = {
        name: "Charlie",
        age: 30,
        tags: ["developer", "designer"]
    }
    
    // Convert to JSON
    let jsonStr, err1 = JSON.stringify(original)
    if err1 {
        fail err1
    }
    
    print("JSON string created")
    
    // Parse back to object
    let parsed, err2 = JSON.parse(jsonStr)
    if err2 {
        fail err2
    }
    
    print("Round-trip successful!")
}
```

---

## Implementation Notes

- JSON parsing and serialization use Rust's `serde_json` crate
- Type-safe parsing (v0.10.0+) uses serde derives for zero-cost deserialization
- All operations are performed at runtime
- Error messages include details from the underlying JSON library
- Type validation happens at compile-time (semantic analysis phase)
- Classes used with `JSON.parse()` automatically get serde derives

---

## Migration Guide: v0.9.x → v0.10.0

### Before (v0.9.x)
```liva
let data = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n.as_i32().unwrap() * 2)

User {
    name: String
    age: i32
}
let userJson = "{...}"
let jsonValue = JSON.parse(userJson)
let name = jsonValue.get_field("name").unwrap().as_string().unwrap()
```

### After (v0.10.0)
```liva
let data: [i32], err = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n * 2)  // Clean! ✨

User {
    name: String
    age: i32
}
let user: User, err = JSON.parse("{...}")
let name = user.name  // Direct access! ✨
```

**Benefits:**
- ✅ No more `.as_i32().unwrap()` chains
- ✅ Direct field access on classes
- ✅ Better type safety
- ✅ Cleaner, more readable code
- ✅ Same performance (serde is zero-cost)

---

## Future Enhancements (v0.10.1+)

### Optional Fields (Phase 3 - Planned)
```liva
User {
    name: String
    age: i32
    email?: String  // Optional field
    phone?: String
}

let user: User, err = JSON.parse("{\"name\": \"Alice\", \"age\": 30}")
// email and phone will be None
```

### Default Values (Phase 3 - Planned)
```liva
Config {
    host: String = "localhost"
    port: i32 = 8080
    debug: bool = false
}

let config: Config, err = JSON.parse("{}")
// Uses defaults for missing fields
```

### Nested Classes (Phase 4 - Planned)
```liva
Address {
    street: String
    city: String
}

User {
    name: String
    address: Address  // Nested class
}

let user: User, err = JSON.parse("{\"name\": \"Alice\", \"address\": {...}}")
```

### Snake_case Conversion (Phase 2.2 - In Progress)
```liva
User {
    userId: u64      // Liva uses camelCase
    firstName: String
}

// Automatically maps from JSON snake_case:
// {"user_id": 1, "first_name": "Alice"}
```

### JSON Schema Validation (Future)
```liva
const schema = {
    type: "object",
    required: ["name", "age"]
}

let data = JSON.parse(jsonStr, schema)
// Validates against schema
```

---

## See Also

- [Error Handling Guide](./error-handling.md)
- [Type System Reference](./types.md)
- [String Templates](./string-templates.md)
- [Classes Documentation](./classes.md)
- [CHANGELOG](../../CHANGELOG.md) - See v0.10.0 release notes

---

**Last Updated:** 2025-01-25  
**Version:** v0.10.0  
**Changes:** Added type-safe JSON parsing with type hints and custom classes
