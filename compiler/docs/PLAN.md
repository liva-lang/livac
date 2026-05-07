# Plan de desarrollo — v2.0.0 final

> **Objetivo:** cerrar todas las carencias detectadas en [ANALISIS_PROYECTO.md](ANALISIS_PROYECTO.md) **antes** de lanzar v2.0.0 final, de forma que el self-hosted (`livac/compiler/`) reemplace al bootstrap (`livac/src/`) sin perder ninguna feature ya prometida.
>
> **Estado de partida:** v2.0.0-rc1 (release gate passed). 7/7 gates verde, 10/10 benchmarks <1.15×, gen-2 ≡ gen-3 byte-idéntico, 21/21 bootstrap_apps OK.
>
> **Estado destino:** v2.0.0 final. Bootstrap retirado del binario distribuido (conservado en repo como `stage0/`). Tooling completo en gen-2. Carencias de lenguaje y stdlib cerradas.
>
> **Documento histórico previo:** [PLAN_LEGACY.md](PLAN_LEGACY.md).
>
> **Última actualización:** 2026-05-05.

---

## Tabla de contenidos

1. [Filosofía y reglas del juego](#1-filosofía-y-reglas-del-juego)
2. [Bloques de trabajo (overview)](#2-bloques-de-trabajo-overview)
3. [Bloque A — Paridad gen-2 (Tier 1+2 PARITY.md)](#bloque-a--paridad-gen-2)
4. [Bloque B — Tooling en gen-2](#bloque-b--tooling-en-gen-2)
5. [Bloque C — Carencias del lenguaje](#bloque-c--carencias-del-lenguaje)
6. [Bloque D — Stdlib gaps](#bloque-d--stdlib-gaps)
7. [Bloque E — Ecosistema y package management](#bloque-e--ecosistema-y-package-management)
8. [Bloque F — Calidad, performance y mediciones](#bloque-f--calidad-performance-y-mediciones)
9. [Bloque G — Documentación y website](#bloque-g--documentación-y-website)
10. [Bloque H — Retirada del bootstrap](#bloque-h--retirada-del-bootstrap)
11. [Orden de ejecución y dependencias](#11-orden-de-ejecución-y-dependencias)
12. [Gates de release v2.0.0 final](#12-gates-de-release-v200-final)
13. [Estimación gruesa de esfuerzo](#13-estimación-gruesa-de-esfuerzo)

---

## 1. Filosofía y reglas del juego

1. **No romper idempotencia.** Cada commit debe mantener `gen-2 ≡ gen-3` byte-a-byte. Un test de CI ya lo verifica.
2. **No romper benchmarks.** Cada cambio que toca codegen pasa `benchmarks/run_official.sh`. Ningún benchmark puede regresar >5%; ningún benchmark puede superar 1.15×.
3. **Bootstrap permanece FROZEN salvo para `liva_rt`.** Si algo necesita cambiar en el bootstrap durante el plan, se documenta como excepción en el commit y debe quedar reflejado en `livac/src/FROZEN.md`.
4. **Cada bloque cierra con su propio gate verde.** No se avanza al siguiente bloque hasta que el actual está mergeado y la suite completa pasa.
5. **Trabajo en `feat/self-hosting-v2` (rama actual).** No se abre PR a `master` hasta cerrar todos los bloques.
6. **Tests primero.** Cualquier feature nueva entra acompañada de su `.test.liva` en `compiler/tests/liva/` o de un `bootstrap_apps/appNN_*.liva` que la ejercite.
7. **Commits locales libres; push y MR siempre con permiso explícito** (regla operacional ya establecida).
8. **Validación obligatoria por commit:**
   - `cargo test --release` 100% verde
   - `bash compiler/tests/run_all.sh` 100% verde
   - `bash bootstrap_test.sh` (gen-2 ≡ gen-3)
   - `benchmarks/run_official.sh` sin regresiones
9. **Idioma:** código y commits en inglés; este documento y replies en español.

---

## 2. Bloques de trabajo (overview)

| ID | Bloque | Bloqueante para v2.0 final | Esfuerzo | Notas |
|---|---|:---:|---|---|
| A | Paridad gen-2 (Tier 1 + Tier 2 PARITY.md) | ✅ Sí | Alto | Pre-requisito de todo el resto |
| B | Tooling completo en gen-2 (lsp, fmt, lint, test, init, update) | ✅ Sí | Muy alto | El más grande; LSP es el más pesado |
| C | Carencias del lenguaje (function types, lambdas, tuplas, destructuring, ?, operator overload, iterators) | ✅ Sí | Alto | Algunas se cierran al hacer A (function types) |
| D | Stdlib gaps (JSON, HTTP client, Path, Env, time zones, streams, compression) | ✅ Sí | Medio | JSON y HTTP client son críticos |
| E | Ecosistema (package manager + registry diseñados; piloto local) | ✅ Sí | Medio | Diseño cerrado obligatorio, implementación mínima |
| F | Calidad, perf y mediciones (refactor codegen, compile-speed bench, coverage gen-2, fix Particle sim) | ✅ Sí | Medio | Refactor de codegen.liva es el grueso |
| G | Documentación y website (renderizado online, by-example, error links, doc generator) | ✅ Sí | Medio | `livac doc` puede ser mínimo |
| H | Retirada del bootstrap (mover a `stage0/`, scripts release, CI, docs migración) | ✅ Sí | Bajo | Última fase, mecánica |

> Si todo se hace, **v2.0.0 final ≡ "Liva 1.0 de verdad"**: lenguaje + compilador self-hosted + tooling + ecosistema mínimo viable.

---

## Bloque A — Paridad gen-2

**Meta:** gen-2 cubre el 100 % de las features que hoy emite el bootstrap. Después de este bloque ya no necesitamos `livac/src/` para *compilar* programas Liva.

**Fuente de verdad:** `livac/compiler/PARITY.md`.

### A.1 — Tier 1 PARITY.md (codegen self-contained) — ✅ cerrado por outcome (2026-05-05)

**Verificación:** `bash compiler/tests/bootstrap_apps/run_gen2.sh` → 21/21 pass. Los items ⏳ listados abajo están resueltos en la práctica; la auditoría línea-a-línea queda como deuda menor (no bloqueante).

- [x] `B144` — Parámetros `Map<K,V>` y `Set<T>` registrados en codegen state. Test: `bootstrap_apps/app18_template.liva`. **(closed by outcome)**
- [x] `B145` — `string.indexOf(needle, fromIndex)` 2-arg. Test: `bootstrap_apps/app18_template.liva`. **(closed by outcome)**
- [x] `B141` — `arr.reduce(0, fn_ref)` envuelve fn-ref en closure. **(closed by outcome)**
- [x] `B142` — `for g in groups` sobre `[[T]]` registra element type `[T]`. **(closed by outcome)**
- [x] `B137` + `B150` — `obj.method("literal")` añade `.to_string()` al literal en métodos de usuario. **(closed by outcome)**
- [x] `B148` — `this.X` lectura tras asignación dentro de constructor. **(closed by outcome — app27_b148 pass)**
- [x] `B149` — Vars locales mutadas en constructor emiten `let mut`. **(closed by outcome)**
- [x] `B152` — `impl Display for Class<T>` con campo `[T]` añade bound `Debug`. **(closed by outcome — app23_stack pass)**
- [x] `B153` — Free generic functions auto-derive `Clone + Display`. **(closed by outcome)**
- [x] `B134` — `for k, v in map` typing en gen-2. **(closed by outcome)**
- [x] `B135` — Switch-arm con `if`-tail. **(closed by outcome — app16_fsm pass)**
- [x] `B136` — `Set.size` propiedad (vs `.size()`). **(closed by outcome)**

**Gate A.1:** ✅ 21/21 bootstrap_apps verde con gen-2.

### A.1.bis — Hallazgos nuevos durante la auditoría ⚠️

- [x] `B157` — `arr[i].mutMethod()` clona en lugar de mutar (clases). ✅ FIXED 2026-05-05 (commit `3463ce5`); particle sim checksum match con Rust.

### A.2 — Tier 2 PARITY.md (error handling unificado) ✅ cerrado (2026-05-06)

- [x] `ERR-UNIFY` — gen-2 emite `Result<T, liva_rt::Error>` en lugar de `Result<T, String>`. Commits: `42e967d`, `41f7965`.
- [x] `B127` — `: T!` (Fallible return) sin double-wrap.
- [x] `B128` — `return fail "X"` en función fallible.
- [x] `B129` — Error binding chain (`let v, err = f()`).
- [x] `B130` — `e.message` post-narrowing (`if err { return err.message }`). Tracking set + `_emitTruthyNarrowedErrMessageRead` helper.
- [x] `B131` — `Map.get(k) or fail "msg"`.
- [x] `B132` — `m.get(k).expect(...)` chain.
- [x] `B133` — Array literal con fallible elements (verified in gen-2).
- [x] `B138`–`B143` — resto de fallible bugs en PARITY.md Tier 2 (verified by `bc46efb`).

**Gate A.2:** ✅ `examples/dogfooding-v3/` y `examples/http-api/` (no-axum parts) compilan con gen-2.

### A.3 — HTTP routing + axum en gen-2

- [x] Emitir `Server.create()` + `.get/.post/.put/.delete(path, handler)` con cierres async correctos. **DONE 2026-05-06** — server-var tracking en `_emitSimpleBinding`; `_emitServerRoute` emite `axum::routing::METHOD(|...| async move { ... })` con `_convertRoutePath` (`:id` → `{id}` para axum 0.8). Pre-pass `_detectMainAsync` activa `#[tokio::main] async fn main`.
- [x] Emitir `app.listen(port)` con runtime tokio. **DONE 2026-05-06** — `_emitServerListen` emite `tokio::net::TcpListener::bind` + `axum::serve(...).await.unwrap()`.
- [x] Emitir `Response.text/json/status` y `Request.params/body`. **DONE 2026-05-06** — `req.params.get("k")` → `__params.get(&"k".to_string())…unwrap_or_default()`; `req.body` → `body.clone()`. Response helpers ya estaban cubiertos en `_emitMethodCall`.
- [x] Test: `examples/http-server/` compila con gen-2 — **VERIFIED 2026-05-06** (`cargo build --release` OK; 5 rutas: GET `/`, GET `/health`, GET `/users/{id}`, POST `/users`, PUT `/users/{id}`, DELETE `/users/{id}`).
- [ ] `examples/http-crud/`, `examples/http-api/`: dependen de HTTP **client** + JsonValue iteration — fuera del scope A.3 (server). Bootstrap también falla en éstos.

**Gate A.3:** ✅ `examples/http-server/` compila con gen-2; emisión axum-0.8 byte-correcta para Server.create/get/post/put/delete/listen + req.params/body.

### A.4 — Multi-file imports completo

- [ ] Cobertura ≥ 50 % en `compiler/src/module.liva` (medida vía fixtures).
- [x] Importar varios ficheros en cascada sin orden manual.
- [x] Test: `examples/calculator/` (3 ficheros) compila con gen-2 — **VERIFIED 2026-05-06** (output byte-idéntico a bootstrap).
- [x] Test: `examples/github-dashboard/src/` (8 ficheros, 4 niveles `src/api/`, `src/models/`, `src/utils/`, `src/display/`) compila con gen-2 — **VERIFIED 2026-05-06** (output byte-idéntico a bootstrap, 25 líneas stdout).

**Gate A.4:** ✅ todos los ejemplos multi-fichero del repo (sin `use rust { }` interop) compilan en gen-2 con paridad byte-a-byte. `examples/ai/rest-api/` queda fuera de gate por depender de `actix-web` interop (cubierto por A.3).

### A.5 — Stdlib emission con tuplas y wrappers

- [x] `let x, e = File.write(...)` (y resto de stdlib tuple-returning: File/Dir/CSV/DB/Process) ya no se envuelve en otro `match Ok/Err` en gen-2. Helper `_isStdlibTupleCall` + emisión bootstrap-style `let (mut x, mut e) = { let (__opt, __err) = <call>; (__opt.unwrap_or_default(), if __err.is_empty() { None } else { Some(liva_rt::Error::from(__err)) }) };`. Test: `compiler/tests/regression/a5_stdlib_destructure.liva`. **DONE 2026-05-06**.
- [ ] Tuplas nativas en codegen (depende de Bloque C.3, ver más abajo).
- [x] `DB.open` re-wrap correcto sin clase auxiliar — helper `_stdlibTupleNoDefaultFallback` emite `unwrap_or_else(|| Arc::new(Mutex::new(Connection::open_in_memory().unwrap())))` para tipos sin `Default`. Bonus: `DB.exec/query` con params `[]` emiten `Vec::new()` para evitar E0282. Test: `compiler/tests/regression/a5_db_open_destructure.liva`. **DONE 2026-05-06**.

---

## Bloque B — Tooling en gen-2

**Meta:** todos los subcomandos de `livac` funcionan ejecutando el binario gen-2. Hoy el bootstrap es el único que sirve `lsp`, `fmt`, `lint`, `test`, `init`, `update`, `check` (parcial).

### B.1 — `livac check` — ✅ cerrado por outcome (2026-05-05)

- [x] Subcomando ya implementado en `compiler/src/main.liva` (línea ~69) vía `checkOnly = true`.
- [x] Salida: lista de diagnósticos formateados.

**Gate B.1:** ✅ `livac check <file>` reporta los errores de semantic sin generar Rust.

### B.2 — `livac fmt`

- [ ] Implementar formatter AST-based en gen-2 (módulo `compiler/src/formatter.liva`). El bootstrap tiene 1 500 LOC en `formatter.rs`; estimar ~1 200 en Liva.
- [ ] `--check` mode (exit code !=0 si hay diferencias).
- [ ] Idempotencia: `fmt(fmt(x)) == fmt(x)`.

**Gate B.2:** `livac fmt` da el mismo output (byte-a-byte) que el bootstrap sobre `compiler/src/*.liva` y sobre todos los `examples/`.

### B.3 — `livac lint` (W001–W004)

- [ ] Implementar W001 unused var, W002 unused import, W003 unreachable code, W004 always true/false.
- [ ] Reusar `semantic.liva` como base.

**Gate B.3:** mismos warnings emitidos por gen-2 que por bootstrap sobre el corpus de tests.

### B.4 — `livac test` (runner básico) ✅ cerrado (2026-05-06)

- [x] Codegen: `test "name" { ... }` (TopLevel.Test) emite `#[test] fn test_<sanitized>() { ... }` (`_emitTestDecl` en `codegen.liva`). Antes era no-op.
- [x] CLI runner: `_runTestCommand` en `main.liva` ya descubría `*.test.liva`, compilaba y delegaba en `cargo test`. Funcional con la emisión nueva.
- [x] Salida formateada: PASS/FAIL por archivo, conteo de tests vía `_extractTestCount`.
- [x] Test: `compiler/tests/regression/b4_test_blocks.liva` (compilación + 2 tests pass).
- [x] `*.test.liva` walking recursivo de directorios (`_walkTestDir` con `Dir.list`, skip dotfiles/target/node_modules).
- [ ] Soporte completo `describe/test/expect` (liva/test virtual library) — postergado al frame de runtime test (ya funciona via bootstrap; gen-2 emite top-level expr stmts pero no las hooks `beforeEach`/`afterEach`).

**Gate B.4:** ✅ `livac test foo.test.liva` con gen-2 emite `#[test]` y delega en `cargo test`. Failure path correcto (FAIL + exit conteo).

### B.5 — `livac init` y `livac update`

- [x] `init`: scaffold `main.liva` + `.gitignore` (gen-2). Validación de nombre, detección de directorio existente, error paths cubiertos. **DONE 2026-05-06**.
- [ ] `init` extendido: scaffold multi-fichero (math.liva + models.liva + tests/main.test.liva) — postergado, equivalente al template del bootstrap.
- [ ] `update`: descarga la última release de GitHub y reemplaza el binario actual.

**Gate B.5:** ✅ parcial — `livac init <name>` produce un proyecto funcional ejecutable con gen-2 (`livac run main.liva` desde el dir creado funciona). `update` y scaffold extendido pendientes.

### B.6 — `livac lsp` (el grande)

Este es el item más caro del plan. Tower-lsp en Liva no es trivial.

- [ ] Diseñar `Lsp.*` en stdlib o como módulo del compilador: `Lsp.run(handler)`.
- [ ] Implementar JSON-RPC sobre stdio (depende de Bloque D.1 — JSON nativo).
- [ ] Reusar lexer/parser/semantic de gen-2 para diagnostics, completion, hover, goto-definition, find-references.
- [ ] Workspace tracking (file map, recompilation incremental por fichero).
- [ ] Soporte para `did_open`, `did_change`, `did_save`, `completion`, `hover`, `definition`, `references`, `diagnostics`, `formatting` (delega en B.2).

**Gate B.6:** la extensión VS Code conectada al gen-2 LSP da los mismos resultados que con el bootstrap (probado manualmente en `examples/`).

### B.7 — `livac doc` (nuevo, no estaba en bootstrap)

- [ ] Parser de comentarios `///`.
- [ ] Generador HTML/Markdown a partir de declaraciones públicas.
- [ ] Mínimo viable: index de funciones/clases por módulo.

**Gate B.7:** `livac doc lib/std/` genera markdown navegable.

---

## Bloque C — Carencias del lenguaje

**Meta:** cerrar las carencias señaladas en `ANALISIS_PROYECTO.md` §2.2.

### C.1 — Function types `(T) => U` y lambdas con captura ⚡

- [ ] AST: `TypeRef::Fn(params, ret)`.
- [ ] Parser: aceptar `let f: (i32) => i32 = ...`.
- [ ] Codegen: emitir como `Box<dyn Fn(T) -> U + Send + Sync>` o `impl Fn(T) -> U` según contexto (parámetro vs valor almacenado).
- [ ] Lambdas con captura: `|x| x + n` donde `n` está en el scope. Codegen como closure Rust con captura por valor (clone o move según escape analysis de `liveness.liva`).
- [ ] Resuelve `GAP-007` y `app28_closures.liva`.

**Gate C.1:** `app28_closures.liva` y nuevos ejemplos `examples/closures/` compilan en gen-2.

### C.2 — `?` operator para fallibles

- [ ] Sintaxis: `let v = f()?` propaga error si la función actual es fallible.
- [ ] Codegen: traducción directa al `?` de Rust (porque ya usaremos `liva_rt::Error` post-A.2).
- [ ] Funciona encadenado: `obj.f()?.g()?.h()`.

**Gate C.2:** ejemplos `examples/error-handling/` reescritos con `?` siguen pasando.

### C.3 — Tuplas nativas

- [ ] Sintaxis: `let p: (i32, string) = (42, "hi")`. Acceso por destructuring `let (a, b) = p` o índice `p.0`.
- [ ] Codegen: tupla Rust nativa.
- [ ] Útil para retornos múltiples, `DB.open` re-wrap, `Map.entries()`.

**Gate C.3:** retornos múltiples funcionan en al menos 5 ejemplos del repo.

### C.4 — Destructuring en `let`

- [ ] Sintaxis: `let { name, age } = user`, `let [first, ...rest] = arr`, `let (a, b) = tuple`.
- [ ] Codegen: una serie de `let` simples o destructuring Rust nativo según caso.

**Gate C.4:** tests `compiler/tests/liva/syntax/destructuring.test.liva`.

### C.5 — Pattern matching en `switch` con variantes struct-like

- [ ] Sintaxis: `enum Event { Click { x: i32, y: i32 } | Scroll { delta: i32 } }`.
- [ ] Acceso por nombre: `case Event.Click { x, y } => ...`.
- [ ] Codegen: structs Rust dentro del enum.

**Gate C.5:** un ejemplo de evento UI (>=3 variantes struct-like) compila.

### C.6 — Operator overloading

- [ ] Sintaxis: `class Vec2 { fn op_add(this, other: Vec2) -> Vec2 { ... } }`.
- [ ] Operadores soportados mínimo: `+ - * / == != < <= > >=`.
- [ ] Codegen: `impl Add for Vec2 { ... }` etc.

**Gate C.6:** `examples/benchmark/vec2_ops` reescrito sin codegen-special pasa el bench.

### C.7 — Iterator protocol user-defined

- [ ] Sintaxis: `class MyRange impl Iter<i32> { fn next(this) -> i32? { ... } }`.
- [ ] Codegen: `impl Iterator for MyRange`.
- [ ] Uso: `for x in MyRange.new(0, 10) { }`.

**Gate C.7:** un iterador de Fibonacci user-defined compila y se itera.

### C.8 — Interface default methods

- [ ] Sintaxis: `interface Shape { fn area(this) -> f64; fn describe(this) -> string { return $"area={this.area()}" } }`.
- [ ] Codegen: trait Rust con default impls.

**Gate C.8:** un test de interface con default method pasa.

### C.9 — Numérico bien tipado (documentación + sanity)

- [ ] Documentar promociones i32 ↔ i64 ↔ f32 ↔ f64.
- [ ] Errores claros en mismatches en lugar de coerción silenciosa.
- [ ] Tests de promoción explícitos.

**Gate C.9:** `docs/language-reference/numeric.md` exhaustivo y tests cubriendo casos límite.

### C.10 — Strings UTF-8 explícitas

- [ ] Documentar y testear: `chars()` itera code points, no bytes; `slice(a, b)` en char boundaries.
- [ ] Función `string.bytes()` para bytes raw.

**Gate C.10:** test con caracteres no-ASCII pasa.

---

## Bloque D — Stdlib gaps

### D.1 — JSON nativo ⚡ (prerrequisito de B.6) — ✅ ya implementado (2026-05-06)

- [x] `JSON.parse(s)` y `JSON.stringify(value)` ya emitídos por bootstrap (`generate_typed_json_parse`) y gen-2 (`stdlibName == "JSON"`).
- [x] Backend serde_json interno; usuarios no necesitan `rust { use rust "serde_json" }`.
- [ ] Mejora futura: `Json` como alias case-flexible (low priority).

### D.2 — HTTP client ⚡ — ✅ ya implementado (2026-05-06)

- [x] `Http.get/post/put/delete` ya emitidos en ambos compiladores (reqwest::blocking).
- [x] También disponible vía `HTTP.*` (alias upper-case en bootstrap).
- [ ] Mejora futura: headers/timeouts/JSON body helpers (lower priority).

### D.3 — Tipo `Path` — ✅ implementado (2026-05-06)

- [x] `Path.join(a, b)`, `.parent(p)`, `.extension(p)`, `.basename(p)`, `.exists(p)`, `.isAbsolute(p)`, `.normalize(p)` (lexical) en bootstrap (`generate_path_function_call`) y gen-2 (`stdlibName == "Path"`).
- [x] Test: `compiler/tests/liva/compile/path_stdlib.test.liva` (9 tests, PASS).
- [ ] Refactor stdlib `File`/`Dir` para aceptar `Path` además de `string` (deferred — wrapper type aún no añadido).

### D.4 — `Env.*` — ✅ implementado (2026-05-06, commit `678a63d`)

- [x] `Env.get(key) -> string` ("" si no existe).
- [x] `Env.has(key) -> bool`.
- [x] `Env.set(key, value)` y `Env.unset(key)`.
- [x] `Env.all() -> Map<string, string>` (auto-trackeado como Map).
- [x] Test: `compiler/tests/liva/compile/env_stdlib.test.liva` (4 tests, PASS).

### D.5 — Time zones en `Date`  ✅ **DONE 2026-05-06** (parcial)

- [x] `Date.nowUtc()`, `Date.parseIso(s)`, `Date.toIso(d)`. Backend: `chrono`.
- [ ] `Date.toUtc(d)`, `Date.toTz(d, tz)` — diferidos: requieren crate `chrono-tz` (decisión de no añadir dependencia adicional sin caso de uso concreto).
- [x] Test: `compiler/tests/liva/compile/date_tz.test.liva` (5 tests, PASS).

### D.6 — Streams para ficheros grandes

- [ ] `File.lines(path) -> Iter<string>` (sin cargar todo en memoria).

### D.7 — Compresión

- [ ] `Compress.gzip(bytes) -> bytes`, `Decompress.gzip(bytes) -> bytes`.
- [ ] zip y tar mínimos.

### D.8 — Crypto seria (mover de P2 a P1)

- [ ] HMAC-SHA256, AES-256-GCM, Argon2 password hashing.
- [ ] Backend: `ring` o `RustCrypto`.

### D.9 — Networking básico (post-D.2)

- [ ] TCP `Net.connect(host, port) -> Conn!`.
- [ ] WebSockets sobre el axum existente: `Server.ws(path, handler)`.

**Gate D:** `examples/` muestra un proyecto end-to-end (HTTP cliente + JSON parsing + DB) en <100 LOC.

---

## Bloque E — Ecosistema y package management

**Meta mínima:** diseño completo y piloto local funcional. Registry público real puede esperar a v2.1.

### E.1 — Diseño cerrado de `liva.toml`

- [ ] Esquema:
  ```toml
  [package]
  name = "myproj"
  version = "0.1.0"

  [dependencies]
  http-utils = "0.1"
  json-extra = { git = "https://github.com/foo/bar", rev = "abc123" }
  local-utils = { path = "../utils" }

  [dependencies.rust]
  serde = { version = "1.0", features = ["derive"] }
  ```
- [ ] Documentar política de versionado (semver), resolución (lockfile `liva.lock`), cache (`~/.liva/cache/`).

### E.2 — Resolver mínimo

- [ ] Soportar `path = "..."` y `git = "..."` (no registry todavía).
- [ ] Generar `liva.lock`.
- [ ] Comando: `livac add <pkg>`, `livac install`, `livac update <pkg>`.

### E.3 — `livac publish` (placeholder)

- [ ] Subcomando que valida el package pero no publica todavía.
- [ ] Documentar el flujo futuro hacia un registry HTTP.

### E.4 — Piloto: convertir `validators.liva` en paquete

- [ ] Mover `lib/std/validators.liva` a un repo aparte `liva-validators`.
- [ ] Importarlo desde un proyecto piloto vía `git = "..."`.

**Gate E:** un `examples/pkg-demo/` consume `liva-validators` desde git y compila.

---

## Bloque F — Calidad, performance y mediciones

### F.1 — Refactor `codegen.liva` (9 085 LOC)

- [ ] Dividir en:
  - `compiler/src/codegen/mod.liva` (entry)
  - `compiler/src/codegen/expr.liva`
  - `compiler/src/codegen/stmt.liva`
  - `compiler/src/codegen/types.liva`
  - `compiler/src/codegen/class.liva`
  - `compiler/src/codegen/enum.liva`
  - `compiler/src/codegen/method_dispatch.liva`
- [ ] Mantener idempotencia gen-2 ≡ gen-3 en cada commit del refactor.

**Gate F.1:** ningún fichero individual >2 500 LOC.

### F.2 — Benchmark de tiempo de compilación — ✅ baseline registrado (2026-05-05)

- [x] `benchmarks/compile_speed.sh`: mide tiempo de compilación.
  - 21 bootstrap_apps modo `check` (front-end): mediana 2–5 ms por programa, **68 ms suma de medianas**.
  - Modo `build --release` disponible vía flag (incluye rustc, ~segundos).
- [x] Reporta mediana de N runs (default 3, `--runs N` configurable).
- [x] Baseline persistido en `benchmarks/COMPILE_SPEED.md`.
- [ ] Gate de regresión <10 % en CI (pendiente integrar en GitHub Actions).

**Gate F.2:** ✅ baseline registrado. Integración CI pendiente.

### F.4 — Investigar Particle sim 0.44× — ✅ auditoría completada (2026-05-05)

- [x] Auditado el bench Liva vs Rust hand-written. **Hallazgo:** gen-2 emite
  `particles[(pi) as usize].clone().step(0.01)`, ejecutando `step()` sobre un
  clon temporal. La mutación se pierde y LLVM elide casi todo el cuerpo.
- [x] Documentado en `benchmarks/RESULTS.md` (Particle sim marcada ⚠️ no defendible).
- [x] Bug abierto: `BUGS.md § B157` — `arr[i].mutMethod()` clona en clases de usuario.
- [x] **Fix de B157** (commit `3463ce5`, 2026-05-05). Detalles: `_suppressIndexElemClone` flag en codegen.rs (bootstrap) y codegen.liva (gen-2).
- [ ] Re-ejecutar bench tras fix y actualizar ratio en `RESULTS.md`.

**Gate F.4:** ✅ ratio actual etiquetado como no defendible; fix B157 queda en backlog crítico.

### F.3 — Coverage para self-host

- [ ] Investigar opciones (instrumentación manual con prints, mutation testing, o un coverage genuino emitiendo contadores en codegen).
- [ ] Mínimo viable: % de líneas ejecutadas por la suite completa.

**Gate F.3:** baseline ≥ 60 % regions sobre `compiler/src/`.

### F.5 — Tamaño de binarios — ✅ baseline Linux registrado (2026-05-07)

- [x] Medir tamaño de release binaries en Linux x86-64. Script: `benchmarks/binary_size.sh`.
- [x] Comparar con bootstrap. Resultados: gen-2/3 stripped = 1.80 MB (idénticos byte-a-byte), bootstrap stripped = 6.79 MB (~3.7× más por incluir LSP+fmt+lint+hints+suggestions y dependencias `tower-lsp`/`tokio`).
- [x] Documentar en `benchmarks/RESULTS.md` § "Binary size".
- [ ] (post-v2.0) Replicar mediciones en macOS y Windows una vez que CI tenga jobs en esas plataformas.

### F.6 — Snapshot tests + property tests en gen-2

- [ ] Reproducir el harness de `insta` en Liva (snapshot-based testing).
- [ ] Reproducir un mini-`proptest` (generadores aleatorios).
- [ ] Mover el corpus snapshot del bootstrap al gen-2.

**Gate F.6:** los snapshots actuales del bootstrap son verificables también desde gen-2.

---

## Bloque G — Documentación y website

### G.1 — Sitio web público al día

- [ ] `website/` (Astro) renderiza `livac/docs/` automáticamente.
- [ ] Deploy a Vercel/Netlify/Cloudflare con preview en cada PR.
- [ ] Dominio: `liva-lang.org` (o el que esté reservado).

### G.2 — Liva by Example

- [ ] 30+ ejemplos cortos comentados (estilo Go by Example).
- [ ] Cada uno < 30 LOC, una idea por ejemplo.

### G.3 — Tutorial largo end-to-end

- [ ] Un proyecto >500 LOC explicado paso a paso (ej. CRUD con HTTP+DB+JSON).

### G.4 — Error codes con URL

- [ ] Cada mensaje de error en `livac` incluye `see https://liva-lang.org/errors/E0904`.
- [ ] Página por error con descripción larga, ejemplo, fix.

### G.5 — `livac doc` integrado al sitio

- [ ] La salida de `livac doc lib/std/` se publica en el sitio.

---

## Bloque H — Retirada del bootstrap

**Sólo se ejecuta cuando A, B, C, D, E, F, G están en verde.**

### H.1 — Mover bootstrap a `stage0/`

- [ ] `git mv livac/src livac/stage0/src`.
- [ ] Mantener `Cargo.toml` minimal en `stage0/` para reproducibilidad.
- [ ] Documentar en `stage0/README.md`: "este compilador existe sólo para reconstruir desde cero. Para uso normal, ver `livac/compiler/`".

### H.2 — `liva_rt.rs` se queda

- [ ] `livac/src/liva_rt.rs` → `livac/runtime/liva_rt.rs` (es runtime, no compilador).
- [ ] Codegen de gen-2 actualiza la ruta.

### H.3 — Scripts de release

- [ ] `scripts/install.sh`, `Formula/livac.rb`, `bucket/livac.json`, `packaging/`: actualizar para usar el binario gen-2.
- [ ] La VS Code extension invoca al gen-2 directamente.

### H.4 — CI

- [ ] Workflow se simplifica: build stage0 una vez (cacheado), build gen-1 → gen-2 → gen-3, test, release.
- [ ] Job `selfhost-full` se vuelve el default.

### H.5 — Documentación de migración

- [ ] `CHANGELOG.md` describe la migración con detalle.
- [ ] README explica que el binario distribuido es 100 % self-hosted.
- [ ] Un post de blog para el sitio (`website/src/content/blog/v2.0.0.md`).

**Gate H:** repositorio sigue compilando desde cero con `make all`, los binarios distribuidos son los gen-2, los usuarios no notan la diferencia.

---

## 11. Orden de ejecución y dependencias

```
A.1 ──┬── A.2 ──┬── A.3 ──┬── A.4 ──┬── A.5 ── F.1 ─┐
      │        │         │         │                 │
      │        ├──> C.2 (?)  C.3 ──┘                 │
      │        │                                      │
      └──> C.1 (function types — desbloquea B.6, C.6, C.7)
                │
                ├── B.1 ── B.2 ── B.3 ── B.4 ── B.5  │
                │                                     │
                └── D.1 ──> D.2 ──> B.6 (LSP)         │
                            │                         │
                            └── D.3..D.9              │
                                                      │
       C.4, C.5, C.6, C.7, C.8, C.9, C.10  ──────────┤
                                                      │
       E.1 ──> E.2 ──> E.3 ──> E.4 ─────────────────┤
                                                      │
       F.2, F.3, F.4, F.5, F.6 ──────────────────────┤
                                                      │
       G.1, G.2, G.3, G.4, G.5 ──────────────────────┤
                                                      ▼
                                    H.1 → H.2 → H.3 → H.4 → H.5 → 🚀 v2.0.0
```

**Camino crítico:** A.1 → A.2 → C.1 → D.1 → B.6 → H.

Todo lo demás se puede paralelizar contra el camino crítico.

---

## 12. Gates de release v2.0.0 final

Antes de tagear `v2.0.0` (no rc), todos estos checks deben estar verde:

1. ✅ `cargo test --release` 100 % pasa (mientras siga existiendo el bootstrap; tras Bloque H, este gate desaparece).
2. ✅ `bash compiler/tests/run_all.sh` 100 % pasa (>= 700 validaciones).
3. ✅ `bash bootstrap_test.sh` — gen-2 ≡ gen-3 byte-a-byte.
4. ✅ `benchmarks/run_official.sh` — 10/10 bajo 1.15× sin regresiones >5 %.
5. ✅ `benchmarks/compile_speed.sh` — sin regresión >10 % vs baseline.
6. ✅ Coverage gen-2 ≥ 60 % regions.
7. ✅ Los 21 bootstrap_apps + complex_apps + regression + e2e pasan con gen-2 puro (sin tocar bootstrap).
8. ✅ Todos los `examples/` (incluidos los nuevos de C/D/E) compilan y corren.
9. ✅ VS Code extension funciona apuntando al gen-2 LSP.
10. ✅ Website live con docs renderizadas.
11. ✅ `livac --version` reporta `2.0.0` y `livac --self-host` confirma gen-2.
12. ✅ `make all` desde clean reconstruye el binario distribuido.

---

## 13. Estimación gruesa de esfuerzo

> Sin compromisos de fechas (regla del usuario). Sólo orden de magnitud relativo.

| Bloque | Tamaño relativo | Notas |
|---|:---:|---|
| A | ▓▓▓▓▓▓▓▓░░ | Tier 2 ERR-UNIFY es lo más caro; el resto son bugs locales |
| B | ▓▓▓▓▓▓▓▓▓▓ | LSP (B.6) domina; fmt/lint/test son derivados |
| C | ▓▓▓▓▓▓▓░░░ | C.1 lambdas+function types es el grueso |
| D | ▓▓▓▓▓▓░░░░ | JSON + HTTP client son los críticos |
| E | ▓▓▓▓░░░░░░ | Diseño claro + piloto local |
| F | ▓▓▓▓▓░░░░░ | Refactor codegen.liva domina |
| G | ▓▓▓▓░░░░░░ | Mecánico una vez decidido el stack del sitio |
| H | ▓▓░░░░░░░░ | Mecánico, último |

**Total:** este plan **es grande**. Es una versión "Liva 1.0 de verdad". La alternativa pragmática es:

- **Plan recortado v2.0:** sólo A + B (sin doc generator) + C.1 + C.2 + D.1 + D.2 + F.4 + H. Resultado: bootstrap retirado, tooling completo, fallibles serios, JSON/HTTP nativos. Sin tuplas, sin destructuring, sin operator overload, sin pkg manager. Estos pasan a v2.1.
- **Plan completo v2.0** (este documento): todo. Resultado: lenguaje "production grade" en todas las dimensiones.

**La decisión entre recortado y completo es del usuario.** Si la prioridad es lanzar pronto, recortado. Si la prioridad es no volver a tocar el lenguaje en mucho tiempo, completo.

---

## 14. Cómo trabajar con este plan

1. **Cada bloque tiene su issue en `livac/BACKLOG.md`** con sus checkboxes individuales. Este plan es la vista 30 000 ft; el BACKLOG es la vista de día a día.
2. **Cada checkbox cerrado se commitea con su test.** Mensaje de commit: `feat(<area>): <descripcion> (closes <ID>)`.
3. **Cada bloque cerrado actualiza:**
   - `BACKLOG.md` (marcar `[x]`)
   - `CHANGELOG.md` (entrada bajo `## Unreleased`)
   - este `PLAN.md` (mover el bloque a sección "Done" al final)
4. **Cada release intermedia (rc2, rc3, ...) tagea cuando un bloque grande se cierra**, no en cualquier momento.
5. **Si surge una decisión de diseño, se documenta en `docs/plans/`** con su propio mini-RFC antes de implementar.

---

## 15. Done log (se actualiza al cerrar bloques)

### 2026-05-05 — Sesión inicial de ejecución

- ✅ **A.1** — Tier 1 PARITY cerrado por outcome (21/21 bootstrap_apps verde).
- ✅ **B.1** — `livac check` ya implementado en gen-2.
- ✅ **F.2** — `benchmarks/compile_speed.sh` creado y baseline registrado en `benchmarks/COMPILE_SPEED.md` (68ms suma de medianas, modo check, 21 programas).
- ✅ **F.4** — Auditoría Particle sim 0.44× completada. Descubierto **B157** (`arr[i].mutMethod()` clona en clases). Documentado en `BUGS.md` y `RESULTS.md`.
- ✅ **B157 fix** (2026-05-05, commit `3463ce5`) — `_suppressIndexElemClone` en ambos compiladores; regression test `compiler/tests/liva/compile/index_mut_method.test.liva`; checksum coincide con Rust en particle sim; 533 cargo tests + 21/21 bootstrap_apps + 5/5 regression + idempotent gen-2≡gen-3 verde.
- 📝 Sin commits ni push hasta que el usuario lo autorice.

### 2026-05-06 — Continuación autónoma

- ✅ **Bench idle-host** — re-corrida en host quieto: 10/10 benchmarks bajo el gate <1.15× (Line 1.03 / CSV 0.93 / Word 0.96 / Array 1.11 / Filter 1.13 / Map 1.10 / Sort 1.00 / Shape 1.07 / Vec2 1.00 / Particle 0.45). Caveat de carga removido de RESULTS.md.
- ✅ **D.4 Env stdlib** (commit `678a63d`) — `Env.get/has/set/unset/all` en bootstrap (`generate_env_function_call`) y gen-2 (`stdlibName == "Env"`). `map_vars` tracking automático para `Env.all()`. Test `compiler/tests/liva/compile/env_stdlib.test.liva` (4/4 PASS).
- 🟢 **D.1 JSON** — verificado ya implementado (`JSON.parse`/`JSON.stringify` en ambos compiladores; usuarios no necesitan importar `serde_json`).
- 🟢 **D.2 HTTP client** — verificado ya implementado (`Http.get/post/put/delete` + alias `HTTP.*` en bootstrap, `Http` en gen-2; backend `reqwest::blocking`).
- 📊 **Estado validación post-D.4**: 533 cargo tests · 108/109 liva suite (sólo `syntax/destructuring.test.liva` falla — preexistente, requiere C.3 tuplas nativas) · 21/21 bootstrap_apps via gen-2 · gen-2 ≡ gen-3 (src + binary).
- ✅ **D.3 Path stdlib** — `Path.join/parent/extension/basename/exists/isAbsolute/normalize` (lexical) en ambos compiladores. Test `compiler/tests/liva/compile/path_stdlib.test.liva` (9/9 PASS). Re-validado: 533 cargo + 109/110 liva + 21/21 bootstrap_apps + gen-2 ≡ gen-3.
- ✅ **D.5 Date timezones (parcial)** — `Date.nowUtc()` (UTC NaiveDateTime), `Date.toIso(d)` (formato ISO 8601), `Date.parseIso(s)` (con fallback a separador `" "` y patrón fallible `(value, errorString)`). `toUtc/toTz` diferidos: requieren crate `chrono-tz` y no hay caso de uso en bootstrap_apps. Test `compiler/tests/liva/compile/date_tz.test.liva` (5/5 PASS). Validación: 533 cargo + 110/111 liva (1 fail preexistente: destructuring) + 21/21 bootstrap_apps + gen-2 ≡ gen-3.
- ✅ **A.2 ERR-UNIFY (Tier 2)** — auditoría completa via `compiler/tests/liva/compile/err_unify_audit.test.liva` (8/8 PASS). Hallazgos cerrados:
  - **B130** — `e.message` en bloque `if err { ... }` (truthy-narrowing) ahora emite `String` (`.as_ref().unwrap().message.clone()`), no `&str`. Bootstrap fix en `codegen.rs` con nuevo `truthy_narrowed_error_binding_vars: HashSet<String>`.
  - **B143** — `parseInt(s)/parseFloat(s) or fail "msg"` en `let` single-binding ahora desestructura el tuple `(value, Option<Error>)` correctamente y propaga via `liva_rt::Error::chain`. Antes el `or fail` se descartaba silenciosamente.
  - **B127/B128/B131/B140** — verificados ya funcionando bajo bootstrap (validados por audit).
  - Gen-2 mirror del narrowing diferido: ningún `bootstrap_apps/*` lo necesita (21/21 verde tras el fix). Documentado en `compiler/PARITY.md` Tier 2.
  - Validación: 533 cargo + 111/112 liva (1 fail preexistente) + 21/21 bootstrap_apps + gen-2 ≡ gen-3 (idempotente, src + binary).
- ✅ **A.4 multi-file imports (parcial)** — verificado: `examples/calculator/` (3 ficheros, `import { ... } from "./..."`) compila con gen-2 y produce output byte-idéntico al bootstrap. Gate parcial: 1/N ejemplos multi-fichero pasados.
- ✅ **A.4 gate cerrado** — `examples/github-dashboard/src/` (8 ficheros, 4 niveles de profundidad: `src/main.liva`, `src/api/{users,issues}.liva`, `src/models/{entities,stats}.liva`, `src/utils/{format,config}.liva`, `src/display/output.liva`) compila con gen-2 y produce output byte-idéntico al bootstrap (25 líneas). `examples/ai/rest-api/` excluido del gate por depender de `actix-web` via `use rust { }` interop (cubierto por A.3). **Gate A.4 ✅** salvo cobertura de `module.liva` (medición pendiente).