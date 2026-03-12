# 🔧 Liva Compiler Context

> **Proyecto:** livac - El compilador de Liva  
> **Versión:** v1.5.0-dev (tag: v1.2.0)  
> **Lenguaje:** Rust  
> **Tests:** 374 passing  
> **Próximo objetivo:** v1.5 — Config + livac init (rust {} + Logging done)  
> **Última actualización:** 2026-03-13

---

## 📦 Qué es este proyecto

**livac** es el compilador del lenguaje Liva. Transforma código `.liva` en código Rust, que luego se compila a binario nativo.

```
Liva Source → Lexer → Parser → AST → Semantic → IR → Codegen → Rust → Binary
```

**Para la referencia completa del lenguaje**, consulta `skills/liva-lang/SKILL.md` (resumen compacto) o `docs/` (documentación detallada).

---

## 🏗️ Arquitectura

```
src/
├── main.rs           # CLI (clap) - punto de entrada
├── lib.rs            # API pública del compilador
├── lexer.rs          # Tokenización (logos)
├── parser.rs         # Parser (chumsky) → AST
├── ast.rs            # Definición del AST
├── semantic.rs       # Análisis semántico y tipos
├── desugaring.rs     # Transformaciones sintácticas
├── ir.rs             # Representación intermedia
├── lowering.rs       # AST → IR
├── codegen.rs        # IR → Código Rust (~490KB, ~14500 líneas)
├── formatter.rs      # Code formatter (--fmt)
├── module.rs         # Sistema de módulos e imports
├── traits.rs         # Sistema de traits/interfaces
├── error.rs          # Sistema de errores
├── error_codes.rs    # Códigos E0xxx
├── hints.rs          # Sugerencias de errores
├── suggestions.rs    # "Did you mean?" (Levenshtein)
├── span.rs           # Ubicaciones en código fuente
├── liva_rt.rs        # Runtime embebido
└── lsp/              # Language Server Protocol
    ├── server.rs     # Servidor LSP (tower-lsp)
    ├── document.rs   # Gestión de documentos
    ├── symbols.rs    # Tabla de símbolos
    ├── diagnostics.rs
    ├── imports.rs
    └── workspace.rs
```

---

## 🛠️ Comandos

```bash
# Build & test
cargo build --release
cargo test

# Compilar y ejecutar
livac run archivo.liva

# Verificar sintaxis / formatear / tests
livac check archivo.liva
livac fmt archivo.liva
livac test archivo.test.liva

# LSP
livac lsp

# Build skill (genera dist/skills/liva-lang/)
make build-skill
```

---

## 🤖 Agent Skills

El compilador incluye una **skill** siguiendo el estándar [Agent Skills](https://agentskills.io):

```
skills/liva-lang/
└── SKILL.md          # Referencia compacta (~400 líneas, <5000 tokens)
```

**Pipeline:**
1. `scripts/build-skill.sh` — Ensambla SKILL.md + copia docs/ como references/ → `dist/skills/liva-lang/`
2. `scripts/install-skills.sh` — Instala en `~/.agents/skills/liva-lang/` (estándar cross-client)
3. Compatible con: GitHub Copilot, Cursor, Claude Code, Windsurf, Goose, Amp, y 15+ agentes más

**Principio clave:** `docs/` es la única fuente de verdad. El build script copia de ahí para generar la skill, sin duplicar documentación.

---

## 📚 Documentación

| Recurso | Contenido |
|---------|-----------|
| `docs/QUICK_REFERENCE.md` | Cheat sheet completo de sintaxis |
| `docs/README.md` | Índice de toda la documentación |
| `docs/language-reference/` | Referencia detallada por tema (~30 archivos) |
| `docs/guides/` | Tutoriales y best practices |
| `ROADMAP.md` | Historial del proyecto (sesiones completadas) |
| `BACKLOG.md` | **Tareas pendientes** por versión (v1.4→v3.x) — checkboxes |
| `docs/plans/PLAN_PRODUCTION_READINESS.md` | **Diseño** de features futuras (sintaxis, decisiones, alternativas) |
| `CHANGELOG.md` | Historial de versiones |
| `BUGS.md` | Bugs encontrados en dogfooding |

---

## 🔄 Estado Actual (v1.5.0-dev)

### Features completados
- **CLI Subcomandos** — `build`, `run`, `check`, `fmt`, `test`, `lsp`, `update` (reemplaza flags planos)
- **`rust { }` Interop** — Inline Rust blocks + `use rust` with version/features + E9002 protection
- **Logging (`Log` module)** — info/warn/error/debug + variadic args + table rendering (Map/Array/JSON) + `setLevel`
- **Stdlib P0** — 38 nuevos métodos/funciones (15 String + 20 Array + 3 Math)
- **Enum Types** — Algebraic data types con pattern matching y destructuring
- **Error Trace Chaining** — Trazabilidad automática con función + línea
- **`or <value>`** — Default value para fallibles: `divide(10, 0) or 42`
- **Map<K,V> / Set<T>** — Colecciones completas con literales, métodos, iteración
- **Code Formatter** (`fmt`, `fmt --check`)
- **Test Framework** — Jest-like API con describe/test/expect + lifecycle hooks + async
- **`or fail`** — Propagación simplificada de errores
- **`=>` one-liners** — if/for/while de una expresión
- **Point-free references** — `items.forEach(print)` sin lambda
- **Method refs `::`** — `names.map(fmt::format)`
- **`break`/`continue`** — Control de flujo en loops
- **`..=` inclusive range** — `for i in 1..=10`
- **Auto data classes** — Sin keyword `data`, inferido por estructura
- **CI/CD** — GitHub Actions en 3 OSes, releases con .deb/.rpm/.tar.gz/.zip
- **Agent Skills** — Skill portable siguiendo estándar agentskills.io

### Stdlib actual
- **String (28 métodos):** toUpperCase, toLowerCase, trim, trimStart, trimEnd, split, replace, replaceAll, contains, startsWith, endsWith, substring, charAt, indexOf, lastIndexOf, slice, padStart, padEnd, repeat, capitalize, isBlank, isEmpty, reverse, truncate, countMatches, removePrefix, removeSuffix, chars
- **Array (31 métodos):** map, filter, reduce, forEach, find, findIndex, some, every, includes, indexOf, join, length, flat, flatMap, slice, sort, distinct, zip, take, drop, first, last, isEmpty, chunks, reversed, sum, min, max, count, sortBy(pending), groupBy(pending)
- **Math (14):** sqrt, pow, abs, floor, ceil, round, min, max, random, PI, E, clamp, sign, log

### Dogfooding
- **79/79 bugs corregidos** (Dogfooding v1: 9 bugs #63-#74, v2: 8 bugs #75-#82)
- **374 tests** totales (179 codegen, 6 desugar, 17 semantic snapshot tests)
- **63 Liva assertion tests** (28 string + 26 array + 9 math) — cobertura completa de stdlib

---

## 🚀 Roadmap de Producción

Liva está en camino a producción. El plan completo está en `docs/plans/PLAN_PRODUCTION_READINESS.md`, las tareas accionables en `BACKLOG.md`.

```
v1.4  Stdlib P0 — String (+15), Array (+20), Math (+3)       ✅ completado
v1.5  rust { } interop + Logging + Config + livac init       ← IN PROGRESS (rust {} + Log done)
v1.6  Stdlib P1 — File, Dir, Date, Regex, CSV/Table          ← scripts reales
v1.7  Stdlib P2 + HTTP Server                                ← backends reales
v1.8  DB + REPL + Linter                                     ← adopción
v2.0  Dogfooding — API REST completa con DB                  ← validación
```

**Documentos clave:**
- `BACKLOG.md` — **qué hacer y cuándo** (checkboxes por versión)
- `docs/plans/PLAN_PRODUCTION_READINESS.md` — **cómo hacerlo** (diseño, sintaxis, decisiones)

---

## ⚠️ Notas para Desarrollo

1. **codegen.rs** es el archivo más grande (~14500 líneas) — toda la generación de Rust
2. **formatter.rs** maneja el formateo de código
3. Tests en `tests/` → `cargo test`
4. LSP se comunica por stdio con la extensión VS Code
5. Archivos `.test.liva` → `livac test` genera tests Rust nativos
6. Switch **statements** usan `case X:` (colon); switch **expressions** usan `X =>` (arrow)
7. Function `=>` tiene implicit return; `if`/`for`/`while =>` NO (solo reemplaza `{}`)
8. `data` keyword **removido** en v1.3.0 — data classes se auto-detectan

---

## 🔁 Regla: Actualizar Contextos

**Al terminar cada tarea o fase, SIEMPRE actualizar:**
- `BACKLOG.md` — marcar `[x]` las tareas completadas
- `livac/.github/copilot-instructions.md` — este archivo (versión, próximo objetivo)
- `.github/copilot-instructions.md` (workspace) — versión y estado general
- `ROADMAP.md` y `CHANGELOG.md` — progreso y changelog
- `docs/plans/PLAN_PRODUCTION_READINESS.md` — si cambia una decisión de diseño


En el chat, responde siempre en español, a menos que se indique lo contrario.