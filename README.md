# 🧩 Liva Programming Language

> *The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.*

**Liva** is a modern programming language that compiles to Rust, featuring:
- 🎯 **Clean, minimal syntax** - Write less, express more
- ⚡ **Hybrid concurrency** - Mix async (I/O) and parallel (CPU) seamlessly  
- 🛡️ **Explicit error handling** - Fallibility system with error binding
- 🔒 **Memory safety** - Compiles to Rust for zero-cost abstractions
- 🚀 **High performance** - Native speed with no runtime overhead

## ⚡ Quick Example

```liva
// Define a class
User {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  canVote() => this.age >= 18
}

// Async function with error handling
fetchUserData(id: number): string {
  if id < 0 fail "Invalid ID"
  return $"User data for {id}"
}

// Parallel computation
heavyCalc(n: number) => n * n

main() {
  // Create instances
  let user = User("Alice", 25)
  print($"{user.name} can vote: {user.canVote()}")
  
  // Hybrid concurrency
  let data = async fetchUserData(123)    // I/O-bound: async
  let result = par heavyCalc(1000)       // CPU-bound: parallel
  
  print($"Data: {data}, Result: {result}")
  
  // Error handling
  let value, err = fetchUserData(-1)
  if err != "" {
    print($"Error: {err}")
  }
}
```

**Output:**
```
Alice can vote: true
Data: User data for 123, Result: 1000000
Error: Invalid ID
```

## 🚀 Installation

### Prerequisites
- Rust 1.70+ and Cargo
- Git

### Install from Source

```bash
git clone https://github.com/liva-lang/livac.git
cd livac
cargo build --release
cargo install --path .
```

### Verify Installation

```bash
livac --version  # Should output: livac 0.6.0
```

## 📖 Usage

### Compile and Run

```bash
# Compile
livac program.liva

# Compile and run immediately
livac program.liva --run

# Check syntax only
livac program.liva --check

# Show generated Rust code
livac program.liva --verbose
```

### Options

```
livac <file.liva> [OPTIONS]

OPTIONS:
  -r, --run         Compile and execute immediately
  -c, --check       Check syntax without compiling
  -v, --verbose     Show generated Rust code
  -o, --output DIR  Set output directory (default: ./target/liva_build)
      --json        Output errors in JSON (for IDE integration)
  -h, --help        Show help
```

## 🎯 Key Features

### 1. Hybrid Concurrency

Mix **async** (for I/O) and **par** (for CPU) in the same program:

```liva
main() {
  let data = async fetchFromAPI()      // Non-blocking I/O
  let result = par complexCalc()       // Parallel computation
  
  fire async logEvent("started")       // Fire-and-forget
  
  let task1 = task async operation()   // Explicit task handle
  let value = await task1              // Await when needed
}
```

### 2. Explicit Error Handling

No exceptions - errors are values with **error binding**:

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

main() {
  let result, err = divide(10, 0)
  
  if err != "" {
    print($"Error: {err}")  // "Division by zero"
  } else {
    print($"Result: {result}")
  }
}
```

### 3. Clean Syntax

```liva
// One-liner functions
square(x) => x * x

// Block functions with types
calculate(a: number, b: number): number {
  return a + b * 2
}

// String templates
greet(name) {
  print($"Hello, {name}!")
}

// Classes with visibility
Person {
  name: string           // public
  _age: number           // protected (pub(super))
  __ssn: string          // private (no pub)
  
  isAdult() => this._age >= 18
}
```

### 4. Full Rust Interoperability

```liva
// Use Rust types directly
let count: u64 = 0
let temp: f32 = 21.5

// Use Rust crates (planned)
use rust "reqwest" as http

fetchData() {
  let res = async http.get("https://api.example.com")
  return res.json()
}
```

## 📚 Documentation

**Complete documentation is available in [`docs/`](docs/README.md):**

### 🚀 Getting Started
- **[Installation Guide](docs/getting-started/installation.md)** - Set up Liva
- **[Quick Start](docs/getting-started/quick-start.md)** - Your first program in 5 minutes
- **[Basic Concepts](docs/getting-started/basic-concepts.md)** - Core language concepts
- **[Examples](docs/getting-started/examples.md)** - Common patterns

### 📘 Language Reference
- **[Syntax Overview](docs/language-reference/syntax-overview.md)** - Grammar and syntax
- **[Types](docs/language-reference/types.md)** - Type system
- **[Functions](docs/language-reference/functions.md)** - Function declarations
- **[Classes](docs/language-reference/classes.md)** - Object-oriented programming
- **[Concurrency](docs/language-reference/concurrency.md)** - async, par, task, fire
- **[Error Handling](docs/language-reference/error-handling.md)** - Fallibility system
- **[Control Flow](docs/language-reference/control-flow.md)** - if, for, while, switch

### 🔧 Compiler Internals
- **[Architecture](docs/compiler-internals/architecture.md)** - Compiler pipeline
- **[Lexer](docs/compiler-internals/lexer.md)** - Tokenization
- **[Parser](docs/compiler-internals/parser.md)** - AST construction
- **[Semantic Analysis](docs/compiler-internals/semantic.md)** - Type checking
- **[IR](docs/compiler-internals/ir.md)** - Intermediate representation
- **[Code Generation](docs/compiler-internals/codegen.md)** - Rust emission

### 📚 Guides
- **[Async Programming](docs/guides/async-programming.md)** - Mastering async/await
- **[Parallel Computing](docs/guides/parallel-computing.md)** - CPU-bound parallelism
- **[Hybrid Concurrency](docs/guides/hybrid-concurrency.md)** - Mixing async + parallel
- **[Error Handling Patterns](docs/guides/error-handling-patterns.md)** - Best practices

## 🏗️ How It Works

```
Liva Source (.liva)
       ↓
[1] Lexer → Tokens (logos)
       ↓
[2] Parser → AST
       ↓
[3] Semantic Analysis
       ├─ Type inference
       ├─ Async inference
       └─ Visibility validation
       ↓
[4] IR Lowering → Typed IR
       ↓
[5] Code Generation → Rust
       ├─ main.rs
       ├─ liva_rt.rs (if async/par used)
       └─ Cargo.toml
       ↓
[6] Cargo Build → Native Binary
```

**Key Transformations:**

| Liva | Rust |
|------|------|
| `let x = 10` | `let mut x: i32 = 10;` |
| `const PI = 3.14` | `const PI: f64 = 3.14;` |
| `async call()` | `liva_rt::run_async(async { call() })` |
| `par call()` | `liva_rt::run_parallel(\|\| call())` |
| `fail "msg"` | `return Err("msg".to_string());` |
| `$"Hello {x}"` | `format!("Hello {}", x)` |
| `and`, `or`, `not` | `&&`, `\|\|`, `!` |

**Learn more:** See [Compiler Architecture](docs/compiler-internals/architecture.md)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test lexer
cargo test parser
cargo test codegen

# Run with output
cargo test -- --nocapture

# Update snapshots (after changing codegen)
cargo insta review
```

**Test Coverage:**
- ✅ Lexer: Token recognition
- ✅ Parser: AST construction  
- ✅ Semantic: Type checking, async inference
- ✅ IR: Lowering and type resolution
- ✅ Codegen: Rust code generation
- ✅ Integration: End-to-end compilation

## �️ Development

### Build from Source

```bash
git clone https://github.com/liva-lang/livac.git
cd livac
cargo build --release
```

### Development Workflow

```bash
# Run in dev mode
cargo run -- examples/hello.liva --run

# Watch for changes (requires cargo-watch)
cargo watch -x 'run -- test.liva --run'

# Format code
cargo fmt

# Lint
cargo clippy
```

### Project Structure

```
livac/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library interface
│   ├── lexer.rs         # Tokenization (300 lines)
│   ├── parser.rs        # AST construction (1500 lines)
│   ├── semantic.rs      # Type checking (600 lines)
│   ├── ir.rs            # IR definitions (400 lines)
│   ├── lowering.rs      # AST → IR (800 lines)
│   ├── codegen.rs       # IR → Rust (2000 lines)
│   ├── error.rs         # Error reporting (400 lines)
│   └── span.rs          # Source locations (100 lines)
├── docs/
│   ├── README.md        # Documentation index
│   ├── getting-started/ # Installation, quick start, examples
│   ├── language-reference/  # Complete language spec
│   ├── compiler-internals/  # Architecture, AST, IR, codegen
│   ├── guides/          # Advanced topics, patterns
│   └── api/             # Standard library reference
├── tests/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   ├── semantics_tests.rs
│   ├── codegen_tests.rs
│   └── integration_tests.rs
├── Cargo.toml
└── README.md            # This file
```

## 🎯 Current Status

**Version:** 0.6.0  
**Status:** Alpha - Production-ready for experimentation

### ✅ Fully Implemented

- Core syntax (variables, functions, classes, control flow)
- Hybrid concurrency (async, par, task, fire)
- Fallibility system (fail, error binding)
- String templates and interpolation
- Visibility modifiers (public, protected, private)
- Type inference (basic)
- Async inference (complete)
- Full Rust code generation
- Comprehensive error reporting
- IR-based compilation pipeline

### 🚧 Work in Progress

- Strict type checking (currently permissive)
- Cross-module imports
- Generic types
- Pattern matching
- Module system

### 📋 Roadmap

**v0.7 - Type System** (Q4 2025)
- Strict type checking
- Generic functions and classes
- Type aliases and unions

**v0.8 - Modules** (Q1 2026)
- Module system with imports
- Package manager integration
- Standard library

**v0.9 - Advanced Features** (Q2 2026)
- Pattern matching
- Traits/interfaces
- Macros

**v1.0 - Stable Release** (Q3 2026)
- Language Server Protocol (LSP)
- Debugger integration
- Production-ready toolchain

## � Error Reporting

Liva provides exceptional error messages with:

✅ Unique error codes (E1xxx, E2xxx, E0xxx, E3xxx)  
✅ Precise source locations  
✅ Code snippets with visual indicators  
✅ Helpful suggestions  
✅ Color-coded terminal output  
✅ JSON format for IDE integration

**Example:**
```
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → test.liva:6:7

     6 │
       │ let x = 20
       │     ^^^
       │

  ⓘ Variable 'x' already defined in this scope

  💡 Consider using a different name or removing the previous declaration
────────────────────────────────────────────────────────────
```

See [Error System Documentation](docs/compiler-internals/error-system.md) for complete details.

## � IDE Support

### VS Code Extension

Full IDE support with:
- ✅ Syntax highlighting
- ✅ Code completion (IntelliSense)
- ✅ Hover documentation
- ✅ Signature help
- ✅ Go to Definition (F12)
- ✅ Find All References (Shift+F12)
- ✅ Outline view and breadcrumbs
- ✅ Real-time error diagnostics

**Install:**
```bash
cd vscode-extension
npm install && npm run compile
code --install-extension liva-vscode-0.1.0.vsix
```

Or search for "Liva" in the VS Code Marketplace (coming soon).

## 🤝 Contributing

Contributions are welcome! Here's how to get started:

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** your changes: `git commit -m 'Add amazing feature'`
4. **Push** to the branch: `git push origin feature/amazing-feature`
5. **Open** a Pull Request

### Coding Standards

- Follow Rust conventions (`cargo fmt`, `cargo clippy`)
- Add tests for new features
- Update documentation
- Write clear commit messages

### Areas for Contribution

- 📝 Documentation improvements
- 🐛 Bug fixes
- ✨ New language features
- 🧪 More test cases
- 🎨 Error message improvements
- 📚 Example programs

## 📄 License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

## 👤 Author

**Fran Nadal**  
Creator and maintainer of Liva

## 🙏 Acknowledgments

- **Rust Community** - For excellent tooling and libraries
- **Logos** - Fast lexer generation
- **Tokio** - Async runtime
- **Insta** - Snapshot testing
- All contributors and early adopters!

## 📞 Support & Community

- � **Documentation**: [docs/README.md](docs/README.md)
- 💬 **Discussions**: GitHub Discussions
- � **Bug Reports**: GitHub Issues
- 📧 **Email**: fran@liva-lang.org
- 🌐 **Website**: https://liva-lang.org (coming soon)

## 🔗 Related Projects

- **[vscode-extension](vscode-extension/)** - VS Code language extension
- **[docs](docs/)** - Complete documentation
- **[examples](examples/)** - Example Liva programs (coming soon)

---

**Made with ❤️ for developers who want Python's simplicity, TypeScript's clarity, and Rust's performance.**

**Start coding with Liva today! 🚀**

```bash
livac --help
```
