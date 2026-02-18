# 🚀 Liva Programming Language

[![CI](https://github.com/liva-lang/livac/actions/workflows/ci.yml/badge.svg)](https://github.com/liva-lang/livac/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/liva-lang/livac?include_prereleases&label=latest)](https://github.com/liva-lang/livac/releases/latest)
[![Tests](https://img.shields.io/badge/tests-278%20passing-brightgreen.svg)]()
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> **The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.**

**Liva** is a modern programming language that compiles to Rust, giving you native performance with clean, intuitive syntax. If you know any programming language, you'll feel at home with Liva.

---

## ✨ Why Liva?

| | Liva | TypeScript | Python | Rust |
|---|:---:|:---:|:---:|:---:|
| 🎯 Clean syntax | ✅ | ✅ | ✅ | ⚠️ |
| 🛡️ Type safety | ✅ | ✅ | ⚠️ | ✅ |
| ⚡ Native performance | ✅ | ❌ | ❌ | ✅ |
| 🔒 Memory safety | ✅ | N/A | N/A | ✅ |
| 📦 No garbage collector | ✅ | ❌ | ❌ | ✅ |
| 📚 Easy to learn | ✅ | ✅ | ✅ | ⚠️ |

---

## 🚀 Installation

### Pre-built Binaries (Recommended)

Download the latest release from [**GitHub Releases**](https://github.com/liva-lang/livac/releases/latest):

#### 🐧 Linux (Ubuntu/Debian)

```bash
curl -LO https://github.com/liva-lang/livac/releases/latest/download/livac_amd64.deb
sudo dpkg -i livac_amd64.deb
livac --version
```

#### 🎩 Linux (Fedora/RHEL)

```bash
curl -LO https://github.com/liva-lang/livac/releases/latest/download/livac.x86_64.rpm
sudo rpm -i livac.x86_64.rpm
livac --version
```

#### 🐧 Linux (Generic)

```bash
curl -L https://github.com/liva-lang/livac/releases/latest/download/livac-linux-x64.tar.gz | tar xz
sudo mv livac /usr/local/bin/
livac --version
```

#### 🍎 macOS

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/liva-lang/livac/releases/latest/download/livac-darwin-arm64.tar.gz | tar xz
sudo mv livac /usr/local/bin/

# Intel Mac
curl -L https://github.com/liva-lang/livac/releases/latest/download/livac-darwin-x64.tar.gz | tar xz
sudo mv livac /usr/local/bin/

livac --version
```

#### 🪟 Windows

1. Download [`livac-windows-x64.zip`](https://github.com/liva-lang/livac/releases/latest/download/livac-windows-x64.zip)
2. Extract `livac.exe`
3. Add the folder to your `PATH`
4. Open a terminal: `livac --version`

#### Uninstall

```bash
# Debian/Ubuntu
sudo apt remove livac

# Fedora/RHEL
sudo rpm -e livac

# macOS / Linux (tar.gz)
sudo rm /usr/local/bin/livac
```

### Build from Source

Requires [Rust](https://rustup.rs/) 1.70+:

```bash
git clone https://github.com/liva-lang/livac.git
cd livac
cargo build --release
cargo install --path .
livac --version
```

---

## 👋 Hello, Liva!

Create `hello.liva`:

```liva
main() => print("Hello, World!")
```

Run it:

```bash
livac hello.liva --run
# Hello, World!
```

---

## 📖 Language Tour

### Variables & Constants

```liva
let name = "Alice"          // Mutable, type inferred
let age: number = 25        // Explicit type
const MAX = 100             // Immutable constant
```

### Functions

```liva
// One-liner
greet(name) => print($"Hello, {name}!")

// With types
add(a: number, b: number): number => a + b

// Block function
calculate(a: number, b: number): number {
    let result = a + b * 2
    return result
}
```

### String Templates

```liva
let name = "Bob"
let age = 30
print($"Hello, {name}! Next year you'll be {age + 1}")
```

### Control Flow

```liva
// If/else
if score >= 90 {
    print("Grade: A")
} else if score >= 80 {
    print("Grade: B")
} else {
    print("Needs improvement")
}

// Ternary
let status = age >= 18 ? "adult" : "minor"

// For loops
for i in 0..10 { print(i) }           // 0 to 9
for item in items { print(item) }      // Iterate array

// While
while count < 5 { count = count + 1 }
```

### Pattern Matching

```liva
let result = switch value {
    0 => "zero",
    1 | 2 | 3 => "small",       // Or-pattern
    4..=10 => "medium",         // Range
    n if n > 100 => "huge",     // Guard
    _ => "other"                // Wildcard
}
```

### Arrays & Functional Operations

```liva
let nums = [1, 2, 3, 4, 5]

let doubled = nums.map(x => x * 2)              // [2, 4, 6, 8, 10]
let evens = nums.filter(x => x % 2 == 0)        // [2, 4]
let sum = nums.reduce((acc, x) => acc + x, 0)    // 15
let found = nums.find(x => x > 3)               // Some(4)
let hasEven = nums.some(x => x % 2 == 0)        // true

// Point-free: pass functions directly (v1.1.0)
nums.forEach(print)                              // no lambda needed
let strs = nums.map(toString)

// Method references with :: (v1.1.0)
let fmt = Formatter("Item")
let labels = nums.map(fmt::format)               // ["Item: 1", "Item: 2", ...]
```

### Classes & Interfaces

```liva
// Interface — only method signatures
Animal {
    makeSound(): string
    getName(): string
}

// Class implementing an interface
Dog : Animal {
    constructor(name: string) {
        this.name = name
    }
    name: string

    makeSound() => "Woof!"
    getName() => this.name
}

let dog = Dog("Rex")
print($"{dog.getName()} says {dog.makeSound()}")
```

### Error Handling

No exceptions — errors are explicit values:

```liva
divide(a: number, b: number) {
    if b == 0 { fail "Cannot divide by zero" }
    return a / b
}

let result, err = divide(10, 0)
if err {
    print($"Error: {err}")
} else {
    print($"Result: {result}")
}
```

### Concurrency

```liva
let data = async fetchFromAPI()       // I/O async (non-blocking)
let result = par heavyCalc()          // CPU parallel (multi-threaded)

fire async logEvent("started")       // Fire-and-forget
let t = task async operation()        // Task handle
let value = await t                   // Await when ready
```

### Modules

```liva
// math.liva — functions are public by default
add(a, b) => a + b
_helper(x) => x * 2     // Private (underscore prefix)

// main.liva
import { add } from "./math.liva"
import * as utils from "./utils.liva"
```

### Generics

```liva
Stack<T> {
    constructor() { this.items = [] }
    items: [T]

    push(item: T) { this.items.push(item) }
    pop(): T => this.items.pop()
    isEmpty(): bool => this.items.length == 0
}

let stack = Stack<number>()
stack.push(42)
```

---

## 📦 Standard Library

| Module | Functions | Description |
|--------|-----------|-------------|
| **Console** | `print()`, `console.log()`, `console.input()`, `console.error()` | I/O and colored output |
| **Math** | `Math.sqrt()`, `Math.pow()`, `Math.abs()`, `Math.random()`, ... | Mathematical operations |
| **Strings** | `.split()`, `.replace()`, `.trim()`, `.toUpperCase()`, ... | 11 string methods |
| **Arrays** | `.map()`, `.filter()`, `.reduce()`, `.find()`, `.some()`, ... | 9 array methods |
| **File I/O** | `File.read()`, `File.write()`, `File.exists()`, `File.delete()` | File system operations |
| **HTTP** | `HTTP.get()`, `HTTP.post()`, `HTTP.put()`, `HTTP.delete()` | HTTP client |
| **JSON** | `JSON.parse()`, `JSON.stringify()` | JSON serialization |
| **Conversions** | `parseInt()`, `parseFloat()`, `toString()` | Type conversions |

---

## 💡 Error Messages

Liva provides best-in-class error messages:

```
● E0701: Fallible function must be called with error binding [Semantic]
────────────────────────────────────────────────────────────
  → test.liva:7:16

     5 │ divide(a, b) => b == 0 ? fail "Division by zero" : a / b
     6 │
     7 │ let result = divide(10, 2)
       │              ^^^^^^

  ⓘ Function 'divide' can fail but is not being called with error binding.

  💡 Use error binding: let result, err = fallibleFunc(...)

  📝 Example:
     let result, err = divide(10, 2)
     if err == "" { print(result) }

  📚 Learn more: docs/ERROR_CODES.md#e0701
────────────────────────────────────────────────────────────
```

**Features:**
- 🎯 Precise source location with context lines
- 💡 Smart suggestions ("Did you mean `userName`?")
- 📝 Code examples showing the correct fix
- 📚 Links to documentation for each error code
- 🏷️ Error categories (Parser, Semantic, Modules, etc.)

---

## 🛠️ IDE Support

Full VS Code / Cursor support via the [Liva VS Code Extension](https://github.com/liva-lang/vscode-extension):

- ✅ Syntax highlighting
- ✅ Intelligent autocompletion (30+ items)
- ✅ Go to Definition (F12)
- ✅ Find All References (Shift+F12)
- ✅ Hover type information
- ✅ Real-time diagnostics
- ✅ Signature help
- ✅ 60+ code snippets

```bash
# Install the extension
cd vscode-extension
npm install && npm run compile
npx vsce package
code --install-extension liva-vscode-*.vsix
```

---

## 🏗️ How It Works

```
Liva Source (.liva)
       ↓
  [1] Lexer → Tokens (logos)
       ↓
  [2] Parser → AST (chumsky)
       ↓
  [3] Semantic Analysis
       ├─ Type inference
       ├─ Async/fallibility inference
       └─ Visibility validation
       ↓
  [4] IR Lowering → Typed IR
       ↓
  [5] Code Generation → Rust
       ├─ main.rs
       ├─ liva_rt.rs (runtime)
       └─ Cargo.toml
       ↓
  [6] Cargo Build → Native Binary
```

---

## 🧪 Battle-Tested

Liva was built through extensive **dogfooding** — 10+ real applications were built with Liva, uncovering and fixing **71 bugs** (100% resolved):

| App | What it tests |
|-----|--------------|
| 🔧 GitHub CLI | HTTP + JSON + Arrays |
| 🔧 Config Tool | File I/O + JSON + Dynamic keys |
| 🔧 Task Manager | CRUD + File persistence |
| 🔧 Notes App | Classes + Methods + File I/O |
| 🔧 Weather CLI | Real API + Nested JSON |
| 🔧 Crypto Tracker | CoinGecko API + null checking |
| 🔧 Todo API | HTTP POST/PUT/DELETE |
| 🔧 Log Analyzer | Pattern matching + File.exists |
| 🔧 Modular App | Multi-file imports |
| 🔧 Generics Tests | Box\<T\>, Stack\<T\>, Pair\<A,B\> |
| 🎓 Student Grade Tracker | Data classes + Interfaces + Switch ranges + Error handling |

---

## 📖 Commands

```bash
livac file.liva           # Compile to Rust project
livac file.liva --run     # Compile and execute
livac file.liva --check   # Syntax check only (fast!)
livac file.liva --verbose # Show generated Rust code
livac --lsp               # Start language server
livac --help              # Show all options
```

---

## 🧪 Testing

```bash
cargo test                 # Run all 278 tests
cargo test lexer           # Run lexer tests
cargo test codegen         # Run codegen tests
cargo test -- --nocapture  # With output
```

---

## 📚 Documentation

| Resource | Description |
|----------|-------------|
| **[Quick Reference](docs/QUICK_REFERENCE.md)** | Cheat sheet — all syntax on one page |
| **[Getting Started](docs/getting-started/)** | Installation & first program |
| **[Language Reference](docs/language-reference/)** | Complete language specification |
| **[Error Codes](docs/ERROR_CODES.md)** | All error codes explained |
| **[Compiler Internals](docs/compiler-internals/)** | How the compiler works |
| **[LSP Guide](docs/lsp/)** | IDE integration details |
| **[Troubleshooting](docs/TROUBLESHOOTING.md)** | Common issues & fixes |
| **[Changelog](CHANGELOG.md)** | Full version history |

---

## 🏗️ Project Structure

```
livac/
├── src/                    # Compiler source (~12,000 lines of Rust)
│   ├── main.rs             # CLI entry point
│   ├── lexer.rs            # Tokenization (logos)
│   ├── parser.rs           # Parser (chumsky) → AST
│   ├── ast.rs              # AST definitions
│   ├── semantic.rs         # Type checking & inference
│   ├── codegen.rs          # IR → Rust code generation
│   ├── module.rs           # Module system & imports
│   ├── error.rs            # Error reporting system
│   └── lsp/                # Language Server Protocol
├── docs/                   # Complete documentation (27+ files)
├── tests/                  # Test suite (110+ tests)
└── examples/               # Example Liva programs
```

---

## 🤝 Contributing

Contributions are welcome!

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes with tests
4. Submit a pull request

**Areas for contribution:** bug fixes, documentation, new examples, error messages, test cases.

---

## 📄 License

MIT License — See [LICENSE](LICENSE) for details.

## 👤 Author

**Fran Nadal** — Creator and maintainer of Liva

🐙 GitHub: [github.com/liva-lang](https://github.com/liva-lang)

---

<p align="center">
  <b>Made with ❤️ for developers who want Python's simplicity and Rust's performance.</b>
  <br><br>
  <code>livac hello.liva --run</code>
</p>
