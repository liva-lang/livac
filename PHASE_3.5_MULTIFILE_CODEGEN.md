# Phase 3.5: Multi-File Code Generation

**Branch:** `feature/phase-3.5-multifile-codegen`  
**Status:** 🚧 Planning  
**Target Version:** v0.12.0  
**Created:** 2025-10-29

## Overview

Currently, the Liva compiler supports:
- ✅ Phase 3.1: Module syntax parsing
- ✅ Phase 3.2: AST representation
- ✅ Phase 3.3: Module resolver
- ✅ Phase 3.4: Semantic analysis with imports
- ❌ Phase 3.5: **Multi-file code generation** (THIS PHASE)

The module system can parse `import` statements and validate symbols across files, but the code generator only compiles the entry point file. This phase will implement full multi-file compilation.

## Problem Statement

**Current Behavior:**
```liva
// main.liva
import { Task } from "./domain/task"

main() {
    let task = Task("Do something", Status("PENDING"))
    print(task.title)
}
```

```bash
$ livac main.liva -r
# ❌ Rust compilation error: cannot find type `Task`
# The code generator doesn't include imported modules
```

**Expected Behavior:**
```bash
$ livac main.liva -r
# ✅ Compiles all imported files
# ✅ Generates proper Rust module structure
# ✅ Runs successfully
```

## Current Architecture

### Module Resolution (✅ Working)

Located in `src/resolver/mod.rs`:
- `ModuleResolver` collects all files
- Builds dependency graph
- Validates import paths
- Resolves symbols across modules

### Code Generation (❌ Incomplete)

Located in `src/codegen/mod.rs`:
- Only generates code for entry point
- Ignores imported modules
- Creates monolithic `main.rs`

## Implementation Plan

### Step 1: Analyze Current Code Generator

**Files to review:**
- `src/codegen/mod.rs` - Main code generator
- `src/codegen/rust_codegen.rs` - Rust code generation
- `src/resolver/module_resolver.rs` - Module resolution

**Key questions:**
1. How does `generate_rust()` currently work?
2. Where is the entry point determined?
3. How are symbols currently exported?

### Step 2: Design Multi-File Generation Strategy

**Option A: Inline Everything (Current)**
```
Entry File → Parse → Resolve Imports → Generate Single main.rs
```

**Option B: Rust Modules (Proposed)**
```
Entry File → Parse → Resolve Imports → Generate Multiple .rs Files
           ↓
    Import Files → Parse → Generate Module Files
           ↓
    Link All → cargo build
```

**Option C: Hybrid Approach**
```
Entry File → Parse → Resolve Imports → Generate main.rs + mod.rs
           ↓
    Generate helper modules for each .liva file
```

### Step 3: Implement Module Code Generation

**3.1 Create Module Generator**
```rust
// src/codegen/module_codegen.rs

pub struct ModuleCodegen {
    module_graph: ModuleGraph,
    output_dir: PathBuf,
}

impl ModuleCodegen {
    pub fn generate_modules(&self) -> Result<Vec<GeneratedModule>> {
        // For each module in graph:
        // 1. Generate Rust code
        // 2. Create .rs file
        // 3. Track exports
    }
}
```

**3.2 Update Main Codegen**
```rust
// src/codegen/mod.rs

pub fn generate_rust_multifile(
    entry_point: &str,
    module_graph: &ModuleGraph,
    output_dir: &Path,
) -> Result<()> {
    let module_gen = ModuleCodegen::new(module_graph, output_dir);
    
    // Generate all module files
    let modules = module_gen.generate_modules()?;
    
    // Generate main.rs with proper use statements
    let main_rs = generate_main_with_imports(&modules)?;
    
    // Write mod.rs to expose modules
    let mod_rs = generate_mod_file(&modules)?;
    
    Ok(())
}
```

**3.3 Generate Rust Module Structure**
```
target/liva_build/src/
├── main.rs           # Entry point with imports
├── lib.rs            # Or mod.rs
├── domain/
│   ├── mod.rs        # Module declarations
│   ├── task.rs       # From domain/task.liva
│   └── status.rs     # From domain/status.liva
└── shared/
    ├── mod.rs
    └── result.rs     # From shared/result.liva
```

### Step 4: Handle Symbol Exports

**4.1 Track Exported Symbols**
```rust
struct ModuleExports {
    module_path: String,
    classes: Vec<String>,
    functions: Vec<String>,
    type_aliases: Vec<String>,
}
```

**4.2 Generate Use Statements**
```rust
// In main.rs
use crate::domain::task::Task;
use crate::domain::status::Status;
use crate::shared::result::{Result, ok, err};
```

### Step 5: Resolve Import Paths

**5.1 Convert Liva Imports to Rust Paths**
```liva
import { Task } from "./domain/task"
import { Result } from "../shared/result"
```

↓

```rust
use crate::domain::task::Task;
use crate::shared::result::Result;
```

**5.2 Handle Different Import Styles**
- Named imports: `import { A, B } from "./mod"`
- Default imports: `import A from "./mod"`
- Wildcard imports: `import * from "./mod"`

### Step 6: Testing Strategy

**6.1 Unit Tests**
```rust
#[test]
fn test_generate_module_file() {
    let module = parse_module("task.liva");
    let rust_code = generate_module_code(module);
    assert!(rust_code.contains("pub struct Task"));
}

#[test]
fn test_import_path_conversion() {
    let liva_path = "./domain/task";
    let rust_path = convert_to_rust_path(liva_path);
    assert_eq!(rust_path, "crate::domain::task");
}
```

**6.2 Integration Tests**
Create test cases in `tests/multifile/`:
- Simple two-file project
- Nested modules
- Circular dependencies
- Re-exports

**6.3 Test with Real Application**
Use the task management system from `test_workspace/`:
```bash
cd test_workspace/task-management-system
livac main.liva -r
# Should compile and run with all imports
```

## Implementation Checklist

### Phase 3.5.1: Foundation (Week 1)
- [ ] Analyze current code generator architecture
- [ ] Design module generation strategy
- [ ] Create `ModuleCodegen` struct
- [ ] Implement basic module file generation
- [ ] Write unit tests

### Phase 3.5.2: Import Resolution (Week 2)
- [ ] Convert Liva import paths to Rust paths
- [ ] Handle relative imports (./,  ../)
- [ ] Generate `use` statements
- [ ] Track symbol exports
- [ ] Test import resolution

### Phase 3.5.3: Multi-File Generation (Week 3)
- [ ] Generate main.rs with imports
- [ ] Generate mod.rs files
- [ ] Create module directory structure
- [ ] Handle module visibility (pub)
- [ ] Test with 2-3 file projects

### Phase 3.5.4: Advanced Features (Week 4)
- [ ] Handle re-exports
- [ ] Support wildcard imports
- [ ] Implement circular dependency detection
- [ ] Add incremental compilation
- [ ] Performance optimization

### Phase 3.5.5: Testing & Documentation (Week 5)
- [ ] Create comprehensive test suite
- [ ] Test with task management system
- [ ] Write migration guide
- [ ] Update language documentation
- [ ] Create examples

## Success Criteria

1. **Functionality**
   - ✅ Can compile multi-file Liva projects
   - ✅ Generates proper Rust module structure
   - ✅ Resolves all imports correctly
   - ✅ Maintains symbol visibility

2. **Compatibility**
   - ✅ Backward compatible with single-file projects
   - ✅ Works with existing module resolver
   - ✅ No breaking changes to syntax

3. **Quality**
   - ✅ All tests pass
   - ✅ No compilation errors
   - ✅ Generated code is idiomatic Rust
   - ✅ Performance acceptable

4. **Documentation**
   - ✅ Updated language reference
   - ✅ Migration guide written
   - ✅ Examples provided
   - ✅ CHANGELOG updated

## Example: Before & After

### Before (v0.11.3)

**Project Structure:**
```
src/
├── domain/
│   ├── task.liva
│   └── status.liva
└── main.liva
```

**Compilation:**
```bash
$ livac main.liva -r
Error: cannot find type `Task` in this scope
```

**Workaround:** Concatenate all files into one

### After (v0.12.0)

**Same Project Structure:**
```
src/
├── domain/
│   ├── task.liva
│   └── status.liva
└── main.liva
```

**Compilation:**
```bash
$ livac main.liva -r
🧩 Liva Compiler v0.12.0
→ Compiling main.liva
→ Compiling domain/task.liva
→ Compiling domain/status.liva
✓ Generated at ./target/liva_build
✓ Compilation successful!

Running program:
============================================================
Task created: Implement authentication
Status: PENDING
```

## Related Issues

- Task management system requires multi-file support
- Module system documentation promises this feature
- Community requests for larger projects

## References

- **Module Resolver:** `src/resolver/module_resolver.rs`
- **Code Generator:** `src/codegen/mod.rs`
- **Module Documentation:** `docs/language-reference/modules.md`
- **Test Application:** `test_workspace/task-management-system/`

## Notes

- This is a **critical** feature for production readiness
- Enables proper code organization
- Allows for larger, maintainable projects
- Completes the module system implementation (Phase 3)

---

**Next Steps:**
1. Review current code generator
2. Create proof-of-concept for 2-file compilation
3. Implement full multi-file generation
4. Test with task management system
5. Document and release v0.12.0
