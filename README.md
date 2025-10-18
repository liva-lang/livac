# 🚀 Liva Programming Language

> *The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.*

**Liva** is a modern, statically-typed programming language that compiles to Rust. Designed for developers who want expressive syntax without sacrificing performance or safety.

## ✨ Why Liva?

- 🎯 **Clean, minimal syntax** - Write less, express more
- ⚡ **Hybrid concurrency** - Mix async (I/O) and parallel (CPU) seamlessly  
- 🛡️ **Explicit error handling** - No exceptions, errors are values
- 🏗️ **Interface-based design** - Clean abstractions without inheritance
- 🔒 **Memory safety** - Compiles to Rust for zero-cost abstractions
- 🚀 **Native performance** - No runtime overhead, no garbage collector

## ⚡ Quick Example

```liva
// Define an interface (only method signatures)
Greetable {
    greet(): string
    introduce(): string
}

// Implement interface in a class
User : Greetable {
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  name: string
  age: number
  
  // Interface implementation
  greet() => $"Hello!"
  introduce() => $"I'm {this.name}, {this.age} years old"
  
  // Additional methods
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
  print(user.introduce())
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
I'm Alice, 25 years old
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

### 1. Interface-Based Design

Define contracts with interfaces, implement in classes:

```liva
// Interface: only method signatures
Animal {
    makeSound(): string
    getName(): string
}

// Multiple interfaces
Drawable {
    draw(): void
}

// Class implementing multiple interfaces
Dog : Animal, Drawable {
    constructor(name: string) {
        this.name = name
    }
    
    name: string
    
    // Implement Animal
    makeSound() => "Woof!"
    getName() => this.name
    
    // Implement Drawable
    draw() {
        print($"Drawing a dog named {this.name}")
    }
}
```

**Key points:**
- Interfaces have **only method signatures** (no fields, no constructor)
- Classes implement interfaces using `:` syntax
- Support for **multiple interfaces** with comma-separated list
- Compiles to Rust traits for zero-cost abstractions

### 2. Hybrid Concurrency

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

### 2. Hybrid Concurrency

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

### 3. Explicit Error Handling

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

### 4. Clean Syntax

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

### 5. Full Rust Interoperability

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

## 📚 Language Guide & Documentation

**Complete documentation:** [`docs/`](docs/README.md)

### � Language Guide (Start Here!)

#### Core Concepts
- **[Variables](docs/language-reference/variables.md)** - `let`, `const`, mutability, type inference
- **[Functions](docs/language-reference/functions.md)** - One-liners, blocks, parameters, return types
- **[Types](docs/language-reference/types.md)** - Primitives, arrays, optionals, type system
- **[Operators](docs/language-reference/operators.md)** - Arithmetic, logical, comparison, bitwise

#### Object-Oriented Programming
- **[Classes](docs/language-reference/classes.md)** - Constructors, fields, methods, visibility
- **[Interfaces](docs/language-reference/classes.md#interfaces)** - Contracts, implementation, multiple interfaces
- **[Visibility](docs/language-reference/visibility.md)** - Public, protected, private

#### Control Flow & Logic
- **[Control Flow](docs/language-reference/control-flow.md)** - `if`, `while`, `for`, `switch`
- **[Error Handling](docs/language-reference/error-handling.md)** - `fail`, error binding, patterns
- **[Concurrency](docs/language-reference/concurrency.md)** - `async`, `par`, `task`, `fire`, `await`

#### Advanced Features
- **[String Templates](docs/language-reference/string-templates.md)** - Interpolation, formatting
- **[Collections](docs/language-reference/collections.md)** - Arrays, vectors, operations
- **[Syntax Overview](docs/language-reference/syntax-overview.md)** - Complete grammar reference

### 🚀 Getting Started

- **[Installation](docs/getting-started/installation.md)** - Install Liva compiler
- **[Quick Start](docs/getting-started/quick-start.md)** - Your first program in 5 minutes

### 🔧 For Compiler Developers

- **[Architecture](docs/compiler-internals/architecture.md)** - Compilation pipeline
- **[Lexer](docs/compiler-internals/lexer.md)** - Tokenization with Logos
- **[Parser](docs/compiler-internals/parser.md)** - Recursive descent parsing
- **[Semantic Analysis](docs/compiler-internals/semantic.md)** - Type checking, inference
- **[IR](docs/compiler-internals/ir.md)** - Intermediate representation
- **[Desugaring](docs/compiler-internals/desugaring.md)** - AST transformations
- **[Code Generation](docs/compiler-internals/codegen.md)** - Rust code emission
- **[Grammar](docs/compiler-internals/grammar.md)** - Complete EBNF grammar

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
├── src/                       # Compiler source code
│   ├── main.rs                # CLI entry point
│   ├── lib.rs                 # Library interface
│   ├── lexer.rs               # Tokenization (~300 lines)
│   ├── parser.rs              # AST construction (~1,750 lines)
│   ├── semantic.rs            # Type checking (~600 lines)
│   ├── ir.rs                  # IR definitions (~400 lines)
│   ├── lowering.rs            # AST → IR (~800 lines)
│   ├── codegen.rs             # IR → Rust (~4,700 lines)
│   ├── desugaring.rs          # AST transformations (~200 lines)
│   ├── error.rs               # Error reporting (~400 lines)
│   └── span.rs                # Source locations (~100 lines)
│
├── docs/                      # Complete documentation
│   ├── README.md              # Documentation index
│   ├── getting-started/       # Installation, tutorials
│   ├── language-reference/    # Complete language spec (14 files)
│   └── compiler-internals/    # Architecture, design (8 files)
│
├── tests/                     # Comprehensive test suite
│   ├── lexer_tests.rs         # Tokenization tests
│   ├── parser_tests.rs        # Parser tests with snapshots
│   ├── semantics_tests.rs     # Type checking tests
│   ├── codegen_tests.rs       # Code generation tests
│   ├── integration_tests.rs   # End-to-end tests
│   ├── snapshots/             # Insta snapshot files
│   └── [codegen|parser|...]/  # Test input files
│
├── examples/                  # Example Liva programs
│   ├── main.liva              # Basic example
│   └── manual-tests/          # Manual test files
│
├── scripts/                   # Build and utility scripts
│   ├── run_tests.sh           # Test runner
│   └── setup_and_commit.sh    # Dev workflow
│
├── Cargo.toml                 # Rust package manifest
├── Makefile                   # Build shortcuts
└── README.md                  # This file
```

## 🎯 Current Status

**Version:** 0.6.0  
**Status:** Alpha - Feature-complete for core language

### ✅ Fully Implemented

**Core Language:**
- ✅ Variables (`let`, `const`) with type inference
- ✅ Functions (one-liner, block, typed parameters/returns)
- ✅ Classes (constructors, fields, methods)
- ✅ Interfaces (method signatures, multiple implementation)
- ✅ Control flow (`if`, `while`, `for`, `switch`, ternary)
- ✅ Operators (arithmetic, logical, comparison, bitwise)
- ✅ String templates with interpolation
- ✅ Visibility modifiers (public, protected, private)

**Concurrency:**
- ✅ Async/await for I/O-bound operations
- ✅ Parallel execution for CPU-bound operations
- ✅ Task handles (`task`, `fire`, `await`)
- ✅ Hybrid concurrency (mix async + parallel)

**Error Handling:**
- ✅ Explicit `fail` statements
- ✅ Error binding (`let value, err = ...`)
- ✅ Fallibility inference (automatic detection)
- ✅ Comprehensive error messages with suggestions

**Compiler:**
- ✅ Complete lexer with 50+ tokens
- ✅ Recursive descent parser
- ✅ Type inference and checking
- ✅ Async/fallibility inference
- ✅ IR-based compilation pipeline
- ✅ Full Rust code generation
- ✅ Error reporting with JSON output

**Tooling:**
- ✅ VS Code extension with IntelliSense
- ✅ Real-time interface validation
- ✅ Syntax highlighting and snippets
- ✅ Comprehensive test suite (600+ tests)

### 🚧 In Development

- 🔄 Strict type checking (currently permissive)
- 🔄 Generic types and functions
- 🔄 Module system with imports
- 🔄 Pattern matching
- 🔄 Trait system refinements

### 📋 Roadmap

**v0.7 - Type System Enhancement** (Q1 2026)
- Strict type checking with inference
- Generic functions and classes
- Type aliases and unions
- Better error messages for type mismatches

**v0.8 - Module System** (Q2 2026)
- Import/export statements
- Module resolution
- Package manager integration
- Standard library foundation

**v0.9 - Advanced Features** (Q3 2026)
- Pattern matching (`match` expressions)
- Trait refinements
- Macro system (hygenic)
- Compile-time evaluation

**v1.0 - Production Release** (Q4 2026)
- Language Server Protocol (LSP)
- Debugger support
- Performance optimizations
- Stability guarantees
- Production-ready documentation

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
