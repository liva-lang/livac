# 🚀 Liva Programming Language

> *A simple, modern programming language that compiles to Rust*

**Liva** is designed to be easy to learn while being powerful and safe. If you know any programming language, you'll feel at home with Liva.

## ✨ Why Liva?

- 🎯 **Simple syntax** - Easy to read and write
- ⚡ **Fast performance** - Compiles to Rust for native speed
- 🛡️ **Safe** - Catch errors at compile time
- 🔧 **Practical** - Built for real-world applications

## 🚀 Installation

### Prerequisites

Before installing Liva, make sure you have:
- **Rust** 1.70 or newer ([Install Rust](https://rustup.rs/))
- **Git** (to clone the repository)

### Install Liva

```bash
# 1. Clone the repository
git clone https://github.com/liva-lang/livac.git
cd livac

# 2. Build and install
cargo build --release
cargo install --path .

# 3. Verify installation
livac --version
```

You should see: `livac 0.7.0` (or `0.8.0-dev` if on feature branch)

## 👋 Your First Liva Program

Let's start with the simplest program possible:

**Create a file called `hello.liva`:**
```liva
main() => print("Hello, World!")
```

**Run it:**
```bash
livac hello.liva --run
```

**Output:**
```
Hello, World!
```

Congratulations! You just wrote and ran your first Liva program! 🎉

## 📖 Basic Commands

Before we dive into the language, let's learn the compiler commands:

```bash
# Compile a Liva program
livac program.liva

# Compile and run immediately
livac program.liva --run

# Check syntax without compiling (fast!)
livac program.liva --check

# See what Rust code is generated (advanced)
livac program.liva --verbose
```

**Pro tip:** Use `--check` while coding to quickly catch errors!

## 📚 Language Basics

### 1. Variables

Variables store values. Use `let` to create them:

```liva
main() {
  let name = "Alice"
  let age = 25
  let pi = 3.14159
  
  print(name)
  print(age)
}
```

**Key points:**
- `let` creates a variable
- Liva automatically figures out the type (string, number, etc.)
- Variables are **mutable** by default (you can change their value)

**Change a variable's value:**
```liva
main() {
  let count = 0
  count = count + 1
  print(count)  // Prints: 1
}
```

**Constants** (values that never change):
```liva
main() {
  const MAX_USERS = 100
  const APP_NAME = "MyApp"
  
  print(MAX_USERS)
  // MAX_USERS = 200  ← ERROR! Can't change constants
}
```

### 2. String Templates

Want to mix text with variables? Use string templates with `$`:

```liva
main() {
  let name = "Bob"
  let age = 30
  
  // Use {} inside $"..." to insert variables
  print($"Hello, my name is {name}")
  print($"I am {age} years old")
  print($"Next year I'll be {age + 1}")
}
```

**Output:**
```
Hello, my name is Bob
I am 30 years old
Next year I'll be 31
```

### 3. Basic Operations

You can do math and logic with Liva:

```liva
main() {
  // Math
  let sum = 10 + 5        // 15
  let diff = 10 - 5       // 5
  let product = 10 * 5    // 50
  let quotient = 10 / 5   // 2
  let remainder = 10 % 3  // 1 (modulo)
  
  // Comparison
  let isEqual = 5 == 5         // true
  let isNotEqual = 5 != 3      // true
  let isGreater = 10 > 5       // true
  let isLessOrEqual = 5 <= 10  // true
  
  // Logic (use and, or, not)
  let canVote = age >= 18 and age < 100
  let isWeekend = day == "Saturday" or day == "Sunday"
  let isNotReady = not isReady
}
```

### 4. Control Flow: if/else

Make decisions in your code:

```liva
main() {
  let age = 20
  
  if age >= 18 {
    print("You can vote!")
  } else {
    print("Too young to vote")
  }
}
```

**Multiple conditions:**
```liva
main() {
  let score = 85
  
  if score >= 90 {
    print("Grade: A")
  } else if score >= 80 {
    print("Grade: B")
  } else if score >= 70 {
    print("Grade: C")
  } else {
    print("Grade: F")
  }
}
```

**One-line if (ternary operator):**
```liva
main() {
  let age = 20
  let status = age >= 18 ? "adult" : "minor"
  print(status)  // Prints: adult
}
```

**Logical operators:**
```liva
main() {
  let age = 25
  let hasLicense = true
  
  if age >= 18 and hasLicense {
    print("Can drive!")
  }
  
  if age < 18 or not hasLicense {
    print("Cannot drive")
  }
}
```

### 5. Loops

**While loop** (repeat while condition is true):
```liva
main() {
  let count = 0
  
  while count < 5 {
    print(count)
    count = count + 1
  }
}
```

**Output:** `0 1 2 3 4`

**For loop** (iterate over a range):
```liva
main() {
  // From 0 to 4 (5 iterations)
  for i in 0..5 {
    print(i)
  }
  
  // From 1 to 10
  for i in 1..11 {
    print($"Number: {i}")
  }
}
```

**Break and continue:**
```liva
main() {
  let i = 0
  while i < 10 {
    i = i + 1
    
    if i == 5 {
      continue  // Skip 5
    }
    
    if i == 8 {
      break  // Stop at 8
    }
    
    print(i)
  }
}
```

**Output:** `1 2 3 4 6 7`

### 6. Functions

Functions let you reuse code:

**Simple function (one-liner):**
```liva
greet() => print("Hello!")

main() {
  greet()  // Call the function
}
```

**Function with parameters:**
```liva
greet(name) => print($"Hello, {name}!")

main() {
  greet("Alice")
  greet("Bob")
}
```

**Function that returns a value:**
```liva
add(a, b) => a + b

main() {
  let result = add(5, 3)
  print(result)  // 8
}
```

**Function with multiple lines:**
```liva
calculateDiscount(price, percent) {
  let discount = price * percent / 100
  let finalPrice = price - discount
  return finalPrice
}

main() {
  let price = calculateDiscount(100, 20)
  print(price)  // 80
}
```

**Type annotations** (optional but recommended):
```liva
multiply(a: number, b: number): number => a * b

greetPerson(name: string, age: number) {
  print($"{name} is {age} years old")
}

main() {
  let result = multiply(5, 10)
  greetPerson("Alice", 25)
}
```

### 7. Arrays and Switch

**Arrays:**
```liva
main() {
  let numbers = [1, 2, 3, 4, 5]
  let names = ["Alice", "Bob", "Charlie"]
  
  // Access by index (starts at 0)
  print(numbers[0])  // 1
  print(names[1])    // Bob
  
  // Iterate over array
  for num in numbers {
    print(num)
  }
}
```

**Switch statement:**
```liva
main() {
  let day = "Monday"
  
  switch day {
    "Monday" => print("Start of week")
    "Friday" => print("Almost weekend!")
    "Saturday" | "Sunday" => print("Weekend!")
    _ => print("Regular day")
  }
}
```

### 8. Classes and Objects

Create your own types with classes:

```liva
Person {
  // Constructor
  constructor(name: string, age: number) {
    this.name = name
    this.age = age
  }
  
  // Fields (properties)
  name: string
  age: number
  
  // Methods
  introduce() {
    print($"Hi, I'm {this.name} and I'm {this.age} years old")
  }
  
  canVote() => this.age >= 18
}

main() {
  // Create an instance
  let person = Person("Alice", 25)
  
  // Call methods
  person.introduce()
  
  // Check if can vote
  if person.canVote() {
    print($"{person.name} can vote")
  }
}
```

**Output:**
```
Hi, I'm Alice and I'm 25 years old
Alice can vote
```

### 9. Error Handling

Liva doesn't use exceptions. Instead, errors are explicit values:

```liva
divide(a: number, b: number) {
  if b == 0 {
    fail "Cannot divide by zero"
  }
  return a / b
}

main() {
  // Get both result and error
  let result, err = divide(10, 0)
  
  if err {
    print($"Error: {err}")  // "Cannot divide by zero"
  } else {
    print($"Result: {result}")
  }
}
```

**Output:**
```
Error: Cannot divide by zero
```

**Why this is good:**
- ✅ You **can't forget** to handle errors
- ✅ Errors are **visible** in the code
- ✅ No surprise exceptions crashing your program

## 🎯 Complete Example

Let's put it all together in a grade calculator:

```liva
// Calculate letter grade from score
getGrade(score: number): string {
  if score >= 90 {
    return "A"
  } else if score >= 80 {
    return "B"
  } else if score >= 70 {
    return "C"
  } else if score >= 60 {
    return "D"
  } else {
    return "F"
  }
}

// Student class
Student {
  constructor(name: string, score: number) {
    this.name = name
    this.score = score
  }
  
  name: string
  score: number
  
  getGrade() => getGrade(this.score)
  
  isPassing() => this.score >= 60
  
  report() {
    let grade = this.getGrade()
    let status = this.isPassing() ? "PASS" : "FAIL"
    
    print($"Student: {this.name}")
    print($"Score: {this.score}")
    print($"Grade: {grade}")
    print($"Status: {status}")
    print("---")
  }
}

main() {
  let students = [
    Student("Alice", 92),
    Student("Bob", 78),
    Student("Charlie", 55)
  ]
  
  print("=== Grade Report ===")
  
  for student in students {
    student.report()
  }
}
```

**Output:**
```
=== Grade Report ===
Student: Alice
Score: 92
Grade: A
Status: PASS
---
Student: Bob
Score: 78
Grade: C
Status: PASS
---
Student: Charlie
Score: 55
Grade: F
Status: FAIL
---
```

---

## 📦 Standard Library (v0.7.0)

Liva comes with a comprehensive standard library for common operations.

### 🖨️ Output & Input

**Two ways to output:**

```liva
// print() - Simple, clean output for users
print("Hello, World!")           // → Hello, World!
print($"Name: {name}")           // → Name: Alice

// console.log() - Debug output for developers
console.log("Hello, World!")     // → "Hello, World!" (with quotes)
console.log([1, 2, 3])           // → [1, 2, 3] (formatted)
console.log(data)                // → Shows full structure
```

**Interactive input:**

```liva
// Read user input
let name = console.prompt("Enter your name: ")
let age = console.prompt("Enter your age: ")

print($"Hello, {name}! You are {age} years old.")
```

**Error output:**

```liva
console.error("Something went wrong!")  // → stderr
console.warn("Be careful!")             // → stderr with Warning:
```

### 📊 Arrays

Transform and analyze collections:

```liva
let numbers = [1, 2, 3, 4, 5]

// Transform
let doubled = numbers.map(x => x * 2)       // [2, 4, 6, 8, 10]

// Filter
let evens = numbers.filter(x => x % 2 == 0) // [2, 4]

// Reduce
let sum = numbers.reduce((acc, x) => acc + x, 0)  // 15

// Find
let found = numbers.find(x => x > 3)        // Some(4)

// Check
let hasEven = numbers.some(x => x % 2 == 0) // true
let allPositive = numbers.every(x => x > 0) // true

// Search
let index = numbers.indexOf(3)              // 2
let exists = numbers.includes(5)            // true

// Chain operations
let result = numbers
    .filter(x => x > 2)
    .map(x => x * 2)
    .reduce((acc, x) => acc + x, 0)         // 24
```

### 🔤 Strings

Manipulate text easily:

```liva
let text = "Hello, World!"

// Split and join
let words = text.split(", ")                // ["Hello", "World!"]

// Case conversion
let upper = text.toUpperCase()              // "HELLO, WORLD!"
let lower = text.toLowerCase()              // "hello, world!"

// Trim whitespace
let clean = "  hello  ".trim()              // "hello"

// Replace
let greeting = text.replace("World", "Liva") // "Hello, Liva!"

// Query
let starts = text.startsWith("Hello")       // true
let ends = text.endsWith("!")               // true

// Extract
let sub = text.substring(0, 5)              // "Hello"
let char = text.charAt(0)                   // 'H'
let pos = text.indexOf("World")             // 7
```

### 🧮 Math

Mathematical operations:

```liva
// Basic math
let x = Math.sqrt(16.0)         // 4.0
let y = Math.pow(2.0, 3.0)      // 8.0
let z = Math.abs(-10.5)         // 10.5

// Rounding
let a = Math.floor(3.7)         // 3
let b = Math.ceil(3.2)          // 4
let c = Math.round(3.5)         // 4

// Comparison
let min = Math.min(10.5, 20.3)  // 10.5
let max = Math.max(10.5, 20.3)  // 20.3

// Random
let rand = Math.random()        // 0.0 to 1.0
```

### 🔄 Type Conversion

Convert between types with error handling:

```liva
// Parse integers
let num, err = parseInt("42")
if err == "" {
    print($"Number: {num}")     // Number: 42
} else {
    console.error($"Parse error: {err}")
}

// Parse floats
let value, err2 = parseFloat("3.14")
if err2 == "" {
    print($"Float: {value}")    // Float: 3.14
}

// To string
let str = toString(42)          // "42"
let strFloat = toString(3.14)   // "3.14"
let strBool = toString(true)    // "true"
```

**Full documentation:** [Standard Library Reference](docs/language-reference/stdlib/)

---

## 🎓 What's Next?

Now that you know the basics, you can:

1. **Practice** - Write your own programs
2. **Explore** - Try more complex examples
3. **Learn Advanced Features**:
   - Async/await for concurrent operations
   - Interfaces for clean abstractions
   - Parallel computation
   - Custom types and generics (coming soon)

```

## 📦 Module System (v0.8.0 - In Development)

Liva now supports multi-file projects with a simple import/export system:

### Basic Usage

**math.liva** - Public functions (exported by default):
```liva
// Public function - no prefix
add(a, b) {
    ret a + b
}

subtract(a, b) {
    ret a - b
}

// Private function - with _ prefix
_internal_helper(x) {
    ret x * 2
}
```

**main.liva** - Import and use:
```liva
import { add, subtract } from "./math.liva"

main() {
    let result = add(10, 20)
    print($"10 + 20 = {result}")
}
```

### Import Syntax

```liva
// Named imports
import { add, multiply } from "./math.liva"

// Wildcard imports (namespace)
import * as utils from "./utils.liva"
let x = utils.square(5)

// Multiple imports
import { add } from "./math.liva"
import { log } from "./logger.liva"
```

### Key Features

- ✅ **Public by default** - All functions without `_` prefix are exported
- ✅ **Private with `_`** - Consistent with Liva's naming convention
- ✅ **JavaScript-style syntax** - Familiar and intuitive
- ✅ **Cycle detection** - Prevents circular dependencies
- ✅ **Relative paths** - `./`, `../` for easy navigation
- ⏳ **Symbol validation** - Coming in v0.8.0 final release

### Current Status

**Phase 3.3 Complete:**
- ✅ Parser handles all import syntax
- ✅ Module resolution with caching
- ✅ Circular dependency detection
- ✅ Topological sort for compilation order

**Coming Soon (Phase 3.4-3.5):**
- ⏳ Import symbol validation
- ⏳ Multi-file Rust project generation
- ⏳ Comprehensive test suite

**Try it now:**
```bash
git checkout feature/modules-v0.8.0
cargo build --release
./target/release/livac examples/modules/test_import_syntax.liva
```

## 📖 Full Documentation

Want to learn more? Check out the complete documentation:

- **[Language Reference](docs/language-reference/)** - Complete guide to all features
- **[Getting Started](docs/getting-started/)** - Detailed tutorials
- **[Compiler Internals](docs/compiler-internals/)** - How Liva works under the hood

## ⚡ Quick Reference

### Compile & Run
```bash
livac file.liva --run     # Compile and run
livac file.liva --check   # Check syntax only
livac file.liva           # Just compile
```

### Variables
```liva
let x = 10           // Mutable variable
const PI = 3.14      // Constant
```

### Functions
```liva
greet() => print("Hi")                    // One-liner
add(a, b) => a + b                        // With params
calculate(x: number): number => x * 2     // With types
```

### Control Flow
```liva
if condition { }                          // If
if x > 0 { } else { }                    // If-else
let result = x > 0 ? "pos" : "neg"       // Ternary
while condition { }                       // While loop
for i in 0..10 { }                       // For loop
```

### Classes
```liva
Person {
  constructor(name: string) {
    this.name = name
  }
  name: string
  greet() => print($"Hi, I'm {this.name}")
}
```

### Error Handling
```liva
if error_condition {
  fail "Error message"
}

let result, err = someFunction()
if err {
  // Handle error
}
```

## 🏗️ Project Structure

```
livac/
├── src/           # Compiler source code
├── docs/          # Full documentation
├── examples/      # Example programs
├── tests/         # Test suite
└── README.md      # This file
```

## 🤝 Contributing

Want to help improve Liva? Contributions are welcome!

1. Fork the repository
2. Create your feature branch
3. Make your changes
4. Submit a pull request

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 👤 Author

**Fran Nadal** - Creator and maintainer of Liva

---

**Happy coding! 🚀**

```bash
# Start now!
livac --help
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
livac --version  # Should output: livac 0.7.0
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

### 3. Explicit Error Handling

No exceptions - errors are values with **error binding**:

```liva
divide(a: number, b: number) => b == 0 ? fail "Division by zero" : a / b

main() {
  let result, err = divide(10, 0)
  
  if err {
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
  name: string      // public
  _ssn: string      // private
  
  isAdult() => this.age >= 18
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
- **[Visibility](docs/language-reference/visibility.md)** - Public, private

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

**Version:** 0.7.0 (0.8.0-dev on feature branches)  
**Status:** Alpha - Core language complete, stdlib released, modules in development

### ✅ Fully Implemented

**Core Language:**
- ✅ Variables (`let`, `const`) with type inference
- ✅ Functions (one-liner, block, typed parameters/returns)
- ✅ Classes (constructors, fields, methods)
- ✅ Interfaces (method signatures, multiple implementation)
- ✅ Control flow (`if`, `while`, `for`, `switch`, ternary)
- ✅ Operators (arithmetic, logical, comparison, bitwise)
- ✅ String templates with interpolation
- ✅ Visibility modifiers (public, private)

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

- � **Module System (v0.8.0)** - Currently in development!
  - ✅ Import/export syntax (JavaScript-style)
  - ✅ Module resolution with cycle detection
  - ✅ Public by default, private with `_` prefix
  - ⏳ Semantic validation (in progress)
  - 📋 Multi-file code generation (planned)
- �🔄 Strict type checking (currently permissive)
- 🔄 Generic types and functions
- 🔄 Pattern matching
- 🔄 Trait system refinements

### 📋 Roadmap

**v0.7.0 - Standard Library** ✅ RELEASED (Oct 2025)
- ✅ String manipulation (37 functions)
- ✅ Math operations (sqrt, sin, cos, abs, etc.)
- ✅ Type conversions (parseInt, parseFloat, toString)
- ✅ Console I/O (console.log, console.readLine, etc.)
- ✅ Array/collection utilities

**v0.8.0 - Module System** 🚧 IN PROGRESS (Oct 2025)
- ✅ Import/export statements (Phase 3.2 complete)
- ✅ Module resolution with cycle detection (Phase 3.3 complete)
- ⏳ Import validation (Phase 3.4 in progress)
- 📋 Multi-file Rust project generation (Phase 3.5 planned)
- **ETA:** 2-3 weeks

**v0.9.0 - Type System Enhancement** (Q1 2026)
- Strict type checking with inference
- Generic functions and classes
- Type aliases and unions
- Better error messages for type mismatches

**v1.0.0 - Advanced Features** (Q2 2026)
- Pattern matching (`match` expressions)
- Trait refinements
- Package manager integration
- Standard library expansion

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
