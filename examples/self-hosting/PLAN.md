# Self-Hosting: Compilador de Liva escrito en Liva

## Objetivo

Reimplementar el compilador `livac` (actualmente en Rust, en `src/`) usando Liva. El compilador self-hosted debe producir el mismo output que el compilador Rust para los mismos inputs.

## Referencia

El compilador Rust actual está en `src/` y sigue este pipeline:

```
lexer.rs → parser.rs → ast.rs → codegen.rs → Rust source
```

La implementación en Liva debe replicar este pipeline con la misma estructura.

## Validación

Los **513+ tests existentes** (`cargo test --tests`) definen el comportamiento correcto del compilador. El compilador self-hosted en Liva debe pasar esos mismos tests — es decir, para cada input de test, debe generar el mismo Rust output que el compilador Rust actual.

## Tracking de issues

Todos los bugs, errores o carencias del lenguaje encontrados durante el desarrollo se documentan en **`ISSUES.md`** para corregirlos después en el compilador Rust.

