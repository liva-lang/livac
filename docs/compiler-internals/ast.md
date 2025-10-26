# Abstract Syntax Tree (AST)

The AST is the core data structure representing parsed Liva programs.

## Location

**File**: `src/ast.rs` (~1,200 lines)

## Overview

The AST provides:
1. **Type-safe representation**: Rust enums and structs for all syntax nodes
2. **Source tracking**: Every node has a `Span` for error reporting
3. **Semantic information**: Type annotations, visibility modifiers, etc.
4. **Pattern matching support**: Destructuring patterns (v0.10.2+)

## Core Types

### Program

```rust
pub struct Program {
    pub items: Vec<TopLevel>,
}
```

Root node containing all top-level declarations.

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

## Variable Bindings (v0.10.2+)

### VarBinding Structure

```rust
pub struct VarBinding {
    pub pattern: BindingPattern,
    pub type_ref: Option<TypeRef>,
    pub span: Span,
}
```

**Changed in v0.10.2**: Previously had `name: String` field. Now uses `pattern: BindingPattern` to support destructuring.

### BindingPattern Enum

```rust
pub enum BindingPattern {
    Identifier(String),
    Object(ObjectPattern),
    Array(ArrayPattern),
}
```

**Variants**:
- **Identifier**: Simple variable name (e.g., `x`, `userName`)
- **Object**: Object destructuring pattern (e.g., `{name, age}`)
- **Array**: Array destructuring pattern (e.g., `[first, second]`)

### Object Destructuring

```rust
pub struct ObjectPattern {
    pub fields: Vec<ObjectPatternField>,
}

pub struct ObjectPatternField {
    pub key: String,      // Property name in source object
    pub binding: String,  // Local variable name (may differ if renamed)
}
```

**Examples**:
```liva
let {name, age} = user
// ObjectPattern { fields: [
//   ObjectPatternField { key: "name", binding: "name" },
//   ObjectPatternField { key: "age", binding: "age" }
// ]}

let {name: userName, age: userAge} = user
// ObjectPattern { fields: [
//   ObjectPatternField { key: "name", binding: "userName" },
//   ObjectPatternField { key: "age", binding: "userAge" }
// ]}
```

### Array Destructuring

```rust
pub struct ArrayPattern {
    pub elements: Vec<Option<String>>,  // None = skip element
    pub rest: Option<String>,           // Rest pattern variable
}
```

**Examples**:
```liva
let [first, second] = array
// ArrayPattern { elements: [Some("first"), Some("second")], rest: None }

let [first, , third] = array
// ArrayPattern { elements: [Some("first"), None, Some("third")], rest: None }

let [head, ...tail] = array
// ArrayPattern { elements: [Some("head")], rest: Some("tail") }
```

### Helper Methods

```rust
impl VarBinding {
    /// Returns the simple identifier name, or None for complex patterns
    pub fn name(&self) -> Option<&str> {
        match &self.pattern {
            BindingPattern::Identifier(name) => Some(name),
            _ => None,
        }
    }
    
    /// Returns true if this is a simple identifier binding
    pub fn is_simple(&self) -> bool {
        matches!(self.pattern, BindingPattern::Identifier(_))
    }
}
```

**Usage**: Most codegen and semantic analysis code uses these helpers for backward compatibility while we implement full destructuring support.

## Statements

### VarDecl

```rust
pub struct VarDecl {
    pub bindings: Vec<VarBinding>,  // Can be multiple for fallible patterns
    pub init: Expr,
    pub is_fallible: bool,          // true for 'let x, err = ...'
}
```

**Examples**:
```liva
let x = 10
// VarDecl { bindings: [VarBinding { pattern: Identifier("x"), ... }], is_fallible: false }

let result, err = parseInt("123")
// VarDecl { bindings: [Identifier("result"), Identifier("err")], is_fallible: true }

let {name, age} = user
// VarDecl { bindings: [VarBinding { pattern: Object(...), ... }], is_fallible: false }
```

### Other Statements

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

## Expressions

```rust
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnOp, operand: Box<Expr> },
    Ternary { condition, then_expr, else_expr },
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    Member { object: Box<Expr>, property: String },
    Index { object: Box<Expr>, index: Box<Expr> },
    ObjectLiteral(Vec<(String, Expr)>),
    StructLiteral { type_name: String, fields: Vec<(String, Expr)> },
    ArrayLiteral(Vec<Expr>),
    Lambda(LambdaExpr),
    StringTemplate { parts: Vec<StringTemplatePart> },
    Fail(Box<Expr>),
    RangeLiteral { start: Box<Expr>, end: Box<Expr>, inclusive: bool },
}
```

## Type References

```rust
pub enum TypeRef {
    Simple(String),                              // number, string, Person
    Array(Box<TypeRef>),                         // [number]
    Map { key: Box<TypeRef>, value: Box<TypeRef> },  // [string: number]
    Optional(Box<TypeRef>),                      // number?
    Generic { name: String, type_args: Vec<TypeRef> },  // Array<T>
    Function { params: Vec<TypeRef>, return_type: Box<TypeRef> },  // (number, number) => number
    Tuple(Vec<TypeRef>),                         // (number, string)
}
```

## Execution Policies

### Function Call Policies

```rust
pub enum ExecPolicy {
    Normal,       // func()
    Async,        // async func()
    Par,          // par func()
    TaskAsync,    // task async func()
    TaskPar,      // task par func()
    FireAsync,    // fire async func()
    FirePar,      // fire par func()
}
```

### Data Parallel Policies (For Loops)

```rust
pub enum DataParallelPolicy {
    Seq,      // for seq x in items
    Par,      // for par x in items
    Vec,      // for vec x in items
    ParVec,   // for parvec x in items
}
```

## Visibility

```rust
pub enum Visibility {
    Public,    // pub
    Private,   // (default)
}
```

## Source Tracking

```rust
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub file: String,
}
```

Every AST node includes span information for:
- Error messages
- IDE features (go-to-definition, hover)
- Code formatting

## Migration Notes (v0.10.2)

**Breaking Change**: `VarBinding.name` → `VarBinding.pattern`

**Before**:
```rust
let name = &binding.name;
```

**After**:
```rust
let name = binding.name().unwrap();  // For simple identifiers
// or
if let Some(name) = binding.name() {
    // Handle simple binding
}
```

**Affected Modules**:
- `src/codegen.rs`: 13 locations updated
- `src/parser.rs`: 2 locations updated
- `src/semantic.rs`: 6 locations updated
- `src/lowering.rs`: 3 locations updated

## Summary

- **Type-safe**: Rust's type system ensures AST validity
- **Extensible**: Easy to add new node types
- **Destructuring-ready**: v0.10.2 adds pattern matching support
- **Well-tracked**: Every node has source location information

**Next**: [Semantic Analysis →](semantic.md)

**See Also**:
- [Parser](parser.md)
- [Language Reference](../language-reference/syntax-overview.md)
