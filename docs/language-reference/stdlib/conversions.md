# Type Conversion Functions

> **Status:** ✅ Complete  
> **Version:** v0.7.0

Built-in functions for converting between types.

---

## 📚 Available Functions (3/3)

### parseInt(str: String) -> (Int, Error?)

Parse a string into an integer.

**Parameters:**
- `str` - The string to parse

**Returns:**
- A tuple `(value, error)` where:
  - `value` is the parsed integer (0 on error)
  - `error` is `None` on success, or an error message on failure

**Examples:**

```liva
let num, err = parseInt("42")
if err == "" {
    print($"Parsed: {num}")
} else {
    print($"Error: {err}")
}

// Valid cases
let valid, _ = parseInt("123")    // valid = 123
let negative, _ = parseInt("-456") // negative = -456

// Invalid cases
let bad1, err1 = parseInt("abc")   // bad1 = 0, err1 = "Invalid integer format"
let bad2, err2 = parseInt("12.5")  // bad2 = 0, err2 = "Invalid integer format"
```

---

### parseFloat(str: String) -> (Float, Error?)

Parse a string into a floating-point number.

**Parameters:**
- `str` - The string to parse

**Returns:**
- A tuple `(value, error)` where:
  - `value` is the parsed float (0.0 on error)
  - `error` is `None` on success, or an error message on failure

**Examples:**

```liva
let pi, err = parseFloat("3.14159")
if err == "" {
    print($"Pi: {pi}")
}

// Valid cases
let float1, _ = parseFloat("3.14")  // float1 = 3.14
let float2, _ = parseFloat("42")    // float2 = 42.0
let negative, _ = parseFloat("-2.5") // negative = -2.5

// Invalid cases
let bad, err = parseFloat("xyz")    // bad = 0.0, err = "Invalid float format"
```

---

### toString(value: Any) -> String

Convert any value to its string representation.

**Parameters:**
- `value` - The value to convert (can be Int, Float, Bool, or any type)

**Returns:**
- A string representation of the value

**Examples:**

```liva
let s1 = toString(42)      // "42"
let s2 = toString(3.14)    // "3.14"
let s3 = toString(true)    // "true"
let s4 = toString(false)   // "false"

// Use in string templates
let num = 123
let str = toString(num)
print($"The number is: {str}")
```

---

## 💡 Usage Patterns

### Safe Parsing with Error Handling

```liva
let input = "123"
let num, err = parseInt(input)
if err != "" {
    print($"Failed to parse '{input}': {err}")
    return
}
print($"Parsed value: {num}")
```

### Converting Between Types

```liva
// String to number
let str = "42"
let num, _ = parseInt(str)

// Number to string
let value = 123
let text = toString(value)

// Float to string
let pi = 3.14159
let piStr = toString(pi)
```

### Batch Processing with Error Checking

```liva
let inputs = ["123", "456", "abc", "789"]
for input in inputs {
    let num, err = parseInt(input)
    if err == "" {
        print($"✓ Parsed {input} = {num}")
    } else {
        print($"✗ Failed to parse '{input}': {err}")
    }
}
```

---

## 📝 Notes

- **Error Binding Pattern**: `parseInt` and `parseFloat` use Liva's error binding pattern with tuples `(value, error?)`.
- **Default Values**: On parse failure, numeric functions return 0 (for `parseInt`) or 0.0 (for `parseFloat`).
- **Error Messages**: 
  - `parseInt`: "Invalid integer format"
  - `parseFloat`: "Invalid float format"
- **toString Compatibility**: Works with all primitive types and can be extended for custom types.

---

## 🧪 Testing

All conversion functions have been tested with:
- ✅ Valid integer strings
- ✅ Valid float strings  
- ✅ Invalid format strings
- ✅ Boolean conversions
- ✅ Numeric conversions

Test file: `test_conversions.liva`

---

## 📝 See Also

- [Math Functions](./math.md)
- [Array Methods](./arrays.md)
- [String Methods](./strings.md)
- [Standard Library Overview](./README.md)
