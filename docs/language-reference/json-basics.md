# JSON: Basics & Type-Safe Parsing

**Version:** v0.10.0  
**Status:** Stable  
**New in v0.10.0:** Type-safe JSON parsing with type hints and custom classes ✨

---

## Table of Contents

- [Overview](#overview)
- [Quick Start (v0.10.0)](#quick-start-v0100)
- [Functions](#functions)
  - [JSON.parse() - Basic Usage (v0.9.x)](#jsonparse---basic-usage-v09x)
  - [JSON.stringify()](#jsonstringify)
- [Type-Safe Parsing (v0.10.0+)](#type-safe-parsing-v0100-)
  - [Supported Types](#supported-types)
  - [Syntax](#syntax)
  - [Examples](#examples)
  - [Error Handling](#error-handling)
  - [Implementation Details](#implementation-details)
- [Optional Fields (v0.10.4+)](#optional-fields-v0104-)
- [See Also](#see-also)

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

---

## Optional Fields (v0.10.4+) ✨

**New in v0.10.4:** Optional fields with `?` syntax for handling nullable/missing JSON fields.

### Why Optional Fields?

Real-world APIs often have fields that may be absent, null, or undefined:

```json
{
  "users": [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob"},  // Missing email
    {"id": 3, "name": "Carol", "email": null}  // Null email
  ]
}
```

**Without optional fields (v0.10.3-):** JSON parsing **fails** for Bob and Carol.  
**With optional fields (v0.10.4+):** All users parse successfully! ✅

### Syntax

Declare optional fields with `?` after the field name:

```liva
User {
    id: u32           // Required field
    name: String      // Required field
    email?: String    // ✨ Optional - can be null or missing
    age?: u32         // ✨ Optional - can be null or missing
}
```

### How It Works

- **Required fields** (`field: Type`): Must be present and non-null in JSON
- **Optional fields** (`field?: Type`): Can be absent, null, or present
- Generates `Option<T>` wrapper in Rust code
- Automatically adds `#[serde(skip_serializing_if = "Option::is_none")]` for efficient serialization

### Generated Code

```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,  // ✅ Wrapped in Option
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
}
```

### Example Usage

```liva
User {
    id: u32
    name: String
    email?: String
    phone?: String
}

main() {
    // Case 1: All fields present
    let json1 = "{\"id\": 1, \"name\": \"Alice\", \"email\": \"alice@example.com\", \"phone\": \"555-0001\"}"
    let user1: User, err1 = JSON.parse(json1)
    // ✅ Success: email = Some("alice@example.com")
    
    // Case 2: Optional field missing
    let json2 = "{\"id\": 2, \"name\": \"Bob\", \"email\": \"bob@example.com\"}"
    let user2: User, err2 = JSON.parse(json2)
    // ✅ Success: email = Some("bob@example.com"), phone = None
    
    // Case 3: Optional field null
    let json3 = "{\"id\": 3, \"name\": \"Carol\", \"email\": null, \"phone\": \"555-0003\"}"
    let user3: User, err3 = JSON.parse(json3)
    // ✅ Success: email = None, phone = Some("555-0003")
    
    // Case 4: All optional fields missing
    let json4 = "{\"id\": 4, \"name\": \"Dave\"}"
    let user4: User, err4 = JSON.parse(json4)
    // ✅ Success: email = None, phone = None
}
```

### Real-World Example: API Integration

```liva
Post {
    id: u64
    title: String
    content: String
    publishedAt?: String  // May not be published yet
    authorEmail?: String  // Author may not have public email
    tags?: [String]       // Optional array of tags
    likes?: u32           // New field, old posts don't have it
    imageUrl?: String     // Not all posts have images
}

main() {
    let response, err = async HTTP.get("https://api.example.com/posts")
    
    if err == "" && response.status == 200 {
        let posts: [Post], parseErr = JSON.parse(response.body)
        
        if parseErr == "" {
            print($"Loaded {posts.length} posts")
            // All posts parse successfully, regardless of which optional fields are present! ✅
        }
    }
}
```

### Benefits

✅ **Type Safety:** Explicitly document which fields can be absent/null  
✅ **No More Crashes:** Missing fields don't cause parse failures  
✅ **Better DX:** Code shows intent - optional vs required  
✅ **API Ready:** Handle real-world APIs with nullable fields  
✅ **Zero Overhead:** Direct mapping to Rust's `Option<T>`

### Best Practices

**DO:**
- ✅ Use `?` for fields that may be absent/null in JSON
- ✅ Use optional fields for API responses with partial data
- ✅ Combine with error binding for robust error handling

**DON'T:**
- ❌ Make all fields optional "just in case" (be explicit!)
- ❌ Use optional for required API fields (document your API contract)

### Comparison: Required vs Optional

| Feature | Required (`field: Type`) | Optional (`field?: Type`) |
|---------|-------------------------|--------------------------|
| JSON missing | ❌ Parse error | ✅ Parse success (None) |
| JSON null | ❌ Parse error | ✅ Parse success (None) |
| JSON present | ✅ Parse success | ✅ Parse success (Some) |
| Rust type | `T` | `Option<T>` |
| Default value | Type default | `None` |

---

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

## See Also

- [JSON: Advanced Usage](./json-advanced.md) — Error handling, type mapping, complete examples, migration guide, future enhancements
- [Error Handling Guide](./error-handling.md)
- [Type System Reference](./types.md)
- [String Templates](./string-templates.md)
- [Classes Documentation](./classes.md)
- [CHANGELOG](../../CHANGELOG.md) - See v0.10.0 release notes

---

**Last Updated:** 2025-01-25  
**Version:** v0.10.0  
**Changes:** Added type-safe JSON parsing with type hints and custom classes
