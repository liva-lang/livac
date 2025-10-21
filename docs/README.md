# 📚 Liva Language Documentation# 📚 Liva Language Documentation# 📚 Documentación de Liva



> **The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.**



Welcome to the Liva language documentation. This guide will help you learn and master Liva, from basic syntax to advanced compiler internals.> **The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.**## Estructura de Documentación



## 📖 Documentation Structure



### 🚀 [Getting Started](getting-started/)Welcome to the Liva language documentation. This guide will help you learn and master Liva, from basic syntax to advanced compiler internals.### 🔄 Concurrencia



Start here if you're new to Liva.Documentación completa del sistema de concurrencia de Liva.



- **[Installation](getting-started/installation.md)** - Install the Liva compiler## 📖 Documentation Structure

- **[Quick Start](getting-started/quick-start.md)** - Your first Liva program in 5 minutes

📂 **[docs/concurrency/](concurrency/)** - Sistema de concurrencia

### 📘 [Language Reference](language-reference/)

### 🚀 [Getting Started](getting-started/)- **[README.md](concurrency/README.md)** - Índice y guía de aprendizaje

Complete language specification and syntax reference.

Start here if you're new to Liva.- **[EXECUTION_MODES.md](concurrency/EXECUTION_MODES.md)** - Las 7 formas de ejecutar funciones ⭐ LEER PRIMERO

- **[Syntax Overview](language-reference/syntax-overview.md)** - Grammar and basic syntax

- **[Types](language-reference/types.md)** - Type system, primitives, and inference- **[ERROR_HANDLING.md](concurrency/ERROR_HANDLING.md)** - Manejo de errores

- **[Variables & Constants](language-reference/variables.md)** - Variable declarations and mutability

- **[Functions](language-reference/functions.md)** - Function syntax, parameters, and return types- **[Installation](getting-started/installation.md)** - Install the Liva compiler- **[CONCURRENCIA_SISTEMA.md](concurrency/CONCURRENCIA_SISTEMA.md)** - Especificación técnica completa

- **[Classes & Objects](language-reference/classes.md)** - Object-oriented programming

- **[Control Flow](language-reference/control-flow.md)** - if, for, while, switch statements- **[Quick Start](getting-started/quick-start.md)** - Your first Liva program in 5 minutes- **[PLAN_CONCURRENCIA.md](concurrency/PLAN_CONCURRENCIA.md)** - Roadmap de implementación

- **[Operators](language-reference/operators.md)** - Arithmetic, logical, comparison operators

- **[Concurrency](language-reference/concurrency.md)** - async, par, task, fire keywords- **[Basic Concepts](getting-started/basic-concepts.md)** - Core language concepts- **[PHASE1_PROGRESS.md](concurrency/PHASE1_PROGRESS.md)** - Progreso Phase 1 (✅ completada)

- **[Error Handling](language-reference/error-handling.md)** - Fallibility system with fail/error binding

- **[Visibility](language-reference/visibility.md)** - Public, protected, and private access- **[Examples](getting-started/examples.md)** - Common patterns and use cases- **[RESUMEN_DOCUMENTACION.md](concurrency/RESUMEN_DOCUMENTACION.md)** - Resumen ejecutivo

- **[String Templates](language-reference/string-templates.md)** - String interpolation

- **[Collections](language-reference/collections.md)** - Arrays, vectors, and data structures

- **[Modules](language-reference/modules.md)** - 🚧 Multi-file projects and imports (v0.8.0-dev)- **[INICIO_RAMA.md](concurrency/INICIO_RAMA.md)** - Quick start para desarrollo



### 🔧 [Compiler Internals](compiler-internals/)### 📘 [Language Reference](language-reference/)



Deep dive into how the Liva compiler works.Complete language specification and syntax reference.### 📝 Especificaciones del Lenguaje



- **[Architecture](compiler-internals/architecture.md)** - Compiler pipeline overview- **[Liva_v0.6_spec.md](Liva_v0.6_spec.md)** - Especificación completa v0.6

- **[Lexer](compiler-internals/lexer.md)** - Tokenization and lexical analysis

- **[Parser](compiler-internals/parser.md)** - AST construction- **[Syntax Overview](language-reference/syntax-overview.md)** - Grammar and basic syntax- **[Liva_v0.6_EBNF_AST.md](Liva_v0.6_EBNF_AST.md)** - Gramática EBNF y AST

- **[Semantic Analysis](compiler-internals/semantic.md)** - Type checking and validation

- **[IR (Intermediate Representation)](compiler-internals/ir.md)** - Internal representation- **[Types](language-reference/types.md)** - Type system, primitives, and inference- **[Liva_v0.6_Desugaring.md](Liva_v0.6_Desugaring.md)** - Reglas de desugaring

- **[Code Generation](compiler-internals/codegen.md)** - Rust code emission

- **[Desugaring](compiler-internals/desugaring.md)** - Metadata collection- **[Variables & Constants](language-reference/variables.md)** - Variable declarations and mutability

- **[Grammar](compiler-internals/grammar.md)** - EBNF grammar reference

- **[Functions](language-reference/functions.md)** - Function syntax, parameters, and return types### 🛡️ Sistema de Errores

---

- **[Classes & Objects](language-reference/classes.md)** - Object-oriented programming- **[ERROR_SYSTEM.md](ERROR_SYSTEM.md)** - Sistema de errores completo

## 🎓 Learning Paths

- **[Control Flow](language-reference/control-flow.md)** - if, for, while, switch statements- **[ERROR_CODES.md](ERROR_CODES.md)** - Códigos de error

### For New Users

- **[Operators](language-reference/operators.md)** - Arithmetic, logical, comparison operators- **[error_messages_improvements.md](error_messages_improvements.md)** - Mejoras de mensajes

1. Start with [Quick Start](getting-started/quick-start.md) - Learn basics in 5 minutes

2. Read [Syntax Overview](language-reference/syntax-overview.md) - Understand core syntax- **[Concurrency](language-reference/concurrency.md)** - async, par, task, fire keywords

3. Explore [Concurrency](language-reference/concurrency.md) - Master async/parallel programming

4. Learn [Error Handling](language-reference/error-handling.md) - Fallibility system- **[Error Handling](language-reference/error-handling.md)** - Fallibility system with fail/error binding### 🔍 Auditoría y Análisis



### For Compiler Developers- **[Visibility](language-reference/visibility.md)** - Public, protected, and private access- **[AUDITORIA_COMPLETA_LIVA.md](AUDITORIA_COMPLETA_LIVA.md)** - Auditoría completa del proyecto



1. Read [Architecture](compiler-internals/architecture.md) - Understand the pipeline- **[String Templates](language-reference/string-templates.md)** - String interpolation

2. Study [Lexer](compiler-internals/lexer.md) → [Parser](compiler-internals/parser.md) → [Semantic](compiler-internals/semantic.md) - Grasp each phase

3. Dive into [Code Generation](compiler-internals/codegen.md) - See how Rust code is emitted- **[Collections](language-reference/collections.md)** - Arrays, vectors, and data structures### 📋 Planificación

4. Review [Grammar](compiler-internals/grammar.md) - EBNF reference

- **[Rust Interop](language-reference/rust-interop.md)** - Using Rust crates and types- **[feature_plan_lambdas_concurrency.md](feature_plan_lambdas_concurrency.md)** - Plan de features

---

- **[refactor_plan.md](refactor_plan.md)** - Plan de refactoring

## 📦 Quick Links

### 🔧 [Compiler Internals](compiler-internals/)

- **[GitHub Repository](https://github.com/liva-lang/livac)** - Source code

- **[Installation Guide](getting-started/installation.md)** - Get startedDeep dive into how the Liva compiler works.## 🎓 Rutas de Aprendizaje

- **[Language Tour](language-reference/syntax-overview.md)** - Comprehensive overview

- **[Examples](../../main.liva)** - Real working code



---- **[Architecture](compiler-internals/architecture.md)** - Compiler pipeline overview### Para Nuevos Usuarios de Liva



## 🌟 Key Features- **[Lexer](compiler-internals/lexer.md)** - Tokenization and lexical analysis1. 📖 [Liva_v0.6_spec.md](Liva_v0.6_spec.md) - Aprende la sintaxis básica



### Hybrid Concurrency- **[Parser](compiler-internals/parser.md)** - AST construction2. 🎯 [concurrency/EXECUTION_MODES.md](concurrency/EXECUTION_MODES.md) - Concurrencia simplificada

- **`async`**: I/O-bound operations

- **`par`**: CPU-bound parallelism- **[Semantic Analysis](compiler-internals/semantic.md)** - Type checking and validation3. 🛡️ [concurrency/ERROR_HANDLING.md](concurrency/ERROR_HANDLING.md) - Manejo de errores

- **`task`**: Explicit task handles

- **`fire`**: Fire-and-forget execution- **[IR (Intermediate Representation)](compiler-internals/ir.md)** - Internal representation



### Fallibility System- **[Code Generation](compiler-internals/codegen.md)** - Rust code emission### Para Desarrolladores del Compilador

- **`fail`**: Explicit error raising

- **Error binding**: `let value, err = fallibleFn()`- **[Desugaring](compiler-internals/desugaring.md)** - AST transformations1. 🚀 [concurrency/INICIO_RAMA.md](concurrency/INICIO_RAMA.md) - Setup del proyecto

- **Result types**: Automatic Result<T, Error> wrapping

- **[Error System](compiler-internals/error-system.md)** - Error codes and reporting2. 📋 [concurrency/PLAN_CONCURRENCIA.md](concurrency/PLAN_CONCURRENCIA.md) - Roadmap

### Data-Parallel Loops

- **`for par`**: Parallel iteration (Rayon)- **[Runtime](compiler-internals/runtime.md)** - liva_rt module and concurrency support3. 🔄 [concurrency/CONCURRENCIA_SISTEMA.md](concurrency/CONCURRENCIA_SISTEMA.md) - Spec técnica

- **`for vec`**: SIMD vectorization

- **`for parvec`**: Combined parallel + SIMD4. 📝 [Liva_v0.6_EBNF_AST.md](Liva_v0.6_EBNF_AST.md) - Gramática y AST



### Modern Syntax### 📚 [Guides](guides/)

- **Arrow functions**: `add(a, b) => a + b`

- **String templates**: `$"Hello, {name}!"`Practical guides for common tasks and advanced topics.### Para Contribuidores

- **Type inference**: Optional type annotations

1. 🔍 [AUDITORIA_COMPLETA_LIVA.md](AUDITORIA_COMPLETA_LIVA.md) - Estado del proyecto

---

- **[Async Programming](guides/async-programming.md)** - Working with asynchronous code2. 📋 [feature_plan_lambdas_concurrency.md](feature_plan_lambdas_concurrency.md) - Features planificadas

## 📝 Documentation Status

- **[Parallel Computing](guides/parallel-computing.md)** - CPU-bound parallel tasks3. 🎯 [concurrency/README.md](concurrency/README.md) - Sistema de concurrencia

✅ **Complete**: All core language features documented  

✅ **Accurate**: Verified against source code (October 2025)  - **[Hybrid Concurrency](guides/hybrid-concurrency.md)** - Mixing async and parallel

✅ **Comprehensive**: 20+ documentation files, ~15,000 lines  

✅ **Up-to-date**: Reflects current implementation- **[Error Handling Patterns](guides/error-handling-patterns.md)** - Best practices for fallibility## 🔗 Enlaces Rápidos



---- **[Testing](guides/testing.md)** - Writing tests for Liva code



**Version**: Liva v0.6  - **[Performance](guides/performance.md)** - Optimization tips and benchmarking### Repositorio

**Last Updated**: October 18, 2025  

**Documentation Source**: Verified from `src/` code- **[Migration from TypeScript](guides/typescript-migration.md)** - Porting TypeScript code- 📂 [Código fuente](../src/)


- **[Migration from Python](guides/python-migration.md)** - Porting Python code- 🧪 [Tests](../tests/)

- 📦 [VS Code Extension](../vscode-extension/)

### 🔌 [API Reference](api/)

Standard library and built-in functions.### Ejemplos

- 🎨 [main.liva](../main.liva) - Demo completa de features

- **[Built-in Functions](api/builtins.md)** - Core functions (print, length, etc.)- 🧪 [Tests de concurrencia](../tests/concurrency/)

- **[Array Methods](api/arrays.md)** - push, pop, map, filter, reduce- 🔬 [Tests de integración](../tests/integration/)

- **[String Methods](api/strings.md)** - split, join, substring, trim

- **[Math Functions](api/math.md)** - abs, sqrt, pow, min, max---

- **[Type Conversions](api/conversions.md)** - parseInt, parseFloat, toString

- **[I/O Operations](api/io.md)** - File and console operations**Última actualización:** 18 de octubre de 2025


## 🎯 Quick Links

### By Topic
- 🔰 **New to Liva?** → [Quick Start](getting-started/quick-start.md)
- ⚡ **Concurrency** → [Concurrency Reference](language-reference/concurrency.md)
- 🛡️ **Error Handling** → [Error Handling Guide](language-reference/error-handling.md)
- 🔧 **Compiler Development** → [Architecture](compiler-internals/architecture.md)
- 📊 **Examples** → [Examples Gallery](getting-started/examples.md)

### By Experience Level
- **Beginner**: Getting Started → Basic Concepts → Examples
- **Intermediate**: Language Reference → Guides → Common Patterns
- **Advanced**: Compiler Internals → Performance → Custom Extensions

## 📦 Language Version

This documentation is for **Liva v0.6**.

### Version Features
- ✅ Core syntax and type system
- ✅ Functions and classes with visibility
- ✅ Hybrid concurrency (async + parallel)
- ✅ Fallibility system with error binding
- ✅ String templates and interpolation
- ✅ Full Rust interoperability
- ✅ Comprehensive error reporting

## 🤝 Contributing to Documentation

Found an error or want to improve the docs?

1. Fork the repository
2. Make your changes in the `docs/` directory
3. Submit a pull request

## 📞 Need Help?

- 💬 **GitHub Discussions** - Ask questions and share ideas
- 🐛 **Issues** - Report bugs or request features
- 📧 **Email** - Contact the maintainers

---

**Happy coding with Liva! 🧩**

*Last updated: October 2025*
