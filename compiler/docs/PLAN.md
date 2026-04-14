# Self-Hosting: Compilador de Liva escrito en Liva

> **Estado:** Fase 5 completada ✅ — 84 test files, 83/83 passing  
> **Última actualización:** 2026-04-14  
> **Próximo:** Fase 6 — Madurez arquitectónica + cobertura 100%  
> **Branch:** `feat/self-hosting-v2`

---

## Objetivo

Reescribir el compilador `livac` en Liva. No es un port 1:1 del compilador Rust —
es un **rediseño** que corrige los errores arquitectónicos del compilador actual.

El compilador Rust actual (`src/`) es el **bootstrap compiler**: solo existe
para compilar el compilador Liva la primera vez. Después, el compilador Liva se
compila a sí mismo.

## Estructura del repo

```
livac/
├── src/              ← bootstrap compiler (Rust) — compila compiler/src/
├── compiler/
│   ├── src/          ← compilador Liva (100% puro Liva, 0 rust {} blocks)
│   │   ├── token.liva      (312 líneas)
│   │   ├── ast.liva         (450 líneas)
│   │   ├── lexer.liva       (612 líneas)
│   │   ├── parser.liva      (2,323 líneas)
│   │   ├── semantic.liva    (1,709 líneas)
│   │   ├── liveness.liva    (520 líneas)
│   │   ├── codegen.liva     (4,941 líneas)
│   │   ├── module.liva      (243 líneas)
│   │   └── main.liva        (744 líneas)   ← TOTAL: 11,854 líneas
│   ├── tests/
│   │   ├── liva/            ← Liva Test Suite (84 archivos, 83 passing)
│   │   └── bootstrap_test.sh
│   └── docs/
│       ├── PLAN.md          ← este archivo
│       └── ISSUES.md        ← bugs encontrados durante self-hosting
├── tests/                   ← tests del bootstrap (Rust, 518 tests)
└── Cargo.toml
```

## Por qué reescribir (no reparar)

El compilador Rust tiene un defecto fundamental: **no tiene sistema de tipos en codegen**.

| Métrica | codegen.rs (Rust) | codegen.liva (Liva) | Ratio |
|---------|-------------------|---------------------|-------|
| Líneas | 16,930 | 4,941 | 3.4x menos |
| Campos de tracking | 95 | 30 | 3.2x menos |
| HashSets para adivinar tipos | 47 | 4 | **12x menos** |
| Referencias a `_vars` tracking | 294 | 23 | **13x menos** |
| HACK/TODO/Bug-reference markers | 18+ | 2 | 9x menos |
| Bug-specific workarounds (B09, B39...) | 10+ | 0 | ∞ |

El compilador Rust **adivina tipos** con HashSets (`string_vars`, `float_vars`, `map_vars`...).
El compilador Liva **sabe tipos** con TypeContext del semantic analyzer.

```
BOOTSTRAP (Rust):    Lexer → Parser → Semantic(valida) → Codegen(17K, adivina tipos)
SELF-HOSTED (Liva):  Lexer → Parser → Semantic(valida + TIPA) → Codegen(5K, mecánico)
```

## Referencia

| Documento | Qué contiene |
|-----------|-------------|
| `ISSUES.md` | Bugs encontrados durante self-hosting |
| `docs/plans/PLAN_LIVENESS_ANALYSIS.md` | Diseño técnico del liveness analysis |
| `docs/guides/style-guide.md` | Guía de estilos idiomáticos de Liva |
| `docs/QUICK_REFERENCE.md` | Referencia rápida con gotchas y features |
| `skills/liva-lang/SKILL.md` | Skill portable para agentes AI |

---

## Historial de fases completadas

### Fase 0: Fix Bootstrap ✅
Arreglar los bugs del compilador Rust que bloqueaban la escritura del self-hosting.
- FIX-1 a FIX-6: `let x: T? = value`, switch `match &expr`, params clone, enums Copy, borrar dead code
- 518 tests verdes

### Fase 1: Frontend en Liva ✅
4 módulos (token, ast, lexer, parser) — 3,626 líneas idiomáticas.
Rewrite con compound assignment, one-liners, comentarios WHY-not-WHAT.

### Fase 2: Análisis Semántico ✅
`semantic.liva` (1,709 líneas) + `liveness.liva` (520 líneas).
TypeContext con type pool, scope chain, expression typing, function signatures, class/enum metadata.
Liveness analysis para move/borrow/clone.

### Fase 3: Codegen Limpio ✅
`codegen.liva` (4,941 líneas). Type-directed dispatch via TypeContext.
78 stdlib methods (string/array/map/set), ownership emission, Cargo.toml generation.

### Fase 4: Main + CLI + Bootstrap ✅
`main.liva` (744 líneas) + `module.liva` (243 líneas).
CLI: build/run/check/test subcommands. 7/9 módulos compilan a Rust válido.

### Fase 5: Liva Test Suite ✅
84 archivos de test, 83/83 passing. 6 capas: syntax (18), compile (8), e2e (43), stdlib (12), stdlib-io (1), errors (1+14 fixtures).

---

## Estado actual (2026-04-14)

### Lo que funciona
- **Compilador 100% Liva puro** — 0 bloques `rust {}` en los 9 módulos
- **11,854 líneas** de Liva que se compilan a Rust válido
- **83/83 tests passing** en la Liva Test Suite
- **TypeContext-first architecture** — codegen mecánico, sin adivinaciones de tipo

### Problemas identificados

#### A. Bugs de codegen activos (RC = Root Cause)

| RC | Bug | Impacto | Esfuerzo |
|----|-----|---------|----------|
| RC2 | `toBeTruthy`/`toBeFalsy` en `Option<T>` genera `assert!(!(x))` en vez de `.is_none()` | Assertions en tests con valores nullable | Bajo |
| RC3 | `self.field.clone().push(x)` empuja al clon, no al campo | Cualquier método que muta un campo de clase | Medio |
| RC6 | `.par()` no implementado | Iteración paralela | Bajo |
| RC7 | `async fn` nunca se emite (siempre `pub fn`) | Todo código async/HTTP | Medio |
| RC9 | `!(expr)` pierde paréntesis → `!a == b` en vez de `!(a == b)` | Negaciones compuestas | Bajo |

**RCs ya corregidos:** RC1 (Map.get or), RC5 (rust {} multistatement), RC8 (const string)

#### B. Stdlib sin codegen en self-hosted (7 módulos, ~1,400 líneas)

| Módulo | Funciones faltantes | Esfuerzo |
|--------|---------------------|----------|
| Config | load, get, getInt, getBool, getAll | Bajo (~50 loc) |
| CSV | read, readTable, write, writeTable, parse, parseTable, toTable, fromTable | Medio (~150 loc) |
| JSON | parse (typed + untyped), stringify + serde derives | Alto (~300 loc) |
| DB (SQLite) | open, exec, query, close | Medio (~200 loc) |
| HTTP Server | Server.create, app.get/post/put/delete, app.listen, Response.* | Alto (~400 loc) |
| HTTP Client | get, post, put, delete | Medio (~200 loc) |
| File/Dir/Process (extendido) | 9 + 6 + 3 funciones extras | Bajo (~100 loc) |

**Stdlib que SÍ tiene codegen:** String (28), Array (31), Map (10), Set (10), Math (14), Log (4), Date (8), Regex (4), Random (6), Crypto (5), Process.exec, File.read/write, Dir.create, Sys.args.

#### C. Debilidades arquitectónicas (3 puntos)

1. **Stdlib dispatch es if-else chain** — `_emitStringMethod()`, `_emitArrayMethod()`, `_emitGenericMethodCall()` son ~200 líneas cada uno. Deberían ser dispatch tables (Map de método → generador).
2. **`_emitGenericMethodCall()` duplica lógica** de los métodos tipados como fallback. Código duplicado que puede divergir.
3. **Sin error propagation en codegen** — escribe `/* unknown */` o `todo!()` en vez de reportar. El compilador Rust downstream detecta los errores.

#### D. Tests con `rust {}` workaround (2 archivos)
- `errors.test.liva` — usa `rust { env!("CARGO_MANIFEST_DIR") }` para path del proyecto
- `http_server.test.liva` — ídem

#### E. Features documentadas sin test (23 features)
Console API, JSON module, HTTP Client, File I/O (extendido), Dir I/O (extendido), CSV, Config, DB/SQLite, System module, try-catch, union types, sortBy/groupBy, par/vec/parvec execution policies, data-parallel for, async/par/task/await, optional chaining `?.`, unwrap operator `!`, method references `::`, object/struct literals, parameter destructuring, string toInt/toFloat, `for => pointfree`, polymorphic interfaces.

#### F. Features parcialmente testeadas (18 features)
Error handling (falta err.message), switch (faltan tuple patterns, ranges), generics (faltan generic classes, multiple params), enums (faltan partial wildcards), arrays (faltan take/drop/chunks/zip/findIndex/flatMap/count), strings (faltan trimStart/trimEnd/slice/chars/replaceAll), map (faltan clear/forEach/for-in), set (faltan clear/forEach/for-in), date (faltan parse/comparison), crypto (faltan md5/base64Decode), regex (falta match), process (faltan spawn/pid/exit), defer (falta block form/LIFO), classes (faltan field defaults/async methods), destructuring (faltan array/rest/object/skip), type aliases (faltan generic/function aliases), logging (faltan setLevel/table), math (faltan random/log).

#### G. Error codes sin test (28+ de 42+)
Solo 14 codes testeados: E0001, E0310, E0701, E0901-E0904, E1000, E2000, E4004, W001-W004.

---

## Roadmap: Fase 6 — Madurez (plan de acción)

### Principios

1. **Primero lo que no requiere cambios al compilador** — tests para features que ya compilan
2. **Después bugs de codegen** — cada fix desbloquea más tests
3. **Después arquitectura** — dispatch tables, error propagation
4. **Después stdlib faltante** — cada módulo nuevo solo toca codegen.liva
5. **Al final eliminar `rust {}` de tests** — requiere alternativa Liva para path resolution

### 6.1 — Tests de features que ya compilan (sin tocar el compilador)

> **Objetivo:** Subir cobertura de tests sin tocar codegen.liva
> **Esfuerzo:** Bajo — solo escribir archivos .test.liva
> **Prioridad:** 🔴 ALTA — es la forma más rápida de encontrar bugs

Tests nuevos para features que codegen.liva ya maneja:

| Test | Capa | Features a validar |
|------|------|--------------------|
| `optional_chaining.test.liva` | e2e | `?.` operator, null safety |
| `unwrap_operator.test.liva` | e2e | `!` operator en valores opcionales |
| `try_catch.test.liva` | e2e | try/catch blocks, error propagation |
| `switch_advanced.test.liva` | e2e | Tuple patterns, ranges, case syntax |
| `generics_advanced.test.liva` | e2e | Generic classes, multiple type params, constraint mixing |
| `enum_wildcards.test.liva` | e2e | Partial wildcards en switch |
| `defer_block.test.liva` | e2e | Block form `defer { }`, LIFO verification |
| `destructuring_advanced.test.liva` | e2e | Array destructuring, rest `...`, object patterns |
| `type_aliases_advanced.test.liva` | e2e | Generic aliases, function type aliases |
| `union_types.test.liva` | syntax | Union type declarations |
| `struct_literals.test.liva` | e2e | Struct literal syntax |
| `method_references.test.liva` | e2e | `::` method references |
| `for_pointfree.test.liva` | e2e | `for => ref` point-free style |
| `array_methods_extended.test.liva` | stdlib | take, drop, chunks, zip, findIndex, flatMap, count |
| `string_methods_extended.test.liva` | stdlib | trimStart, trimEnd, slice, chars, replaceAll |
| `map_methods_extended.test.liva` | stdlib | clear, forEach, for-in iteration |
| `set_methods_extended.test.liva` | stdlib | clear, forEach, for-in iteration |
| `date_extended.test.liva` | stdlib | parse, comparison, interpolation |
| `regex_extended.test.liva` | stdlib | match, capture groups |
| `crypto_extended.test.liva` | stdlib | md5, base64Decode |
| `math_extended.test.liva` | stdlib | random, log, log2, log10 |
| `class_advanced.test.liva` | e2e | Field defaults, constructor validation |
| `polymorphic_interfaces.test.liva` | syntax | Interfaces con generic constraints |

**Criterio de salida:** Cada test que falla se clasifica como:
- **Bug del compilador** → se registra en ISSUES.md y se arregla en 6.2
- **Bug del self-hosted codegen** → se arregla en 6.2
- **Feature no implementada** → se documenta y se mueve a 6.4

### 6.2 — Fix RC bugs en codegen.liva

> **Objetivo:** Arreglar los 5 bugs activos del codegen self-hosted
> **Esfuerzo:** Medio — ~100 líneas de cambios en codegen.liva
> **Prioridad:** 🔴 ALTA — desbloquea tests que fallan por RC bugs

| Orden | RC | Fix | Líneas aprox |
|-------|-----|-----|-------------|
| 1 | RC9 | Añadir paréntesis en `_emitUnary` para Not con operandos compuestos | ~5 |
| 2 | RC2 | Detectar Option en `toBeTruthy`/`toBeFalsy` → generar `.is_some()`/`.is_none()` | ~10 |
| 3 | RC6 | Añadir dispatch para `.par()` → `.par_iter()` (rayon) | ~20 |
| 4 | RC7 | Tracking de `async` en funciones → emitir `async fn` + `.await` | ~50 |
| 5 | RC3 | Detección de lvalue en method calls sobre `self.field` — no clonar si es target de mutación | ~30 |

**Proceso:** Cada fix se hace en codegen.liva → se recompila el self-hosted → se valida con tests existentes + test nuevo específico.

### 6.3 — Mejoras arquitectónicas

> **Objetivo:** Preparar codegen.liva para escalar sin acumular deuda técnica
> **Esfuerzo:** Medio — refactor sin cambiar comportamiento
> **Prioridad:** 🟡 MEDIA — previene que el self-hosted se convierta en otro codegen.rs

| Orden | Mejora | Descripción |
|-------|--------|-------------|
| 1 | **Dispatch tables para stdlib** | Reemplazar if-else chains en `_emitStringMethod()`, `_emitArrayMethod()`, etc. con `Map<string, fn>` dispatch. Cada método es una entrada en un map, no un if-else. |
| 2 | **Eliminar `_emitGenericMethodCall()` duplicado** | Unificar la lógica de fallback con los métodos tipados. Si el tipo es desconocido, intentar resolverlo antes de caer en el fallback. |
| 3 | **Error propagation en codegen** | En vez de `/* unknown */` o `todo!()`, acumular errores en un `[Diagnostic]` y reportarlos al final. El usuario ve qué NO se pudo compilar. |
| 4 | **Reducir `.clone()` innecesarios** | Usar liveness info para evitar clone en `let` bindings cuando la variable no se usa después del bind. |

### 6.4 — Codegen para stdlib faltante

> **Objetivo:** Agregar codegen en codegen.liva para los 7 módulos ausentes
> **Esfuerzo:** Alto — ~1,400 líneas nuevas
> **Prioridad:** 🟡 MEDIA — solo necesario si quieres compilar programas que usen estos módulos

**Orden por dependencia y utilidad:**

| Orden | Módulo | Líneas aprox | Dependencias |
|-------|--------|-------------|-------------|
| 1 | **File/Dir/Process extendido** | ~100 | Solo std::fs, std::process |
| 2 | **Config** | ~50 | Solo std::fs (lee .env) |
| 3 | **JSON** | ~300 | Crate `serde` + `serde_json`, serde derives en classes |
| 4 | **CSV** | ~150 | Crate `csv` |
| 5 | **DB (SQLite)** | ~200 | Crate `rusqlite`, connection tracking |
| 6 | **HTTP Server** | ~400 | Crate `axum` + `tokio`, async main, route handlers |
| 7 | **HTTP Client** | ~200 | Crate `reqwest`, async calls |

**Nota:** HTTP Server y HTTP Client requieren que RC7 (async fn) esté arreglado primero.

### 6.5 — Eliminar `rust {}` de tests

> **Objetivo:** Los 2 test files que usan `rust { env!("CARGO_MANIFEST_DIR") }` deben ser Liva puro
> **Esfuerzo:** Bajo — requiere alternativa para obtener la ruta del proyecto
> **Prioridad:** 🟢 BAJA — es cosmético, los tests funcionan

**Opciones:**
1. `Process.exec("pwd")` — ejecuta pwd y usa el resultado
2. `Sys.cwd()` — si se implementa como método del módulo Sys
3. Variable de entorno — pasar `LIVAC_PROJECT_DIR` desde el test runner

### 6.6 — Error codes: cobertura completa

> **Objetivo:** Subir de 14/42+ error codes testeados a 100%
> **Esfuerzo:** Medio — crear fixtures para cada error code
> **Prioridad:** 🟢 BAJA — los error codes principales ya están cubiertos

Error codes sin test (referencia en `docs/ERROR_CODES.md`):
- E01xx: duplicados
- E02xx: type mismatches
- E03xx: undefined functions/params
- E04xx: missing returns
- E05xx: duplicate definitions
- E06xx: invalid imports
- E07xx: fallible handling
- E09xx: exhaustiveness (parcialmente cubiertos)
- W0xx: warnings (parcialmente cubiertos)

---

## Orden de ejecución recomendado

```
Fase 6.1  Tests de features existentes        ← EMPEZAR AQUÍ
  ↓ (cada test que falla genera trabajo para 6.2)
Fase 6.2  Fix RC bugs
  ↓ (con bugs arreglados, más tests pasan)
Fase 6.1  Segunda ronda de tests              ← re-run tests que fallaban
  ↓
Fase 6.3  Mejoras arquitectónicas             ← antes de añadir stdlib
  ↓ (dispatch tables hacen más fácil añadir módulos)
Fase 6.4  Stdlib faltante                     ← File/Dir → Config → JSON → CSV → DB → HTTP
  ↓ (cada módulo habilita tests de stdlib-io)
Fase 6.6  Error codes                         ← en paralelo con lo anterior
  ↓
Fase 6.5  Eliminar rust {} de tests           ← al final, es cosmético
```

---

## Checklist de hitos

```
Fase 0: Fix Bootstrap ✅
  [x] FIX-1 a FIX-6 completados
  [x] 518 tests verdes

Fase 1: Frontend ✅
  [x] token.liva, ast.liva, lexer.liva, parser.liva — idiomáticos

Fase 2: Semantic Analyzer ✅
  [x] 2.1-2.7 completadas (TypeContext, type resolver, liveness analysis)

Fase 3: Codegen ✅
  [x] 3.1-3.7 completadas (RustEmitter, 78 stdlib methods, ownership)

Fase 4: Bootstrap ✅
  [x] main.liva + module.liva + CLI + 7/9 modules → valid Rust

Fase 5: Liva Test Suite ✅
  [x] 84 test files, 83/83 passing
  [x] 6 capas: syntax(18), compile(8), e2e(43), stdlib(12), stdlib-io(1), errors(1+14)

Fase 6: Madurez
  [ ] 6.1: Tests de features existentes (~23 archivos nuevos)
  [ ] 6.2: Fix RC bugs (RC2, RC3, RC6, RC7, RC9)
  [ ] 6.3: Mejoras arquitectónicas (dispatch tables, error propagation)
  [ ] 6.4: Stdlib faltante (File ext, Config, JSON, CSV, DB, HTTP Server, HTTP Client)
  [ ] 6.5: Eliminar rust {} de tests
  [ ] 6.6: Error codes cobertura completa
```

---

## Regla: Todo código Liva sigue la documentación

> **OBLIGATORIO:** Todo código del self-hosting DEBE seguir `docs/guides/style-guide.md`.
> Antes de escribir cualquier módulo nuevo, leer:
> 1. `docs/guides/style-guide.md` — convenciones idiomáticas
> 2. `docs/QUICK_REFERENCE.md` — features del lenguaje con gotchas
> 3. `skills/liva-lang/SKILL.md` — reglas críticas y anti-patterns

