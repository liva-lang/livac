# 🚀 Liva Quickstart Guide

Get up and running with Liva in 5 minutes!

## Prerequisites

- **Rust** 1.70+ ([Install from rustup.rs](https://rustup.rs/))
- **Cargo** (comes with Rust)
- A code editor (VSCode recommended)

## Installation

### Option 1: Quick Install (Linux/macOS)

```bash
curl -sSf https://raw.githubusercontent.com/liva-lang/livac/main/install.sh | sh
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/liva-lang/livac.git
cd livac

# Build and install
make install

# Or manually
cargo build --release
cp target/release/livac ~/.local/bin/
```

### Verify Installation

```bash
livac --help
```

You should see the help message for the Liva compiler.

## Your First Liva Program

### 1. Create a file `hello.liva`

```liva
main() {
  print("Hello, Liva! 🧩")
}
```

### 2. Compile and run

```bash
livac hello.liva --run
```

**Output:**
```
🧩 Liva Compiler v0.6
→ Compiling hello.liva
  → Lexical analysis...
  → Parsing...
  → Semantic analysis...
  → Desugaring to Rust...
  → Lowering to IR...
  → Generating Rust code...
  → Running cargo build...
✓ Compilation successful!

Running program:
============================================================
Hello, Liva! 🧩
```

### What just happened?

- The compiler analysed your program, lowered it to the intermediate representation, and generated Rust directly from it.
- Because the snippet does not use async/parallel helpers, no `liva_rt` runtime module was emitted. Try the concurrency samples below to see it appear in the generated Rust source.
- The full Rust project lives in `./target/liva_build` so you can inspect, rebuild, or run it manually with Cargo.

## 5-Minute Tutorial

### Variables and Types

```liva
main() {
  // Variables (mutable by default)
  let name = "Fran"
  let age: number = 41
  
  // Constants
  const PI = 3.1416
  
  // Type inference works
  let x = 10        // number (i32)
  let y = 3.14      // float (f64)
  let ok = true     // bool
  
  print($"Name: {name}, Age: {age}")
}
```

**Run:** `livac variables.liva --run`

### Functions

```liva
// One-liner function
square(x: number): number = x * x

// Block function
greet(name: string) {
  print($"Hello, {name}!")
}

// With inference
add(a, b) => a + b

main() {
  print(square(5))      // 25
  greet("World")        // Hello, World!
  print(add(10, 20))    // 30
}
```

### Classes and OOP

```liva
Person {
  name: string
  _age: number       // protected
  __id: string       // private
  
  greet() {
    print($"Hi, I'm {this.name}")
  }
  
  // One-liner method
  isAdult(): bool = this._age >= 18
}

main() {
  let p = Person("Ana", 25, "12345")
  p.greet()
  print(p.isAdult())
}
```

### Async/Await

```liva
use rust "reqwest" as http

// Auto-async: contains async call
fetchUser(id: number) {
  let url = $"https://api.example.com/users/{id}"
  let response = async http.get(url)
  return response.json()
}

main() {
  // Async call - spawns and awaits on use
  let user = async fetchUser(1)
  print($"User: {user.name}")
}
```

### Parallel Computing

```liva
// CPU-intensive task
fibonacci(n: number): number {
  if n <= 1 return n
  return fibonacci(n - 1) + fibonacci(n - 2)
}

main() {
  // Run on separate threads
  let a = parallel fibonacci(35)
  let b = parallel fibonacci(36)
  
  // Results computed in parallel
  print($"Fib(35) = {a}")
  print($"Fib(36) = {b}")
}
```

### Control Flow

```liva
main() {
  // If/else with natural operators
  let age = 25
  if age >= 18 and age < 65 {
    print("Working age")
  }
  
  // For loops
  for i in 0..5 {
    print($"Count: {i}")
  }
  
  // Switch
  let color = "red"
  switch color {
    case "red": print("Rojo")
    case "blue": print("Azul")
    default: print("Otro")
  }
}
```

## Common Commands

```bash
# Compile only
livac program.liva

# Compile and run
livac program.liva --run

# Check syntax without compiling
livac program.liva --check

# Show generated Rust code
livac program.liva --verbose

# Specify output directory
livac program.liva -o ./my_build

# Get help
livac --help
```

## IDE Setup (VSCode)

1. Install **Rust Analyzer** extension
2. Associate `.liva` files with JavaScript syntax (for now):

```json
// In settings.json
{
  "files.associations": {
    "*.liva": "javascript"
  }
}
```

## Project Structure

A typical Liva project:

```
my-project/
├── main.liva           # Entry point
├── utils.liva          # Utility functions
├── models.liva         # Data models
└── README.md
```

Compile with:
```bash
livac main.liva --run
```

## What's Next?

### Learn More

- 📖 Read the [Language Specification](docs/Liva_v0.6_spec.md)
- 🎓 Check [Example Programs](examples/)
- 🔧 See [Advanced Features](docs/advanced.md)

### Try These Examples

```bash
# Download examples
git clone https://github.com/liva-lang/livac.git
cd livac/examples

# Try them out
livac functions.liva --run
livac classes.liva --run
livac async.liva --run
livac parallel.liva --run
```

### Join the Community

- 💬 [Discord Server](https://discord.gg/liva)
- 🐦 [Twitter](https://twitter.com/livalang)
- 📝 [Blog](https://liva-lang.org/blog)

## Tips & Tricks

### Debugging

Use `--verbose` to see generated Rust code:

```bash
livac program.liva --verbose
```

### Performance

Liva compiles to optimized Rust. For maximum performance:

```bash
livac program.liva
cd target/liva_build
cargo build --release
./target/release/liva_project
```

### Using Rust Crates

```liva
use rust "serde_json"
use rust "tokio"
use rust "regex"

main() {
  // Use Rust libraries directly!
}
```

### Mixing Sync and Async

```liva
main() {
  let sync_result = calculate()        // Regular call
  let async_result = async fetchData() // Async call
  let parallel_result = parallel heavy()  // Parallel call
  
  // Mix them freely!
}
```

## ✅ Verify Your Toolchain

Run the complete compiler test suite (unit, integration, property, and snapshot tests):

```bash
cargo test
```

To focus on the new IR → Rust pipeline (including the injected runtime helpers), run:

```bash
cargo test --test codegen_ir_tests -- --nocapture
```

The corresponding snapshots live under `tests/snapshots/codegen_ir_tests__*.snap`.

## Common Issues

**Error: `livac: command not found`**
- Add `~/.local/bin` to your PATH
- Or install to `/usr/local/bin` instead

**Error: Rust not found**
- Install Rust from [rustup.rs](https://rustup.rs/)

**Compilation slow?**
- First compilation downloads dependencies
- Subsequent builds are much faster

**Need help?**
- Check [Troubleshooting Guide](docs/troubleshooting.md)
- Ask in [GitHub Discussions](https://github.com/liva-lang/livac/discussions)

## Cheat Sheet

| Feature | Liva Syntax |
|---------|-------------|
| Variable | `let x = 10` |
| Constant | `const PI = 3.14` |
| Function | `add(a, b) => a + b` |
| Class | `Person { name: string }` |
| Protected | `_field: type` |
| Private | `__field: type` |
| Async | `async call()` |
| Parallel | `parallel compute()` |
| String template | `$"Hello {name}"` |
| If/else | `if cond { } else { }` |
| For loop | `for i in 0..10 { }` |
| While | `while cond { }` |
| Switch | `switch x { case val: ... }` |

---

**Congratulations!** 🎉 You're now ready to write Liva code!

For more information, check the [full documentation](README.md).
