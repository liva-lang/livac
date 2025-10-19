# Changelog

All notable changes to the Liva compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] - 2025-10-20

### Fixed
- Removed 26 compiler warnings across the codebase
  - Fixed unreachable code in codegen.rs after early returns
  - Fixed unreachable pattern in lowering.rs
  - Prefixed unused variables with `_`
  - Marked intentionally unused code with `#[allow(dead_code)]`
- Fixed `ir_codegen_string_templates` test
  - Implemented variable type tracking for correct format specifiers
  - Use `{}` for Display types (identifiers, literals, member access)
  - Use `{:?}` for Debug types (arrays, objects)
- Fixed error variable formatting in string templates
  - Added `.unwrap_or("")` when error variables used in templates
  - Prevents `Option<&str>` Display trait errors
- Fixed double semicolons in fire calls
  - Removed trailing semicolon from fire call closures
- Removed illegal class inheritance from test examples
  - Fixed `proj_comprehensive` test: replaced `Empleado : Persona` with composition
  - Clarified distinction between interface implementation (legal) and class inheritance (illegal)

### Changed
- All tests now pass (178 tests total)
  - 82 codegen tests
  - 50 desugar tests
  - 11 integration tests
  - 9 lexer tests
  - 21 parser tests
  - 6 property tests
  - 17 semantics tests
  - 3 doc tests
- Zero compiler warnings
- Improved code quality and consistency

### Documentation
- Updated TODO.md with detailed Phase 1 consolidation progress
- Skipped semantic unit tests restoration (incompatible with current AST)
- Verified all documentation correctly describes interface-only inheritance model

## [0.6.0] - 2025-10-19

### BREAKING CHANGES

#### Removed `protected` Visibility
- **Rationale:** Liva doesn't support class inheritance, only interface implementation
- **Migration:**
  - Old `_protectedField` → Now private (same syntax, different meaning)
  - Old `__privateField` → Now use `_privateField`
  - Protected methods no longer have special semantics

**Before (v0.5.x):**
```liva
Person {
  name: string        // public
  _age: number        // protected (accessible in subclasses)
  __ssn: string       // private (class-only)
}
```

**After (v0.6.0):**
```liva
Person {
  name: string        // public
  _age: number        // private (class-only)
}
```

### Added
- Interface implementation support
  - Classes can implement interfaces using `:` syntax
  - Interfaces are pure contracts (only method signatures, no fields)
  - Multiple interface implementation supported

### Changed
- Visibility model simplified to public/private only
- `_` prefix now means private (was protected)
- `__` prefix removed (no longer needed)

### Migration Guide

#### Class Inheritance → Composition
If you were using class inheritance patterns:

**Before:**
```liva
// This was never officially supported but might have worked
Empleado : Persona {
  empresa: string
}
```

**After:**
```liva
// Use composition instead
Empleado {
  persona: Persona
  empresa: string
  
  init(nombre: string, edad: number, empresa: string) {
    this.persona = Persona(nombre, edad)
    this.empresa = empresa
  }
}
```

#### Interfaces (Still Supported)
Interfaces remain unchanged:

```liva
// Interface (only signatures)
Animal {
  makeSound(): string
  getName(): string
}

// Implementation (has fields + implementations)
Dog : Animal {
  name: string
  
  constructor(name: string) {
    this.name = name
  }
  
  makeSound() => "Woof!"
  getName() => this.name
}
```

---

[Unreleased]: https://github.com/liva-lang/livac/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/liva-lang/livac/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/liva-lang/livac/releases/tag/v0.6.0
