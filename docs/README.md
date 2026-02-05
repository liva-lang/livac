# 📚 Liva Language Documentation

> **The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.**

**Version:** 1.0.0 (Stable Release)

Welcome to the Liva programming language documentation. Liva compiles to Rust, giving you memory safety, zero-cost abstractions, and native performance with a clean, intuitive syntax.

---

## 🎯 Why Liva?

| Feature | Liva | TypeScript | Python | Rust |
|---------|------|------------|--------|------|
| Clean syntax | ✅ | ✅ | ✅ | ⚠️ |
| Type safety | ✅ | ✅ | ⚠️ | ✅ |
| Native performance | ✅ | ❌ | ❌ | ✅ |
| Memory safety | ✅ | N/A | N/A | ✅ |
| No garbage collector | ✅ | ❌ | ❌ | ✅ |
| Learning curve | Easy | Easy | Easy | Steep |

---

## 📖 Documentation Structure

### 🚀 [Getting Started](getting-started/)

Start here if you're new to Liva.

- **[Installation](getting-started/installation.md)** - Install the Liva compiler
- **[Quick Start](getting-started/quick-start.md)** - Your first Liva program in 5 minutes

### 📘 [Language Reference](language-reference/)

Complete language specification and syntax reference.

| Topic | Description |
|-------|-------------|
| [Syntax Overview](language-reference/syntax-overview.md) | Grammar and basic syntax |
| [Types](language-reference/types.md) | Type system, primitives, and inference |
| [Variables](language-reference/variables.md) | Variable declarations and mutability |
| [Functions](language-reference/functions.md) | Function syntax, parameters, and return types |
| [Classes](language-reference/classes.md) | Object-oriented programming |
| [Generics](language-reference/generics.md) | Generic types and constraints |
| [Control Flow](language-reference/control-flow.md) | if, for, while, switch statements |
| [Pattern Matching](language-reference/pattern-matching.md) | match expressions with exhaustiveness |
| [Error Handling](language-reference/error-handling.md) | Fallibility system with fail/error |
| [Concurrency](language-reference/concurrency.md) | async, par, task, fire keywords |
| [Modules](language-reference/modules.md) | Multi-file projects and imports |
| [Collections](language-reference/collections.md) | Arrays, vectors, and data structures |
| [String Templates](language-reference/string-templates.md) | String interpolation |
| [Union Types](language-reference/union-types.md) | Type unions and matching |
| [Type Aliases](language-reference/type-aliases.md) | Custom type definitions |

### 📦 [Standard Library](language-reference/stdlib/)

Built-in modules and APIs.

| Module | Description |
|--------|-------------|
| [Console](language-reference/console-api.md) | `print()`, `println()`, `debug()` |
| [JSON](language-reference/json.md) | JSON parsing and serialization |
| [HTTP](language-reference/http.md) | HTTP client for API calls |
| [File I/O](language-reference/file-io.md) | Reading and writing files |

### 📝 [Guides](guides/)

Practical tutorials and best practices.

- **[Generics Quick Start](guides/generics-quick-start.md)** - Using generics effectively
- **[JSON Typed Parsing](guides/json-typed-parsing.md)** - Type-safe JSON handling
- **[Module Best Practices](guides/module-best-practices.md)** - Organizing large projects
- **[Tuples](guides/tuples.md)** - Working with tuple types
- **[Trait Aliases](guides/trait-aliases-guide.md)** - Creating reusable constraints

### 🔧 [Compiler Internals](compiler-internals/)

Deep dive into how the Liva compiler works.

- **[Architecture](compiler-internals/architecture.md)** - Compiler pipeline overview
- **[Lexer](compiler-internals/lexer.md)** - Tokenization and lexical analysis
- **[Parser](compiler-internals/parser.md)** - AST construction
- **[Code Generation](compiler-internals/codegen.md)** - Rust code generation

### 🛠️ [LSP Integration](lsp/)

IDE support and language server protocol.

---

## ⚡ Quick Example

```liva
// A simple HTTP client example
import { HttpClient } from "http"
import { Json } from "json"

interface User {
  name: string
  email: string
}

main() {
  let client = HttpClient.new()
  let response = client.get("https://api.example.com/user/1").send()
  
  if response.ok {
    let user: User = Json.parse_as(response.body)
    print($"Welcome, {user.name}!")
  } else {
    print($"Error: {response.status}")
  }
}
```

---

## 📚 Additional Resources

- **[Quick Reference](QUICK_REFERENCE.md)** - Cheat sheet for all syntax
- **[Error Codes](ERROR_CODES.md)** - Complete error code reference
- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions
- **[Changelog](../CHANGELOG.md)** - Version history

---

## 🤝 Contributing

Liva is open source! Contributions are welcome.

- Report bugs in [BUGS.md](../BUGS.md)
- Check the [ROADMAP.md](../ROADMAP.md) for planned features
- See [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) for codebase overview

---

**Ready to start?** → [Installation Guide](getting-started/installation.md)
