# AI-Generated Code Audit — Informe Consolidado

> **Fecha:** 2026-03-18  
> **Auditor:** GitHub Copilot (Claude Opus 4.6)  
> **Compilador:** livac v1.3.0  
> **Proyectos auditados:** 10  
> **Archivos totales:** 26  
> **Líneas totales:** ~4,990

---

## Resumen Ejecutivo

Se auditaron 10 proyectos generados por IAs usando la skill `liva-lang` como guía. **Ninguno compilaba en su estado original.** Todos fueron corregidos, compilados y ejecutados exitosamente.

El hallazgo principal: **el 76% de los errores son bugs del compilador**, no errores de la IA.  
Las IAs generaron código razonable que debería funcionar según la spec del lenguaje, pero el codegen de livac produce Rust incorrecto en muchos patrones comunes.

---

## Estadísticas Globales

### Errores por categoría

| Categoría | Count | % | Descripción |
|-----------|------:|--:|-------------|
| **BUG** | 70 | 76.1% | Bug del compilador — código válido genera Rust incorrecto |
| **SKILL** | 7 | 7.6% | Error inducido por la skill (documentación insuficiente o confusa) |
| **DESIGN** | 7 | 7.6% | Problema de diseño del lenguaje |
| **AI** | 6 | 6.5% | Error inventado por la IA sin base |
| **GAP** | 2 | 2.2% | Feature que Liva debería tener pero no tiene |
| **Total** | **92** | **100%** | |

### Resumen por proyecto

| # | Proyecto | Archivos | Líneas | Skill | Errores | BUG | SKILL | GAP | AI | DESIGN | rust {} | Estado |
|---|----------|----------|--------|-------|---------|-----|-------|-----|----|--------|--------|--------|
| 1 | web-scraper | 1 | 115 | No | 5 | 3 | 0 | 1 | 0 | 1 | Sí | ✅ |
| 2 | todo-list | 3 | 251 | No | 12 | 8 | 2 | 0 | 1 | 1 | Mínimo | ✅ |
| 3 | text-search | 1 | 253 | No | 9 | 5 | 3 | 0 | 0 | 1 | No | ✅ |
| 4 | snake-game | 1 | 270 | Sí | 1 | 1 | 0 | 0 | 0 | 0 | 8 blocks | ✅ |
| 5 | calculator | 3 | 274 | No | 7 | 7 | 0 | 0 | 0 | 0 | Parser | ✅ |
| 6 | csv-reader | 1 | 283 | No | 9 | 7 | 1 | 0 | 0 | 1 | 1 block | ✅ |
| 7 | chat-server | 1 | 195 | No | 1 | 1 | 0 | 0 | 0 | 0 | 97% | ✅ |
| 8 | json-parser | 5 | 882 | Sí | 16 | 13 | 0 | 0 | 1 | 2 | 0% | ✅ |
| 9 | rest-api | 4 | 973 | Sí | 17 | 14 | 1 | 1 | 1 | 0 | ~450 líneas | ✅ |
| 10 | mini-interpreter | 6 | 1494 | Sí | 15 | 11 | 0 | 0 | 3 | 1 | 49% | ✅ |
| | **TOTAL** | **26** | **~4990** | **4** | **92** | **70** | **7** | **2** | **6** | **7** | | **10/10** |

---

## Top 13 Bugs Más Frecuentes

Bugs que aparecen en 2+ proyectos, ordenados por impacto:

| # | Bug | Proyectos | Count | Impacto |
|---|-----|-----------|------:|---------|
| 1 | **Ownership/move al pasar valores a funciones** | text-search, csv-reader, json-parser, rest-api, mini-interpreter | 5 | Crítico — afecta todo programa no-trivial |
| 2 | **Valores movidos en iteraciones de loops** | csv-reader, rest-api, mini-interpreter | 3 | Crítico — cualquier loop con function calls |
| 3 | **`get_field()` heuristic para acceso a campos** | todo-list, json-parser, mini-interpreter | 3 | Alto — params, loop vars, locals tratados como JSON |
| 4 | **`_` no aceptado en error binding** | csv-reader, json-parser, rest-api | 3 | Medio — patrón `let val, _ = fn()` rechazado |
| 5 | **Strings dentro de template interpolation** | text-search, csv-reader, json-parser | 3 | Medio — `$"{fn("arg")}"` no funciona |
| 6 | **`&mut self` detección incompleta** | todo-list, calculator, mini-interpreter | 3 | Alto — mutación indirecta no detectada |
| 7 | **Arrow method return type no inferido** | calculator, json-parser, rest-api | 3 | Alto — `=> expr` genera `-> ()` |
| 8 | **`.filter()` genera `.copied()` en vez de `.cloned()`** | text-search, csv-reader | 2 | Medio — falla con String/Vec |
| 9 | **Error binding para method calls roto** | calculator, json-parser | 2 | Crítico — `let val, err = this.method()` no funciona |
| 10 | **`or fail` codegen no funcional** | calculator, json-parser | 2 | Crítico — error silenciosamente ignorado |
| 11 | **Cross-file error binding roto** | calculator, rest-api | 2 | Alto — imports multi-archivo no destructuran |
| 12 | **`main()` no async con `rust {}` .await** | chat-server, rest-api | 2 | Alto — async en rust block no detectado |
| 13 | **Array index access mueve en vez de clonar** | csv-reader, json-parser | 2 | Medio — `arr[i]` como argumento consume |

---

## Catálogo Completo de Bugs (47 únicos)

### Ownership y Move Semantics (9 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 17 | Struct/Map pasado a función por valor — consume ownership | text-search, csv-reader, json-parser, rest-api, mini-interpreter |
| 36 | Valores movidos en iteraciones de loop | csv-reader, rest-api, mini-interpreter |
| 35 | Array index access mueve en vez de clonar | csv-reader, json-parser |
| 21 | `self.tokens[idx]` mueve en vez de clonar | calculator, json-parser |
| 44 | `.clone()` no añadido para campos non-Copy de `&self` | todo-list |
| 45 | `for item in this.collection` itera sobre copia — mutaciones perdidas | todo-list |
| 47 | Array concat `arr + [value]` mueve el valor | rest-api |
| 34 | Error binding vars no marcadas `mut` | csv-reader |
| 33 | Single-var binding para fallible genera tuple | csv-reader |

### Error Binding y `or fail` (6 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 19 | Error binding para method calls roto — `(self.method(), None)` | calculator, json-parser |
| 22 | `or fail` codegen no funcional — ni `?` ni `.map_err()` | calculator, json-parser |
| 23 | Cross-file error binding roto — imports generan `(fn(), None)` | calculator, rest-api |
| 20 | `fail "msg"` genera Error::chain con variable fuera de scope | calculator |
| 38 | Error variable scope leak entre ramas if/else | mini-interpreter |
| 1 | `_` en error binding rechazado como identifier | csv-reader, json-parser, rest-api |

### Codegen — Acceso a Campos (5 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 7 | `get_field()` heuristic — locals/params caen al path JSON | todo-list, json-parser, mini-interpreter |
| 6 | `enum_names` no populado en `generate_module_code()` | todo-list |
| 37 | `type` como nombre de campo — keyword reservada en Rust | mini-interpreter |
| 5 | `resp.body` con async genera `get_field("body")` | web-scraper |
| 10 | `.count()` colisión con array method built-in | todo-list |

### Codegen — Métodos de Clase (5 bugs)
| # | Bug | Proyectos |main()
|---|-----|-----------|
| 8 | `&mut self` detección incompleta — solo directo | todo-list, calculator, mini-interpreter |
| 9 | `&mut self` transitivo no propagado | mini-interpreter |
| 18 | Arrow method return type `=> expr` → `-> ()` | calculator, json-parser, rest-api |
| 14 | Enum field rompe `Default` derive | todo-list |
| 46 | serde derives no triggereados por `JSON.stringify` | rest-api |

### Codegen — Strings y Templates (5 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 2 | Strings dentro de template interpolation | text-search, csv-reader, json-parser |
| 25 | `charAt()` retorna String no char | json-parser |
| 26 | Char escape sequences truncados (`'\n'` → `'\\'`) | json-parser |
| 28 | String `+` genera `.extend()` en vez de `push_str` | json-parser |
| 29 | Template `{:?}` para vars mutables — Debug format | json-parser |

### Codegen — Arrays y Collections (3 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 15 | `.filter()` genera `.copied()` no `.cloned()` | text-search, csv-reader |
| 39 | Array element assignment genera LHS inválido | mini-interpreter |
| 16 | `parseInt(x) or default` genera tuple | text-search |

### Codegen — Async (3 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 24 | `main()` no async cuando `rust {}` tiene `.await` | chat-server, rest-api |
| 3 | `async HTTP.get()` rompe error binding | web-scraper |
| 4 | `spawn_async` sin inner `.await` para user functions | web-scraper |

### Codegen — `rust {}` Interop (4 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 42 | `find_rust_blocks()` matchea `rust` en comments | mini-interpreter |
| 43 | `find_balanced_brace()` confunde lifetimes/apostrophes | mini-interpreter |
| 12 | `use` hoisting genera duplicados (E0252) | snake-game |
| 13 | `use` dentro de `rust {}` no se hoistea al top | todo-list |

### Codegen — Tipos y Conversiones (4 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 40 | `String >= &str` comparación — no PartialOrd | mini-interpreter |
| 41 | Cast priority `pos + 1 as usize` → `pos + (1 as usize)` | mini-interpreter |
| 32 | `f64 / i32` sin cast | csv-reader |
| 31 | `const X: string` genera `const X: String` — &str vs String | rest-api |

### Codegen — Enum (2 bugs)
| # | Bug | Proyectos |
|---|-----|-----------|
| 27 | Enum destructuring field name mapping incorrecto | json-parser |
| 30 | Hyphen en `use rust` crate names — no convierte a `_` | rest-api |

### Codegen — Misc (1 bug)
| # | Bug | Proyectos |
|---|-----|-----------|
| 11 | `console.input` con template string — `print!(format!(...))` | todo-list |

---

## Issues de la Skill (7 únicos)

| # | Issue | Proyectos | Acción |
|---|-------|-----------|--------|
| 1 | `task` es keyword reservada — no prominente en docs | todo-list | Destacar en SKILL.md sección de keywords |
| 2 | `main()` es auto-invocado — no documentado claramente | todo-list, text-search, csv-reader | Documentar en SKILL.md |
| 3 | `console.prompt()` no existe — usar `console.input()` | text-search | Corregir en SKILL.md |
| 4 | `Sys.args()` incluye el programa en args[0] | text-search | Documentar en SKILL.md |
| 5 | Skill no documenta cómo funciones Liva se ven desde `rust {}` (snake_case, Result) | rest-api | Añadir sección de interop |
| 6 | Lista de keywords reservadas necesita más prominencia | todo-list | Mover a sección visible |
| 7 | `main()` llamado explícitamente no es necesario | mini-interpreter | Ya documentado como auto-detect |

---

## Issues de Diseño (7 únicos)

| # | Issue | Proyectos | Impacto |
|---|-------|-----------|---------|
| 1 | Instancias de clase clonadas al pasar a free functions — mutaciones perdidas | todo-list | Alto — rompe OOP patterns |
| 2 | Strings dentro de template expressions no soportados | text-search, json-parser | Medio — `$"{fn("x")}"` |
| 3 | `main()` standalone call parseado como función declaration | csv-reader | Bajo |
| 4 | `return expr or fail` — `or fail` solo con `let` | json-parser | Medio |
| 5 | Proyectos multi-archivo requieren merge por bugs de codegen | mini-interpreter, calculator, json-parser | Alto — multi-file broken |
| 6 | Ownership por defecto move — necesita smart default | todos | Crítico |
| 7 | Falta struct literal syntax | mini-interpreter | Alto |

---

## Stdlib GAPS Identificados (desde análisis de `rust {}`)

Features que eliminarían la necesidad de `rust {}` en los proyectos:

| Feature | Prioridad | Proyectos que lo necesitan | Versión candidata |
|---------|-----------|----------------------------|-------------------|
| HTTP Server module (`HTTP.serve()`) | **Alta** | rest-api, chat-server | v1.7 |
| TCP/Net module | **Alta** | chat-server | v1.7+ |
| `array.sortBy(fn)` | **Alta** | csv-reader | v1.6 |
| `Sys.sleep(ms)` | **Alta** | snake-game | v1.6 |
| `Math.randomInt(min, max)` | **Alta** | snake-game | v1.6 |
| Struct literal syntax | **Alta** | mini-interpreter | v1.8+ |
| `console.clear()` / `console.flush()` | Media | snake-game | v1.7 |
| Cast `toInt()` / `toFloat()` | Media | snake-game, mini-interpreter | v1.6 |
| HashMap/Option en stdlib | Media | mini-interpreter | v1.7 |
| Terminal module (raw mode, key polling) | Baja | snake-game | v2.0+ |

---

## Patrones Recurrentes de la IA

### Lo que las IAs hacen bien
1. **Diseño de clases**: Correcto uso de `class`, `fn`, campos tipados
2. **Control de flujo**: if/else, while, for — bien estructurados
3. **Error handling conceptual**: Intentan usar `let val, err = fn()` y `or fail` correctamente
4. **Modularización**: Separación lógica en archivos (tokens, parser, evaluator)
5. **`rust {}` interop**: Cuando la skill lo documenta, lo usan razonablemente

### Lo que las IAs hacen mal
1. **`number` en vez de `float`**: 3 proyectos usan `number` como tipo (no existe en Liva)
2. **Ternarios no-op**: Ocasionalmente generan `if x then a else a` (ambas ramas iguales)
3. **`.message` en strings**: Asumen que errores son objetos con campo `.message`
4. **`console.prompt()` inventado**: No existe, pero suena razonable
5. **Campos con keywords de Rust**: `type`, `match`, etc. como nombres de campo

### Conclusión sobre las IAs
La calidad del código generado es **notablemente alta** para un lenguaje con documentación limitada. El 76% de los errores son bugs del compilador, lo que significa que las IAs generaron código que **debería funcionar** según la spec. Los proyectos con la skill local (snake-game: 1 error, json-parser: 16 pero 0 rust {}) muestran que una skill completa mejora significativamente la calidad del output.

---

## Acciones Recomendadas

### 🔴 Prioridad Alta — Bugs Críticos del Compilador

| Acción | Bugs que resuelve | Impacto |
|--------|-------------------|---------|
| Fix ownership: auto-clone para paso de argumentos | #17, #36, #21, #35, #44, #47 | 5 proyectos, ~30% de todos los errores |
| Fix `get_field()` heuristic — usar `.field` para locals | #7, #37's | 3 proyectos |
| Fix error binding para method calls | #19, #22, #23 | 3 proyectos |
| Fix `&mut self` detección (indirecta + transitiva) | #8, #9 | 3 proyectos |
| Fix `or fail` codegen | #22 | 2 proyectos |
| Fix `find_balanced_brace` apostrophe/lifetime | #43 | Preventivo — cualquier `rust {}` |
| Fix `find_rust_blocks` skip comments | #42 | Preventivo — cualquier `rust {}` |

### 🟡 Prioridad Media — Bugs de Codegen

| Acción | Bugs que resuelve |
|--------|-------------------|
| Fix `.filter()` → `.cloned()` para non-Copy types | #15 |
| Fix arrow method return type inference | #18 |
| Fix strings en template interpolation | #2 |
| Fix `_` en error binding | #1 |
| Fix `async` detection para `rust {}` blocks | #24 |
| Fix `charAt()` return type (char vs String) | #25 |

### 🟢 Prioridad Baja — Mejoras de la Skill

| Acción | Issues que resuelve |
|--------|---------------------|
| Documentar `main()` auto-detect prominentemente | SKILL #2 |
| Destacar keywords reservadas (incl. Rust keywords) | SKILL #1, #6 |
| Añadir sección de `rust {}` interop (snake_case, Result types) | SKILL #5 |
| Corregir `console.prompt()` → `console.input()` | SKILL #3 |
| Documentar `Sys.args()` behavior | SKILL #4 |

---

## Métricas Finales

| Métrica | Valor |
|---------|------:|
| Proyectos auditados | 10 |
| Proyectos que compilan y ejecutan | **10/10** |
| Total errores encontrados | 92 |
| Bugs únicos del compilador | **47** |
| Issues únicos de la skill | 7 |
| Issues de diseño | 7 |
| Errores de la IA | 6 |
| Feature gaps | 2 |
| Líneas de código auditadas | ~4,990 |
| Líneas de código corregidas (fixed/) | ~6,500+ |
| Tests de ejecución pasados | 10/10 |

---

*Generado automáticamente como parte de la auditoría de código AI-generated para el proyecto Liva.*
