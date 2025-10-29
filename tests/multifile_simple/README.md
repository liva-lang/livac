# Multi-File Compilation Tests

This directory contains tests for Liva's multi-file compilation system (Phase 3.5).

## Test Files

### Basic Tests

**`main.liva`** - Simple two-module test
- Imports: `Container`, `createContainer`
- Tests: Class instantiation, factory pattern

**`test_all.liva`** - Comprehensive multi-module test  
- Imports from 3 modules: `container.liva`, `math.liva`
- Tests: Classes, functions, cross-module calls

### Module Files

**`container.liva`** - Container class module
```liva
Container {
    value: string
    constructor(value: string) { ... }
    getValue(): string { ... }
}
createContainer(value: string): Container { ... }
```

**`math.liva`** - Math utilities module
```liva
add(a: int, b: int): int { ... }
multiply(a: int, b: int): int { ... }
square(n: int): int { ... }
```

**`result.liva`** - Generic Result type (WIP)
- Note: Generics in imports need more work

## Running Tests

### Run Individual Test
```bash
$ livac main.liva -r
```

### Run Comprehensive Test
```bash
$ livac test_all.liva -r
```

Expected output:
```
========================================
  Multi-File Compilation Test
========================================

Test 1: Container Module
Factory pattern works!
Constructor works!

Test 2: Math Module
10 + 20 =
30
5 * 7 =
35
8^2 =
64

========================================
  All Tests Passed!
========================================
```

## What's Tested

✅ **Import Resolution**
- Relative paths (`./module.liva`)
- Multiple imports from same module
- Mixed type and function imports

✅ **Symbol Types**
- Classes (PascalCase preserved)
- Functions (converted to snake_case)
- Factory functions
- Constructors

✅ **Code Generation**
- Module files with `pub` visibility
- Main file with `mod` declarations
- Proper `use` statements
- Rust module structure

## Known Limitations

❌ **Generics in Imports**
- `Result<T,E>` from imports doesn't work yet
- Type inference issues in generated Rust code

❌ **Circular Dependencies**
- Not detected or prevented yet

❌ **Parent Directory Imports**
- `../module.liva` simplified to `crate::module`
- Nested paths not fully tested

## Implementation Status

- **Phase 3.5.1** (Foundation): ✅ COMPLETE
- **Phase 3.5.2** (Import Resolution): ✅ COMPLETE  
- **Phase 3.5.3** (Multi-File Generation): ✅ COMPLETE
- **Phase 3.5.4** (Advanced Features): 🚧 TODO
- **Phase 3.5.5** (Testing & Docs): 🚧 IN PROGRESS

## Next Steps

1. Fix generic type imports
2. Add circular dependency detection
3. Test nested directory structures
4. Add more comprehensive test cases
5. Document best practices
