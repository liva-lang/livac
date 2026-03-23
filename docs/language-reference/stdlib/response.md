# Response Module

The `Response` module provides helper functions for creating HTTP responses in server handlers. Used with the [Server](server.md) module.

## Functions

### `Response.text(content: String): Response`

Creates a plain text response with HTTP 200 OK:

```liva
app.get("/hello", (req) => {
    Response.text("Hello, World!")
})
```

### `Response.json(data: String): Response`

Creates a JSON response with HTTP 200 OK. The data should be a JSON string:

```liva
app.get("/api/status", (req) => {
    Response.json("{ \"status\": \"ok\" }")
})
```

### `Response.status(code: Int): Response`

Creates a response with only a status code (no body):

```liva
app.delete("/items/:id", (req) => {
    Response.status(204)
})
```

Common status codes:
- `200` — OK
- `201` — Created
- `204` — No Content
- `400` — Bad Request
- `404` — Not Found
- `500` — Internal Server Error

## See Also

- [Server Module](server.md) — HTTP server and routing
- [HTTP Module](../../guides/http.md) — HTTP client
