# WebSocket Module

Synchronous WebSocket client using the `tungstenite` crate.

## Functions

### `WS.connect(url: string) → ws`

Opens a WebSocket connection to the given URL. Supports `ws://` and `wss://` (TLS).

```liva
let ws = WS.connect("wss://echo.websocket.org")
```

### `ws.send(message: string)`

Sends a text message over the connection.

```liva
ws.send("hello")
```

### `ws.recv() → string`

Reads the next text message from the server. Returns an empty string if the message is not text or on error.

```liva
let reply = ws.recv()
print(reply)
```

### `ws.close()`

Closes the WebSocket connection gracefully.

```liva
ws.close()
```

## Example

```liva
main() {
    let ws = WS.connect("wss://echo.websocket.org")
    ws.send("hello liva")
    let reply = ws.recv()
    print(reply)   // hello liva
    ws.close()
}
```

## Notes

- `WS.connect` panics if the connection cannot be established.
- `ws.recv()` is blocking and returns `""` on non-text frames or errors.
- TLS connections (`wss://`) require the system's native TLS library.
