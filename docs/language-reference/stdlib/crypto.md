# Crypto Module

The `Crypto` module provides cryptographic hashing and Base64 encoding/decoding functions.

**Crates auto-injected:** `sha2 = "0.10"`, `md-5 = "0.10"`, `base64 = "0.22"`

---

## Functions

### Crypto.sha256(input) → `string`

Computes the SHA-256 hash of a string, returned as lowercase hexadecimal.

```liva
let hash = Crypto.sha256("hello world")
print(hash)  // "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
```

### Crypto.md5(input) → `string`

Computes the MD5 hash of a string, returned as lowercase hexadecimal.

```liva
let hash = Crypto.md5("hello")
print(hash)  // "5d41402abc4b2a76b9719d911017c592"
```

> **Note:** MD5 is cryptographically broken. Use SHA-256 for security-sensitive applications.

### Crypto.base64Encode(input) → `string`

Encodes a string to Base64.

```liva
let encoded = Crypto.base64Encode("Hello, World!")
print(encoded)  // "SGVsbG8sIFdvcmxkIQ=="
```

### Crypto.base64Decode(input) → `string, error`

Decodes a Base64 string. **Fallible** — returns error if the input is not valid Base64 or not valid UTF-8.

```liva
let decoded, err = Crypto.base64Decode("SGVsbG8sIFdvcmxkIQ==")
if err {
    print($"Error: {err}")
} else {
    print(decoded)  // "Hello, World!"
}
```

---

## Complete Example

```liva
main() {
    // Hash a password
    let password = "my-secret-password"
    let hash = Crypto.sha256(password)
    print($"SHA-256: {hash}")

    // Encode data for transmission
    let data = "user:password"
    let encoded = Crypto.base64Encode(data)
    print($"Authorization: Basic {encoded}")

    // Decode received data
    let decoded, err = Crypto.base64Decode(encoded)
    if err == "" {
        print($"Decoded: {decoded}")
    }

    // File integrity check
    let content = "important data"
    let checksum = Crypto.sha256(content)
    print($"Checksum: {checksum}")
}
```

---

## Error Handling

| Function | Fallible? | Error pattern |
|----------|-----------|---------------|
| `Crypto.sha256` | No | `let hash = Crypto.sha256(input)` |
| `Crypto.md5` | No | `let hash = Crypto.md5(input)` |
| `Crypto.base64Encode` | No | `let encoded = Crypto.base64Encode(input)` |
| `Crypto.base64Decode` | Yes | `let decoded, err = Crypto.base64Decode(input)` |
