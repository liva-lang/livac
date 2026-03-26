# 📋 Backlog — Production Readiness

> **Objetivo:** Llevar Liva a producción real  
> **Plan de diseño:** `docs/plans/PLAN_PRODUCTION_READINESS.md`  
> **Prioridad:** Orden por versión = orden de implementación  
> **Última actualización:** 2026-03-23

---

## v1.4 — Stdlib P0: String, Array, Math ✅

> **Foco:** Ampliar tipos existentes para que el lenguaje sea usable en el día a día.  
> **Estado:** ✅ Completado — 38 nuevos métodos/funciones, 19 snapshot tests, 341 tests totales  
> **Esfuerzo real:** ~6h

### String — ampliar `generate_string_method_call()` ✅

**Ya existían (pre-v1.4):** `contains`, `startsWith`, `endsWith`, `indexOf`, `trimStart`, `trimEnd`, `toUpperCase`, `toLowerCase`, `trim`, `split`, `replace`, `substring`, `charAt`

**Nuevos en v1.4 (15 métodos):**
- [x] `s.lastIndexOf(sub)` → `int`
- [x] `s.slice(start, end?)` → `string`
- [x] `s.padStart(len, char?)` → `string`
- [x] `s.padEnd(len, char?)` → `string`
- [x] `s.repeat(n)` → `string`
- [x] `s.replaceAll(old, new)` → `string`
- [x] `s.chars()` → `[string]`
- [x] `s.capitalize()` → `string`
- [x] `s.isBlank()` → `bool`
- [x] `s.isEmpty()` → `bool`
- [x] `s.reverse()` → `string`
- [x] `s.truncate(len)` → `string`
- [x] `s.countMatches(sub)` → `int`
- [x] `s.removePrefix(pre)` → `string`
- [x] `s.removeSuffix(suf)` → `string`

### Array — ampliar generación de métodos ✅

**Ya existían (pre-v1.4):** `map`, `filter`, `reduce`, `forEach`, `find`, `some`, `every`, `includes`, `indexOf`, `join`, `length`

**Nuevos en v1.4 (20 métodos):**
- [x] `arr.findIndex(fn)` → `int`
- [x] `arr.flat()` → `[T]`
- [x] `arr.flatMap(fn)` → `[T]`
- [x] `arr.slice(start, end?)` → `[T]`
- [x] `arr.sort()` → `[T]`
- [x] `arr.distinct()` → `[T]`
- [x] `arr.zip(other)` → `[(T, U)]`
- [x] `arr.take(n)` / `arr.drop(n)` → `[T]`
- [x] `arr.first()` / `arr.last()` → `T?`
- [x] `arr.isEmpty()` → `bool`
- [x] `arr.chunks(n)` → `[[T]]`
- [x] `arr.reversed()` → `[T]`
- [x] `arr.sum()` → `T` (arrays numéricos)
- [x] `arr.min()` / `arr.max()` → `T`
- [x] `arr.count(fn)` → `int`

**Completados en v2.0:**
- [x] `arr.sortBy(fn)` → `[T]`
- [x] `arr.groupBy(fn)` → `Map<K, [T]>`

### Math — ampliar `generate_math_function_call()` ✅

- [x] `Math.clamp(val, min, max)` → `number`
- [x] `Math.sign(val)` → `int` (-1, 0, 1)
- [x] `Math.log(x)` → `float` (logaritmo natural)

---

## v1.5 — Rust Interop + Logging + Config + `livac init` ✅

> **Foco:** `rust { }` desbloquea todo el ecosistema. Logging/Config/init hacen proyectos "reales".  
> **Esfuerzo estimado:** ~18h (12h rust interop + 2h×3 tooling)  
> **Impacto:** Force multiplier — de "usable" a "viable".  
> **Estado:** ✅ Completado — 387 tests totales

### `rust { }` interop — Ver plan §Línea 2 ✅

- [x] Parser: reconocer `rust { ... }` como expresión
- [x] Parser: reconocer `use rust "crate" version "x.y"` con features opcionales
- [x] Desugaring: registrar crates del `rust { }` y `use rust`
- [x] Codegen: emitir bloque Rust inline tal cual
- [x] Codegen: hoisting de `use` statements del bloque rust al top del archivo
- [x] Codegen: `generate_cargo_toml()` — inyectar crates de usuario con versión/features + internos
- [x] Protección: error E9002 si intenta override de versión de crate interno
- [x] Protección: features adicionales a crates internos permitidas (merge)
- [x] Formatter: soporte `rust { }` y `use rust` con version/features
- [x] Tests: bloque rust básico, nested braces, use hoisting, versión/features, E9002, desugar
- [x] Docs: documentar sintaxis y limitaciones

### Logging — módulo `Log` ✅

- [x] `Log.info(msg, ...context)` → stderr con timestamp + nivel + contexto
- [x] `Log.warn(msg, ...context)`
- [x] `Log.error(msg, ...context)`
- [x] `Log.debug(msg, ...context)` — solo con `--verbose`
- [x] `Log.setLevel(level)` — cambiar nivel en runtime
- [x] Variadic args — `Log.info("User", name, "logged in")` concatena con espacios
- [x] Table rendering — Map 4+ keys → Key/Value table (box-drawing Unicode)
- [x] Table rendering — Map ≤3 keys → inline `{k: v}`
- [x] Table rendering — Array<Map> → columnar table (console.table style)
- [x] JSON runtime tables — `JSON.parse()` results auto-detected → table/inline
- [x] Tests (14 snapshot tests)
- [x] Docs (`docs/language-reference/stdlib/logging.md`)

### CLI — Subcomandos ✅

- [x] Migrar de flags (`--run`, `--check`, `--fmt`, `--test`, `--lsp`, `--update`) a subcomandos (`build`, `run`, `check`, `fmt`, `test`, `lsp`, `update`)
- [x] Struct `CompileArgs` interna para `compile()`
- [x] Actualizar `run_format()` y `run_tests()` a parámetros directos
- [x] Actualizar LSP client en vscode-extension (`'--lsp'` → `'lsp'`)
- [x] Actualizar toda la documentación, ejemplos, scripts y CI

### Config / .env — módulo `Config`

- [x] `Config.load(path)` — parsear archivo `.env` (KEY=VALUE)
- [x] `Config.get(config, key)` → `string, error`
- [x] `Config.getInt(config, key)` → `int, error`
- [x] `Config.getBool(config, key)` → `bool, error`
- [x] `Config.getAll(config)` → `Map<string, string>`
- [x] Tests (7 Rust snapshot tests + 11 Liva tests)
- [x] Documentación (docs/language-reference/stdlib/config.md + QUICK_REFERENCE)

### `livac init` — scaffolding ✅

- [x] `livac init <name>` — crea directorio con main.liva + tests/ + .gitignore
- [x] `livac init <name> --template cli` — template CLI
- [x] `livac init <name> --template data` — template data processing
- [x] Tests (6 integration tests)

---

## v1.6 — Stdlib P1: File, Dir, Date, Regex, CSV/Table

> **Foco:** Módulos para scripts y procesamiento de datos.  
> **Esfuerzo estimado:** ~16h  
> **Impacto:** Scripts reales, posicionamiento vs Python.

### File — ampliar `generate_file_function_call()` ✅

- [x] `File.copy(src, dest)` → `error?`
- [x] `File.move(src, dest)` → `error?`
- [x] `File.size(path)` → `int, error`
- [x] `File.extension(path)` → `string`
- [x] `File.readLines(path)` → `[string], error`
- [x] `File.writeLines(path, lines)` → `error?`
- [x] Parser: permitir `move` como nombre de método

### Dir — ampliar `generate_dir_function_call()` ✅

- [x] `Dir.exists(path)` → `bool`
- [x] `Dir.create(path)` → `error?`
- [x] `Dir.delete(path)` → `error?`
- [x] `Dir.listRecursive(path)` → `[string], error`
- [x] `Dir.walk(path)` → `[string], error` (alias de listRecursive)
- [x] Tests (4 snapshot tests)
- [x] Docs (`docs/language-reference/file-io.md` actualizado)

### Date — tipo nuevo (first-class) ✅

**Tipo en compilador:**
- [x] Tipo `Date` en `ast.rs` → `chrono::NaiveDateTime`
- [x] `has_date` flag en `DesugarContext` + crate `chrono` auto-inyectado
- [x] `generate_date_function_call()` para constructores estáticos
- [x] `generate_date_method_call()` para métodos de instancia
- [x] Soporte en interpolación de strings (`$"{date}"` → `.format("%Y-%m-%dT%H:%M:%S")`)

**Constructores estáticos:**
- [x] `Date.now()` → `Date`
- [x] `Date.new(year, month, day)` → `Date` (también acepta 6 args: year, month, day, hour, minute, second)
- [x] `Date.parse(str, pattern)` → `Date, error`
- [x] `Date.timestamp()` → `int` (unix epoch ms)

**Propiedades:** `.year`, `.month`, `.day`, `.hour`, `.minute`, `.second`
- [x] Acceso a propiedades de instancia → `int`

**Métodos de instancia:**
- [x] `d.format(pattern)` → `string`
- [x] `d.add(n, unit)` → `Date`
- [x] `d.diff(other, unit)` → `int`
- [x] `d.toString()` → `string` (ISO 8601)

**Operadores:** `>`, `<`, `>=`, `<=`, `==`, `!=`
- [x] Comparación entre dos `Date` (nativo — `NaiveDateTime` implementa `PartialOrd`)

- [x] Tests (3 snapshot tests)
- [x] Docs (`docs/language-reference/stdlib/date.md`)

### Regex — módulo nuevo (crate `regex` auto-inyectado) ✅

- [x] `Regex.test(pattern, str)` → `bool`
- [x] `Regex.match(pattern, str)` → `string, error`
- [x] `Regex.findAll(pattern, str)` → `[string]`
- [x] `Regex.replace(pattern, str, replacement)` → `string`
- [x] `Regex.split(pattern, str)` → `[string]`
- [x] Crate `regex` auto-inyectado via `has_regex` flag
- [x] Parser: permitir `test` como nombre de método
- [x] Tests (2 snapshot tests)
- [x] Docs (`docs/language-reference/stdlib/regex.md`)

### CSV — módulo nuevo

- [x] `CSV.read(path)` → `[[string]], error`
- [x] `CSV.write(path, data)` → `bool, error`
- [x] `CSV.parse(str)` → `[[string]]`
- [x] `CSV.stringify(data)` → `string`
- [x] `CSV.readTable(path)` → `Table, error` (con headers)
- [x] `CSV.writeTable(path, table)` → `bool, error`
- [x] `CSV.headers(table)` → `[string]`
- [x] `CSV.column(table, colName)` → `[string]`
- [x] Table operations via standard array methods (`filter`, `sortBy`, `groupBy`)
- [x] Tests (2 snapshot tests)

---

## v1.7 — Stdlib P2: Random, Crypto, Process + HTTP Server

> **Foco:** Completar stdlib + poder servir HTTP.  
> **Esfuerzo estimado:** ~16h

### Random — módulo nuevo ✅

- [x] `Random.nextInt(min, max)` → `int`
- [x] `Random.nextFloat([min, max])` → `float`
- [x] `Random.choice(arr)` → `T`
- [x] `Random.shuffle(arr)` → `[T]`
- [x] `Random.uuid()` → `string`
- [x] Tests

### Crypto — módulo nuevo (crates `sha2`/`md-5`/`base64` auto-inyectados) ✅

- [x] `Crypto.sha256(data)` → `string`
- [x] `Crypto.md5(data)` → `string`
- [x] `Crypto.base64Encode(data)` → `string`
- [x] `Crypto.base64Decode(data)` → `string, error`
- [x] Tests

### Process — módulo nuevo ✅

- [x] `Process.exec(cmd)` → `string, error`
- [x] `Process.spawn(cmd)` → `int, error` (PID)
- [x] `Process.pid()` → `int`
- [x] `Process.exit(code)`
- [x] Tests

### HTTP Server — Ver plan §Línea 4 ✅

- [x] `Server.create()` — crear router (axum::Router::new())
- [x] `app.get(path, handler)`, `app.post(...)`, `app.put(...)`, `app.delete(...)` — route registration con axum
- [x] `app.listen(port)` — arrancar servidor (tokio::net::TcpListener + axum::serve)
- [x] `Request` type: `req.params.get("key")` path params, `req.body` body access
- [x] `Response` type: `Response.text(s)`, `Response.json(s)`, `Response.status(code)`
- [x] Codegen: genera código con axum (auto-injected `axum = "0.8"`, async main inference)
- [x] Tests (test_http_server_basic, test_http_server_routes, test_http_server_params)
- [x] Docs: `server.md`, `response.md`
- [x] Example: `examples/http-server/main.liva`

---

## v1.8 — DB + REPL + Linter

> **Foco:** Persistencia, developer experience, calidad de código.  
> **Esfuerzo estimado:** ~20h

### DB — módulo nuevo (crate `rusqlite` auto-inyectado)

- [x] `DB.open(path)` → `connection, error`
- [x] `DB.exec(db, sql, params?)` → `_, error`
- [x] `DB.query(db, sql, params?)` → `[Map<string, string>], error`
- [x] `DB.close(db)`
- [x] Tests (2 snapshot tests)
- [x] Docs: `db.md`
- [x] Example: `examples/db-demo/main.liva`

### REPL — `livac repl` ⏸️ APLAZADO

> **Nota:** Dejado fuera de v1.8 por ahora. Se retomará en una versión futura si hay demanda.

- [ ] Loop read-eval-print básico
- [ ] Mantener estado entre líneas (variables persisten)
- [ ] Mostrar resultado de expresiones
- [ ] Comandos `.help`, `.exit`, `.clear`
- [ ] Historial con readline
- [ ] Tests

### Linter / Warnings ✅

- [x] W001: Variable declarada pero no usada
- [x] W002: Import no usado
- [x] W003: Código inalcanzable después de `return`/`fail`/`break`/`continue`
- [x] W004: Comparación siempre true/false
- [x] Subcommand `livac lint <file>` con `--json`
- [x] Tests (24 tests)
- [x] Docs: `docs/language-reference/linter.md`

---

## v1.9 — Dogfooding: API REST real ✅

> **Foco:** Validación real construyendo un proyecto completo.  
> **Estado:** ✅ Completado — 7 bugs encontrados y corregidos, 482 tests totales  
> **Esfuerzo real:** ~8h

- [x] Definir proyecto de dogfooding (TODO API con DB + HTTP Server)
- [x] Implementar proyecto completo en Liva
  - [x] POST /tasks — crear tarea
  - [x] GET /tasks — listar tareas
  - [x] GET /tasks/:id — detalle
  - [x] PUT /tasks/:id — actualizar
  - [x] DELETE /tasks/:id — eliminar
  - [x] GET /health — health check
  - [x] SQLite como almacenamiento
- [x] Documentar bugs encontrados en BUGS.md (B83-B89)
- [x] Corregir todos los bugs (7/7)
- [x] Escribir regression tests (3 snapshots actualizados)
- [x] Post-mortem: qué falta, qué mejorar

---

## v2.0 — Enums recursivos + Self-hosting

> **Foco:** Desbloquear estructuras de datos tipo árbol y preparar auto-compilación.  
> **Cambio de lenguaje importante — justifica major version.**

### `defer` statement ✅

- [x] Lexer: nuevo token `Defer`
- [x] AST: `DeferStmt { body: Box<Stmt> }` + variante `Stmt::Defer`
- [x] Parser: `defer <expr>` y `defer { ... }` — dos formas
- [x] Desugaring: recursión en body para concurrency detection
- [x] IR: variante `ir::Stmt::Defer(Block)`
- [x] Lowering: `ast::Stmt::Defer` → `ir::Stmt::Defer`
- [x] Codegen: Rust `_DeferGuard` pattern con `Drop` trait (scope guard)
- [x] Formatter: soporte inline (`defer expr`) y block (`defer { ... }`)
- [x] Semantic: validación del body, propagación de async/fail/await
- [x] Linter: recursión en body para W001-W004
- [x] Tests: 6 tests (5 snapshot + 1 formatter)
- [x] Docs: QUICK_REFERENCE, CHANGELOG

### Enums recursivos (auto-boxing) ✅

- [x] Detectar campos recursivos en enums (`left: Expr` dentro de `enum Expr`)
- [x] Auto-generar `Box<T>` en codegen para campos recursivos
- [x] Soporte en arrays de tipos recursivos (`args: [Expr]`) — no necesita boxing (Vec ya provee indirección)
- [x] Tests (4 snapshot + 1 assertion)
- [x] Docs (QUICK_REFERENCE, CHANGELOG)

### Self-hosting (parcial) — experimento completado

- [x] Implementar lexer de Liva en Liva (~660 líneas)
- [x] Implementar parser (subset) en Liva (~948 líneas, self-referencial con 0 errores)
- [ ] Implementar codegen (subset) en Liva (pendiente — reiniciar tras fixes)
- [x] Comparar output con compilador Rust → verificar equivalencia
- [x] Documentar bugs encontrados (#90-#94) y arreglar en main (#90, #91, #92, #94 ✅)

### LANGUAGE_ISSUES fixes ✅

- [x] **C4**: Compound assignment `+=`, `-=`, `*=`, `/=`, `%=` — desugaring en parser, formatter round-trip, 7 tests
- [x] **A5**: Wildcard `_` en enum switch destructuring — parser + codegen + semantic, 3 tests
- [x] **C2**: `for i, item in array` (enumerate) — codegen detecta Map vs Array, 3 tests
- [x] **A4**: Suprimir warnings de imports no usados — `#[allow(unused_imports)]` en codegen

---

## v2.x — Ecosistema maduro (futuro)

> **Priorizar según demanda de usuarios.**

- [ ] `livac doc` — generación de documentación desde `///` comments
- [ ] `livac test --coverage` — cobertura de tests
- [ ] WebSockets — módulo `WS` (tokio-tungstenite)
- [ ] YAML/TOML parsing — módulos nuevos (crates serde_yaml/toml)
- [ ] `livac bench` — benchmarking built-in
- [ ] REPL — `livac repl` (aplazado desde v1.8)
- [ ] Lazy iterators — fusionar cadenas `filter().map().take()` sin `collect()` intermedios en codegen (optimización de rendimiento para arrays grandes)

---

## v3.x — Largo plazo (futuro lejano)

> **Solo si hay comunidad activa.**

- [ ] Package manager (`livac install`) — registry + resolución + lock files
- [ ] Debugging — breakpoints + DAP protocol para VS Code
- [ ] Profiler — instrumentación de funciones + report

---

## 📝 Notas

- Al completar una tarea, marcar `[x]` y actualizar ROADMAP.md + CHANGELOG.md
- Si un bug aparece durante implementación, añadirlo a BUGS.md
- Si una decisión de diseño cambia, actualizar `docs/plans/PLAN_PRODUCTION_READINESS.md`
- Cada versión se cierra con `git tag` + release en GitHub Actions
