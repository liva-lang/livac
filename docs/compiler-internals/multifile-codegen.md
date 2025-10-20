# Multi-File Code Generation

**Phase 3.5 - Module System v0.8.0**

## Overview

The multi-file code generation system transforms Liva modules into a structured Rust project with separate `.rs` files for each module. This enables proper code organization, visibility control, and seamless integration with Cargo's module system.

## Architecture

### Entry Point: `generate_multifile_project()`

```rust
pub fn generate_multifile_project(
    modules: &[&Module],
    entry_module: &Module,
    ctx: DesugarContext,
) -> Result<HashMap<PathBuf, String>>
```

**Responsibilities:**
- Iterate through all resolved modules
- Generate code for each module
- Create mod declarations for main.rs
- Return map of file paths → Rust code

**Flow:**
1. For each non-entry module → call `generate_module_code()`
2. Collect mod declarations (`mod math;`, `mod utils;`)
3. Generate entry point → call `generate_entry_point()`
4. Return HashMap with all files

### Module Code Generation

```rust
fn generate_module_code(
    module: &Module,
    ctx: &DesugarContext,
) -> Result<String>
```

**Responsibilities:**
- Generate use statements from imports
- Generate functions, classes, types with proper visibility
- Apply `pub` modifiers to public symbols
- Remove `_` prefix from private symbols

**Steps:**
1. Convert each `ImportDecl` to Rust use statement
2. For each top-level item:
   - Check visibility: `item.name.starts_with('_')` → private
   - Generate code using `CodeGenerator`
   - Prepend `pub` for public items
3. Concatenate all code

**Example Output:**
```rust
use crate::utils::square;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn internal_helper(x: i32) -> i32 {  // No pub - private
    x * 2
}
```

### Entry Point Generation

```rust
fn generate_entry_point(
    entry_module: &Module,
    mod_declarations: &[String],
    ctx: &DesugarContext,
) -> Result<String>
```

**Responsibilities:**
- Add mod declarations for all modules
- Convert entry module imports to use statements
- Generate liva_rt runtime module
- Generate main function and other entry-level code

**Structure:**
```rust
// 1. Mod declarations
mod math;
mod operations;
mod utils;

// 2. Use statements from imports
use crate::math::add;
use crate::operations::{multiply, divide};

// 3. liva_rt runtime (concurrency, errors)
mod liva_rt { ... }

// 4. Entry module code (main, tests, etc.)
#[tokio::main]
async fn main() { ... }
```

## Import Conversion

### Function: `generate_use_statement()`

Converts Liva import declarations to Rust use statements:

#### Single Import
```liva
import { add } from "./math.liva"
```
↓
```rust
use crate::math::add;
```

#### Multiple Imports
```liva
import { multiply, divide } from "./operations.liva"
```
↓
```rust
use crate::operations::{multiply, divide};
```

#### Wildcard Import
```liva
import * as utils from "./utils.liva"
```
↓
```rust
// Module already available via `mod utils;`
// No use statement needed
```

**Note:** Wildcard imports with aliases that match the module name are skipped in the entry point because the module is already declared with `mod`.

### Path Resolution

Liva path → Rust module path:
- `./math.liva` → `crate::math`
- `../utils/helper.liva` → `crate::helper` (simplified)
- Extension `.liva` is always stripped

## Visibility Rules

### Public Symbols (no `_` prefix)
- Get `pub` modifier in generated Rust code
- Accessible from other modules via use statements
- Example: `add()` → `pub fn add()`

### Private Symbols (`_` prefix)
- No `pub` modifier in Rust code
- Prefix `_` is removed in Rust output
- Only accessible within the same module
- Example: `_internal_calc()` → `fn internal_calc()`

**Implementation:**
```rust
let is_public = !func.name.starts_with('_');
if is_public {
    output.push_str("pub ");
}
```

## File Writing

### Function: `write_multifile_output()`

```rust
fn write_multifile_output(
    files: &HashMap<PathBuf, String>,
    cargo_toml: &str,
    output_dir: &Path,
) -> Result<PathBuf>
```

**Steps:**
1. Create output directory
2. For each (path, content) in files:
   - Create parent directories if needed
   - Write content to file
3. Write Cargo.toml

**Example Output:**
```
/tmp/liva_project/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── math.rs
    ├── operations.rs
    └── utils.rs
```

## Integration with Compiler Pipeline

### Modified: `compile_with_modules()` in `lib.rs`

**Old Flow (Phase 3.4):**
1. Resolve modules
2. Semantic analysis
3. Desugaring
4. **Generate single file** ← Changed
5. Write output

**New Flow (Phase 3.5):**
1. Resolve modules
2. Semantic analysis
3. Desugaring
4. **Generate multi-file project** ← New
5. Write all files to output directory

**Code:**
```rust
let files = codegen::generate_multifile_project(
    &compilation_order[..],
    entry_module,
    desugar_ctx.clone(),
)?;

let output_dir = if let Some(out_dir) = &options.output {
    Some(write_multifile_output(&files, &cargo_toml, out_dir)?)
} else {
    None
};
```

## Implementation Details

### `CodeGenerator` Access

Made `output` field `pub(crate)` to allow direct access:
```rust
pub struct CodeGenerator {
    pub(crate) output: String,  // Changed from private
    // ...
}
```

This enables:
```rust
codegen.output.clear();
codegen.generate_function(func)?;
let func_code = codegen.output.clone();
```

### DesugarContext Cloning

Added `#[derive(Clone)]` to allow reuse across modules:
```rust
#[derive(Clone)]
pub struct DesugarContext {
    // ...
}
```

Each module's codegen gets a cloned context.

## Examples

### Input: Three-Module Project

**main.liva:**
```liva
import { add } from "./math.liva"
import * as utils from "./utils.liva"

main() {
    let result = add(10, 20)
    print($"Result: {result}")
}
```

**math.liva:**
```liva
add(a: number, b: number): number => a + b
subtract(a: number, b: number): number => a - b
_internal_calc(x: number): number => x * 2
```

**utils.liva:**
```liva
square(x: number): number => x * x
```

### Output: Rust Project

**src/main.rs:**
```rust
mod math;
mod utils;

use crate::math::add;

mod liva_rt { /* runtime */ }

#[tokio::main]
async fn main() {
    let result = add(10, 20);
    println!("{}", format!("Result: {}", result));
}
```

**src/math.rs:**
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

fn internal_calc(x: i32) -> i32 {  // Private
    x * 2
}
```

**src/utils.rs:**
```rust
pub fn square(x: i32) -> i32 {
    x * x
}
```

## Known Issues & Limitations

### 1. Wildcard Import Simplification
**Issue:** `import * as utils from "./utils.liva"` generates no use statement.

**Reason:** Module is already available via `mod utils;`. Using `utils::square()` works directly.

**Future:** May want to generate `use crate::utils::*;` for convenience.

### 2. Path Resolution Simplification
**Current:** All relative paths resolve to `crate::module_name`.

**Limitation:** Nested directories not yet supported (`../parent/child.liva`).

**Future:** Full path resolution with proper parent/child relationships.

### 3. If-Expression Block Bug
**Issue:** If-else blocks get semicolons after expressions, breaking return values.

**Example:**
```liva
divide(a, b) {
    if b == 0 { 0 } else { a / b }
}
```
↓
```rust
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 { 0; } else { a / b; }  // Semicolons break it
}
```

**Status:** Pre-existing codegen bug, not specific to modules.

### 4. Namespace Collisions
**Current:** No check for Rust keyword conflicts.

**Example:** Module named `type.liva` would generate `mod type;` (invalid).

**Future:** Validate module names, suggest alternatives.

## Testing

### Test Case: `examples/modules/test_import_syntax.liva`

**Modules:**
- test_import_syntax.liva (entry)
- math.liva (public functions + private)
- operations.liva (multiple functions)
- utils.liva (utility functions)

**Compilation:**
```bash
./livac examples/modules/test_import_syntax.liva --output /tmp/test_modules
```

**Output:**
```
✓ Generated at /tmp/test_modules
✓ Compilation successful!
```

**Execution:**
```bash
cd /tmp/test_modules && cargo run
# Output: 10 + 20 = 30
```

**Verification:**
✅ 4 files generated (main.rs, math.rs, operations.rs, utils.rs)
✅ Mod declarations present
✅ Use statements correct
✅ Pub modifiers applied correctly
✅ Private functions without pub
✅ Compiles with `cargo build`
✅ Executes correctly

## Performance Notes

**Time Complexity:**
- O(M) where M = number of modules
- Each module generated independently
- No backtracking or multiple passes

**Memory:**
- HashMap stores all files in memory before writing
- Acceptable for reasonable project sizes (< 1000 modules)

**Typical Times:**
- 3-module project: ~0.5s compilation
- Includes: parsing, semantic, codegen, cargo build

## Future Enhancements

### Nested Modules
Support directory structure:
```
src/
├── main.rs
├── utils/
│   ├── mod.rs
│   ├── math.rs
│   └── string.rs
└── models/
    └── user.rs
```

### Re-exports
Support `pub use` for re-exporting:
```liva
// re-export from submodule
export { helper } from "./internal/helper.liva"
```

### Conditional Compilation
Support cfg attributes:
```liva
#[cfg(test)]
test_helper() { ... }
```

## References

- [Rust Module System](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Phase 3.5 Spec](../design/MODULE_SYSTEM_SPEC.md)
- [Module Resolution](./module-resolution.md)
- [Import Validation](./import-validation.md)

## Summary

Phase 3.5 successfully implements multi-file code generation:
- ✅ HashMap-based architecture
- ✅ Per-module code generation
- ✅ Import → use conversion
- ✅ Visibility control (pub/private)
- ✅ Entry point generation
- ✅ File writing integration
- ✅ Tested with 3-module example

**Time:** 2 hours (vs 13h estimated)
**Status:** COMPLETE
**Commits:** fae5280, 23c7335
