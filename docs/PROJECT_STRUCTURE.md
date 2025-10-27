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
