# Liva Examples

This directory contains examples demonstrating various features of the Liva language.

## Directory Structure

### 📁 `basic/`
Basic language features and simple examples:
- `test_basic.liva` - Hello world
- `test_parvec*.liva` - Parallel vector operations
- `test_arrow_foreach.liva` - Arrow function syntax in forEach
- `test_underscore.liva` - Private naming convention
- `simple_switch.liva` - Switch expressions

### 📁 `generics/`
Generic programming examples (Phase 5):
- `test_simple_generic.liva` - Basic generic function
- `test_generic_class.liva` - Generic class definitions
- `test_generic_call.liva` - Generic function calls with type arguments
- `test_advanced_generics.liva` - Complex generic patterns
- `test_trait_aliases.liva` - Trait aliases (Numeric, Comparable, etc.)
- `test_multi_constraints.liva` - Multiple trait constraints
- `test_option_generic.liva` - Option<T> implementation
- `test_result_generic.liva` - Result<T,E> implementation
- `test_type_param_validation.liva` - Type parameter validation
- `test_array_*.liva` - Generic array operations

### 📁 `http-json/`
HTTP client and JSON parsing examples (Phase 6):
- `test_http_*.liva` - HTTP GET/POST requests
- `test_json_*.liva` - JSON parsing and manipulation
- `test_user_*.liva` - Complete HTTP + JSON examples with typed parsing
- `json_*.liva` - JSON API processing patterns
- `test_typed_foreach.liva` - Typed arrays in forEach loops
- `test_auto_clone.liva` - Auto-clone feature for methods

Key features demonstrated:
- Async HTTP requests with error binding
- JSON typed parsing with class definitions
- Direct field access on typed objects
- Parallel processing with parvec()
- Method auto-cloning for ergonomic code

### 📁 `calculator/`
Multi-file module system example (Phase 3):
- `calculator.liva` - Main entry point
- `basic.liva` - Basic operations (+, -, *, /)
- `operations/advanced.liva` - Advanced operations

### 📁 `modules/`
Module system examples:
- Various import/export patterns
- Public/private visibility

### 📁 `stdlib/`
Standard library function examples (Phase 2):
- Array methods (map, filter, reduce, forEach, etc.)
- String methods (split, replace, trim, etc.)
- Math functions (sqrt, pow, abs, etc.)
- Type conversions (parseInt, parseFloat, toString)
- Console/IO operations

### 📁 `manual-tests/`
Legacy manual tests (may need cleanup):
- Various edge cases and debugging scenarios
- Some tests may be outdated

## Running Examples

### Single file:
```bash
cd livac
cargo run -- examples/basic/test_basic.liva --run
```

### With HTTP (requires network):
```bash
cargo run -- examples/http-json/test_http_simple.liva --run
```

### Generic examples:
```bash
cargo run -- examples/generics/test_simple_generic.liva --run
```

### Multi-file project:
```bash
cargo run -- examples/calculator/calculator.liva --run
```

## Example Categories by Phase

- **Phase 1-4**: Core language features (variables, functions, classes, control flow)
- **Phase 5 (v0.9.0)**: Generics system (`generics/`)
- **Phase 6.1 (v0.9.3)**: JSON parsing (`http-json/test_json_*.liva`)
- **Phase 6.2 (v0.9.4)**: File I/O (in `stdlib/`)
- **Phase 6.3 (v0.9.6)**: HTTP client (`http-json/test_http_*.liva`)
- **Phase 6.3.1 (v0.9.7)**: JSON arrays/objects (`http-json/json_*.liva`)

## Notes

- All examples use the latest Liva syntax
- Examples in `http-json/` demonstrate real-world patterns combining HTTP + JSON + generics
- Generic examples show both basic and advanced usage patterns
- Module examples demonstrate proper project organization
