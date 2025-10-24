# Phase 6.1: JSON API Design & Specification

**Version:** v0.9.3  
**Date:** 2025-01-21  
**Status:** Design Phase  
**Estimated Time:** 4 hours total

---

## 1. Overview

This phase adds JSON parsing and serialization capabilities to Liva, enabling seamless interaction with JSON data from APIs, files, and other sources.

### Goals
- ✅ Parse JSON strings into Liva types
- ✅ Serialize Liva values to JSON strings
- ✅ Proper error handling with error binding
- ✅ Type-safe JSON operations
- ✅ Support all JSON types (null, bool, number, string, array, object)

### Non-Goals (Future Phases)
- ❌ JSON Schema validation (Phase 6.7)
- ❌ JSON Path queries (Phase 6.7)
- ❌ Streaming JSON parser (Phase 7+)
- ❌ Custom serialization hooks (Phase 7+)

---

## 2. API Design

### 2.1 JSON.parse() - Parse JSON String

**Signature:**
```liva
function parse(json: string): any?
```

**Description:**  
Parses a JSON string and returns a Liva value. Returns `none` if parsing fails.

**Usage with Error Binding:**
```liva
let data, err = JSON.parse('{"name": "Alice", "age": 30}')

if err {
    print("Parse error: ${err}")
    fail err
}

print(data.name)  // "Alice"
print(data.age)   // 30
```

**Usage without Error Binding:**
```liva
let data = JSON.parse('{"valid": "json"}')
if data == none {
    print("Invalid JSON")
} else {
    print(data.valid)  // "json"
}
```

**Return Type:**
- `any?` - Returns parsed value or `none` on error
- With error binding: `(any?, Error?)`

---

### 2.2 JSON.stringify() - Serialize to JSON

**Signature:**
```liva
function stringify(value: any): string?
```

**Description:**  
Converts a Liva value to a JSON string. Returns `none` if serialization fails (e.g., circular references, unsupported types).

**Usage:**
```liva
const obj = {
    name: "Bob",
    age: 25,
    active: true,
    tags: ["developer", "designer"]
}

let json, err = JSON.stringify(obj)
if err {
    print("Stringify error: ${err}")
} else {
    print(json)  // '{"name":"Bob","age":25,"active":true,"tags":["developer","designer"]}'
}
```

**Return Type:**
- `string?` - Returns JSON string or `none` on error
- With error binding: `(string?, Error?)`

---

### 2.3 JSON.parseTyped<T>() - Type-Safe Parsing (Future)

**Note:** This requires generics and will be implemented in Phase 6.6 after generics are complete.

```liva
interface User {
    name: string
    age: int
    email: string
}

let user, err = JSON.parseTyped<User>('{"name":"Alice","age":30,"email":"alice@example.com"}')
// Type: (User?, Error?)
```

---

## 3. Type Mapping

### 3.1 JSON → Liva Type Mapping

| JSON Type | Liva Type | Example |
|-----------|-----------|---------|
| `null` | `none` | `null` → `none` |
| `true`/`false` | `bool` | `true` → `true` |
| `number` (int) | `int` | `42` → `42` |
| `number` (float) | `float` | `3.14` → `3.14` |
| `string` | `string` | `"hello"` → `"hello"` |
| `array` | `array<any>` | `[1,2,3]` → `[1, 2, 3]` |
| `object` | `object` (Map) | `{"a":1}` → `{a: 1}` |

**Implementation Notes:**
- Numbers: Detect integer vs float by presence of decimal point
- Objects: Use `HashMap<String, JsonValue>` in Rust, expose as object in Liva
- Arrays: Use `Vec<JsonValue>` in Rust

### 3.2 Liva → JSON Type Mapping

| Liva Type | JSON Type | Example |
|-----------|-----------|---------|
| `none` | `null` | `none` → `null` |
| `bool` | `true`/`false` | `true` → `true` |
| `int` | `number` | `42` → `42` |
| `float` | `number` | `3.14` → `3.14` |
| `string` | `string` | `"hello"` → `"hello"` |
| `array<T>` | `array` | `[1, 2]` → `[1,2]` |
| `object` | `object` | `{a: 1}` → `{"a":1}` |
| Class instance | `object` | Fields as object |
| Functions | **ERROR** | Not serializable |

**Unsupported Types:**
- Functions (throw error)
- Closures (throw error)
- Tasks/Futures (throw error)

---

## 4. Error Handling

### 4.1 Parse Errors

**Error Types:**
- Invalid JSON syntax
- Unexpected end of input
- Invalid escape sequences
- Invalid Unicode escapes
- Number overflow

**Example:**
```liva
let data, err = JSON.parse('{"invalid": }')  // Missing value

if err {
    print(err.message)  // "Unexpected token } at position 12"
}
```

### 4.2 Stringify Errors

**Error Types:**
- Circular references
- Unsupported types (functions, tasks)
- Invalid UTF-8 in strings

**Example:**
```liva
const obj = {}
obj.self = obj  // Circular reference

let json, err = JSON.stringify(obj)
if err {
    print(err.message)  // "Circular reference detected"
}
```

---

## 5. Implementation Strategy

### 5.1 Dependencies

**Rust Crate:** `serde_json`
- Add to `Cargo.toml`: `serde_json = "1.0"`
- Use for parsing and serialization
- Map to Liva's type system

### 5.2 AST Additions

**Option 1: Built-in Functions (Recommended)**
```rust
// In semantic.rs - add to stdlib
"JSON" => {
    "parse" => FunctionType { ... }
    "stringify" => FunctionType { ... }
}
```

**Option 2: Special Expressions**
```rust
pub enum Expr {
    // ... existing variants
    JsonParse(Box<Expr>),     // JSON.parse(expr)
    JsonStringify(Box<Expr>), // JSON.stringify(expr)
}
```

**Decision:** Use **Option 1** (built-in functions) for consistency with existing stdlib (Array, String, Math).

### 5.3 Code Generation

**Parse:**
```rust
// Liva: let data, err = JSON.parse(json_str)
// Rust:
match serde_json::from_str::<serde_json::Value>(&json_str) {
    Ok(value) => {
        let data = Some(convert_json_to_liva(value));
        let err = None;
    }
    Err(e) => {
        let data = None;
        let err = Some(liva_rt::Error::new(&format!("JSON parse error: {}", e)));
    }
}
```

**Stringify:**
```rust
// Liva: let json, err = JSON.stringify(data)
// Rust:
match serde_json::to_string(&convert_liva_to_json(data)) {
    Ok(s) => {
        let json = Some(s);
        let err = None;
    }
    Err(e) => {
        let json = None;
        let err = Some(liva_rt::Error::new(&format!("JSON stringify error: {}", e)));
    }
}
```

### 5.4 Runtime Support

**Add to `liva_rt.rs`:**
```rust
pub mod json {
    use serde_json::Value;

    pub fn parse(s: &str) -> Result<Value, String> {
        serde_json::from_str(s)
            .map_err(|e| format!("JSON parse error: {}", e))
    }

    pub fn stringify(v: &Value) -> Result<String, String> {
        serde_json::to_string(v)
            .map_err(|e| format!("JSON stringify error: {}", e))
    }
}
```

---

## 6. Testing Strategy

### 6.1 Test Cases

**Parse Tests:**
1. Valid primitives (null, bool, number, string)
2. Valid arrays
3. Valid objects
4. Nested structures
5. Invalid JSON (syntax errors)
6. Edge cases (empty object, empty array, unicode)

**Stringify Tests:**
1. Primitives (none, bool, int, float, string)
2. Arrays
3. Objects
4. Nested structures
5. Class instances
6. Unsupported types (functions)

**Example Test:**
```liva
// test_json_parse.liva
let data, err = JSON.parse('{"name": "Alice", "age": 30}')
assert(err == none)
assert(data.name == "Alice")
assert(data.age == 30)

let invalid, err2 = JSON.parse('{invalid}')
assert(invalid == none)
assert(err2 != none)
```

---

## 7. Documentation Plan

### 7.1 Files to Create

1. **`docs/language-reference/json.md`** (400 lines)
   - Complete API reference
   - Type mapping tables
   - Error handling guide
   - Examples

2. **`examples/json_demo.liva`** (100 lines)
   - Parse JSON from string
   - Stringify objects
   - Error handling
   - Real-world API example

3. **Update `CHANGELOG.md`**
   - Add v0.9.3 entry
   - List new features
   - Breaking changes (if any)

4. **Update `ROADMAP.md`**
   - Mark Phase 6.1 as complete
   - Update version to v0.9.3

---

## 8. Iteration Plan (4 hours)

### Iteration 1: Design & API (30 min) ✅ CURRENT
- ✅ Create this design document
- ✅ Define function signatures
- ✅ Define type mappings
- ✅ Plan error handling

### Iteration 2: JSON Lexer (1 hour)
- [ ] Tokenize JSON strings
- [ ] Handle all JSON types
- [ ] Add to `lexer.rs` or create `json_lexer.rs`
- [ ] Unit tests

### Iteration 3: JSON Parser (1.5 hours)
- [ ] Parse tokens to AST
- [ ] Build Liva values from JSON
- [ ] Validate structure
- [ ] Error reporting
- [ ] Integration tests

### Iteration 4: Type Mapping (30 min)
- [ ] JSON → Liva conversion
- [ ] Liva → JSON conversion
- [ ] Handle edge cases
- [ ] Nested structures

### Iteration 5: Code Generation (30 min)
- [ ] Generate Rust code for parse
- [ ] Generate Rust code for stringify
- [ ] Error binding support
- [ ] Runtime integration

### Iteration 6: Tests & Docs (30 min)
- [ ] Comprehensive test suite
- [ ] Example programs
- [ ] Update CHANGELOG
- [ ] Update ROADMAP
- [ ] Create json.md reference

---

## 9. Success Criteria

- ✅ Parse valid JSON strings to Liva types
- ✅ Stringify Liva values to JSON strings
- ✅ Error binding works correctly
- ✅ All JSON types supported
- ✅ Nested structures work
- ✅ All tests pass
- ✅ Documentation complete
- ✅ Examples demonstrate real usage

---

## 10. Future Enhancements (Post-v0.9.3)

### Phase 6.6: Generic JSON Parsing
```liva
let user = JSON.parseTyped<User>(json_str)
// Type-safe, validated against interface
```

### Phase 6.7: JSON Schema Validation
```liva
const schema = {
    type: "object",
    properties: {
        name: { type: "string" },
        age: { type: "number" }
    }
}

let data, err = JSON.parse(json_str, schema)
// Validates against schema
```

### Phase 7+: Streaming JSON
```liva
for chunk in JSON.stream(large_file) {
    process(chunk)
}
```

---

**Next Step:** Start Iteration 2 - JSON Lexer Implementation
