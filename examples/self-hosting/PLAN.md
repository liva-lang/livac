# Self-Hosting: Compilador de Liva escrito en Liva

> **Objetivo:** Implementar el compilador de Liva usando el propio lenguaje Liva.  
> **Validación:** El self-hosting es la prueba definitiva de que un lenguaje es viable — si puede compilarse a sí mismo, puede compilar cualquier cosa.

---

## Qué es esto

Un compilador de Liva escrito en Liva (`livac` compila este código a Rust, que a su vez compila programas Liva). No tiene que ser feature-complete — es un **subset funcional** que demuestre que Liva puede expresar lógica de compilador real.

## Arquitectura

```
src/
├── main.liva          # Entry point — orquesta el pipeline
├── token.liva         # Enum Token con todos los tokens del lenguaje
├── lexer.liva         # Tokenizador: string → [Token]
├── ast.liva           # Enums recursivos: Expr, Stmt, TypeRef, etc.
├── parser.liva        # Parser recursivo descendente: [Token] → AST
└── codegen.liva       # Generador de código: AST → string (Rust source)

tests/
├── lexer.test.liva    # Tests del lexer
├── parser.test.liva   # Tests del parser
└── codegen.test.liva  # Tests del codegen (input Liva → output Rust esperado)
```

## Pipeline

```
Source (.liva) → Lexer → [Token] → Parser → AST → Codegen → Rust source
```

Cada fase es una función pura:
1. **Lexer:** `tokenize(source: string): [Token]`
2. **Parser:** `parse(tokens: [Token]): Program`
3. **Codegen:** `generate(program: Program): string`

## Subset del lenguaje a soportar

### Fase 1 — Mínimo viable
- [ ] Variables: `let`, `const`, tipos primitivos (`number`, `float`, `string`, `bool`)
- [ ] Funciones: declaración, arrow (`=>`), parámetros tipados, return
- [ ] Control flow: `if`/`else`, `while`, `for..in`, `switch`
- [ ] Expresiones: binarias, unarias, llamadas, member access, index
- [ ] Literales: int, float, string (con interpolación `$"..."`), bool, null, arrays
- [ ] `print()` y string interpolation
- [ ] `main()` auto-detection

### Fase 2 — Tipos de datos
- [ ] Data classes (positional constructor, Display, PartialEq)
- [ ] Enums simples y con payload
- [ ] Enums recursivos (auto-boxing)
- [ ] Optional types (`T?`, `null`, `or`, `!`, `?.`)
- [ ] Arrays y métodos básicos (`map`, `filter`, `forEach`, `length`, `push`)

### Fase 3 — Features avanzadas
- [ ] Módulos / imports (`use`)
- [ ] Error handling (`fail`, `or fail`, `let x, err = ...`)
- [ ] Pattern matching en switch (destructuring, guards, wildcards)
- [ ] String methods (`contains`, `split`, `replace`, etc.)
- [ ] Closures / lambdas

### Fase 4 — Self-compilation test
- [ ] El self-hosting compiler puede compilar un programa Liva de ejemplo
- [ ] Output Rust es compilable con `rustc`
- [ ] Comparar output con el compilador Rust oficial para verificar equivalencia

## Tests

Cada módulo debe tener tests exhaustivos usando el framework de test de Liva (`test "name" { ... }`):

- **Lexer tests:** tokenizar snippets y verificar secuencia de tokens
- **Parser tests:** parsear código y verificar estructura del AST
- **Codegen tests:** compilar Liva → comparar string Rust generado vs esperado
- **Integration tests:** pipeline completo source → Rust output

## Tracking de bugs

Todos los bugs, errores de compilación o carencias del lenguaje encontrados durante el desarrollo se documentan en:

→ **`ISSUES.md`** (en este mismo directorio)

Formato por issue:
```markdown
### ISSUE-NNN: Título descriptivo
- **Tipo:** BUG | LANGUAGE_GAP | FEATURE_REQUEST
- **Severidad:** BLOCKER | HIGH | LOW
- **Descripción:** Qué pasa
- **Código que falla:** (snippet)
- **Error:** (mensaje de error o comportamiento incorrecto)
- **Workaround:** (si existe)
- **Estado:** OPEN | FIXED | WONTFIX
```

## Referencia

- Sintaxis completa del lenguaje: `docs/QUICK_REFERENCE.md` + skill `SKILL.md`
- Arquitectura del compilador Rust: `docs/PROJECT_STRUCTURE.md`
- Source del compilador Rust: `src/` (lexer.rs, parser.rs, ast.rs, codegen.rs)
- Ejemplos existentes: `examples/` (todos los subdirectorios)

## Criterio de éxito

1. El código compila con `livac build` sin errores
2. Los tests pasan con `livac test`
3. El compilador self-hosted genera Rust válido para al menos un programa de ejemplo
4. Todos los issues encontrados están documentados en `ISSUES.md`
