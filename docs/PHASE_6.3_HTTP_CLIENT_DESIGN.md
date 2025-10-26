# Phase 6.3: HTTP Client Implementation (v0.9.6)

> **Version:** v0.9.6  
> **Status:** ✅ COMPLETED  
> **Started:** 2025-01-24  
> **Completed:** 2025-10-26  
> **Total Time:** 5 hours  
> **Branch:** `feature/http-client-v0.9.6`

---

## 📋 Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Design Decisions](#design-decisions)
- [API Design](#api-design)
- [Implementation Plan](#implementation-plan)
- [Error Handling](#error-handling)
- [Testing Strategy](#testing-strategy)
- [Documentation Plan](#documentation-plan)
- [Timeline](#timeline)

---

## 🎯 Overview

Implement a simple, ergonomic HTTP client for Liva that integrates seamlessly with the language's async/await system and error binding pattern.

**Key Features:**
- 4 HTTP methods: GET, POST, PUT, DELETE
- Request headers support
- Request body support (JSON, text)
- Async by default (uses Liva's async/await)
- Error binding integration (`let response, err = HTTP.get(...)`)
- Response parsing (status, headers, body)

**Why Now?**
- ✅ JSON parsing already implemented (v0.9.3)
- ✅ File I/O already implemented (v0.9.4)
- ✅ Async/await system mature and tested
- ✅ Error binding pattern proven and ergonomic
- 🎯 Completes the "practical web stack" trilogy

---

## 🎯 Goals

### Primary Goals
1. **Simple API**: Developers can make HTTP requests in 1-2 lines
2. **Type Safety**: Clear response types and error handling
3. **Async Native**: All operations are async by default
4. **Error Binding**: Consistent with Liva's error handling philosophy
5. **Production Ready**: Handles timeouts, errors, edge cases

### Non-Goals (Defer to Future)
- ❌ Streaming responses (v0.9.7+)
- ❌ WebSocket support (v0.10.0+)
- ❌ Custom retry logic (v0.9.7+)
- ❌ Cookie management (v0.9.7+)
- ❌ Certificate pinning (v1.0.0+)

---

## 🎨 Design Decisions

### 1. Namespace: `HTTP.*`
**Decision:** Use `HTTP` namespace (like `JSON`, `File`, `Math`)

**Rationale:**
- Consistent with existing stdlib namespaces
- Clear and discoverable
- Avoids conflicts with user code

**Alternatives Considered:**
- `http.*` (lowercase) - Rejected: inconsistent with JSON, File
- `HttpClient.get()` - Rejected: too verbose
- Top-level functions `httpGet()` - Rejected: pollutes global namespace

### 2. Async by Default
**Decision:** All HTTP functions return async tasks

**Rationale:**
- HTTP operations are inherently I/O-bound
- Consistent with Liva's async philosophy
- Prevents blocking main thread

**Usage:**
```liva
main() {
    let response, err = async HTTP.get("https://api.example.com")
    if err != "" {
        print($"Error: {err}")
        return
    }
    print($"Status: {response.status}")  // Auto-awaits here on first use
}
```

### 3. Error Binding Pattern
**Decision:** Use error binding tuple return `(Response?, Error?)`

**Rationale:**
- Consistent with JSON.parse(), File.read()
- Explicit error handling (no exceptions)
- Type-safe and ergonomic

**Error Cases:**
- Network errors (connection refused, timeout)
- DNS resolution failures
- Invalid URL format
- TLS/SSL errors
- HTTP errors (4xx, 5xx) - Still return response!

**Important:** 4xx/5xx responses are NOT errors. They return valid Response objects.

### 4. Response Type
**Decision:** Return a simple Response object with essential fields

**Structure:**
```liva
Response {
    status: int          // HTTP status code (200, 404, etc.)
    statusText: string   // "OK", "Not Found", etc.
    body: string         // Response body as string
    headers: [string]    // Array of "Key: Value" strings (simplified)
}
```

**Rationale:**
- Simple to use (no complex types yet)
- JSON parsing is separate: `JSON.parse(response.body)`
- Headers as array of strings (simple, no HashMap yet)

### 5. Request Configuration
**Decision:** Use optional parameters for headers and body

**Signatures:**
```liva
// Simple GET/DELETE (no body)
HTTP.get(url: string): (Response?, string?)
HTTP.delete(url: string): (Response?, string?)

// POST/PUT with body
HTTP.post(url: string, body: string): (Response?, string?)
HTTP.put(url: string, body: string): (Response?, string?)

// With headers (future enhancement)
HTTP.get(url: string, headers: [string]): (Response?, string?)
HTTP.post(url: string, body: string, headers: [string]): (Response?, string?)
```

**For v0.9.6:** Start with simple versions (no headers parameter)  
**For v0.9.7:** Add headers support

---

## 🔧 API Design

### HTTP.get(url: string): (Response?, string?)

**Purpose:** Perform HTTP GET request

**Parameters:**
- `url`: string - The URL to request (must start with http:// or https://)

**Returns:**
- Tuple: `(Response?, string?)`
  - Success: `(Response, "")`
  - Failure: `("", "error message")`

**Example:**
```liva
main() {
    let response, err = async HTTP.get("https://api.github.com/users/octocat")
    
    if err != "" {
        console.error($"Request failed: {err}")
        return
    }
    
    print($"Status: {response.status}")  // Auto-awaits here
    print($"Body: {response.body}")
    
    // Parse JSON response
    let user, jsonErr = JSON.parse(response.body)
    if jsonErr == "" {
        print($"User: {user}")
    }
}
```

### HTTP.post(url: string, body: string): (Response?, string?)

**Purpose:** Perform HTTP POST request with body

**Parameters:**
- `url`: string - The URL to request
- `body`: string - Request body (often JSON)

**Returns:**
- Tuple: `(Response?, string?)`

**Example:**
```liva
main() {
    let jsonBody = JSON.stringify({
        "name": "John Doe",
        "email": "john@example.com"
    })
    
    let response, err = async HTTP.post(
        "https://api.example.com/users",
        jsonBody
    )
    
    if err != "" {
        console.error($"Failed to create user: {err}")
        return
    }
    
    print($"User created! Status: {response.status}")  // Auto-awaits here
}
```

### HTTP.put(url: string, body: string): (Response?, string?)

**Purpose:** Perform HTTP PUT request (update resource)

**Parameters:**
- `url`: string - The URL to request
- `body`: string - Request body

**Returns:**
- Tuple: `(Response?, string?)`

**Example:**
```liva
main() {
    let updates = JSON.stringify({"name": "Jane Doe"})
    
    let response, err = async HTTP.put(
        "https://api.example.com/users/123",
        updates
    )
    
    if err == "" {
        print("User updated successfully!")  // Auto-awaits here
    }
}
```

### HTTP.delete(url: string): (Response?, string?)

**Purpose:** Perform HTTP DELETE request

**Parameters:**
- `url`: string - The URL to request

**Returns:**
- Tuple: `(Response?, string?)`

**Example:**
```liva
main() {
    let response, err = async HTTP.delete(
        "https://api.example.com/users/123"
    )
    
    if err == "" && response.status == 204 {
        print("User deleted successfully!")  // Auto-awaits here
    }
}
```

---

## 🛠️ Implementation Plan

### Phase 1: Setup & Dependencies (30 min)

**Tasks:**
1. Add `reqwest` crate to runtime dependencies
   - `reqwest = { version = "0.11", features = ["json"] }`
2. Add tokio runtime support (already exists for async)
3. Create Response struct in liva_rt.rs

**Files to Modify:**
- `src/liva_rt.rs` - Add Response struct and HTTP functions
- Runtime Cargo.toml template - Add reqwest dependency

**Response Struct:**
```rust
pub struct LivaHttpResponse {
    pub status: i32,
    pub status_text: String,
    pub body: String,
    pub headers: Vec<String>,
}
```

### Phase 2: Runtime Functions (1.5 hours)

**Implement 4 HTTP functions in liva_rt.rs:**

1. **`liva_http_get(url: String) -> (Option<LivaHttpResponse>, String)`**
   ```rust
   pub async fn liva_http_get(url: String) -> (Option<LivaHttpResponse>, String) {
       // Validate URL
       // Create reqwest client with timeout (30s default)
       // Perform GET request
       // Handle errors (network, DNS, timeout)
       // Parse response
       // Return (Some(response), "") or (None, error_message)
   }
   ```

2. **`liva_http_post(url: String, body: String) -> (Option<LivaHttpResponse>, String)`**
   - Similar to GET but with body
   - Set Content-Type: application/json (default)

3. **`liva_http_put(url: String, body: String) -> (Option<LivaHttpResponse>, String)`**
   - Similar to POST

4. **`liva_http_delete(url: String) -> (Option<LivaHttpResponse>, String)`**
   - Similar to GET

**Error Handling:**
- Invalid URL format: "Invalid URL format"
- Connection refused: "Connection refused"
- Timeout: "Request timeout (30s)"
- DNS error: "DNS resolution failed"
- TLS error: "SSL/TLS error"
- Generic: Include reqwest error message

**Response Parsing:**
- Extract status code
- Extract status text (OK, Not Found, etc.)
- Read body as string (UTF-8)
- Parse headers into "Key: Value" strings

### Phase 3: Semantic Analysis (30 min)

**Tasks:**
1. Detect HTTP.* calls in semantic analyzer
2. Mark as fallible (returns error tuple)
3. Mark as async (requires await)
4. Validate:
   - First arg is string (URL)
   - POST/PUT have second string arg (body)

**Files to Modify:**
- `src/semantic.rs` - Add HTTP function detection

**Detection Logic:**
```rust
// In validate_call_expr()
if let Expr::MemberAccess { object, member } = &*call.callee {
    if let Expr::Identifier(name) = &**object {
        if name == "HTTP" {
            match member.as_str() {
                "get" | "post" | "put" | "delete" => {
                    // Mark as async and fallible
                    self.async_functions.insert(format!("HTTP.{}", member));
                    self.fallible_functions.insert(format!("HTTP.{}", member));
                }
                _ => {
                    return Err(CompilerError::SemanticError(
                        format!("Unknown HTTP method: {}", member).into()
                    ));
                }
            }
        }
    }
}
```

### Phase 4: Code Generation (1 hour)

**Tasks:**
1. Detect HTTP.* calls in codegen
2. Generate appropriate Rust code
3. Include runtime functions in generated code

**Files to Modify:**
- `src/codegen.rs` - Add HTTP call generation

**Code Generation Examples:**

**Liva:**
```liva
let response, err = async HTTP.get("https://api.github.com")
```

**Generated Rust:**
```rust
// Spawn async task (lazy evaluation)
let response_task = tokio::spawn(async move {
    liva_http_get("https://api.github.com".to_string()).await
});

// Auto-await on first use of response or err
let (response_opt, err_str) = response_task.await.unwrap();
let response = if let Some(r) = response_opt {
    r
} else {
    LivaHttpResponse {
        status: 0,
        status_text: String::new(),
        body: String::new(),
        headers: Vec::new(),
    }
};
let err = err_str;
```

**HTTP.post Generation:**
```rust
// Liva: HTTP.post(url, body)
// Rust: liva_http_post(url, body).await
```

**Access Pattern:**
```liva
response.status      → response.status
response.body        → response.body.clone()
response.headers     → response.headers.clone()
```

### Phase 5: Testing (1 hour)

**Test Files:**

1. **test_http_get.liva** - Basic GET request
   ```liva
   main() {
       let response, err = async HTTP.get("https://httpbin.org/get")
       if err != "" {
           print($"Error: {err}")
           return
       }
       print($"Status: {response.status}")  // Auto-awaits here
       print($"Body length: {response.body.length}")
   }
   ```

2. **test_http_post.liva** - POST with JSON
   ```liva
   main() {
       let data = JSON.stringify({"name": "Test", "value": 42})
       let response, err = async HTTP.post("https://httpbin.org/post", data)
       
       if err != "" {
           console.error($"POST failed: {err}")
           return
       }
       
       print($"Status: {response.status}")  // Auto-awaits here
       let result, jsonErr = JSON.parse(response.body)
   }
   ```

3. **test_http_put.liva** - PUT request

4. **test_http_delete.liva** - DELETE request

5. **test_http_error.liva** - Error handling
   ```liva
   main() {
       // Invalid URL
       let r1, e1 = async HTTP.get("not-a-url")
       print($"Invalid URL error: {e1}")  // Auto-awaits here
       
       // Connection refused
       let r2, e2 = async HTTP.get("http://localhost:99999")
       print($"Connection error: {e2}")  // Auto-awaits here
       
       // 404 response (NOT an error!)
       let r3, e3 = async HTTP.get("https://httpbin.org/status/404")
       if e3 == "" {
           print($"404 status code: {r3.status}")  // Auto-awaits here
       }
   }
   ```

6. **test_http_json_integration.liva** - HTTP + JSON workflow
   ```liva
   main() {
       // Fetch user data
       let response, err = async HTTP.get("https://jsonplaceholder.typicode.com/users/1")
       
       if err != "" {
           console.error($"Failed to fetch user: {err}")
           return
       }
       
       // Parse JSON
       let user, jsonErr = JSON.parse(response.body)
       if jsonErr != "" {
           console.error($"Failed to parse JSON: {jsonErr}")
           return
       }
       
       print($"User loaded: {user}")  // Auto-awaits on HTTP call above
   }
   ```

**Test Strategy:**
- Use httpbin.org for testing (reliable, public API)
- Test happy paths (200 OK)
- Test error cases (invalid URL, timeout, 404)
- Test integration with JSON parsing
- Test all 4 HTTP methods

---

## 🚨 Error Handling

### Error Categories

#### 1. **URL Validation Errors**
- **Code:** E9001
- **Message:** "Invalid URL format: {url}"
- **Example:** `HTTP.get("not-a-url")`
- **Recovery:** Check URL starts with http:// or https://

#### 2. **Network Errors**
- **Code:** E9002
- **Message:** "Network error: {details}"
- **Examples:**
  - "Connection refused"
  - "DNS resolution failed"
  - "Request timeout (30s)"
- **Recovery:** Check network connectivity, verify URL

#### 3. **TLS/SSL Errors**
- **Code:** E9003
- **Message:** "SSL/TLS error: {details}"
- **Recovery:** Check certificate validity, try http:// for testing

#### 4. **HTTP Method Errors** (Compile-time)
- **Code:** E0902
- **Message:** "Unknown HTTP method: {method}"
- **Example:** `HTTP.patch(...)` (not implemented yet)
- **Recovery:** Use GET, POST, PUT, or DELETE

### Error Messages Design

**Good Error Messages:**
```
Error: Invalid URL format: "not-a-url"
Hint: URLs must start with http:// or https://
Example: HTTP.get("https://api.example.com/data")

Error: Connection refused: localhost:99999
Hint: Check if the server is running and the port is correct

Error: Request timeout after 30 seconds
Hint: The server may be slow or unreachable. Consider retrying.
```

---

## 📚 Documentation Plan

### 1. Language Reference (docs/language-reference/http.md)

**Sections:**
- Overview
- HTTP.get() - GET requests
- HTTP.post() - POST requests
- HTTP.put() - PUT requests
- HTTP.delete() - DELETE requests
- Response object structure
- Error handling patterns
- Integration with JSON
- Best practices
- Common patterns (CRUD operations)
- Troubleshooting

**Estimated:** 500-600 lines

### 2. Examples (examples/http/)

**Files:**
- `basic_get.liva` - Simple GET request
- `post_json.liva` - POST with JSON data
- `crud_operations.liva` - Full CRUD example
- `error_handling.liva` - Comprehensive error handling
- `api_client.liva` - Build a simple API client class

### 3. CHANGELOG.md

**Entry for v0.9.6:**
```markdown
## [0.9.6] - 2025-01-24

### Added - HTTP Client (Phase 6.3 - 4h)

**HTTP API:**
- `HTTP.get(url: string): (Response?, Error?)` - Perform GET request
- `HTTP.post(url, body: string): (Response?, Error?)` - Perform POST request
- `HTTP.put(url, body: string): (Response?, Error?)` - Perform PUT request
- `HTTP.delete(url: string): (Response?, Error?)` - Perform DELETE request

**Response Object:**
- `status: int` - HTTP status code (200, 404, etc.)
- `statusText: string` - Status text ("OK", "Not Found")
- `body: string` - Response body as string
- `headers: [string]` - Response headers as array

**Features:**
- Async by default (uses await)
- Error binding integration
- 30-second default timeout
- Comprehensive error messages
- JSON integration (parse response bodies)

**Examples:**
\`\`\`liva
main() {
    let response, err = async HTTP.get("https://api.example.com/data")
    if err != "" {
        console.error($"Request failed: {err}")
        return
    }
    
    let data, jsonErr = JSON.parse(response.body)
    print($"Data: {data}")  // Auto-awaits on response access
}
\`\`\`

**Future Enhancements (v0.9.7+):**
- Custom headers support
- Request timeout configuration
- Streaming responses
- Retry logic
```

### 4. ROADMAP.md Update

Mark Phase 6.3 as complete, update statistics

---

## ⏱️ Timeline

### Hour 1: Setup & Runtime Foundation (✅ COMPLETE)
- [x] Create branch `feature/http-client-v0.9.6`
- [x] Add reqwest dependency
- [x] Create Response struct in liva_rt.rs
- [x] Implement liva_http_get() runtime function
- [x] Basic error handling and timeout

### Hour 2: Complete Runtime Implementation (✅ COMPLETE)
- [x] Implement liva_http_post()
- [x] Implement liva_http_put()
- [x] Implement liva_http_delete()
- [x] Refactor common code (DRY)
- [x] Comprehensive error handling

### Hour 3: Compiler Integration (✅ COMPLETE)
- [x] Add HTTP detection in semantic analyzer
- [x] Mark as async and fallible
- [x] Generate code in codegen.rs
- [x] Handle Response object access
- [x] Basic test (compile + run)

### Hour 4: Testing & Documentation (✅ COMPLETE)
- [x] Create 5 integration tests (GET, POST, PUT, DELETE, errors)
- [x] Create 3 comprehensive examples
- [x] Run all tests (httpbin.org)
- [x] Write http.md documentation
- [x] Update CHANGELOG.md
- [x] Update ROADMAP.md
- [x] Git commit and merge

---

## 📊 Success Criteria

### Functional Requirements
✅ All 4 HTTP methods work (GET, POST, PUT, DELETE)  
✅ Async/await integration works  
✅ Error binding returns correct values  
✅ Response object has all fields  
✅ 30-second timeout works  
✅ Network errors handled gracefully  
✅ 4xx/5xx responses return Response (not error)  

### Code Quality
✅ Zero compiler warnings  
✅ All tests passing  
✅ Code formatted (cargo fmt)  
✅ Comprehensive error messages  

### Documentation
✅ Complete language reference (500+ lines)  
✅ Working examples (6 files)  
✅ CHANGELOG updated  
✅ ROADMAP updated  

### Integration
✅ Works with JSON.parse()  
✅ Works with File.write() (save responses)  
✅ Works in async main()  
✅ Error binding pattern consistent  

---

## 🔄 Future Enhancements (v0.9.7+)

### Custom Headers (v0.9.7)
```liva
let headers = ["Authorization: Bearer token", "Content-Type: application/json"]
let response, err = async HTTP.get(url, headers)
```

### Timeout Configuration (v0.9.7)
```liva
HTTP.setTimeout(60)  // 60 seconds
let response, err = async HTTP.get(url)
```

### Streaming Responses (v0.10.0)
```liva
let stream, err = async HTTP.stream(url)
stream.onChunk((chunk) => {
    print($"Received: {chunk}")
})
```

### Retry Logic (v0.9.7)
```liva
HTTP.setRetries(3)  // Retry up to 3 times
let response, err = async HTTP.get(url)
```

### Request Builder Pattern (v0.10.0)
```liva
let request = HTTP.Request()
    .url("https://api.example.com")
    .method("POST")
    .header("Authorization", "Bearer token")
    .body(jsonData)
    .timeout(60)

let response, err = async request.send()
```

---

## 📝 Notes

- **httpbin.org** is used for testing (reliable, public API for testing HTTP clients)
- **Timeout default:** 30 seconds (prevents hanging)
- **No cookies yet:** Deferred to v0.9.7+
- **No connection pooling:** reqwest handles this internally
- **UTF-8 only:** All request/response bodies are UTF-8 strings
- **No binary support yet:** Deferred to v0.10.0+

---

## 🎉 Impact

After Phase 6.3, Liva developers will have a **complete web stack**:

```liva
main() {
    // 1. Fetch data from API (async, auto-awaits on first use)
    let response, httpErr = async HTTP.get("https://api.example.com/data")
    if httpErr != "" {
        console.error($"HTTP error: {httpErr}")
        return
    }
    
    // 2. Parse JSON response (auto-awaits here when accessing response.body)
    let data, jsonErr = JSON.parse(response.body)
    if jsonErr != "" {
        console.error($"JSON error: {jsonErr}")
        return
    }
    
    // 3. Save to file
    let dataStr = JSON.stringify(data)
    let saved, fileErr = File.write("data.json", dataStr)
    if fileErr != "" {
        console.error($"File error: {fileErr}")
        return
    }
    
    print("✓ Data fetched, parsed, and saved!")
}
```

**This unlocks:**
- 🌐 API clients
- 🤖 Web scrapers
- 🔗 Microservices
- 📊 Data pipelines

---

## ✅ Completion Summary

**Delivered Features:**
- ✅ 4 HTTP methods fully implemented (GET, POST, PUT, DELETE)
- ✅ Async by default with error binding pattern
- ✅ 30-second timeout with comprehensive error handling
- ✅ Response struct with status, statusText, body, headers
- ✅ 5 integration tests (all passing)
- ✅ 3 comprehensive examples with POST and PUT
- ✅ Complete runtime implementation (150+ lines)
- ✅ Semantic analysis integration (120+ lines)
- ✅ Code generation (300+ lines)

**Test Coverage:**
- `tests/integration/proj_http/test_get.liva` - GET requests
- `tests/integration/proj_http/test_post.liva` - POST with JSON body
- `tests/integration/proj_http/test_put.liva` - PUT with JSON body
- `tests/integration/proj_http/test_delete.liva` - DELETE requests
- `tests/integration/proj_http/test_errors.liva` - Error handling

**Examples Created:**
- `examples/http-json/example_http_post.liva` - POST demonstration
- `examples/http-json/example_http_put.liva` - PUT demonstration
- `examples/http-json/example_http_all_methods.liva` - All methods demo

**Metrics:**
- Total implementation time: 5 hours
- Files created: 8 (5 tests + 3 examples)
- Tests passing: 5/5 ✅
- Documentation pages: 1 (800+ lines)
- Code quality: Zero warnings in HTTP implementation
- 🧪 Integration tests

---

**Document Version:** 1.0  
**Author:** Liva Team  
**Last Updated:** 2025-01-24
