# 🧩 Liva Compiler (livac) v0.6

> The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.

A compiler that transforms Liva code into safe, efficient Rust code with full async/parallel support.

## 🏗️ Architecture Snapshot (v0.6)

The current compiler pipeline runs in the following stages:

1. **Lexer** → produces `TokenWithSpan`s using `logos`.
2. **Parser** → builds the high-level AST.
3. **Semantic pass** → performs lightweight checks (async inference, visibility) while deeper type validation remains on the roadmap.
4. **IR lowering** → converts the analysed AST into a typed, compiler-internal IR.
5. **Code generation** → emits Rust directly from the IR when all items are supported; otherwise the legacy AST generator is used as a fallback for that module.
6. **Runtime helpers** → when async/parallel features are detected, the generator injects a `liva_rt` helper module wrapping `tokio::spawn` and `std::thread::spawn`.

> **Heads-up**: semantic validation is intentionally permissive today (unknown identifiers/types may slip through). Follow the roadmap in `docs/refactor_plan.md` for the hardening timeline.

## 🚀 Installation

```bash
git clone <repository>
cd livac
cargo build --release
```

The binary will be available at `target/release/livac`

## 📖 Usage

### Basic Compilation

```bash
livac input.liva
```

This will:
1. Tokenise, parse, and analyse your Liva code
2. Lower the program to the IR and attempt IR-driven code generation (with a legacy fallback when necessary)
3. Generate Rust code in `./target/liva_build/`
4. Compile it with Cargo (unless `LIVAC_SKIP_CARGO` is set)

### Options

```bash
livac input.liva [OPTIONS]

OPTIONS:
  -o, --output <DIR>    Output directory (default: ./target/liva_build)
  -r, --run             Run the program after compilation
  -v, --verbose         Show generated Rust code
  -c, --check           Only check syntax, don't compile
      --json            Output errors in JSON format (for IDE integration)
  -h, --help            Print help
```

### Examples

**Check syntax only:**
```bash
livac my_program.liva --check
```

**Compile and run:**
```bash
livac my_program.liva --run
```

**See generated Rust code:**
```bash
livac my_program.liva --verbose
```

**JSON output for IDE integration:**
```bash
livac my_program.liva --check --json
```

## 🔴 Error Reporting System

Liva v0.6 includes a comprehensive error reporting system with:

- ✅ **Unique error codes** (E1xxx for lexer, E2xxx for parser, E0xxx for semantic, E3xxx for codegen)
- ✅ **Precise location** (file, line, and column)
- ✅ **Code snippets** with visual indicators pointing to the error
- ✅ **Descriptive messages** with context and helpful suggestions
- ✅ **Colorized output** in terminal for better readability
- ✅ **JSON output** for IDE integration
- ✅ **VS Code integration** with real-time error highlighting

### Example Error Output

```
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → test.liva:6:7

     6 │
       │ let x = 20
       │     ^^^
       │

  ⓘ Variable 'x' already defined in this scope

  💡 Consider using a different name or removing the previous declaration of 'x'
────────────────────────────────────────────────────────────
```

For complete documentation on error codes and the error system, see:
- **[ERROR_SYSTEM.md](docs/ERROR_SYSTEM.md)** - Detailed error system documentation
- **[ERROR_CODES.md](docs/ERROR_CODES.md)** - Complete list of error codes

## 🧪 Testing

Run the full compiler and test suite with:

```bash
cargo test
```

IR-specific code generation scenarios (async/parallel helpers, runtime injection) are covered in `tests/codegen_ir_tests.rs`:

```bash
cargo test --test codegen_ir_tests -- --nocapture
```

Snapshots for these tests live under `tests/snapshots/codegen_ir_tests__*.snap` and capture the emitted `liva_rt` helper module when concurrency features are present.

For the current migration status and outstanding work (strict semantics, runtime crate extraction, additional IR coverage), see `docs/refactor_plan.md`.

## 📝 Example Liva Programs

### 1. Hello World

```liva
main() {
  print("Hello from Liva!")
}
```

### 2. Simple Function

```liva
sum(a: number, b: number): number => a + b

main() {
  let result = sum(5, 3)
  print($"Result: {result}")
}
```

### 3. Class with Visibility

```liva
Persona {
  nombre: string
  _edad: number        // protected
  __dni: string        // private

  saludar() {
    print($"Hola, soy {this.nombre}")
  }

  _getEdad(): number = this._edad
}

main() {
  let p = Persona("Fran", 41, "XYZ")
  p.saludar()
}
```

### 4. Async Concurrency

```liva
use rust "reqwest" as http

fetchUser() {
  let res = async http.get("https://api.example.com/user")
  return res.json()
}

main() {
  let user = async fetchUser()
  print($"User: {user.name}")
}
```

### 5. Parallel Computing

```liva
heavyCalc(n: number): number {
  // Simulate heavy computation
  return n * n
}

main() {
  let a = parallel heavyCalc(100)
  let b = parallel heavyCalc(200)
  
  print($"Results: {a}, {b}")
}
```

### 6. Mixed Concurrency

```liva
processData() {
  let data1 = async fetchFromAPI()
  let data2 = parallel computeIntensive()
  
  fire async logMetrics()  // Fire and forget
  
  return [data1, data2]
}
```

### 7. Control Flow

```liva
checkAge(age: number) {
  if age >= 18 and age < 65 {
    print("Working age")
  } else if age >= 65 {
    print("Retired")
  } else {
    print("Minor")
  }
}

main() {
  for i in 0..10 {
    checkAge(i * 10)
  }
}
```

### 8. Error Handling

```liva
divide(a: number, b: number): number {
  if b == 0 {
    throw "Division by zero"
  }
  return a / b
}

main() {
  try {
    let result = divide(10, 0)
    print(result)
  } catch (e) {
    print($"Error: {e}")
  }
}
```

## 🎯 Language Features

### ✅ Implemented

- ✅ **Variables & Constants**: `let`, `const`
- ✅ **Functions**: One-liners and blocks
- ✅ **Classes**: With inheritance support
- ✅ **Visibility**: Public, Protected (`_`), Private (`__`)
- ✅ **Types**: `number`, `float`, `string`, `bool`, all Rust primitives
- ✅ **Control Flow**: `if`, `while`, `for`, `switch`
- ✅ **Operators**: Arithmetic, logical (`and`/`or`/`not`, `&&`/`||`/`!`)
- ✅ **String Templates**: `$"Hello {name}"`
- ✅ **Async/Await**: `async call()`
- ✅ **Parallelism**: `parallel call()`
- ✅ **Tasks**: `task async/parallel call()`
- ✅ **Fire & Forget**: `fire async/parallel call()`
- ✅ **Auto-async inference**: Functions with async calls become async
- ✅ **Rust Interop**: `use rust "crate"`
- ✅ **Error Handling**: `try`/`catch`/`throw`

## 🔧 Project Structure

```
livac/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── ast.rs           # Abstract Syntax Tree
│   ├── lexer.rs         # Tokenizer
│   ├── parser.rs        # Parser
│   ├── semantic.rs      # Semantic analysis
│   ├── desugaring.rs    # AST transformation
│   └── codegen.rs       # Rust code generation
├── docs/
│   ├── Liva_v0.6_spec.md
│   ├── Liva_v0.6_EBNF_AST.md
│   └── Liva_v0.6_Desugaring.md
├── Cargo.toml
└── README.md
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run specific module tests:

```bash
cargo test lexer
cargo test parser
cargo test codegen
```

## 🛠️ Development

### Building from Source

```bash
cargo build --release
```

### Running in Development Mode

```bash
cargo run -- examples/hello.liva --run
```

### Adding New Features

1. Update the AST in `src/ast.rs`
2. Add lexer tokens in `src/lexer.rs`
3. Implement parser rules in `src/parser.rs`
4. Add semantic checks in `src/semantic.rs`
5. Implement code generation in `src/codegen.rs`
6. Write tests for the new feature

## 📚 Documentation

Full language documentation is available in the `docs/` directory:

- **[Liva_v0.6_spec.md](docs/Liva_v0.6_spec.md)** - Complete language specification
- **[Liva_v0.6_EBNF_AST.md](docs/Liva_v0.6_EBNF_AST.md)** - Formal grammar and AST
- **[Liva_v0.6_Desugaring.md](docs/Liva_v0.6_Desugaring.md)** - Transformation rules
- **[ERROR_SYSTEM.md](docs/ERROR_SYSTEM.md)** - 🆕 Comprehensive error reporting system
- **[ERROR_CODES.md](docs/ERROR_CODES.md)** - 🆕 Complete error code reference

## 🎓 Examples Directory

Create an `examples/` directory with sample programs:

```
examples/
├── hello.liva          # Basic hello world
├── functions.liva      # Function examples
├── classes.liva        # Class and OOP
├── async.liva          # Async/await patterns
├── parallel.liva       # Parallel computing
├── mixed.liva          # Mixed concurrency
└── full_app.liva       # Complete application
```

## 🔍 How It Works

### Compilation Pipeline

```
Liva Source (.liva)
    ↓
[1] Lexer → Tokens
    ↓
[2] Parser → AST
    ↓
[3] Semantic Analysis
    ├─ Type checking
    ├─ Async inference
    └─ Visibility validation
    ↓
[4] Desugaring
    └─ Liva AST → Rust concepts
    ↓
[5] Code Generation
    ├─ main.rs
    └─ Cargo.toml
    ↓
[6] Cargo Build
    ↓
Rust Binary
```

### Key Transformations

#### Visibility
```liva
Persona {
  nombre: string      // public
  _edad: number       // protected → pub(super)
  __dni: string       // private → (no pub)
}
```

#### Auto-Async
```liva
fetchUser() {
  let res = async http.get("url")  // Contains async call
  return res.json()
}
// Becomes: async fn fetch_user()
```

#### Concurrency
```liva
let x = async call()      // tokio::spawn().await
let y = parallel calc()   // thread::spawn().join()
let z = task async fn()   // Returns handle, await explicit
fire async log()          // Spawn without handle
```

#### String Templates
```liva
$"Hello {name}, age {age}"
// Becomes: format!("Hello {}, age {}", name, age)
```

## 🐛 Troubleshooting

### Common Issues

**Error: "Expected identifier"**
- Check for typos in variable/function names
- Ensure proper syntax (e.g., `:` for types, `=` for assignment)

**Error: "Type not found"**
- Verify the type is defined or is a built-in type
- Check `use rust` declarations for external types

**Error: "Cargo build failed"**
- Check generated Rust code with `--verbose`
- Ensure all Rust crates are properly specified

**Async errors**
- Make sure Tokio runtime is available (auto-added when async is used)
- Check that async functions are properly awaited

## 🚦 Status

**Version:** 0.6.0  
**Status:** Alpha  
**Rust Version:** 1.70+

### What Works
✅ Basic syntax and compilation  
✅ Functions and classes  
✅ Async/parallel concurrency  
✅ Type inference  
✅ String templates  
✅ Rust interop  

### In Progress
🚧 Advanced type system  
🚧 Pattern matching  
🚧 Generics refinement  
🚧 Module system  
🚧 Standard library  

### Planned
📋 Package manager  
📋 Language server (LSP)  
📋 REPL  
📋 Debugger integration  

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Coding Standards

- Follow Rust conventions
- Add tests for new features
- Update documentation
- Run `cargo fmt` and `cargo clippy`

## 📄 License

This project is licensed under the MIT License.

## 👤 Author

**Fran Nadal**

## 🙏 Acknowledgments

- Rust community for excellent tooling
- Logos for lexer generation
- Chumsky for parser combinators
- Tokio for async runtime

## 📞 Support

- 📧 Issues: Use GitHub Issues
- 💬 Discussions: GitHub Discussions
- 📖 Docs: See `docs/` directory

---

**Happy coding with Liva! 🧩**

_"Write like Python, think like TypeScript, run like Rust."_
