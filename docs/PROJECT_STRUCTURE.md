# 📂 Liva Project Organization

## Directory Structure

```
Liva/
├── livac/                    # Main compiler project
│   ├── src/                  # Compiler source code
│   ├── tests/                # Unit and integration tests
│   ├── examples/             # Example programs (organized)
│   │   ├── basic/            # Basic language features (6 files)
│   │   ├── generics/         # Generic programming (15 files)
│   │   ├── http-json/        # HTTP + JSON examples (31 files)
│   │   ├── stdlib/           # Standard library demos (12 files)
│   │   ├── calculator/       # Multi-file module example (3 files)
│   │   ├── modules/          # Module system examples (4 files)
│   │   ├── manual-tests/     # Legacy manual tests
│   │   └── README.md         # Examples documentation
│   ├── docs/                 # Comprehensive documentation
│   └── Cargo.toml            # Rust project configuration
│
├── vscode-extension/         # VS Code language extension
│   ├── src/                  # Extension TypeScript code
│   ├── syntaxes/             # Syntax highlighting
│   └── snippets/             # Code snippets
│
└── target/                   # Build artifacts (gitignored)
```

## Quick Start

### Run a basic example:
```bash
cd livac
cargo run -- examples/basic/test_basic.liva --run
```

### Run HTTP + JSON example:
```bash
cargo run -- examples/http-json/test_http_simple.liva --run
```

### Run generics example:
```bash
cargo run -- examples/generics/test_simple_generic.liva --run
```

### Run tests:
```bash
cd livac
cargo test
```

### Coverage report:
```bash
# Prerequisites (one-time):
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview

# Print summary table:
make coverage

# HTML report (browser-friendly):
make coverage-html
```

**Baseline (2026-04-29, 518 tests):**

| Module | Region cov | Line cov | Notes |
|--------|-----------:|---------:|-------|
| `desugaring.rs` | 99.16% | 98.54% | ✅ |
| `suggestions.rs` | 98.85% | 99.27% | ✅ |
| `lexer.rs` | 91.97% | 91.82% | ✅ |
| `traits.rs` | 91.80% | 92.68% | ✅ |
| `span.rs` | 90.96% | 88.24% | ✅ |
| `hints.rs` | 86.36% | 90.76% | ✅ |
| `error_codes.rs` | 83.62% | 77.33% | ✅ |
| `lib.rs` | 81.55% | 80.35% | ✅ |
| `parser.rs` | 77.10% | 80.58% | core |
| `codegen.rs` | 67.22% | 66.67% | core; large surface |
| `module.rs` | 67.09% | 56.45% | |
| `error.rs` | 62.27% | 63.79% | |
| `formatter.rs` | 55.88% | 58.81% | |
| `linter.rs` | 49.80% | 55.17% | |
| `semantic.rs` | 48.21% | 45.83% | core |
| `ast.rs` | 46.88% | 50.62% | mostly Display impls |
| `lsp/*` | 0–59% | — | requires LSP harness |
| `liva_rt.rs` | 0% | — | runtime template — only used in compiled output |
| `main.rs` | 18.95% | 36.39% | CLI entry — covered by E2E |
| **TOTAL** | **62.81%** | **62.36%** | — |

**Notes:** `liva_rt.rs`, `main.rs`, and `lsp/*` are intentionally low: they are
exercised end-to-end by `compiler/tests/e2e_selfhost.sh`, the Liva test suite
in `compiler/tests/liva/`, and manual LSP integration tests. Region coverage
in the compiler core (lexer/parser/semantic/codegen) is the primary metric.


## Example Categories

- **basic/** - Hello world, basic syntax, parallel vectors
- **generics/** - Type parameters, constraints, trait aliases
- **http-json/** - Real-world HTTP requests + JSON parsing + typed data
- **stdlib/** - Array methods, string operations, math functions
- **calculator/** - Multi-file project with module system
- **modules/** - Import/export patterns

## Documentation

- **livac/docs/** - Complete language reference and guides
- **livac/examples/README.md** - Detailed examples documentation
- **ROADMAP.md** - Development roadmap and progress

## Development

All `.liva` test files are organized in `livac/examples/` subdirectories.
No loose test files in the project root.

See `livac/examples/README.md` for detailed examples documentation.
