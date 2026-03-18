# JSON: Advanced Usage

**Version:** v0.10.0  
**Status:** Stable  

---

## Table of Contents

- [Error Handling](#error-handling)
- [Type Mapping](#type-mapping)
  - [JSON → Liva](#json--liva)
  - [Liva → JSON](#liva--json)
- [Complete Examples](#complete-examples)
- [Implementation Notes](#implementation-notes)
- [Migration Guide: v0.9.x → v0.10.0](#migration-guide-v09x--v0100)
- [Future Enhancements (v0.10.1+)](#future-enhancements-v0101)
  - [Optional Fields (Phase 3)](#optional-fields-phase-3---planned)
  - [Default Values](#default-values--available-in-v0104)
  - [Nested Classes](#nested-classes--available)
  - [Snake_case Conversion](#snake_case-conversion-phase-22---in-progress)
  - [JSON Schema Validation](#json-schema-validation-future)
- [See Also](#see-also)

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

### Default Values ✅ Available in v0.10.4

Fields can have default values that are used both in constructors and when deserializing from JSON:

```liva
Config {
    host: string = "localhost"
    port: int = 8080
    debug: bool = false
    timeout?: int = 30  // Optional with default
}

let config: Config, err = JSON.parse("{}")
// Uses defaults for missing fields:
// host = "localhost", port = 8080, debug = false, timeout = Some(30)

let config2: Config, err2 = JSON.parse("{\"port\": 3000}")
// Overrides port, uses defaults for others:
// host = "localhost", port = 3000, debug = false, timeout = Some(30)
```

**Supported Default Types:**
- `int`, `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`: Integer literals
- `float`, `f32`, `f64`: Float literals
- `string`, `String`: String literals (automatically converted to `String`)
- `bool`: `true` or `false`

**Constructor Usage:**
```liva
User {
    name: string = "Guest"
    age: int = 18
    role: string = "user"
    active: bool = true
}

let user1 = User.new()  // Uses all defaults
// name = "Guest", age = 18, role = "user", active = true
```

**JSON Deserialization:**
Default values are automatically applied when fields are missing from JSON:

```liva
Settings {
    theme: string = "dark"
    fontSize: int = 14
    autoSave: bool = true
    maxRetries?: int = 3  // Optional with default
}

let json = "{\"theme\": \"light\"}"
let settings: Settings, err = JSON.parse(json)
// theme = "light" (from JSON)
// fontSize = 14 (default)
// autoSave = true (default)
// maxRetries = Some(3) (default for optional field)
```

**Optional Fields with Defaults:**
When combining `?` (optional) with `=` (default value), the default is used when the field is missing from JSON:

```liva
User {
    id: u32
    name: string
    bio?: string = "No bio available"  // Optional with default
}

// JSON without bio field:
let json1 = "{\"id\": 1, \"name\": \"Alice\"}"
let user1: User, err1 = JSON.parse(json1)
// bio = Some("No bio available")

// JSON with bio field:
let json2 = "{\"id\": 2, \"name\": \"Bob\", \"bio\": \"Developer\"}"
let user2: User, err2 = JSON.parse(json2)
// bio = Some("Developer")

// JSON with null bio:
let json3 = "{\"id\": 3, \"name\": \"Carol\", \"bio\": null}"
let user3: User, err3 = JSON.parse(json3)
// bio = None (null overrides default)
```

### Nested Classes ✅ Available
```liva
### Nested Classes ✅ Available
```liva
Geo {
    lat: string
    lng: string
}

Address {
    street: string
    suite: string
    city: string
    zipcode: string
    geo: Geo  // Nested class
}

User {
    name: string
    address: Address  // Nested class
}

let json = """
{
    "name": "Alice",
    "address": {
        "street": "Main St",
        "suite": "Apt 4",
        "city": "NYC",
        "zipcode": "10001",
        "geo": {
            "lat": "40.7128",
            "lng": "-74.0060"
        }
    }
}
"""

let user: User, err = JSON.parse(json)
console.log(user.address.city)           // "NYC"
console.log(user.address.geo.lat)        // "40.7128"

// Works in destructuring too:
users.forEach(({name, address}) => {
    console.log($"{name} lives at {address.zipcode}")
})
```

**Nested Optional Classes:**
```liva
User {
    name: string
    address?: Address  // Optional nested class
}

// JSON without address:
let json1 = "{\"name\": \"Bob\"}"
let user1: User, err1 = JSON.parse(json1)
// address = None

// JSON with address:
let json2 = "{\"name\": \"Alice\", \"address\": {...}}"
let user2: User, err2 = JSON.parse(json2)
// address = Some(Address { ... })
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

- [JSON: Basics & Type-Safe Parsing](./json-basics.md) — Overview, quick start, functions, type-safe parsing, optional fields
- [Error Handling Guide](./error-handling.md)
- [Type System Reference](./types.md)
- [String Templates](./string-templates.md)
- [Classes Documentation](./classes.md)
- [CHANGELOG](../../CHANGELOG.md) - See v0.10.0 release notes

---

**Last Updated:** 2025-01-25  
**Version:** v0.10.0  
**Changes:** Added type-safe JSON parsing with type hints and custom classes
