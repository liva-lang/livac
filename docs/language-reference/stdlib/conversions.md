# Type Conversion Functions

Built-in functions for converting between types.

## `parseInt(str: string)` → `(int, string)`

Parse a string into an integer. Returns `(value, error)` tuple.

- On success: `(parsed_int, "")`
- On failure: `(0, "Invalid integer format")`

```liva
let num, err = parseInt("42")
if err {
    print($"Error: {err}")
}

let valid, _ = parseInt("123")      // 123
let negative, _ = parseInt("-456")  // -456
let bad, err = parseInt("abc")      // 0, "Invalid integer format"
```

## `parseFloat(str: string)` → `(float, string)`

Parse a string into a float. Returns `(value, error)` tuple.

- On success: `(parsed_float, "")`
- On failure: `(0.0, "Invalid float format")`

```liva
let pi, err = parseFloat("3.14159")
if err {
    print($"Error: {err}")
}

let f, _ = parseFloat("3.14")    // 3.14
let bad, err = parseFloat("xyz") // 0.0, "Invalid float format"
```

## `toString(value: any)` → `string`

Convert any value to its string representation.

```liva
let s1 = toString(42)      // "42"
let s2 = toString(3.14)    // "3.14"
let s3 = toString(true)    // "true"
```

## Error Handling Pattern

All parse functions use Liva's error binding `(value, error?)`:

```liva
let num, err = parseInt(input)
if err {
    print($"Failed: {err}")
    return
}
print($"Parsed: {num}")
```

Use `or` for concise fallbacks:

```liva
let port = parseInt(portStr) or 3000
```
