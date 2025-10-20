# 📋 Phase 3: Module System (v0.8.0) - TODO

> **Branch:** `feature/modules-v0.8.0`  
> **Status:** 🚧 In Progress  
> **Started:** 2025-10-20  
> **Goal:** Multi-file projects with import/export

---

## 🎯 Overview

Implement a module system for Liva that allows organizing code across multiple files:

**Syntax:**
```liva
// Public by default (no _ prefix)
add(a: Int, b: Int) -> Int { return a + b }

// Private (with _ prefix)
_helper() { /* not exported */ }

// Import
import { add } from "./math.liva"
import * as math from "./math.liva"
```

**Key Features:**
- JavaScript-style `import` syntax
- Public by default (no `_` prefix)
- Private with `_` prefix (consistent with Liva conventions)
- Wildcard imports for namespaces
- Relative path resolution

---

## 📝 Tasks

### Phase 1: Parser & AST (Days 1-2) 📋 Not Started

**Goal:** Parse import statements and update AST

#### 1.1 Update AST (~2 hours)
- [ ] Define `ImportDecl` struct in `ast.rs`
  ```rust
  pub struct ImportDecl {
      pub imports: Vec<String>,      // ["add", "multiply"]
      pub source: String,             // "./math.liva"
      pub is_wildcard: bool,          // import *
      pub alias: Option<String>,      // as name
  }
  ```
- [ ] Add `Import(ImportDecl)` to `TopLevelItem` enum
- [ ] Implement `Display` for `ImportDecl` (for debugging)

#### 1.2 Implement Parser (~4 hours)
- [ ] Add `import` keyword to lexer (if not already there)
- [ ] Parse `import { name1, name2 } from "path"`
  - Recognize `import` keyword
  - Parse braces with comma-separated identifiers
  - Parse `from` keyword
  - Parse string literal for path
- [ ] Parse `import * as alias from "path"`
  - Recognize `*` for wildcard
  - Parse `as` keyword and alias identifier
- [ ] Handle edge cases:
  - Empty import list: `import {} from "path"` (error)
  - Missing braces: `import add from "path"` (error)
  - Missing path: `import { add }` (error)
  - Invalid path: `import { add } from 123` (error)

#### 1.3 Parser Tests (~2 hours)
- [ ] Test named imports
  - Single: `import { add } from "./math.liva"`
  - Multiple: `import { add, multiply, PI } from "./math.liva"`
- [ ] Test wildcard imports
  - Basic: `import * as math from "./math.liva"`
  - Different aliases: `import * as m from "./math.liva"`
- [ ] Test error cases
  - Missing braces
  - Missing path
  - Invalid syntax
- [ ] Test with other top-level items
  - Imports + functions
  - Imports + classes
  - Multiple imports

**Deliverable:** Parser can parse all import syntax variants

---

### Phase 2: Module Resolver (Days 3-4) 📋 Not Started

**Goal:** Load and resolve imported files

#### 2.1 Module Data Structures (~2 hours)
- [ ] Create `module.rs` file
- [ ] Define `Module` struct
  ```rust
  pub struct Module {
      pub path: PathBuf,
      pub ast: Vec<TopLevelItem>,
      pub public_symbols: HashSet<String>,
      pub private_symbols: HashSet<String>,
      pub imports: Vec<ImportDecl>,
  }
  ```
- [ ] Define `ModuleResolver` struct
  ```rust
  pub struct ModuleResolver {
      modules: HashMap<PathBuf, Module>,
      entry_point: PathBuf,
      root_dir: PathBuf,
  }
  ```
- [ ] Define `DependencyGraph` for cycle detection
  ```rust
  pub struct DependencyGraph {
      edges: HashMap<PathBuf, Vec<PathBuf>>,
  }
  ```

#### 2.2 Path Resolution (~3 hours)
- [ ] Implement `resolve_import_path()`
  - Handle relative paths: `./`, `../`
  - Resolve relative to current file's directory
  - Ensure path ends with `.liva`
  - Check file exists on filesystem
- [ ] Handle different path formats
  - Same directory: `./file.liva`
  - Subdirectory: `./sub/file.liva`
  - Parent directory: `../file.liva`
  - Multiple levels: `../../file.liva`
- [ ] Path normalization
  - Canonicalize paths to avoid duplicates
  - Handle `.` and `..` in paths

#### 2.3 Module Loading (~4 hours)
- [ ] Implement `load_module(path)`
  - Read file from filesystem
  - Parse with existing parser
  - Extract public symbols (no `_` prefix)
  - Extract private symbols (`_` prefix)
  - Cache in `modules` HashMap
- [ ] Implement recursive loading
  - Parse imports from loaded module
  - Recursively load imported modules
  - Build dependency graph
- [ ] Module caching
  - Don't re-parse same file
  - Use canonical paths as keys

#### 2.4 Dependency Graph (~3 hours)
- [ ] Build dependency graph while loading
- [ ] Implement cycle detection
  - Use DFS to detect cycles
  - Report full cycle path in error
- [ ] Topological sort for compilation order

#### 2.5 Resolver Tests (~3 hours)
- [ ] Test path resolution
  - Relative paths
  - Different directory levels
  - Path normalization
- [ ] Test module loading
  - Single file
  - Multiple files
  - Recursive imports
- [ ] Test cycle detection
  - Direct cycle: A → B → A
  - Indirect cycle: A → B → C → A
- [ ] Test caching
  - Same file imported twice
  - Diamond dependency: A → B, A → C, B → D, C → D

**Deliverable:** ModuleResolver can load multi-file projects

---

### Phase 3: Semantic Analysis (Day 5) 📋 Not Started

**Goal:** Validate imports and symbol resolution

#### 3.1 Symbol Validation (~3 hours)
- [ ] Validate imported symbols exist
  - Check against module's public_symbols
  - Error if symbol not found
  - Suggest similar names (did you mean?)
- [ ] Validate imported symbols are public
  - Error if importing `_` prefixed symbol
  - Clear error message about privacy
- [ ] Handle wildcard imports
  - Import all public symbols
  - Store namespace alias

#### 3.2 Scope Resolution (~3 hours)
- [ ] Update semantic analyzer for imports
- [ ] Extend symbol table with imported symbols
  - Named imports: add to current scope
  - Wildcard imports: add namespace to scope
- [ ] Handle name collisions
  - Error if import conflicts with local definition
  - Error if two imports have same name
- [ ] Validate symbol usage
  - Check imported symbols are used
  - Warn about unused imports

#### 3.3 Semantic Tests (~2 hours)
- [ ] Test symbol validation
  - Valid imports
  - Non-existent symbols
  - Private symbol imports
- [ ] Test name collisions
  - Import vs local function
  - Import vs import
- [ ] Test wildcard imports
  - Accessing namespace members
  - Name resolution through namespace

**Deliverable:** Semantic analysis validates all imports

---

### Phase 4: Code Generation (Days 6-7) 📋 Not Started

**Goal:** Generate multi-file Rust projects

#### 4.1 Project Structure Generation (~3 hours)
- [ ] Generate `src/` directory structure
  - Mirror Liva file structure
  - Create subdirectories for nested modules
  - Generate `mod.rs` files for directories
- [ ] Generate module files
  - One `.rs` file per `.liva` file
  - Convert path: `math.liva` → `src/math.rs`
  - Nested: `ops/basic.liva` → `src/ops/basic.rs`

#### 4.2 Module Declarations (~2 hours)
- [ ] Generate `mod` declarations in parent files
  - In `main.rs` for top-level modules
  - In `mod.rs` for nested modules
- [ ] Example:
  ```rust
  // main.rs
  mod math;
  mod operations;
  ```

#### 4.3 Import/Use Statements (~3 hours)
- [ ] Convert Liva imports to Rust `use`
  - `import { add } from "./math.liva"` → `use math::add;`
  - `import * as m from "./math.liva"` → `use math;` (alias later)
- [ ] Handle relative paths in Rust
  - `./file` → `self::file` or direct module name
  - `../file` → `super::file`
- [ ] Handle nested modules
  - `./ops/basic.liva` → `use operations::basic;`

#### 4.4 Visibility Modifiers (~2 hours)
- [ ] Add `pub` to public symbols
  - Functions without `_` prefix
  - Classes without `_` prefix
  - Constants without `_` prefix
- [ ] Keep private symbols without `pub`
  - Functions with `_` prefix
  - Remove `_` prefix in generated Rust code

#### 4.5 Code Generation Tests (~3 hours)
- [ ] Test module structure generation
- [ ] Test `mod` declarations
- [ ] Test `use` statements
- [ ] Test `pub` modifiers
- [ ] Test relative path conversion
- [ ] Full integration: multi-file compilation

**Deliverable:** Multi-file Liva projects compile to Rust

---

### Phase 5: Integration & Examples (Day 8) 📋 Not Started

**Goal:** Working examples and documentation

#### 5.1 Example Projects (~3 hours)
- [ ] Create calculator example
  ```
  calculator/
  ├── main.liva
  ├── operations/
  │   ├── basic.liva
  │   └── advanced.liva
  └── models/
      └── calculator.liva
  ```
- [ ] Create multi-module app example
- [ ] Test end-to-end compilation
- [ ] Verify generated Rust compiles and runs

#### 5.2 Documentation (~2 hours)
- [ ] Write module system user guide
- [ ] Add import/export reference docs
- [ ] Update language reference
- [ ] Create migration guide

#### 5.3 Testing & Polish (~3 hours)
- [ ] Run full test suite
- [ ] Fix any remaining bugs
- [ ] Improve error messages
- [ ] Add more tests if needed

#### 5.4 Release Preparation (~1 hour)
- [ ] Update CHANGELOG.md
- [ ] Update ROADMAP.md
- [ ] Update version to v0.8.0
- [ ] Create release notes

**Deliverable:** Liva v0.8.0 ready for release

---

## 📊 Progress Tracking

| Phase | Tasks | Estimated | Actual | Status |
|-------|-------|-----------|--------|--------|
| **1. Parser** | 3 tasks | 8h | - | 📋 Not Started |
| **2. Resolver** | 5 tasks | 15h | - | 📋 Not Started |
| **3. Semantic** | 3 tasks | 8h | - | 📋 Not Started |
| **4. Codegen** | 5 tasks | 13h | - | 📋 Not Started |
| **5. Integration** | 4 tasks | 9h | - | 📋 Not Started |
| **Total** | **20 tasks** | **53h** | **-** | 📋 **0%** |

---

## 🎯 Success Metrics

- [ ] All parser tests passing (10+ tests)
- [ ] All resolver tests passing (15+ tests)
- [ ] All semantic tests passing (10+ tests)
- [ ] All codegen tests passing (10+ tests)
- [ ] Calculator example compiles and runs
- [ ] Multi-module example compiles and runs
- [ ] Documentation complete
- [ ] Zero compiler warnings

---

## 🚀 Next Steps

1. **Start with Parser** (Phase 1)
   - Define AST nodes
   - Implement import parsing
   - Write parser tests

2. **Then Module Resolver** (Phase 2)
   - Path resolution
   - File loading
   - Dependency graph

3. **Semantic Analysis** (Phase 3)
   - Symbol validation
   - Scope resolution

4. **Code Generation** (Phase 4)
   - Multi-file Rust projects
   - Module declarations
   - Use statements

5. **Polish & Release** (Phase 5)
   - Examples
   - Documentation
   - Testing

---

**Let's build the module system! 🚀**

**Current Status:** Ready to start Phase 1 (Parser & AST)
