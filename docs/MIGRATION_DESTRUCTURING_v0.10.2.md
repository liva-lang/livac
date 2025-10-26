# Migration Guide: Destructuring Support (v0.10.2)

## Overview

Version 0.10.2 introduces **destructuring patterns** for variable bindings. This is a breaking change to the AST structure that affects how variable bindings are represented internally.

## Breaking Changes

### VarBinding Structure Change

**Before (v0.10.1)**:
```rust
pub struct VarBinding {
    pub name: String,              // ❌ Removed
    pub type_ref: Option<TypeRef>,
    pub span: Span,
}
```

**After (v0.10.2)**:
```rust
pub struct VarBinding {
    pub pattern: BindingPattern,   // ✅ New field
    pub type_ref: Option<TypeRef>,
    pub span: Span,
}

pub enum BindingPattern {
    Identifier(String),
    Object(ObjectPattern),
    Array(ArrayPattern),
}
```

## Migration Path

### 1. Direct Field Access

**Before**:
```rust
let name = &binding.name;
```

**After (Option 1 - Safe)**:
```rust
if let Some(name) = binding.name() {
    // Handle simple identifier binding
}
```

**After (Option 2 - Unwrap for now)**:
```rust
// If you know it's always a simple identifier (no destructuring yet)
let name = binding.name().unwrap();
```

**After (Option 3 - Match pattern)**:
```rust
match &binding.pattern {
    BindingPattern::Identifier(name) => {
        // Handle identifier
    }
    BindingPattern::Object(obj_pattern) => {
        // TODO: Implement object destructuring support
    }
    BindingPattern::Array(arr_pattern) => {
        // TODO: Implement array destructuring support
    }
}
```

### 2. Creating Bindings

**Before**:
```rust
VarBinding {
    name: "x".to_string(),
    type_ref: None,
    span: my_span,
}
```

**After**:
```rust
VarBinding {
    pattern: BindingPattern::Identifier("x".to_string()),
    type_ref: None,
    span: my_span,
}
```

### 3. Checking Binding Type

**New Helper Method**:
```rust
if binding.is_simple() {
    // It's a simple identifier, safe to use name()
    let name = binding.name().unwrap();
}
```

## Updated Modules

### src/codegen.rs (13 locations)

**Pattern**: All uses of `binding.name` updated to use helper methods.

**Example Fix**:
```rust
// Before
write!(self.output, "{}", self.sanitize_name(&binding.name))

// After
if let Some(name) = binding.name() {
    write!(self.output, "{}", self.sanitize_name(name))
}
```

### src/parser.rs (2 locations)

**Pattern**: Create `BindingPattern::Identifier` instead of storing name directly.

**Example Fix**:
```rust
// Before
bindings.push(VarBinding {
    name,
    type_ref,
    span,
});

// After
bindings.push(VarBinding {
    pattern: BindingPattern::Identifier(name),
    type_ref,
    span,
});
```

### src/semantic.rs (6 locations)

**Pattern**: Use `binding.name()` helper for symbol declaration.

**Example Fix**:
```rust
// Before
if self.declare_symbol(&binding.name, declared_type.clone()) {
    // ...
}

// After
if let Some(name) = binding.name() {
    if self.declare_symbol(name, declared_type.clone()) {
        // ...
    }
}
```

### src/lowering.rs (3 locations)

**Pattern**: Extract name for IR lowering.

**Example Fix**:
```rust
// Before
name: binding.name.clone()

// After
name: binding.name().unwrap_or("_").to_string()
```

## Helper Methods

### binding.name() -> Option<&str>

Returns the simple identifier name if the pattern is `BindingPattern::Identifier`, otherwise `None`.

**Use when**:
- You need the variable name for codegen
- You're working with simple bindings only
- You want to gracefully handle complex patterns later

**Example**:
```rust
if let Some(name) = binding.name() {
    self.variables.insert(name.to_string(), ty);
}
```

### binding.is_simple() -> bool

Returns `true` if the pattern is a simple identifier.

**Use when**:
- You need to check if it's safe to unwrap `name()`
- You want to error on complex patterns (not yet supported)

**Example**:
```rust
if !binding.is_simple() {
    return Err(CompilerError::new(
        "Destructuring patterns not yet supported here"
    ));
}
let name = binding.name().unwrap();
```

## Implementation Status

### ✅ Completed (v0.10.2-alpha)

- [x] AST types for destructuring patterns
- [x] Helper methods for backward compatibility
- [x] Updated all existing code to use new API
- [x] Unit tests passing
- [x] Snapshot tests updated

### 🚧 In Progress

- [ ] Parser support for `{x, y}` syntax
- [ ] Parser support for `[x, y]` syntax
- [ ] Semantic validation for destructuring
- [ ] Code generation for destructuring

### ⏳ Planned

- [ ] Nested destructuring
- [ ] Default values in destructuring
- [ ] Rest patterns in objects
- [ ] Function parameter destructuring

## Testing Your Code

After updating to v0.10.2, run:

```bash
cargo build
cargo test --lib
```

If you see errors like:
```
error[E0609]: no field `name` on type `&ast::VarBinding`
```

You need to migrate to the new API using the patterns above.

## Future-Proofing

### Don't Do This:
```rust
// ❌ Assumes all bindings are simple identifiers
let name = binding.name().unwrap();
```

### Do This Instead:
```rust
// ✅ Gracefully handles complex patterns
if let Some(name) = binding.name() {
    // Handle simple binding
} else {
    // TODO: Implement destructuring support
    return Err(CompilerError::new(
        "Destructuring patterns not yet fully supported"
    ));
}
```

Or with a TODO comment:
```rust
// TODO(destructuring): Support object and array patterns here
let name = binding.name().unwrap();
```

## Questions?

See:
- [AST Documentation](compiler-internals/ast.md)
- [Parser Documentation](compiler-internals/parser.md)
- [Design Document](PHASE_6.5_DESTRUCTURING_DESIGN.md)

Or check the commit history:
```bash
git log --grep="destructuring"
```
