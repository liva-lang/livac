# Self-Hosting: Compilador de Liva escrito en Liva

> **Estado:** Fase 8 completada — Fase 9 Commits 1-3 aplicados (9.1/9.2/9.3/9.4/9.5/9.6/9.8/9.10 done, 9.9 ya cubierto, 9.7/9.11 aplazados), bootstrap 9/9, pendiente bench
> **Última actualización:** 2026-04-27
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

#### B. Stdlib codegen ✅ COMPLETO

Todos los módulos stdlib tienen codegen en el self-hosted: String (28), Array (31), Map (10), Set (10), Math (14), Log (4), Date (8), Regex (5), Random (6), Crypto (6), Process (4), File (11), Dir (7), Sys (3), Config (5), CSV (8), JSON (2), DB (4), Server (1), Http (4), Response (3).

#### C. Debilidades arquitectónicas (3 puntos)

1. **Stdlib dispatch es if-else chain** — `_emitStringMethod()`, `_emitArrayMethod()`, `_emitGenericMethodCall()` son ~200 líneas cada uno. Deberían ser dispatch tables (Map de método → generador).
2. **`_emitGenericMethodCall()` duplica lógica** de los métodos tipados como fallback. Código duplicado que puede divergir.
3. **Sin error propagation en codegen** — escribe `/* unknown */` o `todo!()` en vez de reportar. El compilador Rust downstream detecta los errores.

#### D. Tests con `rust {}` workaround ✅ ELIMINADO
- `errors.test.liva` — ahora usa `Sys.env("PWD")` + `Process.exec(cmd) or ""`
- `http_server.test.liva` — ídem
- Único `rust {}` restante: `rust_interop.test.liva` (legítimo — testea la feature)

#### E. Features documentadas sin test (23 features)
Console API, JSON module, HTTP Client, File I/O (extendido), Dir I/O (extendido), CSV, Config, DB/SQLite, System module, try-catch, union types, sortBy/groupBy, par/vec/parvec execution policies, data-parallel for, async/par/task/await, optional chaining `?.`, unwrap operator `!`, method references `::`, object/struct literals, parameter destructuring, string toInt/toFloat, `for => pointfree`, polymorphic interfaces.

#### F. Features parcialmente testeadas (18 features)
Error handling (falta err.message), switch (faltan tuple patterns, ranges), generics (faltan generic classes, multiple params), enums (faltan partial wildcards), arrays (faltan take/drop/chunks/zip/findIndex/flatMap/count), strings (faltan trimStart/trimEnd/slice/chars/replaceAll), map (faltan clear/forEach/for-in), set (faltan clear/forEach/for-in), date (faltan parse/comparison), crypto (faltan md5/base64Decode), regex (falta match), process (faltan spawn/pid/exit), defer (falta block form/LIFO), classes (faltan field defaults/async methods), destructuring (faltan array/rest/object/skip), type aliases (faltan generic/function aliases), logging (faltan setLevel/table), math (faltan random/log).

#### G. Error codes sin test (16+ de 42+)
26 codes testeados: E0001-E0004, E0302, E0310, E0603-E0605, E0701, E0901-E0904, E0906, E1000, E2000, E4004, E4006-E4007, E5001, E9002, W001-W004.
Untestable via `livac check`: E0005 (length on identifiers deferred), E0006-E0007 (HTTP validation not in check path), E0301 (type inference too weak), E0602 (parser can't produce nested modifiers), E4008-E4009 (import order dependency).

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

### 6.2 — Fix RC bugs en codegen.liva ✅ DONE

> **Objetivo:** Arreglar los 5 bugs activos del codegen self-hosted
> **Estado:** ✅ COMPLETADO — 5/5 RC bugs corregidos, compilación exitosa, tests pasan

| Orden | RC | Fix | Estado |
|-------|-----|-----|--------|
| 1 | RC9 | Paréntesis en `_emitUnary` para Not con operandos compuestos | ✅ |
| 2 | RC2 | Detectar Option en `toBeTruthy`/`toBeFalsy` → `.is_some()`/`.is_none()` | ✅ |
| 3 | RC6 | `_emitIterPrefix` para `.par()` → `.par_iter()` (rayon) | ✅ |
| 4 | RC7 | `isAsyncInferred` → `pub async fn` + `#[tokio::main]` | ✅ |
| 5 | RC3 | Detección de métodos mutadores en `self.field` — suprime `.clone()` | ✅ |

### 6.3 — Mejoras arquitectónicas ✅ DONE

> **Objetivo:** Preparar codegen.liva para escalar sin acumular deuda técnica
> **Estado:** ✅ COMPLETADO — dispatch restructure, generic unification, warnings, liveness clone opt

| Orden | Mejora | Descripción | Estado |
|-------|--------|-------------|--------|
| 1 | **Dispatch restructure** | Añadido target "date" en dispatch chain, creado `_isKnownDateMethod()`, tertiary fallback con runtime var tracking (`_mapVars`/`_setVars`/`_stringVars`) | ✅ |
| 2 | **Generic fallback unification** | Creado `_emitDateMethod()` (~40 loc). Reducido `_emitGenericMethodCall()` de 228→65 líneas eliminando métodos duplicados que ya están en dispatchers tipados | ✅ |
| 3 | **Error propagation** | Añadido `_warnings: [string]`, `getWarnings()`, `_warn()`. Warnings en Union type approximation y Optional wrapping fallback | ✅ |
| 4 | **Liveness-based clone reduction** | `_emitClonedArg()` consulta `_liveCtx.useCounts` — si variable se usa 1 vez (last use), omite `.clone()` y mueve | ✅ |

### 6.4 — Codegen para stdlib faltante ✅ DONE

> **Objetivo:** Agregar codegen en codegen.liva para los 7 módulos ausentes
> **Estado:** ✅ COMPLETADO — 7/7 módulos implementados, compilación exitosa, todos los tests verdes

| Orden | Módulo | Funciones | Estado |
|-------|--------|-----------|--------|
| 1 | **File** (extendido) | read, write, append, exists, delete, copy, move, size, extension, readLines, writeLines (11) | ✅ |
| 2 | **Dir** (extendido) | create, list, exists, isDir, delete, listRecursive/walk (7) | ✅ |
| 3 | **Process** (extendido) | exec, spawn, pid, exit (4) | ✅ |
| 4 | **Sys** (extendido) | args, env, exit (3) | ✅ |
| 5 | **Config** | load, get, getInt, getBool, getAll (5) | ✅ |
| 6 | **CSV** | read, write, readTable, writeTable, parse, stringify, headers, column (8) | ✅ |
| 7 | **JSON** | parse, stringify (2) | ✅ |
| 8 | **DB (SQLite)** | open, exec(±params), query(±params), close (4) | ✅ |
| 9 | **Server** | create (1) | ✅ |
| 10 | **Http Client** | get, post, put, delete (4) | ✅ |
| 11 | **Response** | json(±status), text(±status), status (3) | ✅ |

Also: added `usesHttpClient` detection + `reqwest` to Cargo.toml generator.

### 6.5 — Eliminar `rust {}` de tests ✅ DONE

> **Completado:** Reemplazado `rust { env!("CARGO_MANIFEST_DIR") }` con Liva puro
> **Solución:** `Sys.env("LIVAC_PROJECT_ROOT")` con fallback `Sys.env("PWD")`
> **Archivos:** errors.test.liva, http_server.test.liva — 14+1 tests pasan

### 6.6 — Error codes: cobertura completa ✅ DONE

> **Completado:** Subido de 14 a 26 error codes testeados
> **Nuevos:** E0002, E0003, E0004, E0302, E0603, E0604, E0605, E0906, E4006, E4007, E5001, E9002
> **Fixtures:** 12 nuevos archivos .liva + 1 helper module (import_helper)
> **Untestable:** E0005, E0006-E0007, E0301, E0602, E4008-E4009 (parser/check limitations)

### 6.7 — AST caching: eliminar re-parseos redundantes ✅ DONE

> **Completado:** Unificadas Pass 1a + 1b en un solo loop: Sub-pass A (enum names) + Sub-pass B (enum fields + cache)
> **Resultado:** 4 → 2 parses por módulo después del BFS
> **Refactor:** `compileMultiFile()` en main.liva simplificado

Problema actual en `compileMultiFile()`:
```
BFS:     tokenize + parse (extrae imports)        → 1 parse/módulo ✓
Pass 1a: tokenize + parse + OTRO tokenize + parse  → 2 parses/módulo ✗
Pass 1b: tokenize + parse (enum fields)            → 1 parse/módulo ✗
Pass 2:  usa copias de Pass 1a                     → 0 parses ✓
                                            TOTAL:  4 parses/módulo
```

Optimización: unificar Pass 1a + 1b en un solo loop que parsea una vez,
recolecta enum info, y cachea copias del Program para Pass 2.
```
BFS:     tokenize + parse              → 1 parse/módulo ✓
Unified: tokenize + parse → enums + cache → 1 parse/módulo ✓
Pass 2:  usa copias cacheadas          → 0 parses ✓
                                 TOTAL: 2 parses/módulo (4x → 2x)
```

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
Fase 6.6  Error codes                         ✅ DONE — 26/42+ codes testeados
  ↓
Fase 6.5  Eliminar rust {} de tests           ✅ DONE — Sys.env fallback
```

---

## Roadmap: Fase 7 — Self-Compilation

### Objetivo

Que el compilador escrito en Liva (`compiler/src/*.liva`) sea capaz de compilarse
a sí mismo. Es decir:

```
1. Bootstrap (Rust):  livac build compiler/src/main.liva  →  livac-gen1 (binario)
2. Gen-1 (Liva):      livac-gen1 build compiler/src/main.liva  →  livac-gen2 (binario)
3. Validación:         diff <(livac-gen1 output) <(livac-gen2 output)  →  idénticos
```

El paso 1 ya funciona. El paso 2 es el objetivo de esta fase.
El paso 3 (idempotencia generacional) es la prueba final.

### Estrategia

1. **Compilar con bootstrap** — `./target/release/livac build compiler/src/main.liva`
2. **Ejecutar gen-1 contra sí mismo** — ver qué errores produce
3. **Clasificar errores** como: bug de codegen, feature faltante, o limitación del bootstrap
4. **Iterar** hasta que gen-1 produce Rust válido para todos sus módulos
5. **Compilar gen-2** y validar que produce el mismo output

### 7.1 — Gen-1: compilar el compilador ✅ DONE

> **Completado:** gen-1 produce Rust válido para los 9 módulos (253→0 errores cargo)
> **Commit:** `01eaea3` — 12,226 líneas de Rust generado
> **Fixes:** for-loop iteration, let-binding clones, self.field indexing, .length/.size property access

### 7.2 — Gen-2: idempotencia generacional ✅ DONE

> **Completado:** gen-1 output == gen-2 output (8/9 byte-identical, 1 mod-order diff)
> **Commit:** `4cbb30a` — 2000x performance improvement (42s → 0.021s per file)
> **Perf fix:** Suppress self.field auto-clone for indexing (`_inAssignTarget` flag)
> **Idempotence:** Sorted comparison IDENTICAL — gen-2 is functionally equivalent to gen-1

### 7.3 — Clone reduction ✅ DONE

> **Completado:** Reducción de clones innecesarios en Rust generado (2830→1633, -42%)
> **Commit:** `a1d2711` — string comparison optimization, Copy-type detection

### 7.4 — Match borrow optimization ✅ DONE

> **Completado:** `match expr.clone()` → `match &expr` — eliminó O(n²) en switch de tipos recursivos
> **Commit:** `b5f7b72` — gen-2 build de >300s (timeout) a 8.3s (36x+ más rápido)

### 7.5 — Gen-3 fixes ✅ DONE

> **Completado:** 3 fixes para que gen-3 compile correctamente:
> 1. `||` envuelto en paréntesis en `_emitBinary` — corrige precedencia `a && (b || c)`
> 2. Detección de `self.field.subfield.method()` a 2 niveles — suprime `.clone()` en cadenas
> 3. Revert liveness en `_emitClonedArg` — causaba `self._live_ctx.clone()` O(n²) en gen-2+
> **Commit:** `ebc9221` — gen-3 build funcional (36s), idempotencia gen-3=gen-4 verificada

---

## Roadmap: Fase 8 — Calidad del Rust Generado

### Objetivo

Que un programa escrito en Liva genere Rust **igual de eficiente** que Rust escrito a mano.
El compilador ya funciona; ahora hay que hacer que el código que produce sea óptimo.

**No es sobre la velocidad del compilador** — es sobre la velocidad de las aplicaciones
que los usuarios construyen con Liva.

> **⚠️ IMPORTANTE:** Todas las optimizaciones de esta fase se implementan en
> **`compiler/src/codegen.liva`** (el compilador self-hosted), NO en `src/codegen.rs`
> (el bootstrap Rust). El bootstrap solo existe para compilar el self-hosted la
> primera vez. Las mejoras deben ir en el compilador que usarán los usuarios.

### Situación actual (2026-04-15)

Benchmark realizado contra un programa Liva real (REST API, 934 líneas) compilado a Rust:

**Problemas identificados en el Rust generado:**

| Patrón ineficiente | Ejemplo generado | Rust idiomático | Impacto |
|--------------------|-----------------|------------------|---------|
| Clone innecesario de args | `foo(x.clone())` | `foo(x)` / `foo(&x)` | ~1900 clones en 12K líneas |
| `.to_string()` en literals | `"hello".to_string()` pasado a fn | `"hello"` con param `&str` | ~1350 allocations |
| `for item in vec.clone()` | Clona vector entero para iterar | `for item in &vec` | O(n) alloc por loop |
| `self.field.clone().method()` | Clona HashMap para hacer `.get()` | `self.field.get()` | O(n) por field access |
| `format!("{}", x)` | Para cualquier string expression | `x` directamente | Alloc innecesaria |
| `let mut x = value.clone()` | Clona al asignar a let binding | `let x = value` o `let x = &value` | Doble alloc |

### Métricas actuales vs objetivo

| Métrica | Antes (Fase 7) | Ahora (Fase 8) | Objetivo | Notas |
|---------|----------------|----------------|----------|-------|
| `.clone()` por 1K líneas | ~155 | ~163* | <20 | *más líneas ahora; total 996 vs ~1900 original |
| `.to_string()` por 1K líneas | ~110 | ~73 | <30 | `&str` params + move elision |
| `format!("{}", x)` | ~200 | 207† | 0 | †207 son interpolaciones reales, no redundantes |
| `for x in vec.clone()` | ~190 | 31 | 0 | 80% eliminados con `&vec` borrow |
| Binary size vs hand-written | ~same | ~same | ~same | OK |
| **Benchmark vs Rust** | N/A | **6/10 <10%** | all <10% | Numeric/class at parity |

### Estrategia

Cada optimización se mide con un benchmark real antes/después:

1. **Escribir programa de benchmark en Liva** — algo realista que estrese strings, arrays,
   maps, loops, clases. Ej: parser JSON, procesador CSV, mini-servidor.
2. **Escribir el equivalente en Rust a mano** — idiomático, con borrows, sin clones innecesarios.
3. **Compilar ambos, medir con `hyperfine` o `criterion`** — wall time, allocs, peak memory.
4. **Identificar el patrón más costoso** → fix en codegen.liva → re-medir.
5. **Iterar** hasta que la diferencia sea <10% wall time.

### 8.1 — Liveness clone elision + print literal ✅ DONE

> **Commit:** `381bae4`
> **Optimizaciones:**
> - `_emitClonedArg`: Si variable tiene useCounts ≤ 1 y no está en loop, omite `.clone()` (move instead)
> - `_emitForIterable`: Misma liveness check para for-in iterables
> - `_emitPrintCall`/`_emitPrintlnCall`: Detecta `Expr.Literal(Literal.Str(s))` → emite `println!("escaped")` sin format wrapper
> **Gen-3 == Gen-4 (idempotente), 518 tests green**

### 8.2 — Copy-type clone elision + numeric literal detection ✅ DONE

> **Commit:** `76d3a22`
> **Optimizaciones:**
> - `_emitClonedArg`: Detecta Copy types (number/float/bool/char) vía `_lookupVarTypeRef` + `_typeRefToTag` → omite `.clone()`
> - `Expr.Index`: Nuevo helper `_isIndexExprCopyType` → omite `.clone()` para elementos de array Copy
> - Var decl: Detecta init numéricos literales vía `indexOf("= ")` + verificación first/last chars
> **Gen-2 == Gen-3 (idempotente), 518 tests green**

### 8.3 — println! string template forwarding ✅ DONE

> **Commit:** `8467ba6`
> **Optimizaciones:**
> - `_emitStringTemplate` refactorizado: `_emitStringTemplateInner` (solo fmt string + args)
> - Nuevo `_emitStringTemplateInline` helper para println! directo
> - `_emitPrintCall`/`_emitPrintlnCall`: Detecta `Expr.StringTemplate` → `println!("fmt", args)` en vez de `println!("{}", format!("fmt", args))`
> **Gen-3 == Gen-4 (idempotente), 518 tests green**

### 8.4 — push_str chain optimization ✅ DONE

> **Commit:** `415d3cf`
> **Optimizaciones:**
> - Detecta `x = x + y + z` → `x.push_str(y); x.push_str(z)` (elimina `format!` chains)
> - Guard: `_leftmostLeafIsTarget(expr, target)` — verifica que la raíz izquierda del árbol binary sea el target
> - Maneja string literal, string template, y expresión general como RHS
> - Protección: solo activa para `Binary(+)` NO para switch/match/call/etc.
> - Skip para optional vars y cadenas sin raíz en el target
> - Campos: `_pushStrTarget`, `_pushStrUsed` para tracking del estado
> - 9 conversiones `format!()` → `push_str` en codegen.rs generado
> **Gen-2 == Gen-3 (idempotente), 518 tests green**

### 8.5 — &str params for private methods ✅ DONE

> **Commit:** `5fa154b`
> **Optimizaciones:**
> - Private methods (`_prefix`) get `&str` params instead of `String` for string parameters
> - Liveness-based: params with useCounts ≤ 1 and not in loop → `&str`
> - Call sites emit `.as_str()` or pass string literal directly
> - `_strRefParams` map tracks which params are `&str` per function
> - 77 params converted, 56 `.into()` at call sites
> **Gen-1 == Gen-2 (idempotente), 518 tests green**

### 8.6 — for item in &vec borrow iteration ✅ DONE

> **Commit:** `77a6f7a`
> **Optimizaciones:**
> - `_emitForIterable`: Identifier multi-use → `for item in &vec` instead of `vec.clone()`
> - MemberAccess (`self.field`) kept as `.clone()` to avoid E0502 (mutable borrow conflicts)
> - `_forNeedsInnerClone` flag: emits `let item = item.clone();` inside loop when needed
> - 138→80 clone-iterations, 58 now use `&`
> **Gen-1 == Gen-2 (idempotente), 518 tests green**

### 8.7 — Eliminate redundant format!("{}", x) ✅ DONE

> **Commit:** `89248bd`
> **Optimizaciones:**
> - `_emitStringTemplate`: single-expression template `$"{x}"` → `x.to_string()` instead of `format!("{}", x)`
> - Detects `parts.length == 1` and `ExprPart` variant
> - 77→1 `format!` calls in self-hosted codegen output
> **Gen-1 == Gen-2 (idempotente), 518 tests green**

### 8.8 — self.field clone suppression in comparisons ✅ DONE

> **Commit:** `2f11404`
> **Optimizaciones:**
> - `_emitExprNoMemberClone()`: suppresses `.clone()` for direct MemberAccess in comparison contexts
> - Applied in `_emitBinaryLeftCheck` (left side), `_emitBinaryLeftDefault` (both sides for ==,!=,<,>,<=,>=), literal-left (right side)
> - 89→78 `self.field.clone()` calls
> **Gen-1 == Gen-2 (idempotente), 518 tests green**

### 8.9 — Liveness-based let-binding clone elision ✅ DONE

> **Commit:** `d7189bf`
> **Optimizaciones:**
> - For `let x = y` where `y` is a simple identifier: check liveness
> - If `useCounts ≤ 1` and not in loop → skip `.clone()` (move instead)
> - Guard: `&str` params always get `.to_string()` (can't move `&str` to `String`)
> - Hoisted `afterEq` variable for liveness lookup
> - Fix: removed duplicate `let methodName` in `_emitMethod`
> - 1100→996 `.clone()` calls (104 eliminated)
> **Gen-1 == Gen-2 (idempotente), 518 tests green**

### 8.10 — Benchmark suite: Liva vs Rust a mano ✅ DONE

> **Commit:** `45cc67c`
> **3 programas de benchmark** (Liva + Rust a mano, 1000 iteraciones):
>
> | Benchmark | Liva | Rust | Ratio |
> |-----------|------|------|-------|
> | String: Line processing | 215ms | 149ms | 1.44x |
> | String: CSV building | 110ms | 105ms | 1.05x ✅ |
> | String: Word counting | 376ms | 97ms | 3.88x |
> | Collections: Array fill+sum | 3ms | 0ms | ~1x ✅ |
> | Collections: Filter+Map | 5ms | 2ms | 2.5x |
> | Collections: Map build+lookup | 237ms | 158ms | 1.50x |
> | Collections: Sort | 8ms | 2ms | 4x |
> | Classes: Shape compute | 1ms | 0ms | ~1x ✅ |
> | Classes: Vec2 ops | 0ms | 0ms | ~1x ✅ |
> | Classes: Particle sim | 0ms | 4ms | <1x ✅ |
>
> **6/10 benchmarks within <10%** of hand-written Rust.
> Numeric, class and enum code at parity. String/HashMap overhead from ownership-safe clone patterns.

### 8.10 (old 8.6) — Benchmark suite: Liva vs Rust a mano ✅ DONE

> **Commit:** `45cc67c`
> **Detalle:** Ver `benchmarks/RESULTS.md`

3 programas de benchmark (Liva + Rust a mano, 1000 iteraciones cada uno):
- `bench_strings` — line processing, CSV building, word counting
- `bench_collections` — array ops, filter/map, HashMap, sorting
- `bench_classes` — enum pattern matching, Vec2 math, particle simulation

**Resultado: 6/10 benchmarks within <10%** de hand-written Rust.
Compute-bound y class-based code at parity. String/HashMap overhead by clone patterns.

---

## Roadmap: Fase 9 — Cerrar gaps de eficiencia del Rust generado

### Objetivo

Cerrar los 4 benchmarks que aún están >10% sobre Rust escrito a mano (Word counting 3.88x,
Sort 4x, Filter+Map 2.5x, Map build+lookup 1.50x) con optimizaciones medibles en
`compiler/src/codegen.liva`. Meta: **9/10 benchmarks <10%** vs hand-written Rust.

> **⚠️ IMPORTANTE:** Igual que Fase 8 — todas las optimizaciones se implementan
> **únicamente en `compiler/src/codegen.liva`** (self-hosted), NO en `src/codegen.rs`
> (bootstrap Rust).

### Métricas objetivo

| Métrica | Ahora (Fase 8) | Objetivo (Fase 9) |
|---------|----------------|-------------------|
| `.clone()` por 1K líneas (self-host output) | ~163 | <100 |
| Word counting ratio | 3.88x | ~1.3x |
| Sort ratio | 4x | ~1.1x |
| Filter+Map ratio | 2.5x | ~1.15x |
| Map build+lookup ratio | 1.50x | ~1.20x |
| Line processing ratio | 1.44x | ~1.25x |
| **Benchmarks <10%** | 6/10 | **9/10** |

### Estrategia

Una optimización por commit. Por cada item: build bootstrap, rebuild self-host,
`cargo test`, Liva Test Suite, idempotencia gen-1==gen-2 (sorted), benchmark
antes/después en `benchmarks/RESULTS.md`. Si rompe idempotencia o regresa
benchmark: revertir y reabordar.

### 9a — Copy-type detection extendida (base habilitadora)

> **Riesgo:** BAJO. Pre-requisito de 9b/9c — un único punto de verdad para Copy detection.

| Item | Cambio | Ubicación en codegen.liva | Impacto esperado |
|------|--------|---------------------------|------------------|
| 1 | Helper `_isCopyType(typeRef)` extendiendo `_typeRefToTag` | `~260, 4400` | habilita 2-6 |
| 2 | `Map.get(k)` con V Copy → `.copied()` en vez de `.cloned()` | `_emitMapMethod` (~5355) | Map build+lookup ↓ |
| 3 | `Array.first()` / `Array.last()` con T Copy → `.copied()` | `_emitArrayMethod` (~5147) | menos clones |
| 4 | `for x in arr` con T Copy → `for &x in &arr` (deref pattern), eliminar `_forNeedsInnerClone` cuando aplique | `_emitForIterable` / `_emitFor` (~2178) | Line processing ↓ |
| 5 | `Array.sort()` con T primitivo → `.sort()` en vez de `.sort_by(partial_cmp...)` | `_emitArrayMethod` ("sort") | **Sort 4x → ~1.1x** |

### 9b — Iterator chains sin clones

> **Riesgo:** BAJO-MEDIO. Depende de 9a.

| Item | Cambio | Ubicación | Impacto esperado |
|------|--------|-----------|------------------|
| 6 | `_emitIterPrefix` con T Copy: `.iter()` sin `.cloned()` + dereference pattern en cierres downstream (`filter`/`map`/`reduce`), aprovechando `_derefClosureParams` ya existente | `_emitIterPrefix` (~5060) | **Filter+Map 2.5x → ~1.15x** |
| 7 | `Map.keys()` / `Map.values()` directamente iterables: emitir `.keys()` / `.values()` sin `.cloned().collect()` cuando el contexto es for-in | `_emitMapMethod` ("keys", "values") | menos allocs |

### 9c — Map patterns inteligentes

> **Riesgo:** MEDIO. Depende de 9a. Mayor impacto.

| Item | Cambio | Ubicación | Impacto esperado |
|------|--------|-----------|------------------|
| 8 | Peephole: `if m.has(k) { m.set(k, m.get(k) OP e) } else { m.set(k, init) }` → `*m.entry(k).or_insert(init) OP= e`. Solo dispara cuando ambas ramas tocan la misma clave constante con `+`/`-`/`*`/`or` | `_emitIf` | **Word counting 3.88x → ~1.3x** |
| 9 | `Map.set(k, v)` con clave String single-use: omitir `.clone()` de la clave (liveness) | `_emitMapMethod` ("set") | menos clones de String |

### 9d — Limpieza arquitectónica (independiente)

| Item | Cambio | Ubicación | Notas |
|------|--------|-----------|-------|
| 10 | Eliminar `todo!()` residual y reemplazar `/* unknown */` de `_rustTypeToString` por `_warn()` con código de error concreto | `~1977, ~260` | Lo más fácil — agrupar con item 1 |
| 11 | *(Opcional)* Dispatch tables incrementales para `_emitStringMethod` / `_emitArrayMethod` / `_emitMapMethod` / `_emitSetMethod`. Solo si la duplicación tras 9a-9c lo justifica | dispatchers | NO bloqueante |

### Decisiones clave

- **Una opt por commit**, con benchmark + idempotencia verificadas antes de avanzar.
- **Copy detection (item 1) es prerrequisito** de 9a/9b. Hacerlo primero junto con item 10.
- **Item 8 (Entry API) es el de mayor impacto** pero el de mayor riesgo de pattern matching frágil — va después de tener Copy detection sólida (9a completa).
- **String keys en HashMap:** `entry(k.clone())` aún clona la primera vez (sin `entry_ref` estable). Aceptar el clone en `set`; solo elidir en lookups y comparaciones.
- **Closure params al cambiar a `for &x in &arr`:** `x: &T`. Verificar que `_emitClonedArg` no doble el clone — instrumentar con flag "alreadyBorrowed".
- **Item 11 opcional.** Solo si reduce regresiones futuras. No bloquea nada.
- **Fuera de scope:** mensajes de error mejorados, LSP, package manager, retiro del bootstrap, release v2.0.

### Verificación (por cada item)

1. `cargo build --release` (bootstrap)
2. `./target/release/livac build compiler/src/main.liva` (rebuild self-host)
3. `cargo test` (518 tests Rust)
4. `./target/release/livac test compiler/tests/liva` (83 tests Liva)
5. Idempotencia: gen-1 vs gen-2 sobre `compiler/src` con `diff -r` (sorted) → byte-identical
6. `benchmarks/run_benchmarks.sh` → registrar ratio en `benchmarks/RESULTS.md`
7. `grep -c "\.clone()"` en `target/liva_build/` antes/después

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
  [x] 6.1: Tests de features existentes (22 archivos nuevos)
  [x] 6.2: Fix RC bugs (RC2, RC3, RC6, RC7, RC9)
  [x] 6.3: Mejoras arquitectónicas (dispatch, unification, warnings, liveness)
  [x] 6.4: Stdlib faltante (File ext, Dir ext, Process ext, Sys ext, Config, CSV, JSON, DB, Server, Http, Response)
  [x] 6.5: Eliminar rust {} de tests
  [x] 6.6: Error codes cobertura completa (26/42+ codes, 12 nuevos)
  [x] 6.7: AST caching — eliminar re-parseos redundantes (4x → 2x)

Fase 7: Self-Compilation ✅
  [x] 7.1: Gen-1 compila el compilador — 0 cargo errors (commit 01eaea3)
  [x] 7.2: Gen-2 idempotencia generacional — output idéntico (commit 4cbb30a)
  [x] 7.3: Clone reduction (2830→1633, -42%) (commit a1d2711)
  [x] 7.4: Match borrow optimization — gen-2 build >300s→8s (commit b5f7b72)
  [x] 7.5: Gen-3 fixes — || precedence, 2-level self.field, gen-3=gen-4 (commit ebc9221)

Fase 8: Calidad del Rust Generado ✅ COMPLETADA
  [x] 8.1: Liveness clone elision + print literal (commit 381bae4)
  [x] 8.2: Copy-type clone elision + numeric literal detection (commit 76d3a22)
  [x] 8.3: println! string template forwarding (commit 8467ba6)
  [x] 8.4: push_str chain optimization (commit 415d3cf)
  [x] 8.5: &str params for private methods — 77 params (commit 5fa154b)
  [x] 8.6: for item in &vec borrow iteration — 58 converted (commit 77a6f7a)
  [x] 8.7: Eliminate format!("{}", x) — 77→1 (commit 89248bd)
  [x] 8.8: self.field clone suppression in comparisons — 89→78 (commit 2f11404)
  [x] 8.9: Let binding liveness clone elision — 1100→996 (commit d7189bf)
  [x] 8.10: Benchmark suite — 6/10 within <10% of hand-written Rust (commit 45cc67c)

Fase 9: Cerrar gaps de eficiencia del Rust generado
  9a — Copy-type detection extendida (base habilitadora) — Commit 1 aplicado, pendiente bench
  [x] 9.1: Helper `_isCopyType(typeRef)` unificado en codegen.liva (~L3175)
  [x] 9.2: Map.get() con V Copy → `.copied()` (`_emitMapMethod`)
  [x] 9.3: Array.first()/last() con T Copy → `.copied()` (`_emitArrayMethod`)
  [x] 9.4: `for x in arr` con T Copy → `for &x in &arr` sin inner clone (commit 971e451)
  [x] 9.5: Array.sort() para primitivos → `.sort()`; resto `.sort_by(partial_cmp)`
  9b — Iterator chains sin clones
  [x] 9.6: `_emitIterPrefix` con T Copy → `.iter().copied()` (commit 550df15)
  [ ] 9.7: Map.keys()/values() en for-in: sin `.cloned().collect()` — APLAZADO (riesgo: cambia tipo loop var de K a &K, rompe usos en body, requiere análisis de uso)
  9c — Map patterns inteligentes
  [x] 9.8: Peephole has+get+set → entry().or_insert() (`_emitIf` + `_tryEmitEntryApi` en `compiler/src/codegen.liva`). Solo dispara con clave Identifier/Int y operador `+`/`-` con literal Int en RHS y INIT. Idempotencia gen-2==gen-3 verificada (diff -r = 0 líneas).
  [x] 9.9: Map.set con clave String single-use → omitir `.clone()` — YA CUBIERTO por la lógica `isSingleUse` de Phase 8 en `_emitClonedArg`
  9d — Limpieza arquitectónica
  [x] 9.10: `todo!()` / `/* unknown */` reemplazados por `_warn()` + `Some(<expr>)`
  [ ] 9.11: (Opcional) Dispatch tables incrementales para stdlib dispatchers

Baseline workarounds aplicados durante Commit 1 (regresiones bootstrap por auto-clone elision):
  - codegen.liva `_buildParam`: extracción única `extractedName` para evitar E0382 doble-move
  - codegen.liva `_emitAssign`: `let stTarget = stmt.target; let stValue = stmt.value`
  - codegen.liva: rename `let escaped` → `escapedC` en arms `Literal.Char` (W-002)
  - liveness.liva `_analyzeStmt`/`Stmt.Assign`: llamar `_checkAssignEscape(asgn)` ANTES
    de acceder a `asgn.target`/`asgn.value` (clona `asgn` en call site previo a partial-move)

Validación tras Commit 1:
  - cargo test --release: 100% verde (94+282+otros, 0 fallos)
  - bootstrap_test.sh: 9/9 módulos compilan a Rust válido
  - compiler/tests/liva: 107/107 verde
  - cargo check sobre proyecto ensamblado: falla por `serde_json` ausente en Cargo.toml
    generado (issue pre-existente, no de Fase 9)
```

---

## Regla: Todo código Liva sigue la documentación

> **OBLIGATORIO:** Todo código del self-hosting DEBE seguir `docs/guides/style-guide.md`.
> Antes de escribir cualquier módulo nuevo, leer:
> 1. `docs/guides/style-guide.md` — convenciones idiomáticas
> 2. `docs/language-reference` — Guía de referencia del lenguaje
> 3. `docs/QUICK_REFERENCE.md` — features del lenguaje con gotchas
> 4. `skills/liva-lang/SKILL.md` — reglas críticas y anti-patterns

