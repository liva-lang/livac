# HTTP Client

> **Version:** v0.9.6  
> **Status:** Stable  
> **Phase:** 6.3 - HTTP Client

## Overview

Liva provides a built-in HTTP client for making web requests. All HTTP methods are **async by default** and use Liva's error binding pattern for explicit error handling.

**Key Features:**
- 🌐 4 HTTP methods: GET, POST, PUT, DELETE
- ⚡ Async by default with lazy evaluation
- 🔒 TLS/SSL support via rustls (no OpenSSL dependency)
- ⏱️ 30-second timeout (configurable in future versions)
- 🎯 Error binding: `let response, err = async HTTP.get(url)`
- 📦 Response object with status, body, headers
- ✨ Ergonomic JSON parsing: `response.json()` (like JavaScript fetch API)
- 🎨 Typed JSON parsing: `let users: [User], err = response.json()`

---

## Quick Start

```liva
main() {
    // Simple GET request
    let response, err = async HTTP.get("https://api.example.com/users")
    
    if err != "" {
        console.error($"Request failed: {err}")
        return
    }
    
    print($"Status: {response.status}")
    
    // ✨ Parse JSON response (ergonomic way)
    let users, jsonErr = response.json()
    
    if jsonErr != "" {
        console.error($"JSON parsing failed: {jsonErr}")
        return
    }
    
    print($"Got {users.length} users")
}
```

---

## HTTP Methods

### HTTP.get()

Make a GET request to retrieve data.

**Signature:**
```liva
HTTP.get(url: string) -> (Option<Response>, string)
```

**Parameters:**
- `url` - The URL to request (must be http:// or https://)

**Returns:**
- Tuple: `(Option<Response>, string)`
  - First element: `Option<Response>` - The response if successful, None if error
  - Second element: `string` - Error message if failed, empty string if success

**Example:**
```liva
let response, err = async HTTP.get("https://httpbin.org/get")

if err != "" {
    console.error($"Error: {err}")
} else {
    print($"Status: {response.status}")
    print($"Headers: {response.headers}")
    
    // ✨ Parse JSON response using response.json()
    let data, jsonErr = response.json()
    if jsonErr == "" {
        print($"Data: {data}")
    }
}
```

**With Typed Parsing:**
```liva
ApiResponse {
    url: string
    headers: JsonValue  // Can use JsonValue for unstructured data
}

main() {
    let response, err = async HTTP.get("https://httpbin.org/get")
    
    if err != "" {
        console.error($"Error: {err}")
        return
    }
    
    // ✨ Typed JSON parsing
    let data: ApiResponse, jsonErr = response.json()
    
    if jsonErr == "" {
        print($"Request URL: {data.url}")
    }
}
```

---

### HTTP.post()

Make a POST request to send data to the server.

**Signature:**
```liva
HTTP.post(url: string, body: string) -> (Option<Response>, string)
```

**Parameters:**
- `url` - The URL to send the request to
- `body` - The request body (typically JSON string)

**Returns:**
- Same as `HTTP.get()`: `(Option<Response>, string)`

**Example:**
```liva
// Create user
let userData, _ = JSON.stringify({
    name: "Alice",
    email: "alice@example.com"
})

let response, err = async HTTP.post(
    "https://api.example.com/users",
    userData
)

if err != "" {
    console.error($"Failed to create user: {err}")
} else {
    print($"User created! Status: {response.status}")
    print($"Response: {response.body}")
}
```

---

### HTTP.put()

Make a PUT request to update existing data.

**Signature:**
```liva
HTTP.put(url: string, body: string) -> (Option<Response>, string)
```

**Parameters:**
- `url` - The URL of the resource to update
- `body` - The updated data (typically JSON string)

**Returns:**
- Same as `HTTP.get()`: `(Option<Response>, string)`

**Example:**
```liva
// Update user
let updateData, _ = JSON.stringify({
    name: "Alice Smith",
    email: "alice.smith@example.com"
})

let response, err = async HTTP.put(
    "https://api.example.com/users/123",
    updateData
)

if err == "" {
    print($"User updated! Status: {response.status}")
}
```

---

### HTTP.delete()

Make a DELETE request to remove data.

**Signature:**
```liva
HTTP.delete(url: string) -> (Option<Response>, string)
```

**Parameters:**
- `url` - The URL of the resource to delete

**Returns:**
- Same as `HTTP.get()`: `(Option<Response>, string)`

**Example:**
```liva
let response, err = async HTTP.delete("https://api.example.com/users/123")

if err != "" {
    console.error($"Failed to delete: {err}")
} else {
    print($"Deleted! Status: {response.status}")
}
```

---

## Response Object

The `Response` object contains information about the HTTP response.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | `i32` | HTTP status code (200, 404, 500, etc.) |
| `statusText` | `string` | Status text ("OK", "Not Found", etc.) |
| `body` | `string` | Response body as a string |
| `headers` | `[string]` | Response headers as array of strings |

### Methods

#### response.json()

Parse the response body as JSON. Returns a tuple for error binding pattern.

**Signature:**
```liva
response.json() -> (JsonValue, string)
```

**Returns:**
- Tuple: `(JsonValue, string)`
  - First element: `JsonValue` - Parsed JSON data (Null if error)
  - Second element: `string` - Error message if parsing failed, empty string if success

**Basic Example:**
```liva
let response, httpErr = async HTTP.get("https://api.example.com/posts")

if httpErr != "" {
    console.error($"HTTP Error: {httpErr}")
    return
}

// ✨ Ergonomic JSON parsing (like JavaScript fetch API)
let posts, jsonErr = response.json()

if jsonErr != "" {
    console.error($"JSON Error: {jsonErr}")
    return
}

// Use the parsed data
print($"Got {posts.length} posts")
print($"First post title: {posts[0]['title']}")
```

**Typed JSON Parsing:**

You can specify a type hint for automatic deserialization into custom classes:

```liva
// Define your data model
User {
    id: u32
    name: string
    email: string
}

main() {
    let response, httpErr = async HTTP.get("https://api.example.com/users")
    
    if httpErr != "" {
        console.error($"HTTP Error: {httpErr}")
        return
    }
    
    // ✨ Typed parsing - automatically deserializes to User array
    let users: [User], jsonErr = response.json()
    
    if jsonErr != "" {
        console.error($"JSON Error: {jsonErr}")
        return
    }
    
    // Access typed fields directly
    print($"First user: {users[0].name} <{users[0].email}>")
    
    // Type-safe iteration
    users.forEach(user => {
        print($"User {user.id}: {user.name}")
    })
}
```

**Comparison with JSON.parse():**

```liva
// Traditional way (still works)
let data1, err1 = JSON.parse(response.body)

// New ergonomic way (recommended)
let data2, err2 = response.json()

// Both produce the same result!
// But response.json() is more concise and follows JavaScript fetch API pattern
```

### Field Access Example

```liva
let response, err = async HTTP.get("https://api.example.com/data")

if err == "" {
    // Check status code
    if response.status == 200 {
        print("Success!")
    } else if response.status == 404 {
        print("Not found")
    } else {
        print($"HTTP error: {response.statusText}")
    }
    
    // Access body
    print($"Body length: {response.body.length}")
    
    // Parse JSON directly from response
    let data, jsonErr = response.json()
    if jsonErr == "" {
        print($"Parsed data: {data}")
    }
    
    // Access headers (future enhancement will allow individual header access)
    print($"Headers count: {response.headers.length}")
}
```

---

## Error Handling

HTTP operations use Liva's error binding pattern. The error string describes what went wrong.

### Common Error Types

| Error Type | Example Message | Description |
|------------|----------------|-------------|
| **Invalid URL** | "Invalid URL: ..." | URL format is incorrect |
| **DNS Resolution** | "DNS resolution failed: ..." | Cannot resolve domain name |
| **Connection Failed** | "Connection failed: ..." | Cannot connect to server |
| **Timeout** | "Request timeout" | Request took longer than 30 seconds |
| **TLS Error** | "TLS handshake failed: ..." | SSL/TLS certificate issue |
| **HTTP Error** | "HTTP error 404: ..." | Server returned error status |

### Error Handling Pattern

```liva
let response, err = async HTTP.get(url)

if err != "" {
    // Handle specific error types
    if err.contains("timeout") {
        print("Request timed out - server too slow")
    } else if err.contains("DNS") {
        print("Domain name not found")
    } else if err.contains("Connection") {
        print("Cannot reach server")
    } else {
        print($"Unknown error: {err}")
    }
    
    return  // Exit early on error
}

// Success - safe to use response
print($"Status: {response.status}")
```

---

## Async Execution

HTTP methods use Liva's **lazy async evaluation**:
- `async HTTP.get()` returns immediately with a task
- The task executes when the result is first used
- Multiple tasks can run in parallel automatically

### Single Request

```liva
main() {
    // Task created but not executed yet
    let response, err = async HTTP.get("https://api.example.com/data")
    
    // Task executes when 'err' is accessed here
    if err != "" {
        console.error($"Error: {err}")
    }
}
```

### Parallel Requests

```liva
main() {
    // Create multiple tasks
    let users, err1 = async HTTP.get("https://api.example.com/users")
    let posts, err2 = async HTTP.get("https://api.example.com/posts")
    let comments, err3 = async HTTP.get("https://api.example.com/comments")
    
    // All three requests execute in parallel when accessed
    if err1 == "" {
        print($"Users: {users.status}")
    }
    if err2 == "" {
        print($"Posts: {posts.status}")
    }
    if err3 == "" {
        print($"Comments: {comments.status}")
    }
}
```

---

## Common Patterns

### REST API CRUD Operations

```liva
class UserAPI {
    base_url: string
    
    constructor(base_url: string) {
        this.base_url = base_url
    }
    
    // GET - Read all
    getAll() -> (Option<Response>, string) {
        return async HTTP.get($"{this.base_url}/users")
    }
    
    // GET - Read one
    getById(id: i32) -> (Option<Response>, string) {
        return async HTTP.get($"{this.base_url}/users/{id}")
    }
    
    // POST - Create
    create(userData: string) -> (Option<Response>, string) {
        return async HTTP.post($"{this.base_url}/users", userData)
    }
    
    // PUT - Update
    update(id: i32, userData: string) -> (Option<Response>, string) {
        return async HTTP.put($"{this.base_url}/users/{id}", userData)
    }
    
    // DELETE - Delete
    delete(id: i32) -> (Option<Response>, string) {
        return async HTTP.delete($"{this.base_url}/users/{id}")
    }
}

main() {
    let api = UserAPI("https://api.example.com")
    
    // Create user
    let userData, _ = JSON.stringify({ name: "Bob" })
    let createResp, createErr = api.create(userData)
    
    if createErr == "" {
        print($"Created user! Status: {createResp.status}")
    }
    
    // Get all users
    let listResp, listErr = api.getAll()
    if listErr == "" {
        print($"Users: {listResp.body}")
    }
}
```

### JSON API Integration

```liva
main() {
    // Fetch JSON data
    let response, err = async HTTP.get("https://api.example.com/data")
    
    if err != "" {
        console.error($"HTTP Error: {err}")
        return
    }
    
    if response.status != 200 {
        console.error($"HTTP {response.status}: {response.statusText}")
        return
    }
    
    // ✨ Parse JSON using response.json() (ergonomic way)
    let data, jsonErr = response.json()
    
    if jsonErr != "" {
        console.error($"JSON Error: {jsonErr}")
        return
    }
    
    // Use the data
    print($"Received data: {data}")
}
```

**With Typed Parsing:**

```liva
Post {
    id: u32
    userId: u32
    title: string
    body: string
}

main() {
    let response, err = async HTTP.get("https://jsonplaceholder.typicode.com/posts")
    
    if err != "" || response.status != 200 {
        console.error("Failed to fetch posts")
        return
    }
    
    // ✨ Typed JSON parsing - automatic deserialization
    let posts: [Post], jsonErr = response.json()
    
    if jsonErr != "" {
        console.error($"JSON Error: {jsonErr}")
        return
    }
    
    // Type-safe access to fields
    print($"Fetched {posts.length} posts")
    
    posts.forEach(post => {
        print($"Post {post.id}: {post.title}")
    })
}
```

### Retry Logic

```liva
fn fetchWithRetry(url: string, maxRetries: i32) -> (Option<Response>, string) {
    let attempt = 0
    
    while attempt < maxRetries {
        let response, err = async HTTP.get(url)
        
        if err == "" {
            return (response, err)
        }
        
        print($"Attempt {attempt + 1} failed: {err}")
        attempt = attempt + 1
    }
    
    return (None, $"Failed after {maxRetries} attempts")
}

main() {
    let response, err = fetchWithRetry("https://api.example.com/data", 3)
    
    if err == "" {
        print($"Success after retries! Status: {response.status}")
    } else {
        console.error($"All retries failed: {err}")
    }
}
```

---

## Best Practices

### ✅ DO

**Always check for errors:**
```liva
let response, err = async HTTP.get(url)
if err != "" {
    // Handle error
    console.error($"Error: {err}")
    return
}
// Use response safely
```

**Use descriptive variable names:**
```liva
let userResponse, userErr = async HTTP.get("/users")
let postResponse, postErr = async HTTP.get("/posts")
```

**Validate status codes:**
```liva
if response.status >= 200 && response.status < 300 {
    // Success
} else if response.status >= 400 && response.status < 500 {
    // Client error
} else if response.status >= 500 {
    // Server error
}
```

**Parse JSON responses:**
```liva
let data, jsonErr = JSON.parse(response.body)
if jsonErr == "" {
    // Use parsed data
}
```

### ❌ DON'T

**Don't ignore errors:**
```liva
// BAD - ignoring error
let response, _ = async HTTP.get(url)
print(response.status)  // May panic if response is None!
```

**Don't use without async:**
```liva
// BAD - HTTP methods must be async
let response, err = HTTP.get(url)  // Parse error!
```

**Don't assume success:**
```liva
// BAD - not checking error
let response, err = async HTTP.get(url)
print(response.body)  // May panic if err != ""
```

---

## Current Limitations

**v0.9.6 Limitations:**
- ❌ No custom headers (planned for v0.9.7)
- ❌ No timeout configuration (fixed at 30 seconds)
- ❌ No request/response interceptors
- ❌ No streaming support
- ❌ No HTTP/2 or HTTP/3
- ❌ No authentication helpers (must set in headers)

**Planned Enhancements (v0.9.7+):**
- Custom headers: `HTTP.get(url, headers: [string])`
- Configurable timeout: `HTTP.setTimeout(60)`
- Better error types (enum instead of string)
- Request/response middleware
- Streaming responses
- WebSocket support

---

## Technical Details

### Dependencies
- **reqwest** 0.11 - HTTP client library
- **rustls** - TLS implementation (no OpenSSL)
- **tokio** - Async runtime

### Timeout
- Default: 30 seconds
- Applied to entire request (connection + transfer)
- Generates error: "Request timeout"

### TLS/SSL
- Uses rustls for TLS 1.2/1.3
- No OpenSSL dependency (pure Rust)
- Automatic certificate verification
- Fails on invalid certificates

### Performance
- Async I/O for non-blocking operations
- Parallel requests when using multiple async calls
- Connection pooling handled by reqwest
- Minimal overhead for Liva runtime

---

## Examples

### Complete CRUD Example

```liva
class TodoAPI {
    base_url: string
    
    constructor(base_url: string) {
        this.base_url = base_url
    }
    
    list() {
        let response, err = async HTTP.get($"{this.base_url}/todos")
        
        if err != "" {
            console.error($"Failed to list todos: {err}")
            return
        }
        
        if response.status == 200 {
            let todos, jsonErr = JSON.parse(response.body)
            if jsonErr == "" {
                print($"Todos: {todos}")
            }
        }
    }
    
    create(title: string) {
        let todoData, _ = JSON.stringify({ title: title, completed: false })
        let response, err = async HTTP.post($"{this.base_url}/todos", todoData)
        
        if err == "" && response.status == 201 {
            print($"Todo created! {response.body}")
        }
    }
    
    update(id: i32, completed: bool) {
        let updateData, _ = JSON.stringify({ completed: completed })
        let response, err = async HTTP.put($"{this.base_url}/todos/{id}", updateData)
        
        if err == "" {
            print($"Todo updated!")
        }
    }
    
    delete(id: i32) {
        let response, err = async HTTP.delete($"{this.base_url}/todos/{id}")
        
        if err == "" && response.status == 204 {
            print("Todo deleted!")
        }
    }
}

main() {
    let api = TodoAPI("https://jsonplaceholder.typicode.com")
    
    api.list()
    api.create("Learn Liva HTTP Client")
    api.update(1, true)
    api.delete(1)
}
```

---

## See Also

- [Error Handling](error-handling.md) - Error binding patterns
- [JSON API](json.md) - JSON parsing and serialization
- [Concurrency](concurrency.md) - Async/await and parallel execution
- [String Templates](string-templates.md) - URL and body construction

---

**Next Steps:**
- Try the [HTTP examples](../../examples/manual-tests/test_http_get.liva)
- Read the [HTTP Client Design](../PHASE_6.3_HTTP_CLIENT_DESIGN.md)
- Explore [JSON integration](json.md)
