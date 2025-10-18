# Parser (Syntax Analysis)

The parser converts tokens into an **Abstract Syntax Tree (AST)** using recursive descent parsing.

## Location

**File**: `src/parser.rs` (1,754 lines)

## Overview

The parser performs:
1. **Token Stream Processing**: Consumes tokens from lexer
2. **AST Construction**: Builds typed AST nodes
3. **Syntax Validation**: Checks grammar rules
4. **Error Recovery**: Reports syntax errors with context

## Architecture

### Recursive Descent Parser

```rust
pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    current: usize,
    source_file: String,
    source_code: String,
    source_map: SourceMap,
}
```

**Key Characteristics**:
- **Top-down parsing**: Starts from `Program` → `TopLevel` → ... → `Expr`
- **Predictive**: Looks ahead (`peek()`) to decide parsing path
- **Hand-written**: Not generated from grammar (flexibility for error recovery)

## Parsing Entry Points

### Main Entry

```rust
pub fn parse_program(&mut self) -> Result<Program>
```

Returns `Program { items: Vec<TopLevel> }`

### Top-Level Items

```rust
fn parse_top_level(&mut self) -> Result<TopLevel>
```

Parses:
- `import` declarations
- `use rust` declarations
- `type` definitions
- `class` definitions
- Function declarations
- `test` declarations

## Key Parsing Functions

### Functions

```rust
fn parse_function(&mut self, name: String) -> Result<FunctionDecl>
```

Handles:
- Type parameters: `<T, U>`
- Parameters with types and defaults
- Return type annotations
- Block body or arrow expression body

### Classes

```rust
fn parse_class(&mut self) -> Result<ClassDecl>
```

Features:
- Inheritance: `ChildClass: ParentClass`
- Constructor with parameters
- Fields with visibility
- Methods (block or arrow)

### Statements

```rust
fn parse_statement(&mut self) -> Result<Stmt>
```

Statement types:
- **Variable**: `let x = 10` or `let x, err = fallible()`
- **Constant**: `const MAX = 100`
- **If**: With optional parentheses, single-statement or block
- **While**: `while condition { ... }`
- **For**: With data-parallel policies (`seq`, `par`, `vec`, `parvec`)
- **Switch**: Case-based branching
- **Try-Catch**: Exception handling
- **Return**: `return value`
- **Throw**: `throw error`
- **Fail**: `fail "message"`
- **Expression**: Any expression as statement

### For Loops with Policies

```rust
// Parses: for <policy> var in iterable with <options> { body }
if self.match_token(&Token::For) {
    let mut policy = DataParallelPolicy::Seq;
    
    // Parse policy keyword (seq, par, vec, parvec)
    if self.match_token(&Token::Seq) {
        policy = DataParallelPolicy::Seq;
    } else if self.match_token(&Token::Par) {
        policy = DataParallelPolicy::Par;
    } // ... etc
    
    let var = self.parse_identifier()?;
    self.expect(Token::In)?;
    let iterable = self.parse_expression()?;
    
    // Parse with options
    let options = if self.match_token(&Token::With) {
        self.parse_for_options()?
    } else {
        ForPolicyOptions::default()
    };
    
    // Parse body
    self.expect(Token::LBrace)?;
    let body = self.parse_block_stmt()?;
    self.expect(Token::RBrace)?;
    
    return Ok(Stmt::For(ForStmt { var, iterable, policy, options, body }));
}
```

### Expressions (Operator Precedence)

```rust
fn parse_expression(&mut self) -> Result<Expr>
fn parse_ternary(&mut self) -> Result<Expr>
fn parse_logical_or(&mut self) -> Result<Expr>
fn parse_logical_and(&mut self) -> Result<Expr>
fn parse_equality(&mut self) -> Result<Expr>
fn parse_comparison(&mut self) -> Result<Expr>
fn parse_range(&mut self) -> Result<Expr>
fn parse_additive(&mut self) -> Result<Expr>
fn parse_multiplicative(&mut self) -> Result<Expr>
fn parse_unary(&mut self) -> Result<Expr>
fn parse_postfix(&mut self) -> Result<Expr>
fn parse_primary(&mut self) -> Result<Expr>
```

**Precedence Climbing**: Each function handles one precedence level.

### Call Expressions

```rust
fn parse_postfix(&mut self) -> Result<Expr>
```

Handles:
- **Simple calls**: `func(args)`
- **Concurrency policies**: `async func(args)`, `par func(args)`
- **Task creation**: `task async func(args)`
- **Fire-and-forget**: `fire par func(args)`
- **Member access**: `obj.property`
- **Index access**: `arr[index]`
- **Method calls**: `obj.method(args)`

### Primary Expressions

```rust
fn parse_primary(&mut self) -> Result<Expr>
```

Parses:
- **Literals**: Numbers, strings, booleans, chars
- **Identifiers**: Variable names
- **Parenthesized**: `(expr)`
- **Array literals**: `[1, 2, 3]`
- **Object literals**: `{ name: "Alice", age: 25 }`
- **Struct literals**: `Person { name: "Alice", age: 25 }`
- **String templates**: `$"Hello {name}"`
- **Lambdas**: `(x, y) => x + y` or `move (x) => { ... }`

### Lambda Parsing

```rust
fn parse_lambda_expr(&mut self) -> Result<LambdaExpr>
```

Features:
- **Move capture**: `move (x) => x * 2`
- **Parameter types**: `(x: number, y: number) => x + y`
- **Return types**: `(x: number): number => x * 2`
- **Block bodies**: `(x) => { return x * 2 }`
- **Expression bodies**: `(x) => x * 2`

## Error Handling

### Error Recovery

```rust
fn error(&self, message: String) -> CompilerError
```

Generates rich error messages with:
- **Line and column**: Precise location
- **Source snippet**: Shows problematic code
- **Error code**: E2xxx codes for parser errors
- **Helpful message**: Explains what went wrong

### Example Error

```
[E2010] Expected ';' after statement
  --> main.liva:15:20
   |
15 |     let x = 10
   |               ^ Expected ';' here
```

## AST Structure

### Top-Level Items

```rust
pub enum TopLevel {
    Import(ImportDecl),
    UseRust(UseRustDecl),
    Type(TypeDecl),
    Class(ClassDecl),
    Function(FunctionDecl),
    Test(TestDecl),
}
```

### Statements

```rust
pub enum Stmt {
    VarDecl(VarDecl),
    ConstDecl(ConstDecl),
    Assign(AssignStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Switch(SwitchStmt),
    TryCatch(TryCatchStmt),
    Throw(ThrowStmt),
    Fail(FailStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
    Block(BlockStmt),
}
```

### Expressions

```rust
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnOp, operand: Box<Expr> },
    Ternary { condition, then_expr, else_expr },
    Call(CallExpr),
    Member { object: Box<Expr>, property: String },
    Index { object: Box<Expr>, index: Box<Expr> },
    ObjectLiteral(Vec<(String, Expr)>),
    StructLiteral { type_name: String, fields: Vec<(String, Expr)> },
    ArrayLiteral(Vec<Expr>),
    Lambda(LambdaExpr),
    StringTemplate { parts: Vec<StringTemplatePart> },
    Fail(Box<Expr>),
}
```

### Call Expression (with Policies)

```rust
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub exec_policy: ExecPolicy,
}

pub enum ExecPolicy {
    Normal,
    Async,
    Par,
    TaskAsync,
    TaskPar,
    FireAsync,
    FirePar,
}
```

### For Loop Policies

```rust
pub enum DataParallelPolicy {
    Seq,
    Par,
    Vec,
    ParVec,
}

pub struct ForPolicyOptions {
    pub ordered: bool,
    pub chunk: Option<i64>,
    pub threads: Option<ThreadOption>,
    pub simd_width: Option<SimdWidthOption>,
    pub prefetch: Option<i64>,
    pub reduction: Option<ReductionOption>,
    pub schedule: Option<ScheduleOption>,
    pub detect: Option<DetectOption>,
}
```

## Public API

```rust
pub fn parse(tokens: Vec<TokenWithSpan>, file_name: &str, source_code: &str) 
    -> Result<Program>
```

## Summary

- **Recursive Descent**: Hand-written, predictive parser
- **Rich AST**: Fully typed nodes with source spans
- **Policy-Aware**: Parses concurrency and data-parallel annotations
- **Error Recovery**: Detailed error messages with source context
- **1,754 Lines**: Comprehensive grammar support

**Next**: [Semantic Analysis →](semantic.md)

**See Also**:
- [AST Reference](../language-reference/syntax-overview.md)
- [Grammar](grammar.md)
