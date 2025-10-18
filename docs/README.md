# 📚 Liva Language Documentation# 📚 Documentación de Liva



> **The simplicity of TypeScript, the expressiveness of Python, and the safety of Rust.**## Estructura de Documentación



Welcome to the Liva language documentation. This guide will help you learn and master Liva, from basic syntax to advanced compiler internals.### 🔄 Concurrencia

Documentación completa del sistema de concurrencia de Liva.

## 📖 Documentation Structure

📂 **[docs/concurrency/](concurrency/)** - Sistema de concurrencia

### 🚀 [Getting Started](getting-started/)- **[README.md](concurrency/README.md)** - Índice y guía de aprendizaje

Start here if you're new to Liva.- **[EXECUTION_MODES.md](concurrency/EXECUTION_MODES.md)** - Las 7 formas de ejecutar funciones ⭐ LEER PRIMERO

- **[ERROR_HANDLING.md](concurrency/ERROR_HANDLING.md)** - Manejo de errores

- **[Installation](getting-started/installation.md)** - Install the Liva compiler- **[CONCURRENCIA_SISTEMA.md](concurrency/CONCURRENCIA_SISTEMA.md)** - Especificación técnica completa

- **[Quick Start](getting-started/quick-start.md)** - Your first Liva program in 5 minutes- **[PLAN_CONCURRENCIA.md](concurrency/PLAN_CONCURRENCIA.md)** - Roadmap de implementación

- **[Basic Concepts](getting-started/basic-concepts.md)** - Core language concepts- **[PHASE1_PROGRESS.md](concurrency/PHASE1_PROGRESS.md)** - Progreso Phase 1 (✅ completada)

- **[Examples](getting-started/examples.md)** - Common patterns and use cases- **[RESUMEN_DOCUMENTACION.md](concurrency/RESUMEN_DOCUMENTACION.md)** - Resumen ejecutivo

- **[INICIO_RAMA.md](concurrency/INICIO_RAMA.md)** - Quick start para desarrollo

### 📘 [Language Reference](language-reference/)

Complete language specification and syntax reference.### 📝 Especificaciones del Lenguaje

- **[Liva_v0.6_spec.md](Liva_v0.6_spec.md)** - Especificación completa v0.6

- **[Syntax Overview](language-reference/syntax-overview.md)** - Grammar and basic syntax- **[Liva_v0.6_EBNF_AST.md](Liva_v0.6_EBNF_AST.md)** - Gramática EBNF y AST

- **[Types](language-reference/types.md)** - Type system, primitives, and inference- **[Liva_v0.6_Desugaring.md](Liva_v0.6_Desugaring.md)** - Reglas de desugaring

- **[Variables & Constants](language-reference/variables.md)** - Variable declarations and mutability

- **[Functions](language-reference/functions.md)** - Function syntax, parameters, and return types### 🛡️ Sistema de Errores

- **[Classes & Objects](language-reference/classes.md)** - Object-oriented programming- **[ERROR_SYSTEM.md](ERROR_SYSTEM.md)** - Sistema de errores completo

- **[Control Flow](language-reference/control-flow.md)** - if, for, while, switch statements- **[ERROR_CODES.md](ERROR_CODES.md)** - Códigos de error

- **[Operators](language-reference/operators.md)** - Arithmetic, logical, comparison operators- **[error_messages_improvements.md](error_messages_improvements.md)** - Mejoras de mensajes

- **[Concurrency](language-reference/concurrency.md)** - async, par, task, fire keywords

- **[Error Handling](language-reference/error-handling.md)** - Fallibility system with fail/error binding### 🔍 Auditoría y Análisis

- **[Visibility](language-reference/visibility.md)** - Public, protected, and private access- **[AUDITORIA_COMPLETA_LIVA.md](AUDITORIA_COMPLETA_LIVA.md)** - Auditoría completa del proyecto

- **[String Templates](language-reference/string-templates.md)** - String interpolation

- **[Collections](language-reference/collections.md)** - Arrays, vectors, and data structures### 📋 Planificación

- **[Rust Interop](language-reference/rust-interop.md)** - Using Rust crates and types- **[feature_plan_lambdas_concurrency.md](feature_plan_lambdas_concurrency.md)** - Plan de features

- **[refactor_plan.md](refactor_plan.md)** - Plan de refactoring

### 🔧 [Compiler Internals](compiler-internals/)

Deep dive into how the Liva compiler works.## 🎓 Rutas de Aprendizaje



- **[Architecture](compiler-internals/architecture.md)** - Compiler pipeline overview### Para Nuevos Usuarios de Liva

- **[Lexer](compiler-internals/lexer.md)** - Tokenization and lexical analysis1. 📖 [Liva_v0.6_spec.md](Liva_v0.6_spec.md) - Aprende la sintaxis básica

- **[Parser](compiler-internals/parser.md)** - AST construction2. 🎯 [concurrency/EXECUTION_MODES.md](concurrency/EXECUTION_MODES.md) - Concurrencia simplificada

- **[Semantic Analysis](compiler-internals/semantic.md)** - Type checking and validation3. 🛡️ [concurrency/ERROR_HANDLING.md](concurrency/ERROR_HANDLING.md) - Manejo de errores

- **[IR (Intermediate Representation)](compiler-internals/ir.md)** - Internal representation

- **[Code Generation](compiler-internals/codegen.md)** - Rust code emission### Para Desarrolladores del Compilador

- **[Desugaring](compiler-internals/desugaring.md)** - AST transformations1. 🚀 [concurrency/INICIO_RAMA.md](concurrency/INICIO_RAMA.md) - Setup del proyecto

- **[Error System](compiler-internals/error-system.md)** - Error codes and reporting2. 📋 [concurrency/PLAN_CONCURRENCIA.md](concurrency/PLAN_CONCURRENCIA.md) - Roadmap

- **[Runtime](compiler-internals/runtime.md)** - liva_rt module and concurrency support3. 🔄 [concurrency/CONCURRENCIA_SISTEMA.md](concurrency/CONCURRENCIA_SISTEMA.md) - Spec técnica

4. 📝 [Liva_v0.6_EBNF_AST.md](Liva_v0.6_EBNF_AST.md) - Gramática y AST

### 📚 [Guides](guides/)

Practical guides for common tasks and advanced topics.### Para Contribuidores

1. 🔍 [AUDITORIA_COMPLETA_LIVA.md](AUDITORIA_COMPLETA_LIVA.md) - Estado del proyecto

- **[Async Programming](guides/async-programming.md)** - Working with asynchronous code2. 📋 [feature_plan_lambdas_concurrency.md](feature_plan_lambdas_concurrency.md) - Features planificadas

- **[Parallel Computing](guides/parallel-computing.md)** - CPU-bound parallel tasks3. 🎯 [concurrency/README.md](concurrency/README.md) - Sistema de concurrencia

- **[Hybrid Concurrency](guides/hybrid-concurrency.md)** - Mixing async and parallel

- **[Error Handling Patterns](guides/error-handling-patterns.md)** - Best practices for fallibility## 🔗 Enlaces Rápidos

- **[Testing](guides/testing.md)** - Writing tests for Liva code

- **[Performance](guides/performance.md)** - Optimization tips and benchmarking### Repositorio

- **[Migration from TypeScript](guides/typescript-migration.md)** - Porting TypeScript code- 📂 [Código fuente](../src/)

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
