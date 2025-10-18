# Code Generation

The code generator emits Rust source code from the IR, handling concurrency transformations, error handling, and runtime integration.

## Location

**File**: `src/codegen.rs` (4,683 lines)

## Overview

The code generator:
1. **Emits Rust Code**: Converts IR → valid Rust syntax
2. **Generates `liva_rt` Module**: Runtime helpers for concurrency and errors
3. **Handles Async/Par**: Transforms concurrency policies into tokio/rayon code
4. **Manages Error Binding**: Implements fallibility system
5. **Tracks Awaitable Tasks**: Lazy await/join optimization

## Architecture

### CodeGenerator

```rust
pub struct CodeGenerator {
    output: String,
    indent_level: usize,
    ctx: DesugarContext,
    in_method: bool,
    in_fallible_function: bool,
    pending_tasks: HashMap<String, TaskInfo>,
    error_binding_vars: HashSet<String>,
    fallible_functions: HashSet<String>,
    // ... other tracking fields
}
```

## Key Responsibilities

### 1. Runtime Module Generation

Generates `liva_rt` module with:

```rust
mod liva_rt {
    use std::future::Future;
    use tokio::task::JoinHandle;
    
    pub struct Error {
        pub message: String,
    }
    
    pub fn spawn_async<F, T>(future: F) -> JoinHandle<T> { ... }
    pub fn fire_async<F>(future: F) { ... }
    pub fn spawn_parallel<F, T>(f: F) -> JoinHandle<T> { ... }
    pub fn fire_parallel<F>(f: F) { ... }
    
    // Display formatting helpers
    pub fn display_array<T: std::fmt::Debug>(arr: &[T]) -> String { ... }
    // ... more helpers
}
```

### 2. Function Generation

**Non-Async, Non-Fallible**:
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Async**:
```rust
async fn fetchUser(id: i32) -> String {
    let response = reqwest::get(...).await;
    response.text().await.unwrap()
}
```

**Fallible**:
```rust
fn divide(a: i32, b: i32) -> Result<i32, liva_rt::Error> {
    if b == 0 {
        return Err(liva_rt::Error::from("Division by zero"));
    }
    Ok(a / b)
}
```

**Async + Fallible**:
```rust
async fn processData(url: String) -> Result<String, liva_rt::Error> {
    let data = fetch_from_api(&url).await;
    if data.is_empty() {
        return Err(liva_rt::Error::from("No data"));
    }
    Ok(data)
}
```

### 3. Concurrency Transformations

**Async Call (Immediate Await)**:
```liva
let user = async fetchUser(1)
```
↓
```rust
let user = fetchUser(1).await;
```

**Task Async (Deferred Await)**:
```liva
let userTask = task async fetchUser(1)
let user = await userTask
```
↓
```rust
let userTask = liva_rt::spawn_async(async move { fetchUser(1).await });
let user = userTask.await.unwrap();
```

**Parallel Call**:
```liva
let result = par heavyComputation(100)
```
↓
```rust
let result = { heavyComputation(100) };  // Eager evaluation
```

**Task Par**:
```liva
let task = task par compute(100)
let result = await task
```
↓
```rust
let task = liva_rt::spawn_parallel(move || compute(100));
let result = task.await.unwrap();
```

**Fire Async**:
```liva
fire async logEvent("message")
```
↓
```rust
liva_rt::fire_async(async move { logEvent("message").await; });
```

**Fire Par**:
```liva
fire par backgroundCleanup()
```
↓
```rust
liva_rt::fire_parallel(move || { backgroundCleanup(); });
```

### 4. Error Binding

**Basic Error Binding**:
```liva
let result, err = divide(10, 2)
```
↓
```rust
let (result, err) = match divide(10, 2) {
    Ok(val) => (val, "".to_string()),
    Err(e) => (Default::default(), e.to_string()),
};
```

**Error Binding with Async**:
```liva
let result, err = async divide(10, 2)
```
↓
```rust
let (result, err) = match divide(10, 2).await {
    Ok(val) => (val, "".to_string()),
    Err(e) => (Default::default(), e.to_string()),
};
```

**Error Binding with Task**:
```liva
let taskHandle = task async divide(10, 2)
let result, err = await taskHandle
```
↓
```rust
let taskHandle = liva_rt::spawn_async(async move { divide(10, 2).await });
let (result, err) = match taskHandle.await.unwrap() {
    Ok(val) => (val, "".to_string()),
    Err(e) => (Default::default(), e.to_string()),
};
```

### 5. Data-Parallel Loops

**Sequential**:
```liva
for item in items {
    process(item)
}
```
↓
```rust
for item in items {
    process(item);
}
```

**Parallel (Rayon)**:
```liva
for par item in items with threads 4 {
    process(item)
}
```
↓
```rust
items.into_par_iter()
    .with_max_threads(4)
    .for_each(|item| {
        process(item);
    });
```

**Vectorized (SIMD)**:
```liva
for vec value in values with simdWidth 4 {
    compute(value)
}
```
↓
```rust
// SIMD generation (simplified)
for chunk in values.chunks_exact(4) {
    compute(chunk[0]);
    compute(chunk[1]);
    compute(chunk[2]);
    compute(chunk[3]);
}
```

**Parallel + Vec**:
```liva
for parvec value in values with simdWidth 4 ordered {
    process(value)
}
```
↓
```rust
values.into_par_iter()
    .with_simd_width(4)
    .ordered()
    .for_each(|value| {
        process(value);
    });
```

### 6. String Templates

```liva
let name = "Alice"
let greeting = $"Hello, {name}!"
```
↓
```rust
let name = "Alice";
let greeting = format!("Hello, {}!", name);
```

Complex templates:
```liva
let msg = $"User {user.name} (age {user.age}) at {timestamp}"
```
↓
```rust
let msg = format!("User {} (age {}) at {}", user.name, user.age, timestamp);
```

### 7. Classes

```liva
Person {
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    
    name: string
    age: number
    
    greet() => $"Hello, I'm {this.name}"
}
```
↓
```rust
#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: i32,
}

impl Person {
    fn new(name: String, age: i32) -> Self {
        Self { name, age }
    }
    
    fn greet(&self) -> String {
        format!("Hello, I'm {}", self.name)
    }
}
```

## Main Entry Point

```rust
pub fn generate(module: ir::Module, ctx: DesugarContext) -> Result<String>
```

Returns complete Rust source code ready for compilation.

## Code Organization

### String Building

```rust
fn writeln(&mut self, s: &str)  // Write line with indentation
fn indent(&mut self)            // Increase indent
fn dedent(&mut self)            // Decrease indent
```

### Generation Methods

```rust
fn generate_function(&mut self, func: &ir::Function)
fn generate_statement(&mut self, stmt: &ir::Stmt)
fn generate_expression(&mut self, expr: &ir::Expr)
fn generate_for_loop(&mut self, for_stmt: &ir::For)
fn generate_call(&mut self, call: &ir::CallExpr)
```

## Error Codes

| Code | Description |
|------|-------------|
| **E5001** | Unsupported IR construct |
| **E5002** | Invalid concurrency combination |
| **E5003** | Codegen internal error |

## Output Format

Generated Rust code includes:

1. **Module header**: `use` statements for dependencies
2. **`liva_rt` module**: Runtime helpers
3. **Type definitions**: Structs, enums
4. **Functions**: Public and private
5. **Main function**: Entry point (if present)

## Summary

- **4,683 Lines**: Most complex compiler phase
- **IR → Rust**: Direct code emission
- **Runtime Integration**: Generates `liva_rt` module
- **Concurrency Transformation**: Handles async/par/task/fire
- **Error Binding**: Implements fallibility system
- **Data-Parallel**: Rayon and SIMD code generation

**Next**: [Desugaring →](desugaring.md)

**See Also**:
- [Runtime Module](runtime.md)
- [IR](ir.md)
- [Concurrency](../language-reference/concurrency.md)
