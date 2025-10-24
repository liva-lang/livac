# JSON API Reference

**Version:** v0.9.3  
**Status:** Stable  

---

## Overview

The JSON module provides functions to parse JSON strings and serialize Liva values to JSON format. All JSON operations use error binding for safe error handling.

---

## Functions

### JSON.parse()

**Signature:**
```liva
JSON.parse(json: string): (any?, Error?)
```

**Description:**  
Parses a JSON string and returns the parsed value. Returns an error if the JSON is invalid.

**Parameters:**
- `json` (string): The JSON string to parse

**Returns:**
- Tuple `(value?, error?)`:
  - `value`: The parsed JSON value (Some) on success, None on error
  - `error`: None on success, Some(Error) on failure

**Example:**
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
- All operations are performed at runtime
- Error messages include details from the underlying JSON library
- No compile-time JSON validation (all errors caught at runtime)

---

## Future Enhancements (v0.9.4+)

### Type-Safe Parsing (Planned)
```liva
interface User {
    name: string
    age: int
}

let user = JSON.parseTyped<User>(jsonStr)
// Validates structure at runtime
```

### JSON Schema Validation (Planned)
```liva
const schema = {
    type: "object",
    required: ["name", "age"]
}

let data = JSON.parse(jsonStr, schema)
// Validates against schema
```

### Pretty Printing (Planned)
```liva
let json = JSON.stringify(obj, {pretty: true})
// Formatted JSON with indentation
```

---

## See Also

- [Error Handling Guide](./error-handling.md)
- [Type System Reference](./types.md)
- [String Templates](./string-templates.md)

---

**Last Updated:** 2025-01-21  
**Version:** v0.9.3
