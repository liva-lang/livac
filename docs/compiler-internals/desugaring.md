# Desugaring

The desugaring pass collects metadata about the program and prepares it for code generation.

## Location

**File**: `src/desugaring.rs` (260 lines)

## Overview

Desugaring is a **pre-codegen analysis pass** that:
1. **Collects Dependencies**: Tracks `use rust` declarations
2. **Detects Concurrency**: Determines if tokio/rayon needed
3. **Builds Context**: Creates `DesugarContext` for codegen

## DesugarContext

```rust
pub struct DesugarContext {
    pub rust_crates: Vec<(String, Option<String>)>,  // (crate, alias)
    pub has_async: bool,
    pub has_parallel: bool,
}
```

This context tells codegen what runtime dependencies to include.

## Main Function

```rust
pub fn desugar(program: Program) -> Result<DesugarContext>
```

**Returns**: Metadata about program's runtime needs.

## Process

### 1. Collect `use rust` Declarations

```liva
use rust reqwest
use rust serde as json
```

Collected as:
```rust
rust_crates: [
    ("reqwest".to_string(), None),
    ("serde".to_string(), Some("json".to_string())),
]
```

### 2. Detect Async Usage

Checks for:
- `async` calls: `async fetchData()`
- Async functions: Functions marked `is_async_inferred`
- Async policies: `ExecPolicy::Async`, `TaskAsync`, `FireAsync`

If found:
```rust
ctx.has_async = true
rust_crates.push(("tokio".to_string(), None))  // Auto-add tokio
```

### 3. Detect Parallel Usage

Checks for:
- `par` calls: `par compute()`
- Parallel loops: `for par x in items`
- Parallel policies: `ExecPolicy::Par`, `TaskPar`, `FirePar`
- Data-parallel: `DataParallelPolicy::Par`, `Vec`, `ParVec`

If found:
```rust
ctx.has_parallel = true
// Rayon included automatically in liva_rt
```

## Example

**Input Program**:
```liva
use rust reqwest

fetchUser(id: number): string {
  let response = async httpGet($"/users/{id}")
  return response.body
}

processData(items) {
  for par item in items with threads 4 {
    compute(item)
  }
}

main() {
  let user = fetchUser(1)
  processData([1, 2, 3, 4])
}
```

**Output Context**:
```rust
DesugarContext {
    rust_crates: [
        ("reqwest", None),
        ("tokio", None),  // Auto-added
    ],
    has_async: true,     // fetchUser has async call
    has_parallel: true,  // processData has par loop
}
```

## Concurrency Detection

### Traversal Functions

```rust
fn check_concurrency(item: &TopLevel, ctx: &mut DesugarContext)
fn check_block_concurrency(body: &IfBody, ctx: &mut DesugarContext)
fn check_stmt_concurrency(stmt: &Stmt, ctx: &mut DesugarContext)
fn check_expr_concurrency(expr: &Expr, ctx: &mut DesugarContext)
```

**Recursively walks AST** to find concurrency usage.

### Async Detection

```rust
match call.exec_policy {
    ExecPolicy::Async | ExecPolicy::TaskAsync | ExecPolicy::FireAsync => {
        ctx.has_async = true;
    }
    _ => {}
}
```

### Parallel Detection

```rust
match for_stmt.policy {
    DataParallelPolicy::Par | DataParallelPolicy::Vec | DataParallelPolicy::ParVec => {
        ctx.has_parallel = true;
    }
    _ => {}
}
```

## Codegen Integration

Codegen uses `DesugarContext` to:

1. **Include Dependencies**:
   ```rust
   use tokio;
   use rayon::prelude::*;
   ```

2. **Generate Runtime Module**:
   ```rust
   mod liva_rt {
       // Include async helpers if ctx.has_async
       // Include parallel helpers if ctx.has_parallel
   }
   ```

3. **Wrap Main**:
   ```rust
   #[tokio::main]  // If ctx.has_async
   async fn main() {
       // ...
   }
   ```

## Why "Desugaring"?

The name is **legacy** from earlier designs where this pass did more transformation. Now it's primarily **metadata collection**.

**Better name**: `MetadataCollector` or `ContextBuilder`

## Summary

- **260 Lines**: Lightweight analysis pass
- **Metadata Collection**: Determines runtime needs
- **Dependency Tracking**: Manages `use rust` crates
- **Concurrency Detection**: Identifies async/parallel usage
- **Codegen Preparation**: Builds context for code generation

**Next**: [Runtime Module →](runtime.md)

**See Also**:
- [Code Generation](codegen.md)
- [Architecture](architecture.md)
