# Self-Hosting: Compilador de Liva escrito en Liva

## Objetivo

Reimplementar el compilador `livac` (actualmente en Rust, en `src/`) usando Liva. El compilador self-hosted debe producir el mismo output que el compilador Rust para los mismos inputs.

## Referencia

El compilador Rust actual está en `src/` y sigue este pipeline:

```
lexer.rs → parser.rs → ast.rs → codegen.rs → Rust source
```

## Decisiones de diseño

**No es una copia 1:1 del compilador Rust.** El compilador Rust tiene deuda técnica — `codegen.rs` tiene ~19.000 líneas con ~20 HashSets (`option_value_vars`, `string_vars`, `float_vars`, `json_value_vars`, `map_vars`...) para rastrear tipos en tiempo de generación. Eso es un sustituto artesanal de un sistema de tipos real.

**Qué se replica tal cual:**
- El pipeline: lexer → parser → AST → codegen
- La estructura del AST (Expr, Stmt, TypeRef) — está bien diseñada
- El lexer y parser — están limpios, se traducen casi directamente

**Qué se rediseña:**
- **Análisis semántico real** — Una fase nueva que anote el AST con tipos resueltos ANTES de codegen. En vez de que codegen adivine "¿esta variable es string? ¿es Option? ¿es Map?", el AST llega ya con esa info.
- **Codegen limpio** — Si cada nodo ya sabe su tipo, generar Rust es mecánico. Se eliminan los 50+ casos especiales para decidir `.to_string()`, `.clone()`, `.unwrap_or()`, etc.
- **Mejor modularidad** — Separar en módulos claros en vez de un archivo gigante.

## Validación

Los **513+ tests existentes** (`cargo test --tests`) definen el comportamiento correcto del compilador. El compilador self-hosted en Liva debe pasar esos mismos tests — es decir, para cada input de test, debe generar el mismo Rust output que el compilador Rust actual.

## Tracking de issues

Todos los bugs, errores o carencias del lenguaje encontrados durante el desarrollo se documentan en **`ISSUES.md`** para corregirlos después en el compilador Rust.

