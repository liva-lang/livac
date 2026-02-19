# HTTP Client

> **Version:** v0.9.6  
> **Status:** Stable  
> **Phase:** 6.3 - HTTP Client

## Overview

Liva provides a built-in HTTP client with 4 methods: GET, POST, PUT, DELETE. All are **async by default** and use error binding for explicit error handling.

- Async by default with lazy evaluation
- TLS/SSL support via rustls (no OpenSSL dependency)
- 30-second timeout (configurable in future versions)
- Error binding: `let response, err = async HTTP.get(url)`
- Ergonomic JSON parsing: `response.json()` with optional typed parsing

---

## Quick Start

```liva
main() {
    let response, err = async HTTP.get("https://api.example.com/users")
    
    if err != "" {
        console.error($"Request failed: {err}")
        return
    }
    
    print($"Status: {response.status}")
    
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

All HTTP methods share the same return type: `(Option<Response>, string)` — the response if successful (None on error), and an error string (empty on success).

### HTTP.get(url: string)

```liva
let response, err = async HTTP.get("https://httpbin.org/get")

if err != "" {
    console.error($"Error: {err}")
} else {
    print($"Status: {response.status}")
    
    let data, jsonErr = response.json()
    if jsonErr == "" {
        print($"Data: {data}")
    }
}
```

**With typed parsing:**

```liva
ApiResponse {
    url: string
    headers: JsonValue
}

main() {
    let response, err = async HTTP.get("https://httpbin.org/get")
    if err != "" { return }
    
    let data: ApiResponse, jsonErr = response.json()
    if jsonErr == "" {
        print($"Request URL: {data.url}")
    }
}
```

### HTTP.post(url: string, body: string)

```liva
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

### HTTP.put(url, body) / HTTP.delete(url)

PUT and DELETE follow the same pattern as POST and GET respectively:

```liva
// PUT - update existing resource
let updateData, _ = JSON.stringify({ name: "Alice Smith" })
let response, err = async HTTP.put("https://api.example.com/users/123", updateData)

// DELETE - remove resource
let response, err = async HTTP.delete("https://api.example.com/users/123")
```

---

## Response Object

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | `i32` | HTTP status code (200, 404, 500, etc.) |
| `statusText` | `string` | Status text ("OK", "Not Found", etc.) |
| `body` | `string` | Response body as a string |
| `headers` | `[string]` | Response headers as array of strings |

### response.json()

Parse response body as JSON. Returns `(JsonValue, string)` — parsed data and error string.

```liva
let response, httpErr = async HTTP.get("https://api.example.com/posts")
if httpErr != "" { return }

// Untyped parsing (returns JsonValue)
let posts, jsonErr = response.json()

// Typed parsing (automatic deserialization)
let posts: [Post], jsonErr = response.json()
```

**Typed parsing example:**

```liva
User {
    id: u32
    name: string
    email: string
}

main() {
    let response, err = async HTTP.get("https://api.example.com/users")
    if err != "" { return }
    
    let users: [User], jsonErr = response.json()
    if jsonErr != "" { return }
    
    users.forEach(user => {
        print($"User {user.id}: {user.name} <{user.email}>")
    })
}
```

> **Note:** `response.json()` is equivalent to `JSON.parse(response.body)` but more concise.

---

## Error Handling

HTTP operations use Liva's error binding pattern. Common error types:

| Error Type | Example Message |
|------------|----------------|
| Invalid URL | "Invalid URL: ..." |
| DNS Resolution | "DNS resolution failed: ..." |
| Connection Failed | "Connection failed: ..." |
| Timeout | "Request timeout" |
| TLS Error | "TLS handshake failed: ..." |

### Handling Specific Errors

```liva
let response, err = async HTTP.get(url)

if err != "" {
    if err.contains("timeout") {
        print("Request timed out - server too slow")
    } else if err.contains("DNS") {
        print("Domain name not found")
    } else if err.contains("Connection") {
        print("Cannot reach server")
    } else {
        print($"Unknown error: {err}")
    }
    return
}

// Check status codes
if response.status >= 200 && response.status < 300 {
    // Success
} else if response.status >= 400 && response.status < 500 {
    // Client error
} else if response.status >= 500 {
    // Server error
}
```

---

## Async Execution

HTTP methods use **lazy async evaluation**: `async HTTP.get()` returns immediately with a task that executes when the result is first used.

### Parallel Requests

```liva
main() {
    // Create multiple tasks — they execute in parallel when accessed
    let users, err1 = async HTTP.get("https://api.example.com/users")
    let posts, err2 = async HTTP.get("https://api.example.com/posts")
    let comments, err3 = async HTTP.get("https://api.example.com/comments")
    
    if err1 == "" { print($"Users: {users.status}") }
    if err2 == "" { print($"Posts: {posts.status}") }
    if err3 == "" { print($"Comments: {comments.status}") }
}
```

---

## Common Patterns

### REST API Class

```liva
class UserAPI {
    base_url: string
    
    constructor(base_url: string) {
        this.base_url = base_url
    }
    
    getAll() -> (Option<Response>, string) {
        return async HTTP.get($"{this.base_url}/users")
    }
    
    getById(id: i32) -> (Option<Response>, string) {
        return async HTTP.get($"{this.base_url}/users/{id}")
    }
    
    create(userData: string) -> (Option<Response>, string) {
        return async HTTP.post($"{this.base_url}/users", userData)
    }
    
    update(id: i32, userData: string) -> (Option<Response>, string) {
        return async HTTP.put($"{this.base_url}/users/{id}", userData)
    }
    
    delete(id: i32) -> (Option<Response>, string) {
        return async HTTP.delete($"{this.base_url}/users/{id}")
    }
}

main() {
    let api = UserAPI("https://api.example.com")
    
    let userData, _ = JSON.stringify({ name: "Bob" })
    let createResp, createErr = api.create(userData)
    if createErr == "" {
        print($"Created user! Status: {createResp.status}")
    }
    
    let listResp, listErr = api.getAll()
    if listErr == "" {
        print($"Users: {listResp.body}")
    }
}
```

### Typed JSON API

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
    
    let posts: [Post], jsonErr = response.json()
    if jsonErr != "" { return }
    
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
        print($"Success! Status: {response.status}")
    } else {
        console.error($"All retries failed: {err}")
    }
}
```

---

## Best Practices

**Always check errors before using response:**
```liva
let response, err = async HTTP.get(url)
if err != "" {
    console.error($"Error: {err}")
    return
}
```

**Never ignore errors — response may be None:**
```liva
// BAD - may panic if response is None!
let response, _ = async HTTP.get(url)
print(response.status)
```

**Always use async — HTTP methods require it:**
```liva
// BAD - parse error!
let response, err = HTTP.get(url)

// GOOD
let response, err = async HTTP.get(url)
```

---

## Current Limitations

**v0.9.6:**
- No custom headers (planned for v0.9.7)
- No timeout configuration (fixed at 30 seconds)
- No request/response interceptors or middleware
- No streaming, HTTP/2, HTTP/3, or WebSocket support

**Planned (v0.9.7+):** Custom headers, configurable timeout, better error types (enum), streaming, WebSocket.

---

## Technical Details

- **reqwest** 0.11 for HTTP, **rustls** for TLS 1.2/1.3, **tokio** for async runtime
- 30-second timeout applied to entire request (connection + transfer)
- Automatic certificate verification; fails on invalid certificates
- Connection pooling handled by reqwest

---

## See Also

- [Error Handling](error-handling.md) — Error binding patterns
- [JSON API](json.md) — JSON parsing and serialization
- [Concurrency](concurrency.md) — Async/await and parallel execution
- [String Templates](string-templates.md) — URL and body construction
