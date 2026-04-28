# 🔧 Liva Compiler Context

> **Proyecto:** livac - El compilador de Liva  
> **Versión:** v2.0.0-dev (tag: v1.2.0)  
> **Lenguaje:** Rust (bootstrap) + Liva (self-hosting)  
> **Tests:** 518 passing  
> **Self-hosting:** Phase 7 idempotente; Phase 8 ✅; Phase 9 ✅ (9.1–9.6, 9.8, 9.9, 9.10 done; 9.7/9.11 absorbidos por Fase 10) — idempotencia gen-2≡gen-3 binario, bench oficial en `benchmarks/RESULTS.md`  
> **Rama activa:** `feat/self-hosting-v2`  
> **Próximo objetivo:** **Fase 10 (optimizaciones del Rust generado) — prerrequisito de v2.0**. Tier 1 ✅ DONE (10.1 last-use+declaredInLoop, 10.2 clone elision en loop, 10.3 iterator chain fusion). Tier 2 10.4 ✅ DONE (&str deref + sort/reverse in-place + split→for fusion). Gate v2.0 (peor bench <1.15x): Word counting bajó de 1.79x→1.28x; CSV 1.17x; Map 1.14x. Sort/Filter+Map <6ms son ruido (DCE). 10.5 (Box<str>) aplazado por baja relación coste/beneficio. Plan en `compiler/docs/PLAN.md` § Fase 10 y `BACKLOG.md`.  
> **Última actualización:** 2026-04-28

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
├── linter.rs         # Linter (W001-W004) — static analysis warnings
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

# Verificar sintaxis / formatear / tests / lint
livac check archivo.liva
livac fmt archivo.liva
livac test archivo.test.liva
livac lint archivo.liva

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

## 🔄 Estado Actual (v2.0.0-dev)

### Features completados
- **CLI Subcomandos** — `build`, `run`, `check`, `fmt`, `test`, `lsp`, `update`, `init`, `lint` (reemplaza flags planos)
- **`rust { }` Interop** — Inline Rust blocks + `use rust` with version/features + E9002 protection
- **Logging (`Log` module)** — info/warn/error/debug + variadic args + table rendering (Map/Array/JSON) + `setLevel`
- **Config (`Config` module)** — `.env` loading + typed getters (`get`, `getInt`, `getBool`, `getAll`)
- **`livac init`** — Project scaffolding con templates (default, cli, data)
- **Stdlib P0** — 38 nuevos métodos/funciones (15 String + 20 Array + 3 Math)
- **Enum Types** — Algebraic data types con pattern matching, destructuring, y **recursive auto-boxing**
- **Error Trace Chaining** — Trazabilidad automática con función + línea
- **`or <value>`** — Default value para fallibles: `divide(10, 0) or 42`
- **Map<K,V> / Set<T>** — Colecciones completas con literales, métodos, iteración
- **Code Formatter** (`fmt`, `fmt --check`)
- **Test Framework** — Jest-like API con describe/test/expect + lifecycle hooks + async
- **`or fail`** — Propagación simplificada de errores
- **`defer`** — Cleanup automático al salir del scope (LIFO, como Go/Swift)
- **`=>` one-liners** — if/for/while de una expresión
- **Point-free references** — `items.forEach(print)` sin lambda
- **Method refs `::`** — `names.map(fmt::format)`
- **`break`/`continue`** — Control de flujo en loops
- **`..=` inclusive range** — `for i in 1..=10`
- **Compound assignment** — `+=`, `-=`, `*=`, `/=`, `%=` (desugared at parser level)
- **Enum wildcard `_`** — `EnumName.Variant(_)` ignores captured value in switch
- **`for i, item in array`** — Enumerate iteration with index and element
- **Extensionless imports** — `import { X } from "./module"` (auto-appends `.liva`)
- **String `push_str` optimization** — `content += ch` generates `push_str()` instead of `format!()`
- **Enum exhaustive switch** — Omit `_` when all variants covered; `E0904` for missing variants
- **Auto data classes** — Sin keyword `data`, inferido por estructura
- **CI/CD** — GitHub Actions en 3 OSes, releases con .deb/.rpm/.tar.gz/.zip
- **Agent Skills** — Skill portable siguiendo estándar agentskills.io

### Stdlib actual
- **String (28 métodos):** toUpperCase, toLowerCase, trim, trimStart, trimEnd, split, replace, replaceAll, contains, startsWith, endsWith, substring, charAt, indexOf, lastIndexOf, slice, padStart, padEnd, repeat, capitalize, isBlank, isEmpty, reverse, truncate, countMatches, removePrefix, removeSuffix, chars
- **Array (33 métodos):** map, filter, reduce, forEach, find, findIndex, some, every, includes, indexOf, join, length, flat, flatMap, slice, sort, sortBy, distinct, zip, take, drop, first, last, isEmpty, chunks, reversed, sum, min, max, count, groupBy
- **Math (14):** sqrt, pow, abs, floor, ceil, round, min, max, random, PI, E, clamp, sign, log

### LANGUAGE_ISSUES — All 21 resolved
- **10 FIXED**: A1-A5, C1-C2, C4-C5, C7, B4
- **4 already-implemented**: B5, B6, C1
- **7 CLOSED**: A6/A7/A8, B1/B2/B3, C3/C6

### Dogfooding
- **90/90 bugs corregidos** (Dogfooding v1: 9 bugs #63-#74, v2: 8 bugs #75-#82, v3: 7 bugs #83-#89, Self-hosting: 4 bugs #90-#94)
- **518 tests** totales
- **Self-hosting:** Fases 0-4 COMPLETAS — 9 módulos, 9,013 líneas Liva, 7/9 generan Rust válido
  - compiler/src/: token, ast, lexer, parser, semantic (1709), liveness (520), codegen (2458), module (234), main (449)
- **63 Liva assertion tests** (28 string + 26 array + 9 math) — cobertura completa de stdlib
- **File (11 funciones):** read, write, append, exists, delete, copy, move, size, extension, readLines, writeLines
- **Dir (7 funciones):** list, isDir, exists, create, delete, listRecursive, walk
- **Regex (5 funciones):** test, match, findAll, replace, split (crate `regex` auto-inyectado)
- **Date (14 funciones):** now, new, parse, timestamp + .year/.month/.day/.hour/.minute/.second + format, add, diff, toString (crate `chrono` auto-inyectado)
- **CSV (8 funciones):** read, write, readTable, writeTable, parse, stringify, headers, column (Table = `[Map<string, string>]`, std puro)
- **Random (5 funciones):** int, float, bool, choice, uuid (crates `rand` + `uuid` auto-inyectados)
- **Crypto (4 funciones):** sha256, md5, base64Encode, base64Decode (crates `sha2` + `md-5` + `base64` auto-inyectados)
- **Process (4 funciones):** run, output, spawn, exit (`std::process`)
- **HTTP Server (6 funciones):** Server.create, app.get/post/put/delete, app.listen + req.params/body + Response.text/json/status (crate `axum` 0.8 + `tokio` auto-inyectados)
- **DB (4 funciones):** DB.open, DB.exec, DB.query, DB.close (crate `rusqlite` 0.32 bundled auto-inyectado)
- **Linter (4 warnings):** W001 unused var, W002 unused import, W003 unreachable code, W004 always true/false — `livac lint <file> [--json]`

---

## 🚀 Roadmap de Producción

Liva está en camino a producción. El plan completo está en `docs/plans/PLAN_PRODUCTION_READINESS.md`, las tareas accionables en `BACKLOG.md`.

```
v1.4  Stdlib P0 — String (+15), Array (+20), Math (+3)       ✅ completado
v1.5  rust { } interop + Logging + Config + livac init       ✅ completado
v1.6  Stdlib P1 — File, Dir, Date, Regex, CSV/Table          ✅ completado
v1.7  Stdlib P2 + HTTP Server                                ✅ completado
v1.8  DB + Linter                                             ✅ completado (REPL ⏸️ aplazado)
v1.9  Dogfooding — API REST completa con DB                  ✅ completado (7 bugs)
v2.0  Enums recursivos + Self-hosting                        ✅ completado (Fases 0-4)
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