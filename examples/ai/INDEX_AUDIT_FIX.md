# AI Audit → Fix Implementation — Contexto de Trabajo

> **Inicio:** 2026-03-18  
> **Objetivo:** Implementar los fixes del compilador y mejoras de la skill identificados en la auditoría de 10 proyectos AI-generated  
> **Fuente:** `examples/ai/REPORT_SUMMARY.md` (informe consolidado) + 10× `REPORT.md` por proyecto  
> **Estado:** En progreso — 10/47 bugs corregidos

---

## Qué estamos haciendo

La auditoría de 10 proyectos AI-generated reveló **47 bugs únicos del compilador** y **7 issues de la skill**. Ahora toca **corregirlos en el compilador** y **mejorar la skill**, validando que:
1. Los 388 tests existentes siguen pasando (`cargo test`)
2. Los 40 archivos .liva de ejemplo (fuera de `examples/ai/`) siguen compilando
3. Los proyectos AI auditados se benefician del fix (idealmente compilan sin workaround)

---

## Rutas clave

| Qué | Ruta |
|-----|------|
| **Compilador (src/)** | `livac/src/` |
| — Codegen principal | `livac/src/codegen.rs` |
| — Lexer | `livac/src/lexer.rs` |
| — Parser | `livac/src/parser.rs` |
| — Semántico | `livac/src/semantic.rs` |
| — AST/IR | `livac/src/ast.rs`, `livac/src/ir.rs` |
| — Lowering | `livac/src/lowering.rs` |
| — Desugaring | `livac/src/desugaring.rs` |
| — Formatter | `livac/src/formatter.rs` |
| **Tests** | `livac/tests/` |
| — Codegen tests | `livac/tests/codegen_tests.rs` |
| — Integration tests | `livac/tests/integration_tests.rs` |
| — Parser tests | `livac/tests/parser_tests.rs` |
| — Semantic tests | `livac/tests/semantics_tests.rs` |
| — Snapshots | `livac/tests/snapshots/` (252 archivos) |
| **Ejemplos (no-AI)** | `livac/examples/` (40 archivos .liva) |
| **Proyectos AI (regresión)** | `livac/examples/ai/` (10 proyectos, 26 archivos) |
| — Reports individuales | `livac/examples/ai/<proyecto>/REPORT.md` |
| — Informe consolidado | `livac/examples/ai/REPORT_SUMMARY.md` |
| — Contexto auditoría (cerrado) | `livac/examples/ai/AI_AUDIT_CONTEXT.md` |
| **Skill** | `livac/skills/liva-lang/SKILL.md` |
| **Quick Reference** | `livac/docs/QUICK_REFERENCE.md` |
| **Backlog del compilador** | `livac/BACKLOG.md` |
| **Este archivo** | `livac/examples/ai/INDEX_AUDIT_FIX.md` |

---

## Baseline de tests

```
cargo test → 404 passed, 0 failed, 3 ignored
Snapshots:   262 archivos en tests/snapshots/
Ejemplos:    40 archivos .liva en examples/ (no-AI)
```

**Regla de oro:** `cargo test` debe dar 388+ passed, 0 failed después de CADA cambio. Si un fix rompe algo, se revierte o corrige antes de continuar. Si cambian snapshots, revisar con `cargo insta review`.

---

## Metodología por fix

```
1. Identificar el bug (leer REPORT_SUMMARY → sección correspondiente)
2. Localizar el código en src/ (codegen.rs, lexer.rs, etc.)
3. Escribir un test que reproduzca el problema ANTES de tocar nada
4. Implementar el fix
5. cargo test → 388+ passed, 0 failed
6. Verificar que el fix mejora algún proyecto AI (compilar original sin workaround)
7. Marcar [x] en este archivo
8. Si cambian snapshots: cargo insta review
```

---

## 🔴 Prioridad Alta — Bugs Críticos

Bugs que afectan 3+ proyectos o bloquean patrones fundamentales del lenguaje.

### Ownership y Move Semantics (`codegen.rs`)
> Área: generación de `.clone()`, paso de argumentos, iteración en loops.  
> Afecta: 5 de 10 proyectos. ~30% de todos los errores de la auditoría.

- [ ] **B17** — Struct/Map pasado a función por valor consume ownership → necesita auto-clone  
  Archivo: `codegen.rs` | Proyectos: text-search, csv-reader, json-parser, rest-api, mini-interpreter
- [ ] **B36** — Valores movidos en iteraciones de loop → variable consumida en primera iteración  
  Archivo: `codegen.rs` | Proyectos: csv-reader, rest-api, mini-interpreter
- [ ] **B35** — Array index access (`arr[i]`) como argumento mueve en vez de clonar  
  Archivo: `codegen.rs` | Proyectos: csv-reader, json-parser
- [ ] **B21** — `self.tokens[idx]` mueve Token del Vec en vez de `.clone()`  
  Archivo: `codegen.rs` | Proyectos: calculator, json-parser
- [ ] **B44** — `.clone()` no añadido para campos non-Copy de `&self`  
  Archivo: `codegen.rs` | Proyectos: todo-list
- [ ] **B45** — `for item in this.collection` itera sobre copia — mutaciones perdidas  
  Archivo: `codegen.rs` | Proyectos: todo-list
- [ ] **B47** — Array concat `arr + [value]` mueve el valor  
  Archivo: `codegen.rs` | Proyectos: rest-api
- [ ] **B34** — Error binding vars no marcadas `mut` cuando se reasignan  
  Archivo: `codegen.rs` | Proyectos: csv-reader

### Error Binding y `or fail` (`codegen.rs`)
> Área: generación de `match Result { Ok/Err }` y operador `?`.  
> Afecta: 3+ proyectos. Patrón fundamental del lenguaje.

- [x] **B19** — Error binding para method calls roto — `(self.method(), None)` en vez de destructurar Result  ✅ 2026-03-18
  Archivo: `codegen.rs` | Proyectos: calculator, json-parser
- [x] **B22** — `or fail` codegen no funcional — ni `?` ni `.map_err()` generado  ✅ 2026-03-18
  Archivo: `codegen.rs` | Proyectos: calculator, json-parser
- [x] **B23** — Cross-file error binding roto — imports generan `(fn(), None)`  ✅ 2026-03-18
  Archivo: `codegen.rs` | Proyectos: calculator, rest-api
- [x] **B20** — `fail "msg"` genera Error::chain con variable de error fuera de scope  ✅ 2026-03-18
  Archivo: `codegen.rs` | Proyectos: calculator
- [x] **B38** — Error variable scope leak entre ramas if/else  ✅ 2026-03-18 (parcial — fail scope corregido, latent codegen OK)
  Archivo: `codegen.rs` | Proyectos: mini-interpreter
- [x] **B01** — `_` no aceptado en error binding (`let val, _ = fn()`)  ✅ 2026-03-18
  Archivo: `parser.rs` + `semantic.rs` | Proyectos: csv-reader, json-parser, rest-api

### Acceso a Campos (`codegen.rs`)
> Área: `get_field()` heuristic vs `.field` directo.

- [ ] **B07** — `get_field()` heuristic — locals/params caen al path JSON en vez de `.field`  
  Archivo: `codegen.rs` | Proyectos: todo-list, json-parser, mini-interpreter
- [ ] **B06** — `enum_names` no populado en `generate_module_code()`  
  Archivo: `codegen.rs` | Proyectos: todo-list
- [ ] **B05** — `resp.body` con async genera `get_field("body")`  
  Archivo: `codegen.rs` | Proyectos: web-scraper
- [ ] **B10** — `.count()` colisión nombre de método de usuario con array built-in  
  Archivo: `codegen.rs` | Proyectos: todo-list

### Métodos de Clase (`codegen.rs`)
> Área: detección de `&mut self`, return type inference.

- [ ] **B08** — `&mut self` detección incompleta — solo detecta `this.field = x` directo  
  Archivo: `codegen.rs` | Proyectos: todo-list, calculator, mini-interpreter
- [ ] **B09** — `&mut self` transitivo no propagado — método que llama `&mut` no se marca  
  Archivo: `codegen.rs` | Proyectos: mini-interpreter
- [x] **B18** — Arrow method return type `=> expr` genera `-> ()` en vez de inferir  ✅ 2026-03-18
  Archivo: `codegen.rs` | Proyectos: calculator, json-parser, rest-api
- [ ] **B14** — Enum field en clase rompe `Default` derive  
  Archivo: `codegen.rs` | Proyectos: todo-list
- [ ] **B46** — serde derives no triggereados por `JSON.stringify` (solo `JSON.parse`)  
  Archivo: `codegen.rs` | Proyectos: rest-api

### `rust {}` Interop (`lexer.rs`)
> Área: detección de bloques rust y balance de braces.

- [x] **B42** — `find_rust_blocks()` matchea `rust` keyword dentro de `//` comments  ✅ 2026-03-18
  Archivo: `lexer.rs` | Proyectos: mini-interpreter
- [x] **B43** — `find_balanced_brace()` confunde lifetimes/apostrophes con char literals  ✅ 2026-03-18
  Archivo: `lexer.rs` | Proyectos: mini-interpreter

---

## 🟡 Prioridad Media — Bugs de Codegen

Bugs que afectan 1-2 proyectos o patrones menos frecuentes.

### Strings y Templates (`codegen.rs`)

- [ ] **B02** — Strings dentro de template interpolation (`$"{fn("arg")}"`) rompe parsing  
  Proyectos: text-search, csv-reader, json-parser
- [ ] **B25** — `charAt()` retorna String en vez de char — comparaciones `ch == '"'` fallan  
  Proyectos: json-parser
- [ ] **B26** — Char escape sequences truncados (`'\n'` → `'\\'`)  
  Proyectos: json-parser
- [ ] **B28** — String `+` genera `.extend()` (iterator) en vez de `push_str`  
  Proyectos: json-parser
- [ ] **B29** — Template `{:?}` para vars mutables — Debug format añade quotes  
  Proyectos: json-parser

### Arrays y Collections (`codegen.rs`)

- [x] **B15** — `.filter()` genera `.copied()` en vez de `.cloned()` para non-Copy types  ✅ 2026-03-18
  Proyectos: text-search, csv-reader
- [ ] **B39** — Array element assignment (`arr[i] = val`) genera LHS inválido  
  Proyectos: mini-interpreter
- [ ] **B16** — `parseInt(x) or default` genera tuple en vez de unwrap  
  Proyectos: text-search

### Async (`codegen.rs`)

- [ ] **B24** — `main()` no async cuando `rust {}` contiene `.await`  
  Proyectos: chat-server, rest-api
- [ ] **B03** — `async HTTP.get()` rompe error binding  
  Proyectos: web-scraper
- [ ] **B04** — `spawn_async` sin inner `.await` para user functions  
  Proyectos: web-scraper

### Tipos y Conversiones (`codegen.rs`)

- [ ] **B40** — `String >= &str` comparación — no existe PartialOrd  
  Proyectos: mini-interpreter
- [ ] **B41** — Cast priority `pos + 1 as usize` → `pos + (1 as usize)` = `i32 + usize`  
  Proyectos: mini-interpreter
- [ ] **B32** — `f64 / i32` sin cast automático  
  Proyectos: csv-reader
- [ ] **B31** — `const X: string` genera `const X: String` — `&str` vs `String`  
  Proyectos: rest-api

### Enum (`codegen.rs`)

- [ ] **B27** — Enum destructuring field name mapping incorrecto  
  Proyectos: json-parser
- [ ] **B30** — Hyphen en `use rust` crate names no se convierte a `_` (**YA CORREGIDO** en auditoría)  
  Proyectos: rest-api

### Misc

- [ ] **B11** — `console.input` con template string — `print!(format!(...))` anidado  
  Proyectos: todo-list
- [ ] **B33** — Single-var binding para fallible genera tuple  
  Proyectos: csv-reader
- [ ] **B37** — `type` como nombre de campo — keyword reservada en Rust  
  Proyectos: mini-interpreter

---

## 🟢 Mejoras de la Skill (`skills/liva-lang/SKILL.md`)

- [ ] **S1** — Destacar keywords reservadas (incluir keywords de Rust: `type`, `match`, `mod`, etc.)
- [ ] **S2** — Documentar `main()` auto-detect prominentemente (no necesita llamada explícita)
- [ ] **S3** — Corregir `console.prompt()` → `console.input()` si aparece en la skill
- [ ] **S4** — Documentar `Sys.args()` behavior (args[0] = programa)
- [ ] **S5** — Añadir sección de `rust {}` interop: snake_case transform, Result types, cómo Liva vars se ven desde Rust
- [ ] **S6** — Documentar que `number` no es un tipo válido — usar `int` o `float`
- [ ] **S7** — Documentar que errores son `string`, no objetos con `.message`

---

## Issues de Diseño (referencia — no son fixes inmediatos)

Estos requieren decisiones de diseño más profundas. Documentados aquí para tracking.

| ID | Issue | Impacto | Notas |
|----|-------|---------|-------|
| D1 | Instancias de clase clonadas al pasar a free functions — mutaciones perdidas | Alto | Requiere decisión: pass by reference? |
| D2 | Strings dentro de template expressions — nested quotes | Medio | Requiere cambio en lexer/parser |
| D3 | `return expr or fail` — `or fail` solo funciona con `let` | Medio | Requiere cambio en parser |
| D4 | Multi-file codegen tiene bugs de imports/error binding | Alto | B23 cubre parte |
| D5 | Ownership por defecto move — necesita smart default | Crítico | B17 cubre workaround |
| D6 | Falta struct literal syntax | Alto | Candidato v1.8+ |

---

## Stdlib GAPS (referencia — ya en BACKLOG.md)

Features que ya están o deberían estar en el backlog del compilador por versión:

| Feature | Versión | Ya en BACKLOG? |
|---------|---------|----------------|
| `arr.sortBy(fn)` | v1.6 | Sí (pendiente) |
| `Sys.sleep(ms)` | v1.6 | No — añadir |
| `Math.randomInt(min, max)` | v1.6 | No — añadir (→ `Random.int` en v1.7) |
| `toInt()` / `toFloat()` casts | v1.6 | No — añadir |
| `console.clear()` / `console.flush()` | v1.7 | No — añadir |
| HTTP Server module | v1.7 | Sí |
| TCP/Net module | v1.7+ | No — añadir |
| HashMap/Option en stdlib | v1.7 | No — añadir |
| Struct literal syntax | v1.8+ | No — añadir |
| Terminal module (raw mode) | v2.0+ | No — añadir |

---

## Progreso

| Fecha | Tarea | Estado | Tests | Notas |
|-------|-------|--------|-------|-------|
| 2026-03-18 | B01: `_` en error binding | ✅ Done | 389 passed, 0 failed | Parser: `Token::Underscore` → `BindingPattern::Identifier("_")`, Semantic: skip declare for `_` |
| 2026-03-18 | B42: `find_rust_blocks()` skip comments | ✅ Done | 399 passed, 0 failed | Lexer: skip `//`, `/* */`, and string literals before scanning for `rust` keyword |
| 2026-03-18 | B43: `find_balanced_brace()` lifetimes | ✅ Done | 399 passed, 0 failed | Lexer: proper char literal vs lifetime detection — lifetimes don't consume braces |
| 2026-03-18 | B15: `.filter()` .copied → .cloned | ✅ Done | 400 passed, 0 failed | Codegen: default to `.cloned()` for untracked arrays — always safe (Copy implies Clone) |
| 2026-03-18 | B18: arrow method return type | ✅ Done | 401 passed, 0 failed | Codegen: expanded `infer_expr_type` — handles Index, Identifier, arithmetic, Ternary, UnaryNot |
| 2026-03-18 | B19: error binding method calls | ✅ Done | 402 passed, 0 failed | Codegen: added `fallible_methods` HashSet, extended `is_fallible_expr` for `Expr::MethodCall` |
| 2026-03-18 | B22: or fail method calls | ✅ Done | 403 passed, 0 failed | Fixed by B19 — `is_fallible_expr` now recognizes method calls, `or fail` path works |
| 2026-03-18 | B20: fail scope tracking | ✅ Done | 404 passed, 0 failed | Codegen: `error_binding_scope_stack` tracks scope via indent/dedent, `fail "msg"` uses Error::new when err out of scope |
| 2026-03-18 | B23: cross-file error binding | ✅ Done | 404 passed, 0 failed | Codegen: `generate_entry_point()` pre-populates `fallible_functions`/`fallible_methods` from imported modules |
| 2026-03-18 | B38: error var scope leak | ✅ Partial | 404 passed, 0 failed | `fail` scope fixed via B20. Latent in other codegen sites but Rust scoping handles it correctly |

---

## Reglas

1. **`cargo test` SIEMPRE** — 388+ passed, 0 failed, después de cada cambio
2. **Test primero** — escribir test que reproduzca el bug antes de implementar el fix
3. **Un fix a la vez** — no mezclar fixes no relacionados en el mismo cambio
4. **Actualizar este archivo** — marcar `[x]` al completar, añadir fecha al progreso
5. **Si cambian snapshots** — `cargo insta review` y verificar que el cambio es correcto
6. **Referencia detallada** — para el detalle de cada bug, ver `REPORT_SUMMARY.md` sección "Catálogo Completo"
