# Análisis exhaustivo del proyecto Liva

> **Fecha:** 2026-05-05
> **Versión analizada:** v2.0.0-rc1 (release gate passed)
> **Alcance:** lenguaje, compilador bootstrap (Rust), compilador self-hosted (Liva), stdlib, herramientas, ecosistema
> **Contexto:** se planea promocionar `livac/compiler/` (self-hosted) como compilador oficial y eliminar el bootstrap `livac/src/` (Rust).

---

## 0. TL;DR

Liva está, hoy, **en muy buen estado para ser un proyecto de un solo desarrollador**: tiene un lenguaje coherente, ~700 validaciones automáticas, idempotencia binaria gen-2 ≡ gen-3, 10/10 benchmarks por debajo de 1.15× respecto a Rust escrito a mano, stdlib usable y LSP funcional.

**Pero NO es realista cortar el bootstrap todavía.** El gen-2 self-hosted sólo pasa los 21 `bootstrap_apps`; el bootstrap en Rust cubre todavía features que el self-host no porta (function types `(T)=>U`, error handling unificado con `liva_rt::Error`, `T!` fallible, multi-file imports al ≥50%, LSP, formatter, linter completo, runner de tests, `livac init`/`update`, HTTP routing en self-host con axum). Eliminar `livac/src/` sin completar Tier 2 + LSP + tooling rompe varias features ya documentadas como GA en v2.0.

**Recomendación:** v2.0.0 final con la pareja bootstrap + gen-2 (status quo); v2.1 cierra Tier 2 + LSP en gen-2; v2.2 retira el bootstrap del binario de release pero lo conserva en repo como **stage-0** (igual que rustc/Go/Zig).

---

## 1. Salud del proyecto en números

| Métrica | Valor | Observación |
|---|---|---|
| Versión Cargo | 2.0.0-rc1 | RC, no final |
| LOC Rust (`livac/src/*.rs`) | **36 495** | bootstrap, marcado FROZEN |
| LOC Liva (`livac/compiler/src/*.liva`) | **16 261** | self-host, idempotente |
| Tests cargo | 531–533 | 100 % verde release |
| Tests `.test.liva` | 141 | en 7 categorías |
| Bootstrap apps gen-2 | 21/21 | byte-idénticos a gen-3 |
| E2E self-host | 5/5 | stdout idéntico bootstrap vs gen-2 |
| Gates de release | 7/7 | verdes |
| Benchmarks bajo 1.15× | 10/10 | peor caso 1.12× |
| Cobertura regions | 62.81 % | baseline `cargo-llvm-cov` |
| Cobertura lines | 62.36 % | idem |
| Errores E0xxx documentados | 50+ | catálogo en `docs/ERROR_CODES.md` |
| Snippets VS Code | ~300 | `vscode-extension/snippets/` |

El bootstrap es **2.24× más grande en LOC** que el self-host. Esto es saludable: significa que el self-host está aprovechando la expresividad del propio lenguaje.

---

## 2. ¿Es bueno el lenguaje? — análisis de diseño

### 2.1 Lo que funciona muy bien

1. **Sintaxis limpia y familiar.** Mezcla TS/Python/Kotlin sin ruido (`fn`/`def`/`function`). Aprendizaje inmediato para cualquier dev.
2. **Errores como valores explícitos.** `T!`, `or <default>`, `or fail`, error binding `let v, err = f()`, error chaining. Es más estricto que Go (no se puede ignorar el error sin pedirlo) y más ergonómico que Rust (`?` requiere `From` impls).
3. **Concurrencia híbrida real.** `async/await` + `for par` + validación `Send/Sync` en compile-time. Esto es muy raro en lenguajes "scripting-like".
4. **Interop Rust de primera clase.** `rust { … }` inline + `use rust "crate" version "x.y" with features [...]`. Esto es un *escape hatch* genuino y bien diseñado.
5. **Self-hosting idempotente** desde la primera release que lo intenta. Pocos lenguajes consiguen `gen-2 ≡ gen-3` byte-a-byte sin años de iteración.
6. **Determinismo en compilación.** El AST tiene orden estable; los `HashMap` se reemplazaron por `BTreeMap` donde aplicaba. Esto habilita reproducible builds y caching.
7. **LSP first-class** desde v0.x, no parche.
8. **Mensajes de error con código + sugerencia + ejemplo.** Catálogo `E0001..E9xxx`. Mejor que la media para un lenguaje joven.

### 2.2 Carencias del **lenguaje** (no del compilador)

Estas son cosas que *faltan en el diseño*, no bugs:

| Carencia | Impacto | Prioridad |
|---|---|---|
| **Tipos función nativos `(T) => U`** | Ergonomía mala para callbacks; obliga a `Box<dyn Fn>` boilerplate | ⚡ alta |
| **Lambdas / closures completos** | Ligado a lo anterior; afecta `arr.reduce(0, |a,b| a+b)` con captura | ⚡ alta |
| **Tuplas como tipo de primera clase** | Hoy se simulan con clases; `DB.open` re-wrapea por esto | 🔶 media |
| **Pattern matching en `let`** (destructuring) | Sólo en `switch`; no `let {a,b} = obj` ni `let [x, ...rest] = arr` | 🔶 media |
| **Trait/Interface con default methods** | Interfaces hoy son sólo signatures; no `default impl` | 🔶 media |
| **Operator overloading** | No hay forma de definir `+` en clases de usuario (Vec2 lo hace por codegen especial) | 🔶 media |
| **`?` operator para fallibles** | El `or fail` lo cubre parcialmente, pero `f()?.g()?.h()` no es cómodo | 🔷 baja |
| **Enum con variantes `struct-like`** | Hoy variantes son tuple-style; falta `Enum.Variant { name: string, age: i32 }` con acceso por nombre | 🔶 media |
| **Macros / metaprogramación** | Ninguna. Ni siquiera estilo `#[derive]` declarable por usuario | 🔷 baja (post-v2) |
| **Const generics / generic bounds explícitos** | `<T: Display + Clone>` no es expresable en código de usuario; lo deriva el compilador | 🔶 media |
| **Async traits / interfaces async** | No definidas | 🔷 baja |
| **Iterator protocol explícito** | Todo método de iterador es codegen-special. No se pueden definir iteradores user-defined | 🔶 media |
| **Numérico bien tipado** | Promoción i32/i64/f32/f64 no documentada exhaustivamente; potencial de sorpresas | 🔶 media |
| **Strings UTF-8 explícitas** | `chars()`, `slice()` operan a nivel de bytes/chars sin distinción documentada | 🔷 baja |

> **Lectura clave:** el lenguaje tiene 80 % de un lenguaje moderno. El 20 % que falta (function types, lambdas con captura, tuplas, destructuring) es lo que más se va a notar el día que un usuario externo lo pruebe en serio. Esto debería formar parte del backlog de v2.x explícitamente.

### 2.3 Carencias del **compilador self-hosted** (gen-2) frente a bootstrap

Esto sale directamente de [livac/compiler/PARITY.md](livac/compiler/PARITY.md):

**Bloqueantes para hacer cutover a self-host únicamente:**

- ⚡ `GAP-007` — Function types `(T) => U`. Sin esto, `app28_closures.liva` y cualquier API con callbacks funciona sólo con bootstrap.
- ⚡ `B144` — Parámetros `Map<K,V>` / `Set<T>` no registrados en codegen state.
- ⚡ `B145` — `string.indexOf(needle, fromIndex)` 2-arg.
- ⚡ `B141`, `B142`, `B147` (✅), `B148` — varios fixes en codegen.
- ⚡ **Tier 2 entero** — `ERR-UNIFY`: gen-2 todavía emite `Result<T, String>`; falta migrar a `Result<T, liva_rt::Error>`. Esto bloquea **B127–B143** (todo el sub-sistema de fallibles, error binding chains, `or fail` en propagación, `e.message` post-narrowing).
- 🔶 `B152`, `B153` — auto-derive `Display`/`Clone` para genéricos.
- 🔶 `B134`, `B135`, `B136` — switch arms con `if`-tail, `for k,v in map` typing, `Set.size` propiedad.

**Features de tooling que **sólo** existen en bootstrap:**

- `livac lsp` — LSP server (tower-lsp, dashmap)
- `livac fmt` — formatter AST-based completo
- `livac lint` — linter W001..W004
- `livac test` — runner de tests `test_*`
- `livac init` — scaffold de proyecto
- `livac update` — self-update
- `livac check` — type-check sin codegen
- HTTP server emission (axum + async closures + routing)
- Multi-file imports cobertura ≥50 %
- Plugin system de stdlib P1/P2 (Date, Regex, CSV, Crypto, Random, Process, DB)

**Conclusión dura:** el self-host hoy es **un compilador batch que sabe traducir programas Liva a Rust**. No es todavía la herramienta `livac` completa.

---

## 3. ¿Es lento? — análisis de rendimiento

### 3.1 Programas compilados

Resultado oficial 2026-05-04 ([RESULTS.md](livac/benchmarks/RESULTS.md)):

| Suite | Métrica | Liva | Rust | Ratio |
|---|---|---:|---:|---:|
| strings | Line processing | 154ms | 147ms | **1.05×** |
| strings | CSV building | 104ms | 103ms | **1.01×** |
| strings | Word counting | 93ms | 94ms | **0.99×** |
| collections | Array fill+sum | 49ms | 44ms | **1.11×** |
| collections | Filter+Map | 27ms | 24ms | **1.12×** |
| collections | Map build+lookup | 168ms | 150ms | **1.12×** |
| collections | Sort | 64ms | 63ms | **1.02×** |
| classes | Shape compute | 15ms | 15ms | **1.00×** |
| classes | Vec2 ops | 115ms | 114ms | **1.01×** |
| classes | Particle sim | 49ms | 111ms | **0.44×** |

**Nota crítica honesta:** Particle sim a 0.44× (Liva más rápido que Rust escrito a mano) **no es algo de lo que presumir**: indica que la versión Rust del benchmark probablemente no aplica las mismas inlinings o layouts que el código generado por Liva. Convendría revisar ese caso para no transmitir una falsa impresión. El resto (1.00×–1.12×) es el resultado *real* y es excelente.

**Veredicto:** Liva **no es lento**. Está al nivel de Rust escrito a mano para casi todo. El gate <1.15× pasa con margen.

### 3.2 Velocidad del compilador (no medida formalmente)

No hay benchmarks de tiempo de compilación. Esto es una **carencia** del proyecto:

- ¿Cuánto tarda el self-host en compilarse a sí mismo?
- ¿Cómo escala con LOC?
- ¿Hay incrementalidad? (Aparentemente no; cada `livac build` recompila todo el módulo.)

**Recomendación:** añadir un `cargo bench` o script `benchmarks/compile_speed.sh` que mida bootstrap vs gen-2 sobre los 21 bootstrap_apps. Esto **debería** ser un gate antes de declarar v2.0 final.

### 3.3 Tamaño de binarios y memoria

Tampoco medidos. Para un compilador que se distribuye como tarball multiplataforma, esto es relevante.

---

## 4. Calidad del código

### 4.1 Bootstrap (`livac/src/`)

**Fortalezas:**
- Buena separación de fases (lexer → parser → semantic → desugaring → codegen).
- Catálogo de errores centralizado (`error_codes.rs`).
- Tests unitarios + snapshot (`insta`) + property-based (`proptest`).
- Cobertura razonable (62 %) considerando la superficie.

**Debilidades:**
- `codegen.rs` con **6 200 líneas en un solo fichero**. Es el principal *code smell* del proyecto. Refactor pendiente que nunca vendrá porque el bootstrap está congelado.
- `semantic.rs` con 4 425 líneas, similar problema.
- Cobertura de `codegen.rs` 67 %, de `semantic.rs` 48 %. Las partes peor testeadas son justamente las más grandes.
- LSP con **0 % de cobertura instrumentada**; sólo se valida manualmente.

### 4.2 Self-host (`livac/compiler/`)

**Fortalezas:**
- Es la mejor demostración del lenguaje: 16 k LOC funcionales, idempotentes y rápidas.
- Modularización razonable (9 ficheros, separados por fase).
- `liveness.liva` (escape analysis) es código de optimización de calidad que valida que el lenguaje sirve para sistemas.

**Debilidades:**
- `codegen.liva` con **9 085 líneas** (más grande que el bootstrap). Mismo problema, agravado: además es código en un lenguaje sin macros ni metaprogramación.
- No hay tests unitarios *internos* del self-host: la validación es 100 % black-box (gen-2 vs gen-3 byte-equal + bootstrap_apps + e2e). Si hay un bug que afecta sólo a una rama poco ejercitada, no lo detectaremos hasta que un usuario lo encuentre.
- Cobertura no medida (no hay `liva-llvm-cov`).

### 4.3 Convivencia de los dos compiladores

`PARITY.md` lleva la cuenta a mano. Funciona pero es frágil. **Sería deseable un script automático** que escanee features del bootstrap y reporte qué está cubierto en gen-2 (por ejemplo, contando coincidencias de fixtures que pasan en ambos).

---

## 5. Stdlib

### 5.1 Cobertura

| Tier | Módulos | Estado |
|---|---|---|
| P0 | String (28), Array (31), Math (14) | ✅ maduro |
| P1 | File (11), Dir (7), Date (14), Regex (5), CSV (8) | ✅ correcto |
| P1.8 | DB SQLite (4) | ✅ funcional, falta MySQL/PG |
| P2 | Random (5), Crypto (4), Process (4) | ⚠️ mínimo |
| HTTP | Server, Routes, Request, Response | ✅ axum |
| Logging | Log.info/warn/error/debug | ✅ table rendering |
| Config | `.env` loader | ✅ |

### 5.2 Carencias notables de stdlib

- **JSON nativo.** Hoy va por interop Rust con `serde_json`. Debería ser stdlib P0.
- **HTTP cliente.** Hay servidor pero no cliente (`Http.get/post`). Es lo primero que pide cualquier usuario nuevo.
- **Crypto seria.** SHA256 + MD5 + base64 es trivial; no hay HMAC, AES, ECDSA, Argon2, password hashing.
- **Time/timezone.** `Date` no maneja zonas horarias.
- **Networking.** Sólo HTTP. No TCP/UDP raw, no WebSockets (ya hay axum, sería barato exponer).
- **Path manipulation.** `File`/`Dir` operan con strings; no hay tipo `Path` (join, parent, extension, normalize).
- **Env vars.** No hay `Env.get("HOME")` documentado.
- **Streams / iterators.** Lectura línea a línea de ficheros grandes.
- **Compression.** gzip/zip/tar.

### 5.3 lib/std en Liva

Sólo existe `validators.liva`. La idea de tener stdlib *en Liva* (no en Rust) está sin desarrollar. Esto es una oportunidad: los módulos puros (validators, formatters de string, parsing de fechas, etc.) podrían vivir en `lib/std/` y servir como prueba de que el lenguaje basta para sí mismo.

---

## 6. Herramientas y ecosistema

| Herramienta | Estado |
|---|---|
| Compilador (`build`/`run`) | ✅ |
| Type checker (`check`) | ✅ |
| Formatter (`fmt`) | ✅ bootstrap; ⏳ gen-2 |
| Linter (`lint`) W001-W004 | ✅ bootstrap; ⏳ gen-2 |
| Test runner (`test`) básico | ✅ bootstrap; ⏳ gen-2 |
| LSP (`lsp`) | ✅ bootstrap; ⏳ gen-2 |
| Init (`init`) | ✅ bootstrap; ⏳ gen-2 |
| Update (`update`) | ✅ bootstrap; ⏳ gen-2 |
| **Package manager** | ❌ no existe |
| **Package registry** | ❌ no existe |
| **REPL** | ❌ deferred |
| **Debugger / DAP** | ❌ no existe (se debugea el Rust generado) |
| **Documentation generator** | ❌ no existe (`livac doc`) |
| **Coverage tool** | ❌ no existe (sólo cargo-llvm-cov sobre el bootstrap) |
| VS Code extension | ✅ v0.14.0 |

**El gap más grande es package management.** Sin esto, no hay ecosistema posible más allá de `rust { use rust "crate" }`. Es lo que distingue un proyecto personal de un lenguaje viable. Convendría diseñarlo *antes* de v2.0 final, aunque no se implemente, para no cerrar puertas en el formato.

---

## 7. Documentación

`livac/docs/` tiene 27+ archivos, organizados en `getting-started/`, `language-reference/`, `compiler-internals/`, `guides/`, `lsp/`, `plans/`. Esto es **mejor que la media** de un lenguaje joven.

Carencias:
- Falta un sitio web público en línea con la documentación renderizada (`website/` está en Astro pero no se ve si está desplegado y al día).
- Falta un *Liva by Example* canónico (estilo Go by Example).
- No hay tutoriales largos (un proyecto end-to-end de >500 LOC explicado paso a paso).
- ERROR_CODES.md no enlaza desde los mensajes de error en línea (`livac` debería decir `see https://liva-lang.org/errors/E0904`).

---

## 8. Riesgos para promover el self-hosted y eliminar el bootstrap

### 8.1 Riesgos altos (bloqueantes)

1. **Pérdida del LSP.** Sin `livac lsp` en gen-2, el VS Code extension deja de funcionar. La extensión es la cara visible del proyecto.
2. **Pérdida de `fmt`/`lint`/`test`/`init`/`update`.** Toda la experiencia "moderna" de tooling desaparece.
3. **Pérdida de Tier 2 (error handling completo).** Programas fallibles no compilan en gen-2 sin migrar a `liva_rt::Error`.
4. **Pérdida de function types.** Cualquier código con callbacks no compila.
5. **Pérdida de HTTP server.** Sin routing/axum integration en gen-2, los ejemplos `http-api/`, `http-server/`, `http-crud/` rompen.
6. **Pérdida de tooling de release.** Bucket Scoop, Formula brew, .deb/.rpm/.vsix dependen de scripts que invocan al bootstrap.

### 8.2 Riesgos medios

7. **Snapshot tests (`insta`)** del bootstrap se pierden si no hay equivalente en gen-2.
8. **Property-based tests (`proptest`)** del bootstrap se pierden.
9. **No hay coverage tool** para gen-2; se entra a ciegas en regresiones.

### 8.3 Riesgos bajos pero a documentar

10. Determinismo `gen-2 ≡ gen-3` se mantiene **mientras el AST iter order sea estable**. Si en algún refactor se introduce un `HashMap` no ordenado en gen-2, la idempotencia se rompe silenciosamente. **Hay que añadir un test que falle si `gen-2 ≠ gen-3` en CI** (creo que ya existe, confirmar en `bootstrap_test.sh`).
11. Re-implementar tower-lsp en Liva no es trivial (requiere async + JSON-RPC + workspace tracking).

---

## 9. Plan recomendado de cutover

### v2.0.0 final — *no romper nada*

- Mantener bootstrap en repo (status FROZEN ya está bien).
- Shippeamos **dos binarios** en el tarball: `livac-stage0` (Rust) + `livac` (gen-2 self-host). El subcomando `build`/`run`/`check` usa gen-2; el resto usa stage0.
- Documentar que esto es **transitional**.
- Añadir gate de tiempo de compilación al CI.
- Revisar el bench Particle sim (0.44× sospechoso).

### v2.1 — *migrar tooling*

- ERR-UNIFY (Tier 2 completo).
- GAP-007 + B144/B145/B141/B142 (function types + map/set params + indexOf 2-arg + reduce closures + nested array typing).
- B152/B153 (auto-derive Display/Clone para genéricos).
- Implementar `livac lsp` en gen-2 (sin tower-lsp; con stdlib + axum o nuevo módulo `Lsp.*`).
- Implementar `livac fmt`, `livac lint`, `livac test`, `livac init` en gen-2.

### v2.2 — *retirar bootstrap*

- Eliminar `livac/src/*.rs` salvo `liva_rt.rs` (que es runtime, no compilador).
- Conservar el último binario stage-0 publicado en GitHub Releases para bootstrap reproducible.
- Documentar en README cómo reconstruir desde stage-0.

### v2.3+ — *llenar carencias del lenguaje*

- Function types + lambdas con captura (refinar lo de v2.1).
- Tuplas nativas.
- Destructuring en `let`.
- Operator overloading.
- `?` operator.
- Iterator protocol user-defined.
- Package manager + registry (diseñar primero, implementar después).

---

## 10. Diagnóstico final

| Aspecto | Nota | Comentario |
|---|---|---|
| Diseño del lenguaje | **8 / 10** | Coherente y moderno; faltan function types, tuplas, destructuring |
| Implementación bootstrap | **8.5 / 10** | Maduro, frozen, bien testeado; codegen.rs gigante |
| Implementación self-host | **7 / 10** | Idempotente pero incompleto; gaps de tooling y Tier 2 |
| Performance runtime | **9 / 10** | Excelente; revisar Particle sim |
| Performance compilador | **? / 10** | No medido — gap |
| Stdlib | **6.5 / 10** | Buena cobertura básica; faltan JSON, HTTP client, Path, time zones |
| Tooling | **7 / 10** | Completo en bootstrap; vacío en self-host; sin pkg manager |
| Documentación | **8 / 10** | Mejor que la media; falta sitio público y by-example |
| Tests | **8.5 / 10** | 700+ validaciones; falta cobertura instrumentada del self-host |
| Ecosistema | **3 / 10** | Sin paquetes, sin comunidad, sin showcase |
| **Listo para v2.0 GA** | **Sí, con bootstrap incluido** | No para retirar bootstrap todavía |
| **Listo para retirar bootstrap** | **No** | Hace falta v2.1 (Tier 2 + tooling en gen-2) |

---

## 11. Acciones concretas sugeridas (ordenadas por impacto/esfuerzo)

1. ⚡ **No eliminar `livac/src/` en v2.0.** Marcarlo `stage0/` y enviarlo junto al gen-2 en el tarball. Documentar la transición.
2. ⚡ **Añadir benchmark de tiempo de compilación** y gate en CI.
3. ⚡ **Investigar Particle sim 0.44×**. Si es real, documentar por qué; si es un fallo de la versión Rust del bench, corregirla — no queremos vender un dato dudoso.
4. 🔶 **Diseñar el package manager** ahora (no implementar): formato `liva.toml`, layout de cache, política de versionado, registry vs git URL. Decisión arquitectónica que cierra puertas si se posterga.
5. 🔶 **Añadir `Http.get/post` cliente** en stdlib P1.
6. 🔶 **Añadir `Json.parse/stringify`** en stdlib P0 (hoy va por interop, debería ser nativo).
7. 🔶 **Empezar Tier 2 (ERR-UNIFY)** — es el bloqueante #1 para retirar bootstrap.
8. 🔶 **GAP-007 (function types)** — bloqueante #2.
9. 🔷 **Refactor de `codegen.liva`** dividiéndolo en `codegen/expr.liva`, `codegen/stmt.liva`, `codegen/types.liva`, `codegen/class.liva`. 9 k LOC en un fichero es mantenimiento débil.
10. 🔷 **Añadir cobertura al self-host** (instrumentación o mutation testing manual).
11. 🔷 **`livac doc`** — generador de documentación a partir de comentarios `///`.
12. 🔷 **Sitio web público al día** con `docs/` renderizado.

---

## 12. Conclusión

Liva, hoy, es un proyecto **sorprendentemente sólido para su tamaño**. La calidad de ingeniería del bootstrap, la idempotencia del self-host, y los benchmarks pegados a Rust son logros poco comunes.

Pero **promocionar el self-host como compilador único y eliminar el bootstrap en v2.0 sería un error**: rompería LSP, formatter, linter, test runner, init, update, error handling completo y function types. La estrategia correcta es la habitual en compiladores serios (rustc, Go, Zig): **stage-0 conservado, stage-1 promocionado, retirada física aplazada hasta paridad real**.

El plan de v2.0/v2.1/v2.2 propuesto en §9 hace eso de forma ordenada y sin perder ninguna feature ya prometida.
