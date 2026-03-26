# HTTP Client

> SKILL.md covers: `HTTP.get/post/put/delete`, `resp.status`/`resp.body`/`resp.json()`, async requirement.
> This file: response object details, error handling patterns, headers, typed JSON parsing.

## Response Object

| Field | Type | Description |
|-------|------|-------------|
| `status` | `i32` | HTTP status code (200, 404, 500…) |
| `statusText` | `string` | Status text ("OK", "Not Found"…) |
| `body` | `string` | Raw response body |
| `headers` | `[string]` | Response headers as string array |

### `response.json()`

Parses `body` as JSON. Returns `(JsonValue, error)`:

```liva
let response, err = async HTTP.get("https://api.example.com/users")
if err { fail $"Request failed: {err}" }

// Untyped
let data, jsonErr = response.json()

// Typed — automatic deserialization into class
let users: [User], jsonErr = response.json()
```

Equivalent to `JSON.parse(response.body)` but more concise.

## Async Requirement

All HTTP methods **must** use `async`:

```liva
// ✅ Correct
let response, err = async HTTP.get(url)

// ❌ Compile error — HTTP methods are async
let response, err = HTTP.get(url)
```

## POST/PUT Body

Body is a string — use `JSON.stringify` for JSON payloads:

```liva
let body = JSON.stringify({ name: "Alice", email: "alice@example.com" })
let response, err = async HTTP.post("https://api.example.com/users", body)
```

## Error Handling Patterns

### Check error first, then status

```liva
let response, err = async HTTP.get(url)
if err {
    // Network-level: DNS, timeout, TLS, connection refused
    print($"Network error: {err.message}")
    return
}

// HTTP-level: check status code
if response.status >= 400 {
    print($"HTTP {response.status}: {response.body}")
    return
}
```

### Common network error messages

| Error type | Message contains |
|------------|-----------------|
| DNS failure | `"DNS"` |
| Timeout | `"timeout"` |
| Connection refused | `"Connection"` |
| TLS error | `"TLS"` |
| Invalid URL | `"Invalid URL"` |

### Propagation with `or fail`

```liva
fetchUser(id: number): User {
    let resp = async HTTP.get($"https://api.example.com/users/{id}") or fail "Request failed"
    if resp.status != 200 { fail $"HTTP {resp.status}" }
    let user: User = resp.json() or fail "Invalid JSON"
    return user
}
```

## Parallel Requests

Lazy async — multiple requests execute concurrently when results are accessed:

```liva
main() {
    let users, err1 = async HTTP.get("https://api.example.com/users")
    let posts, err2 = async HTTP.get("https://api.example.com/posts")
    // Both requests are in flight

    if !err1 { print($"Users: {users.status}") }
    if !err2 { print($"Posts: {posts.status}") }
}
```

## Typed JSON Response

```liva
User { id: u32; name: string; email: string }

main() {
    let response, err = async HTTP.get("https://api.example.com/users")
    if err { return }

    let users: [User], jsonErr = response.json()
    if jsonErr { return }

    for user in users {
        print($"User {user.id}: {user.name}")
    }
}
```

## Limitations

- No custom headers (planned)
- 30-second fixed timeout
- No streaming / WebSocket / HTTP/2
