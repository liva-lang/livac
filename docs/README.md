# 📚 Liva Language Documentation

> **The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.**

**Version:** 1.1.0-dev  
**Repository:** [github.com/liva-lang/livac](https://github.com/liva-lang/livac)

Liva compiles to Rust, giving you memory safety, zero-cost abstractions, and native performance with a clean, intuitive syntax.

---

## ⚡ Quick Example

```liva
// A weather CLI that fetches real API data
main() {
    let city = "London"
    let url = $"https://wttr.in/{city}?format=j1"
    
    let resp, err = async HTTP.get(url)
    if err {
        print($"Error: {err}")
        return
    }
    
    let data, jsonErr = resp.json()
    if jsonErr {
        print($"JSON Error: {jsonErr}")
        return
    }
    
    let temp = data["current_condition"][0]["temp_C"].asString()
    let desc = data["current_condition"][0]["weatherDesc"][0]["value"].asString()
    
    print($"Weather in {city}: {temp}°C, {desc}")
}
```

```bash
livac weather.liva --run
# Weather in London: 12°C, Partly cloudy
```

---

## 📖 Documentation Structure

### 🚀 Getting Started

Start here if you're new to Liva.

| Document | Description |
|----------|-------------|
| [Installation](getting-started/installation.md) | Install the Liva compiler |
| [Quick Start](getting-started/quick-start.md) | Your first Liva program in 5 minutes |
| [v1.0.0 Release Notes](getting-started/v1.0.0-release.md) | What's in the stable release |

### 📘 Language Reference

Complete language specification and syntax reference.

| Topic | Description |
|-------|-------------|
| [Syntax Overview](language-reference/syntax-overview.md) | Grammar and basic syntax |
| [Types](language-reference/types.md) | Type system, primitives, and inference |
| [Variables](language-reference/variables.md) | Variable declarations and mutability |
| [Functions](language-reference/functions.md) | Function syntax, parameters, and return types |
| [Operators](language-reference/operators.md) | Arithmetic, logical, comparison, bitwise |
| [Classes](language-reference/classes.md) | Classes, interfaces, OOP |
| [Generics](language-reference/generics.md) | Generic types and constraints |
| [Control Flow](language-reference/control-flow.md) | if, for, while, switch statements |
| [Pattern Matching](language-reference/pattern-matching.md) | Switch expressions with exhaustiveness checking |
| [Error Handling](language-reference/error-handling.md) | fail, error binding, fallibility |
| [Concurrency](language-reference/concurrency.md) | async, par, task, fire keywords |
| [Modules](language-reference/modules.md) | Multi-file projects and imports |
| [Collections](language-reference/collections.md) | Arrays, vectors, and data structures |
| [String Templates](language-reference/string-templates.md) | String interpolation with `$"..."` |
| [Union Types](language-reference/union-types.md) | Type unions and matching |
| [Type Aliases](language-reference/type-aliases.md) | Custom type definitions |
| [Visibility](language-reference/visibility.md) | Public and private access |

### 📦 Standard Library

Built-in modules and APIs.

| Module | Description |
|--------|-------------|
| [Console](language-reference/console-api.md) | `print()`, `console.log()`, `console.input()` |
| [JSON](language-reference/json.md) | JSON parsing and serialization |
| [HTTP](language-reference/http.md) | HTTP client for API calls |
| [File I/O](language-reference/file-io.md) | Reading and writing files |

<details>
<summary><b>📖 Detailed stdlib docs</b></summary>

| Module | Description |
|--------|-------------|
| [Arrays](language-reference/stdlib/arrays.md) | Array methods (map, filter, reduce, ...) |
| [Strings](language-reference/stdlib/strings.md) | String methods (split, replace, trim, ...) |
| [Math](language-reference/stdlib/math.md) | Math operations (sqrt, pow, random, ...) |
| [Conversions](language-reference/stdlib/conversions.md) | parseInt, parseFloat, toString |
| [I/O](language-reference/stdlib/io.md) | Console I/O and file operations |

</details>

### 📝 Guides

Practical tutorials and best practices.

| Guide | Description |
|-------|-------------|
| [Generics Quick Start](guides/generics-quick-start.md) | Using generics effectively |
| [JSON Typed Parsing](guides/json-typed-parsing.md) | Type-safe JSON handling |
| [Module Best Practices](guides/module-best-practices.md) | Organizing large projects |
| [Tuples](guides/tuples.md) | Working with tuple types |
| [Trait Aliases](guides/trait-aliases-guide.md) | Creating reusable constraints |
| [Destructuring Migration](guides/MIGRATION_DESTRUCTURING_v0.10.2.md) | Upgrading to v0.10.2 destructuring |

### 🔧 Compiler Internals

Deep dive into how the Liva compiler works.

| Document | Description |
|----------|-------------|
| [Architecture](compiler-internals/architecture.md) | Compiler pipeline overview |
| [Lexer](compiler-internals/lexer.md) | Tokenization and lexical analysis |
| [Parser](compiler-internals/parser.md) | AST construction |
| [AST](compiler-internals/ast.md) | Abstract syntax tree definitions |
| [Semantic Analysis](compiler-internals/semantic.md) | Type checking and inference |
| [Desugaring](compiler-internals/desugaring.md) | AST transformations |
| [IR](compiler-internals/ir.md) | Intermediate representation |
| [Code Generation](compiler-internals/codegen.md) | Rust code generation |
| [Grammar](compiler-internals/grammar.md) | Complete EBNF grammar |
| [Multi-file Codegen](compiler-internals/multifile-codegen.md) | How modules compile |
| [Enhanced Errors](compiler-internals/enhanced-error-context.md) | Error reporting system |
| [Import Validation](compiler-internals/import-validation.md) | Module resolution internals |

### 🛠️ LSP & IDE Support

Language Server Protocol for VS Code / Cursor integration.

| Document | Description |
|----------|-------------|
| [LSP User Guide](lsp/LSP_USER_GUIDE.md) | How to use LSP features |
| [LSP API](lsp/LSP_API.md) | LSP capabilities reference |
| [LSP Design](lsp/LSP_DESIGN.md) | Architecture decisions |
| [LSP v0.12.0](lsp/LSP_v0.12.0_COMPLETE.md) | Initial LSP release |
| [LSP Workspace v0.13.0](lsp/LSP_WORKSPACE_v0.13.0.md) | Multi-file workspace support |

### 📐 Design Documents

Internal design docs for language features (historical).

<details>
<summary><b>📂 View design documents</b></summary>

| Document | Feature |
|----------|---------|
| [Module System Spec](design/MODULE_SYSTEM_SPEC.md) | Module system design |
| [Module Syntax Comparison](design/MODULE_SYNTAX_COMPARISON.md) | Import syntax choices |
| [Module Proposal](design/MODULE_SYSTEM_PROPOSAL.md) | Original module proposal |
| [JSON API Design](design/PHASE_6.1_JSON_API_DESIGN.md) | JSON parsing design |
| [File I/O Design](design/PHASE_6.2_FILE_IO_API_DESIGN.md) | File system API design |
| [HTTP Client Design](design/PHASE_6.3_HTTP_CLIENT_DESIGN.md) | HTTP client design |
| [Pattern Matching Design](design/PHASE_6.4_PATTERN_MATCHING_DESIGN.md) | Pattern matching design |
| [Destructuring Design](design/PHASE_6.5_DESTRUCTURING_DESIGN.md) | Destructuring design |
| [Tuple Types Design](design/PHASE_7.1_TUPLE_TYPES_DESIGN.md) | Tuple types design |

</details>

---

## 📋 Quick Reference

| Resource | Description |
|----------|-------------|
| **[Quick Reference](QUICK_REFERENCE.md)** | Cheat sheet — all syntax on one page |
| **[Error Codes](ERROR_CODES.md)** | Complete error code reference |
| **[Error Handling Guide](ERROR_HANDLING_GUIDE.md)** | Understanding error messages |
| **[Troubleshooting](TROUBLESHOOTING.md)** | Common issues and solutions |
| **[Project Structure](PROJECT_STRUCTURE.md)** | Codebase overview |
| **[Changelog](../CHANGELOG.md)** | Full version history |

---

## 🤝 Contributing

Liva is open source! Contributions are welcome.

- Report bugs in [BUGS.md](../BUGS.md)
- Check the [ROADMAP.md](../ROADMAP.md) for planned features
- See [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) for codebase overview

---

**Ready to start?** → [Installation Guide](getting-started/installation.md)
