# IR (Intermediate Representation)

The IR layer sits between the high-level AST and low-level Rust code generation, providing a cleaner abstraction for codegen.

## Location

**File**: `src/ir.rs` (396 lines)

## Overview

The IR (Intermediate Representation) provides:
- **Simplified AST**: Easier to emit Rust code from
- **Effect Tracking**: Async, parallel, fallibility info
- **Type Information**: Explicit type annotations
- **Concurrency Abstraction**: Unified task/fire/par representation

## Purpose

**Decouples** high-level Liva constructs from Rust-specific code generation:

```
AST → Lowering → IR → CodeGen → Rust
```

This allows:
- Easier codegen maintenance
- Multiple backend targets (future: LLVM, WASM)
- Optimization passes on IR

## IR Structures

### Module

```rust
pub struct Module {
    pub items: Vec<Item>,
    pub extern_crates: Vec<ExternCrate>,
}
```

Top-level container for compilation unit.

### Item

```rust
pub enum Item {
    Function(Function),
    Test(Test),
    Unsupported(ast::TopLevel),  // Classes, types (not yet lowered)
}
```

### Function

```rust
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Type,
    pub body: Block,
    pub async_kind: AsyncKind,
    pub visibility: Visibility,
    pub contains_fail: bool,
    pub source: ast::FunctionDecl,  // Original AST for debugging
}
```

**Key fields**:
- `async_kind`: `Sync`, `Async`, `AsyncInferred`
- `contains_fail`: For `Result<T, Error>` return
- `source`: Link back to original AST

### Statement Types

```rust
pub enum Stmt {
    Let { name: String, ty: Option<Type>, value: Expr },
    Const { name: String, ty: Option<Type>, value: Expr },
    Assign { target: Expr, value: Expr },
    Return(Option<Expr>),
    Throw(Expr),
    Expr(Expr),
    If { condition: Expr, then_block: Block, else_block: Option<Block> },
    While { condition: Expr, body: Block },
    For { var: String, iterable: Expr, policy: DataParallelPolicy, 
          options: ForPolicyOptions, body: Block },
    Block(Block),
    TryCatch { try_block: Block, error_var: String, catch_block: Block },
    Switch { discriminant: Expr, cases: Vec<SwitchCase>, 
             default: Option<Vec<Stmt>> },
    Unsupported(ast::Stmt),
}
```

### Expression Types

```rust
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Await(Box<Expr>),
    AsyncCall { callee: Box<Expr>, args: Vec<Expr> },
    ParallelCall { callee: Box<Expr>, args: Vec<Expr> },
    TaskCall { mode: ConcurrencyMode, callee: String, args: Vec<Expr> },
    FireCall { mode: ConcurrencyMode, callee: String, args: Vec<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, operand: Box<Expr> },
    Ternary { condition, then_expr, else_expr },
    StringTemplate(Vec<TemplatePart>),
    Member { object: Box<Expr>, property: String },
    Index { object: Box<Expr>, index: Box<Expr> },
    ObjectLiteral(Vec<(String, Expr)>),
    StructLiteral { type_name: String, fields: Vec<(String, Expr)> },
    ArrayLiteral(Vec<Expr>),
    Range { start: Box<Expr>, end: Box<Expr> },
    Lambda(LambdaExpr),
    Unsupported(ast::Expr),
}
```

### Concurrency Modes

```rust
pub enum ConcurrencyMode {
    Async,    // I/O-bound
    Parallel, // CPU-bound
}
```

**Used in**:
- `TaskCall`: Creates `JoinHandle<T>`
- `FireCall`: Fire-and-forget spawning

### Data-Parallel Policy

```rust
pub enum DataParallelPolicy {
    Seq,     // Sequential
    Par,     // Parallel (rayon)
    Vec,     // Vectorized (SIMD)
    ParVec,  // Parallel + Vectorized
}
```

### Type Representation

```rust
pub enum Type {
    Inferred,
    Unit,
    Int,
    Float,
    Bool,
    String,
    Char,
    Array(Box<Type>),
    Option(Box<Type>),
    Result { ok: Box<Type>, err: Box<Type> },
    Custom(String),
    Generic { base: String, args: Vec<Type> },
    Function { params: Vec<Type>, ret: Box<Type> },
}
```

## Lowering Pass

**File**: `src/lowering.rs`

Converts AST → IR:

```rust
pub fn lower_program(program: ast::Program) -> ir::Module
```

**Transformations**:
1. **Function lowering**: AST functions → IR functions
2. **Statement lowering**: Convert control flow
3. **Expression lowering**: Simplify concurrency policies
4. **Type resolution**: Map Liva types → IR types

### Example Lowering

**AST**:
```liva
calculateSum(numbers) {
  let total = 0
  for num in numbers {
    total = total + num
  }
  return total
}
```

**IR**:
```rust
Function {
    name: "calculateSum",
    params: [Param { name: "numbers", ty: Inferred }],
    ret_type: Inferred,
    body: Block {
        statements: [
            Let { name: "total", ty: None, value: Literal(Int(0)) },
            For {
                var: "num",
                iterable: Identifier("numbers"),
                policy: Seq,
                options: Default,
                body: Block {
                    statements: [
                        Assign {
                            target: Identifier("total"),
                            value: Binary {
                                op: Add,
                                left: Identifier("total"),
                                right: Identifier("num"),
                            }
                        }
                    ]
                }
            },
            Return(Some(Identifier("total")))
        ]
    },
    async_kind: Sync,
    contains_fail: false,
    visibility: Public,
}
```

## IR Advantages

1. **Cleaner Codegen**: Emit Rust from simplified structures
2. **Optimization Opportunities**: Can optimize at IR level
3. **Multiple Backends**: Could target LLVM, WASM, etc.
4. **Easier Testing**: Test lowering and codegen separately

## Current Limitations

- **Classes not fully lowered**: Still in `Unsupported` category
- **Type inference**: Done during codegen, not at IR level
- **Limited optimization**: No IR-level optimization passes yet

## Summary

- **396 Lines**: Lightweight IR representation
- **AST → IR → Rust**: Clean separation of concerns
- **Effect Tracking**: Async, parallel, fallibility explicit
- **Future-Proof**: Ready for optimization passes and new backends

**Next**: [Code Generation →](codegen.md)

**See Also**:
- [Lowering](lowering.md)
- [Architecture](architecture.md)
