# 🏗️ Compiler Architecture

The Liva compiler (`livac`) is a source-to-source compiler that transforms Liva code into Rust code, which is then compiled using Cargo.

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         Liva Source (.liva)                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. LEXER (logos)                                            │
│    - Tokenization                                           │
│    - Produces: TokenWithSpan                                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. PARSER (recursive descent)                               │
│    - Syntax analysis                                        │
│    - Produces: AST (Abstract Syntax Tree)                   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. SEMANTIC ANALYSIS                                        │
│    - Type inference                                         │
│    - Async inference                                        │
│    - Visibility validation                                  │
│    - Symbol resolution                                      │
│    - Produces: Analyzed AST                                 │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. DESUGARING (optional transformations)                    │
│    - AST simplification                                     │
│    - Produces: Simplified AST                               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. IR LOWERING                                              │
│    - Convert AST to typed IR                                │
│    - Produces: IR (Intermediate Representation)             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. CODE GENERATION                                          │
│    - IR → Rust code (primary)                               │
│    - AST → Rust code (fallback for unsupported features)    │
│    - Generates: main.rs, liva_rt.rs, Cargo.toml            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. CARGO BUILD                                              │
│    - Rust compilation                                       │
│    - Dependency resolution                                  │
│    - Produces: Native binary                                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     Executable Binary                        │
└─────────────────────────────────────────────────────────────┘
```

## Pipeline Stages

### 1. Lexer

**Location:** `src/lexer.rs`

**Purpose:** Convert source text into tokens

**Input:** Raw Liva source code (String)

**Output:** `Vec<TokenWithSpan>`

**Key Components:**
- Uses `logos` crate for token recognition
- Tracks source locations (file, line, column)
- Handles keywords, identifiers, literals, operators

**Example:**
```liva
let x = 10
```

Tokens:
```
Token::Let @ 0..3
Token::Ident("x") @ 4..5
Token::Equals @ 6..7
Token::Number(10) @ 8..10
```

**Features:**
- ✅ Keyword recognition
- ✅ String templates (`$"..."`)
- ✅ Operators (`and`, `or`, `not`, `&&`, `||`, `!`)
- ✅ Concurrency keywords (`async`, `par`, `task`, `fire`, `await`)
- ✅ Error handling (`fail`, `,` in bindings)
- ✅ Visibility prefixes (`_`, `__`)

### 2. Parser

**Location:** `src/parser.rs`

**Purpose:** Build Abstract Syntax Tree from tokens

**Input:** `Vec<TokenWithSpan>`

**Output:** `Program` (AST)

**Strategy:** Recursive descent with operator precedence

**AST Node Types:**
- **Program** - Top-level container
- **Function** - Function declarations
- **Class** - Class definitions
- **Statement** - Executable statements
- **Expression** - Value-producing expressions

**Example:**
```liva
add(a, b) => a + b
```

AST:
```rust
Function {
    name: "add",
    params: [
        Param { name: "a", type_: None },
        Param { name: "b", type_: None }
    ],
    return_type: None,
    body: Expression(
        BinaryOp {
            left: Ident("a"),
            op: Plus,
            right: Ident("b")
        }
    )
}
```

**Grammar:**
See `docs_old/Liva_v0.6_EBNF_AST.md` for complete EBNF grammar.

### 3. Semantic Analysis

**Location:** `src/semantic.rs`

**Purpose:** Validate and enrich the AST with type information

**Input:** AST (Program)

**Output:** Analyzed AST + Errors

**Key Analyses:**

#### 3.1 Async Inference

Automatically marks functions as `async` if they contain:
- `async` calls
- `task async` calls
- `await` expressions

```liva
// Automatically inferred as async
fetchUser() {
  let data = async fetchFromAPI()  // Contains async call
  return data
}
```

#### 3.2 Type Inference

Infers types from:
- Literals (`10` → `number`, `3.14` → `float`)
- Operations (`a + b` where `a: number` → result is `number`)
- Function return types
- Variable assignments

#### 3.3 Visibility Validation

Validates access modifiers:
- `public` - No prefix
- `protected` - `_` prefix → `pub(super)` in Rust
- `private` - `__` prefix → no `pub` in Rust

#### 3.4 Symbol Resolution

- Track variable declarations
- Detect undefined variables
- Validate function/class references

**Current State (v0.6):**
- ✅ Async inference fully implemented
- ✅ Visibility validation working
- 🚧 Type checking is permissive (work in progress)
- 🚧 Symbol resolution partially implemented

**Roadmap:**
See `docs_old/refactor_plan.md` for planned improvements:
- Strict type checking
- Complete symbol table
- Cross-module validation
- Generic type inference

### 4. Desugaring (Optional)

**Location:** `src/desugaring.rs`

**Purpose:** Simplify AST by transforming complex constructs

**Input:** AST

**Output:** Simplified AST

**Transformations:**
- Expand string templates into format! calls
- Convert one-liner functions to blocks
- Normalize operator precedence

**Example:**
```liva
// Before desugaring
$"Hello {name}"

// After desugaring
format!("Hello {}", name)
```

**Current State:**
- Most desugaring is done directly in codegen
- This module exists for future optimizations

### 5. IR Lowering

**Location:** `src/ir.rs`, `src/lowering.rs`

**Purpose:** Convert AST to a typed intermediate representation

**Input:** Analyzed AST

**Output:** IR (Internal Representation)

**IR Node Types:**
```rust
pub enum IrItem {
    Function(IrFunction),
    Class(IrClass),
    Const(IrConst),
}

pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParam>,
    pub return_type: IrType,
    pub body: Vec<IrStmt>,
    pub is_async: bool,
}

pub enum IrStmt {
    VarDecl { name: String, type_: IrType, init: IrExpr },
    Assignment { target: String, value: IrExpr },
    Return(IrExpr),
    If { cond: IrExpr, then: Vec<IrStmt>, else_: Option<Vec<IrStmt>> },
    // ...
}

pub enum IrExpr {
    Literal(IrLiteral),
    Variable(String),
    BinaryOp { left: Box<IrExpr>, op: IrBinOp, right: Box<IrExpr> },
    Call { func: String, args: Vec<IrExpr> },
    Async(Box<IrExpr>),
    Par(Box<IrExpr>),
    // ...
}
```

**Benefits:**
- ✅ Type information attached to every node
- ✅ Simplified structure (fewer edge cases)
- ✅ Better foundation for optimizations
- ✅ Easier code generation

**Current State:**
- IR is fully implemented
- Codegen uses IR for supported features
- Falls back to AST for unsupported features

### 6. Code Generation

**Location:** `src/codegen.rs`

**Purpose:** Emit Rust source code

**Input:** IR (primary) or AST (fallback)

**Output:** Rust source files

**Generated Files:**

#### 6.1 main.rs

Main Rust module with all functions and classes:

```rust
// Generated from Liva
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = add(5, 3);
    println!("{}", result);
}
```

#### 6.2 liva_rt.rs (when using concurrency)

Runtime helpers for async/parallel execution:

```rust
pub fn run_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn(f).await.unwrap()
}

pub fn run_parallel<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(f).join().unwrap()
}
```

#### 6.3 Cargo.toml

Dependencies and project configuration:

```toml
[package]
name = "liva_program"
version = "0.1.0"
edition = "2021"

[dependencies]
# Added when async is used:
tokio = { version = "1", features = ["full"] }
```

**Mapping Rules:**

| Liva | Rust |
|------|------|
| `let x = 10` | `let mut x = 10` |
| `const PI = 3.14` | `const PI: f64 = 3.14` |
| `number` | `i32` |
| `float` | `f64` |
| `string` | `String` |
| `async call()` | `liva_rt::run_async(async { call() })` |
| `par call()` | `liva_rt::run_parallel(\|\| call())` |
| `$"Hello {x}"` | `format!("Hello {}", x)` |
| `and`, `or`, `not` | `&&`, `\|\|`, `!` |
| `fail "msg"` | `return Err("msg".to_string())` |

### 7. Cargo Build

**Purpose:** Compile Rust code to native binary

**Process:**
1. Write generated files to `target/liva_build/`
2. Run `cargo build --release` (unless `LIVAC_SKIP_CARGO` is set)
3. Binary available at `target/liva_build/target/release/liva_program`

**Options:**
- `--run`: Also execute the binary after building
- `--verbose`: Print all cargo output
- `LIVAC_SKIP_CARGO=1`: Skip cargo build (testing code generation)

## Error Reporting

**Location:** `src/error.rs`, `src/span.rs`

**Error Codes:**
- **E1xxx**: Lexer errors (invalid tokens)
- **E2xxx**: Parser errors (syntax errors)
- **E0xxx**: Semantic errors (type errors, undefined symbols)
- **E3xxx**: Codegen errors (unsupported features)

**Error Format:**
```
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → test.liva:6:7

     6 │
       │ let x = 20
       │     ^^^
       │

  ⓘ Variable 'x' already defined in this scope

  💡 Consider using a different name
────────────────────────────────────────────────────────────
```

**Features:**
- ✅ Precise source locations
- ✅ Code snippets with visual indicators
- ✅ Helpful suggestions
- ✅ Color output in terminal
- ✅ JSON output for IDE integration

See [Error System](error-system.md) for complete details.

## Module Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library interface
├── ast.rs            # AST definitions (500+ lines)
├── lexer.rs          # Tokenization (300+ lines)
├── parser.rs         # Parsing (1500+ lines)
├── semantic.rs       # Semantic analysis (600+ lines)
├── ir.rs             # IR definitions (400+ lines)
├── lowering.rs       # AST → IR lowering (800+ lines)
├── codegen.rs        # IR/AST → Rust codegen (2000+ lines)
├── desugaring.rs     # AST transformations (300+ lines)
├── error.rs          # Error reporting (400+ lines)
├── span.rs           # Source location tracking (100+ lines)
└── liva_rt.rs        # Runtime helper template (50+ lines)
```

**Total:** ~7,000 lines of Rust code

## Performance

### Compilation Speed

Typical Liva program (100 lines):
- Lexing: < 1ms
- Parsing: < 5ms
- Semantic: < 10ms
- IR Lowering: < 5ms
- Codegen: < 10ms
- Cargo build: 1-5 seconds (first build), < 1 second (incremental)

### Runtime Performance

Generated Rust code has **zero overhead** compared to hand-written Rust:
- Same binary size
- Same execution speed
- Same memory usage

Concurrency overhead:
- `async`: Minimal (Tokio is highly optimized)
- `par`: Thread creation cost (~50µs per thread)

## Testing

**Location:** `tests/`

Test categories:
- **Lexer tests**: Token recognition
- **Parser tests**: AST construction
- **Semantic tests**: Type checking, async inference
- **Codegen tests**: Rust code generation
- **IR tests**: IR construction and lowering
- **Integration tests**: End-to-end compilation

**Snapshot Testing:**
Uses `insta` crate for snapshot testing:
```bash
cargo test                    # Run all tests
cargo insta review           # Review snapshot changes
```

**Example Test:**
```rust
#[test]
fn test_async_inference() {
    let code = r#"
        fetchUser() {
          let data = async fetchFromAPI()
          return data
        }
    "#;
    
    let ast = parse(code);
    let analyzed = analyze(ast);
    
    assert!(analyzed.functions[0].is_async);
}
```

## Build Modes

### Debug Build

```bash
cargo build
```

- Faster compilation
- Larger binaries
- Debug symbols included

### Release Build

```bash
cargo build --release
```

- Slower compilation
- Smaller binaries (~2-3x smaller)
- Full optimizations
- No debug symbols

**Recommendation:** Use release build for `livac` compiler itself.

## Environment Variables

- `RUST_LOG=debug` - Enable debug logging
- `LIVAC_SKIP_CARGO=1` - Skip cargo build step
- `LIVAC_OUTPUT=path` - Override output directory

## Future Improvements

Planned for future versions:

1. **Incremental Compilation**: Only recompile changed functions
2. **Optimizations**: Dead code elimination, constant folding
3. **LLVM Backend**: Direct LLVM IR generation (skip Rust)
4. **JIT Mode**: Execute without full compilation
5. **Debug Info**: Source maps for debugging
6. **Parallel Compilation**: Compile modules in parallel

## See Also

- **[Lexer](lexer.md)** - Tokenization details
- **[Parser](parser.md)** - AST construction
- **[Semantic Analysis](semantic.md)** - Type checking and validation
- **[IR](ir.md)** - Intermediate representation
- **[Code Generation](codegen.md)** - Rust code emission
- **[Error System](error-system.md)** - Error reporting

---

**Next:** [Lexer](lexer.md)
