# Type-Safe JSON Parsing Guide

**Version:** v0.10.0  
**Author:** Liva Team  
**Last Updated:** 2025-01-25

---

## Introduction

Liva v0.10.0 introduces **type-safe JSON parsing** with type hints, eliminating verbose `.as_i32().unwrap()` calls and providing compile-time type checking. This guide walks you through using this feature effectively.

---

## Quick Comparison

### Before (v0.9.x)
```liva
// Verbose and error-prone
let data = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n.as_i32().unwrap() * 2)
let filtered = doubled.filter(n => n.as_i32().unwrap() > 5)
```

### After (v0.10.0)
```liva
// Clean and type-safe! ✨
let data: [i32], err = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n * 2)
let filtered = doubled.filter(n => n > 5)
```

**Benefits:**
- ✅ **No `.unwrap()` chains** - Direct access to typed values
- ✅ **Compile-time validation** - Catch type errors early
- ✅ **Better IDE support** - Autocomplete and type hints
- ✅ **Zero overhead** - Same performance as manual serde usage
- ✅ **Cleaner code** - More readable and maintainable

---

## Basic Usage

### Parsing Primitives

```liva
main() {
    // Integers
    let count: i32, err = JSON.parse("42")
    let bigNum: i64, err = JSON.parse("9223372036854775807")
    let byte: u8, err = JSON.parse("255")
    
    // Floats
    let price: f64, err = JSON.parse("19.99")
    let precise: f32, err = JSON.parse("3.14")
    
    // Booleans
    let active: bool, err = JSON.parse("true")
    
    // Strings
    let name: String, err = JSON.parse("\"Alice\"")
    
    if err == "" {
        print($"All parsed successfully!")
    }
}
```

### Parsing Arrays

```liva
main() {
    // Array of integers
    let numbers: [i32], err = JSON.parse("[1, 2, 3, 4, 5]")
    
    // Array of floats
    let prices: [f64], err = JSON.parse("[19.99, 29.99, 39.99]")
    
    // Array of strings
    let tags: [String], err = JSON.parse("[\"rust\", \"liva\", \"compiler\"]")
    
    // Array of booleans
    let flags: [bool], err = JSON.parse("[true, false, true]")
    
    if err == "" {
        // Process without .unwrap()!
        let doubled = numbers.map(n => n * 2)
        let total = prices.sum()  // Direct operations
    }
}
```

---

## Working with Custom Classes

### Defining Classes for JSON

```liva
// Simple class
User {
    id: u64
    name: String
    email: String
    age: i32
    active: bool
}

// Class with various types
Product {
    id: u32
    name: String
    price: f64
    inStock: bool
    quantity: i32
}

// Classes are automatically serde-compatible when used with JSON.parse()
```

### Parsing Single Objects

```liva
main() {
    let userJson = "{\"id\": 1, \"name\": \"Alice\", \"email\": \"alice@example.com\", \"age\": 30, \"active\": true}"
    let user: User, err = JSON.parse(userJson)
    
    if err == "" {
        print($"User: {user.name}")
        print($"Email: {user.email}")
        print($"Age: {user.age}")
        
        if user.active {
            print("User is active")
        }
    } else {
        print($"Parse error: {err}")
    }
}
```

### Parsing Arrays of Objects

```liva
main() {
    let usersJson = "[
        {\"id\": 1, \"name\": \"Alice\", \"email\": \"alice@example.com\", \"age\": 30, \"active\": true},
        {\"id\": 2, \"name\": \"Bob\", \"email\": \"bob@example.com\", \"age\": 25, \"active\": false}
    ]"
    
    let users: [User], err = JSON.parse(usersJson)
    
    if err == "" {
        print($"Loaded {users.len()} users")
        
        // Process users
        let activeUsers = users.filter(u => u.active)
        let avgAge = users.map(u => u.age).sum() / users.len()
        
        print($"Active users: {activeUsers.len()}")
        print($"Average age: {avgAge}")
    }
}
```

---

## Real-World Example: API Integration

```liva
// Define data structures
Post {
    id: u64
    userId: u32
    title: String
    body: String
}

Comment {
    id: u64
    postId: u64
    name: String
    email: String
    body: String
}

// Fetch and process posts
async fn fetchPosts() {
    let res, err = async HTTP.get("https://jsonplaceholder.typicode.com/posts?_limit=5")
    
    if err != "" {
        print($"HTTP error: {err}")
        return
    }
    
    // Parse directly into typed array! ✨
    let posts: [Post], parseErr = JSON.parse(res.body)
    
    if parseErr != "" {
        print($"Parse error: {parseErr}")
        return
    }
    
    // Process posts with type safety
    print($"Fetched {posts.len()} posts")
    
    // No .unwrap() needed anywhere!
    let longTitles = posts.filter(p => p.title.len() > 50)
    print($"Posts with long titles: {longTitles.len()}")
}

main() {
    async fetchPosts()
}
```

---

## Parallel Processing with Typed JSON

Type-safe JSON parsing integrates perfectly with Liva's parallel processing:

```liva
main() {
    let dataJson = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]"
    let data: [i32], err = JSON.parse(dataJson)
    
    if err == "" {
        // Sequential
        let sequential = data.map(n => n * n)
        
        // Parallel - same clean syntax!
        let parallel = data.parvec().map(n => n * n)
        
        print($"Sequential: {sequential}")
        print($"Parallel: {parallel}")
    }
}
```

---

## Error Handling Best Practices

### Pattern 1: Check and Continue
```liva
let data: [i32], err = JSON.parse(jsonString)

if err == "" {
    // Success path
    processData(data)
} else {
    // Error path
    print($"Failed to parse: {err}")
    // Continue with defaults or alternative logic
}
```

### Pattern 2: Early Return
```liva
fn processJson(jsonString: string) {
    let data: [i32], err = JSON.parse(jsonString)
    
    if err != "" {
        print($"Error: {err}")
        return
    }
    
    // Rest of function works with valid data
    let result = data.map(n => n * 2)
    print($"Result: {result}")
}
```

### Pattern 3: Default Values
```liva
let data: [i32], err = JSON.parse(jsonString)

// On error, data will be [] (empty array)
// Safe to use even if parsing failed
if data.len() > 0 {
    print("Have data to process")
} else {
    print("No data or parse error")
}
```

---

## Type Safety Benefits

### Compile-Time Validation

The semantic analyzer validates types at compile-time:

```liva
// ✅ Valid - i32 is JSON-serializable
let nums: [i32], err = JSON.parse("[1,2,3]")

// ❌ Error - UndefinedClass doesn't exist
let data: [UndefinedClass], err = JSON.parse("[...]")

// ❌ Error - functions aren't JSON-serializable
let funcs: [fn()], err = JSON.parse("[...]")
```

### Runtime Type Matching

JSON structure must match the declared type:

```liva
// ✅ Valid - JSON matches [i32]
let nums: [i32], err = JSON.parse("[1, 2, 3]")
// err == ""

// ❌ Runtime error - JSON is string, not array
let nums: [i32], err = JSON.parse("\"not an array\"")
// err == "JSON parse error: ..."

// ❌ Runtime error - array contains floats, not ints
let nums: [i32], err = JSON.parse("[1.5, 2.7, 3.9]")
// err == "JSON parse error: ..."
```

---

## Performance Considerations

Type-safe JSON parsing has **zero overhead** compared to manual serde usage:

```rust
// What Liva generates (simplified)
let (data, err): (Vec<i32>, String) = match serde_json::from_str::<Vec<i32>>(&json) {
    Ok(v) => (v, String::new()),
    Err(e) => (Vec::new(), format!("{}", e))
};
```

**Key Points:**
- Direct serde deserialization - no intermediate JsonValue
- Stack-allocated tuples - no heap allocations for error handling
- Optimized by Rust compiler - same as hand-written serde code
- No reflection or runtime type checking

---

## Common Patterns

### Pattern 1: HTTP + JSON + Processing
```liva
async fn fetchAndProcess() {
    let res, httpErr = async HTTP.get("https://api.example.com/data")
    
    if httpErr != "" {
        return
    }
    
    let data: [i32], jsonErr = JSON.parse(res.body)
    
    if jsonErr != "" {
        return
    }
    
    let processed = data.parvec().map(n => n * 2)
    print($"Processed: {processed}")
}
```

### Pattern 2: Fallback to Default
```liva
fn loadConfig(configJson: string) {
    let config: Config, err = JSON.parse(configJson)
    
    if err != "" {
        // Use default config
        config = Config {
            host: "localhost",
            port: 8080,
            debug: false
        }
    }
    
    return config
}
```

### Pattern 3: Transform and Re-serialize
```liva
fn transformData(inputJson: string) {
    let data: [i32], err = JSON.parse(inputJson)
    
    if err != "" {
        return ""
    }
    
    let transformed = data.map(n => n * 2)
    let outputJson, err2 = JSON.stringify(transformed)
    
    return outputJson
}
```

---

## Troubleshooting

### Problem: "Type not defined"
```liva
let user: User, err = JSON.parse(json)
// Error: Type 'User' is not defined
```
**Solution:** Define the class before using it:
```liva
User {
    name: String
    age: i32
}

let user: User, err = JSON.parse(json)
```

### Problem: "Parse error" at runtime
```liva
let nums: [i32], err = JSON.parse("[1.5, 2.7]")
// err != "" - floats can't be parsed as integers
```
**Solution:** Match JSON type to Liva type:
```liva
let nums: [f64], err = JSON.parse("[1.5, 2.7]")  // ✅
```

### Problem: Missing fields in JSON
```liva
User {
    name: String
    age: i32
    email: String  // Required!
}

let user: User, err = JSON.parse("{\"name\": \"Alice\", \"age\": 30}")
// err != "" - missing "email" field
```
**Solution:** Make field optional (Phase 3) or provide in JSON:
```liva
let user: User, err = JSON.parse("{\"name\": \"Alice\", \"age\": 30, \"email\": \"\"}")
```

---

## What's Next?

### Phase 2.2: Snake_case Conversion (In Progress)
```liva
User {
    userId: u64      // Liva camelCase
    firstName: String
}

// Will auto-map from JSON snake_case:
// {"user_id": 1, "first_name": "Alice"}
```

### Phase 3: Optional Fields (Planned)
```liva
User {
    name: String
    email?: String  // Optional - can be missing
}
```

### Phase 4: Nested Classes (Planned)
```liva
Address {
    city: String
}

User {
    name: String
    address: Address  // Nested!
}
```

---

## See Also

- [JSON API Reference](../language-reference/json.md)
- [Error Handling Guide](../ERROR_HANDLING_GUIDE.md)
- [Type System Reference](../language-reference/types.md)
- [CHANGELOG](../../CHANGELOG.md) - v0.10.0 release notes

---

**Last Updated:** 2025-01-25  
**Version:** v0.10.0
