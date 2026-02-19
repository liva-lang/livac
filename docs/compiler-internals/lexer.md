# Lexer (Tokenization)

The lexer converts source code into tokens using the **Logos** crate for high-performance lexical analysis.

## Location

**File**: `src/lexer.rs`

## Overview

The lexer performs:
1. **Tokenization**: Converts character stream to tokens
2. **Comment Stripping**: Removes single-line (`//`) and multi-line (`/* */`) comments
3. **Whitespace Handling**: Skips whitespace automatically
4. **Span Tracking**: Records source locations for error reporting

## Token Types

### Keywords (59 tokens)

**Control Flow**:
- `let`, `const`, `if`, `else`, `while`, `for`, `in`, `switch`, `case`, `default`
- `return`, `throw`, `try`, `catch`

**Concurrency**:
- `async`, `parallel`, `par`, `task`, `fire`, `await`
- `move`, `seq`, `vec`, `parvec`, `with`

**Data-Parallel Policies**:
- `ordered`, `chunk`, `threads`, `simdWidth`
- `prefetch`, `reduction`, `schedule`, `detect`
- `auto`, `safe`, `fast`, `static`, `dynamic`

**Error Handling**:
- `fail`

**Types**:
- `number`, `float`, `bool`, `char`, `string`, `bytes`
- `type`, `import`, `use`, `rust`, `test`

**Literals**:
- `true`, `false`

**Operators** (word-based):
- `and`, `or`, `not`, `as`

### Operators

**Arithmetic**: `+`, `-`, `*`, `/`, `%`

**Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`

**Logical**: `&&`, `||`, `!`

**Other**: `=`, `.`, `..`, `=>`, `?`, `:`

### Delimiters

`(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`

### Literals

```rust
IntLiteral(i64)         // 42, 100, 1_000_000
FloatLiteral(f64)       // 3.14, 1.5, 0.001
StringLiteral(String)   // "hello", "world"
CharLiteral(char)       // 'a', 'Z', '\n'
StringTemplate(String)  // $"Hello {name}"
```

### Identifiers

```rust
Ident(String)           // public: myVar, calculate
PrivateIdent(String)    // private: _helper, _secret
```

Visibility is determined by leading underscore:
- No prefix: Public
- `_`: Private

## Implementation

### Using Logos

```rust
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]         // Skip whitespace
#[logos(skip r"//[^\n]*")]             // Skip single-line comments
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]  // Skip multi-line comments
pub enum Token {
    #[token("let")]
    Let,
    
    #[token("async")]
    Async,
    
    #[regex(r"[0-9][0-9_]*", parse_int)]
    IntLiteral(i64),
    
    // ... more tokens
}
```

### Key Features

1. **Automatic Whitespace Skipping**: Using `#[logos(skip)]`
2. **Regex Patterns**: For numbers, strings, identifiers
3. **Underscore Separators**: `1_000_000` → `1000000`
4. **String Templates**: `$"Hello {name}"` tokenized separately

### Integer Parsing

```rust
#[regex(r"[0-9][0-9_]*", |lex| {
    lex.slice().replace('_', "").parse::<i64>().ok()
})]
IntLiteral(i64),
```

Supports underscores for readability: `1_000_000`

### Float Parsing

```rust
#[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| {
    lex.slice().replace('_', "").parse::<f64>().ok()
})]
FloatLiteral(f64),
```

### String Literals

```rust
#[regex(r#""([^"\\]|\\.)*""#, |lex| {
    let s = lex.slice();
    let content = &s[1..s.len()-1];  // Strip quotes
    Some(content.to_string())
})]
StringLiteral(String),
```

### String Templates

```rust
#[regex(r#"\$"([^"\\]|\\.)*""#, |lex| {
    let s = lex.slice();
    let content = &s[2..s.len()-1];  // Strip $" and "
    Some(content.to_string())
})]
StringTemplate(String),
```

## TokenWithSpan

```rust
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}
```

Tracks source location for error reporting:
- `span.start`: Start byte offset
- `span.end`: End byte offset
- `span.start_position(map)`: (line, column)

## Public API

```rust
// Tokenize source code
pub fn tokenize(source: &str, file_name: &str) -> Result<Vec<TokenWithSpan>>
```

Returns `Vec<TokenWithSpan>` or `CompilerError` on invalid tokens.

## Error Handling

Lexer errors include:
- **Invalid tokens**: Unrecognized characters
- **Malformed literals**: Unclosed strings, invalid numbers
- **Location info**: Line, column, source snippet

## Summary

- **50+ Keywords**: Including all concurrency and data-parallel policy tokens
- **Logos-based**: High-performance lexing
- **Span Tracking**: For precise error messages
- **Comment Stripping**: Automatic via regex
- **Visibility-Aware**: Identifier prefixes determine visibility

**Next**: [Parser →](parser.md)
