# Server Module

The `Server` module provides HTTP server functionality for building REST APIs and web services. It generates code using the [axum](https://docs.rs/axum) framework.

## Creating a Server

```liva
let app = Server.create()
```

Creates a new HTTP server (axum Router). The variable is automatically tracked as mutable to allow adding routes.

## Registering Routes

### GET

```liva
app.get("/hello", (req) => {
    Response.text("Hello, World!")
})
```

### POST

```liva
app.post("/users", (req) => {
    let body = req.body
    Response.json(body)
})
```

### PUT

```liva
app.put("/users/:id", (req) => {
    let id = req.params.get("id")
    Response.text("Updated " + id)
})
```

### DELETE

```liva
app.delete("/users/:id", (req) => {
    let id = req.params.get("id")
    Response.status(204)
})
```

## Path Parameters

Use `:param` syntax in route paths. Access parameters via `req.params.get("name")`:

```liva
app.get("/users/:id/posts/:postId", (req) => {
    let userId = req.params.get("id")
    let postId = req.params.get("postId")
    Response.json("{ \"userId\": \"" + userId + "\", \"postId\": \"" + postId + "\" }")
})
```

Parameters are extracted as strings from the URL path.

## Request Body

Access the raw request body via `req.body` (available for POST and PUT handlers):

```liva
app.post("/data", (req) => {
    let body = req.body
    Response.text("Received: " + body)
})
```

## Response Helpers

### `Response.text(content)`

Returns a plain text response with status 200:

```liva
Response.text("Hello!")
// → (StatusCode::OK, "Hello!".to_string())
```

### `Response.json(data)`

Returns a JSON response with status 200:

```liva
Response.json("{ \"message\": \"ok\" }")
// → (StatusCode::OK, Json(json!(...)))
```

### `Response.status(code)`

Returns a response with only a status code:

```liva
Response.status(204)
// → StatusCode::from_u16(204)
```

## Starting the Server

```liva
app.listen(3000)
print("Server running on :3000")
```

Starts listening on the specified port. This call blocks (runs the tokio event loop). The `main()` function is automatically made async when `Server` is used.

## Complete Example

```liva
main() {
    let app = Server.create()

    app.get("/", (req) => {
        Response.json("{ \"message\": \"Hello from Liva!\" }")
    })

    app.get("/users/:id", (req) => {
        let id = req.params.get("id")
        Response.json("{ \"id\": \"" + id + "\" }")
    })

    app.post("/users", (req) => {
        let body = req.body
        Response.text("Created: " + body)
    })

    app.delete("/users/:id", (req) => {
        Response.status(204)
    })

    app.listen(3000)
    print("Server running on :3000")
}
```

## Generated Code

The server compiles to axum with tokio:

| Liva | Rust |
|------|------|
| `Server.create()` | `axum::Router::new()` |
| `app.get(path, handler)` | `app.route(path, axum::routing::get(\|\| async { ... }))` |
| `app.listen(port)` | `tokio::net::TcpListener::bind(addr).await; axum::serve(...)` |
| `req.params.get("key")` | `__params.get(&"key").cloned().unwrap_or_default()` |
| `req.body` | `body.clone()` |
| `Response.text(s)` | `(StatusCode::OK, s.to_string())` |
| `Response.json(s)` | `(StatusCode::OK, Json(json!(s)))` |
| `Response.status(n)` | `StatusCode::from_u16(n)` |

## Auto-injected Dependencies

When `Server` is used, the following crate is automatically added to `Cargo.toml`:

```toml
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

Tokio is always included in Liva projects; axum is added only when `Server` is used.
