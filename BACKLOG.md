# 📋 Backlog — Production Readiness

> **Source of truth for:** pending work, open tasks, deferred items.  
> **Companion docs:** `ROADMAP.md` (high-level vision + phases),
> `CHANGELOG.md` (released versions, Keep-a-Changelog format).  
> **Plan de diseño:** `docs/plans/PLAN_PRODUCTION_READINESS.md`  
> **Prioridad:** Orden por versión = orden de implementación  
> **Última actualización:** 2026-05-12
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

### Self-hosting — Fase 0: Bootstrap fixes ✅

> Arreglar el compilador Rust para que escribir Liva sea cómodo (prerequisito de Fase 2).

- [x] **FIX-5**: `#[derive(Copy)]` para enums unitarios — evita move errors en BinOp, Visibility, etc.
- [x] **FIX-6**: Borrar `IrCodeGenerator` dead code — eliminadas ~4.400 líneas (ir.rs, lowering.rs, IrCodeGenerator, codegen_ir_tests)
- [x] **FIX-1**: `let x: T? = value` → generar `Some(value)` automáticamente
- [x] **FIX-2**: Reassignment de enum sin `Some()` espurio (no reproduce)
- [x] **FIX-3**: `switch` genera `match &expr` si variable se usa después
- [x] **FIX-4**: Parámetros no-Copy: clone at call site

### Self-hosting — Fase 1: Frontend en Liva ✅

- [x] token.liva — 312 líneas, idiomatic
- [x] ast.liva — 450 líneas, idiomatic
- [x] lexer.liva — 610 líneas, idiomatic
- [x] parser.liva — 2254 líneas, idiomatic

### Self-hosting — Fase 2.1: Scope tracker ✅

- [x] semantic.liva — 647 líneas, compila a Rust sin errores
- [x] TypeContext, Scope, Symbol, FunctionSig, ClassInfo, EnumInfo, Diagnostic
- [x] SemanticAnalyzer: scope management, symbol table, registration + analysis passes
- [x] Factory functions (makeParamSig, makeFunctionSig, makeFieldInfo)
- [x] Bootstrap fix SH-011: Switch expression mutation scanner
- [x] Bootstrap fix SH-012: init_is_already_optional() para Expr::Member
- [x] Bootstrap fix SH-013: For-loop var_types tracking
- [x] 518 tests verdes

### Self-hosting — Fase 2.7: Liveness analysis ✅

- [x] liveness.liva — 519 líneas, nuevo módulo, compila a Rust sin errores
- [x] LivenessContext: useCounts + loopUseCounts + paramBorrow maps
- [x] LivenessAnalyzer: walks AST counting variable uses per function/method
- [x] Loop tracking: _inLoop flag for for/while — uses inside loops never eligible for move
- [x] Parameter borrow detection: Copy types owned, non-Copy borrow
- [x] Full Expr/Stmt coverage: 22 Expr variants, all Stmt variants, lambdas, switch arms
- [x] Public API: analyzeLiveness(program)
- [x] Removed examples/self-hosting/ legacy (canonical: compiler/)
- [x] 518 tests verdes
- [x] **Phase 2 COMPLETE**

### Self-hosting — Fase 3: Codegen Limpio ✅

- [x] codegen.liva — 2458 líneas, nuevo módulo, compila a Rust sin errores
- [x] RustEmitter class: output buffer, indent management, name sanitization
- [x] Type emission: all 9 TypeRef variants → Rust types
- [x] Declarations: functions, classes (struct+impl+constructor), enums, type aliases, imports
- [x] Statements: all 16 Stmt variants
- [x] Expressions: all 22+ Expr variants
- [x] Stdlib mapping: 78 methods (28 string + 30 array + 10 map + 10 set)
- [x] Ownership helpers: type-directed _emitRefArg
- [x] Cargo.toml generation with feature-aware dependencies
- [x] Public API: generateRust(program, typeCtx, liveCtx)
- [x] 520 tests verdes

### Self-hosting — Fase 4: Main + CLI + Bootstrap ✅

- [x] main.liva — 449 líneas, CLI entry point (build/run/check)
- [x] module.liva — 234 líneas, module resolver (BFS, topological sort)
- [x] bootstrap_test.sh — validation script
- [x] Full pipeline: read → lex → parse → semantic → liveness → codegen → write → cargo build
- [x] Bootstrap test: 7/9 modules → valid Rust (10,859 lines from 9,013 Liva)
- [x] Modules passing: token, ast, lexer, parser, semantic, liveness, module
- [x] **Phases 0-4 COMPLETE**

### Self-hosting — Fase 2.6: Import resolution ✅

- [x] semantic.liva — 1708 líneas (+62), compila a Rust sin errores
- [x] Import registration: _registerImport processes TopLevel.Import
- [x] Shallow type stubs for imported uppercase names
- [x] TypeContext: importedSymbols + importSources
- [x] Query methods: isImportedSymbol, getImportSource
- [x] ImportDecl added to imports
- [x] 518 tests verdes

### Self-hosting — Fase 2.5: Class/Enum metadata ✅

- [x] semantic.liva — 1646 líneas (+140), compila a Rust sin errores
- [x] Constructor validation: _validateStructLiteral + _countRequiredFields
- [x] Map method type table: 10 methods
- [x] Set method type table: 7 methods
- [x] Enum variant inference in _inferCallByName and _inferMemberOnSimple
- [x] Method dispatch on MapType and SetType
- [x] Metadata query API: 5 methods for codegen
- [x] ObjField import added
- [x] 518 tests verdes

### Self-hosting — Fase 2.4: Function signatures ✅

- [x] semantic.liva — 1506 líneas (+178), compila a Rust sin errores
- [x] Current function tracking: _currentFuncName + _currentFuncFallible
- [x] Param type storage: _storeParamType stores types in pool during analysis
- [x] Return type validation: _validateReturn warns on mismatch
- [x] Call argument count validation: _validateCallArgs + _countRequiredParams
- [x] Fallibility tracking: _trackCallFallibility + _checkCalleeFallible
- [x] Ownership workarounds: store-before-declare, string-compare patterns
- [x] 1 new workaround: W-006 (bare return after => not supported)
- [x] 518 tests verdes

### Self-hosting — Fase 2.3: Expr typing ✅

- [x] semantic.liva — 1328 líneas (+116), compila a Rust sin errores
- [x] Type index maps: _funcRetTypeIdx, _fieldTypeIdx, _methodRetTypeIdx
- [x] Second indexing pass: _indexTypeInfo populates maps after registration
- [x] Lookup methods: lookupFuncReturnType, _lookupMethodReturnType, _lookupFieldType
- [x] Expression analysis: _analyzeExpr exercises inferExprType during analysis
- [x] Statement analysis: Assign, Switch, ExprStmt, Return, Throw, Fail
- [x] Control flow analysis: _analyzeIf/_analyzeWhile condition analysis
- [x] Factory: _addTypeOpt(optRef: TypeRef?) for safe Optional→pool
- [x] TypeContext enriched: funcRetTypes, fieldTypes, methodRetTypes
- [x] 1 new workaround: W-005 (option_value_vars leak across methods)
- [x] 518 tests verdes

### Self-hosting — Fase 2.2: Type resolver ✅

- [x] semantic.liva — 1212 líneas (+564), compila a Rust sin errores
- [x] Type pool: _typePool + _varTypeIdx for resolved type storage
- [x] resolveTypeRef: recursive resolution of all 9 TypeRef variants
- [x] inferExprType: type inference for all Expr variants (literals, ops, calls, collections)
- [x] String/Array method type tables (15+15 methods)
- [x] For-loop iterable element type inference (_inferIterableElemType)
- [x] Type utilities: _typeToString, typesEqual, isUnknownType, _unwrapOptionalType
- [x] 4 new workarounds documented (W-001 through W-004)
- [x] 518 tests verdes

### Self-hosting (parcial) — experimento completado

- [x] Implementar lexer de Liva en Liva (~660 líneas)
- [x] Implementar parser (subset) en Liva (~948 líneas, self-referencial con 0 errores)
- [x] Implementar codegen completo en Liva (~7000 líneas, gen-2≡gen-3 idempotente)
- [x] Comparar output con compilador Rust → verificar equivalencia
- [x] Documentar bugs encontrados (#90-#94) y arreglar en main (#90, #91, #92, #94 ✅)

### LANGUAGE_ISSUES fixes ✅

- [x] **C4**: Compound assignment `+=`, `-=`, `*=`, `/=`, `%=` — desugaring en parser, formatter round-trip, 7 tests
- [x] **A5**: Wildcard `_` en enum switch destructuring — parser + codegen + semantic, 3 tests
- [x] **C2**: `for i, item in array` (enumerate) — codegen detecta Map vs Array, 3 tests
- [x] **A4**: Suprimir warnings de imports no usados — `#[allow(unused_imports)]` en codegen
- [x] **C7**: Imports sin extensión `.liva` — module.rs + semantic.rs fallback, 1 integration test
- [x] **C5**: String append `push_str` optimization — codegen detecta `x = x + expr`, 3 tests
- [x] **B4**: Enum exhaustive switch checking — semantic stores variant lists, E0904, 2 tests
- [x] **C1**: `parseInt(s) or 0` ya funciona con `or value` (B16 fix)
- [x] **B5**: Type alias ya implementado (lexer + parser + semantic + codegen)
- [x] **B6**: Switch guards ya implementados (parser + codegen + semantic)
- [x] **A7**: Closed (won't-fix — theoretical, no real failure)
- [x] **B3**: Closed (not an issue — enums already fully supported)
- [x] **A6/A8/C3**: Closed (deferred to C6 ref params)
- [x] **C6**: Closed (future enhancement, post-v2.0)
- [x] **B1/B2**: Closed (design decisions, deferred post-v2.0)

### Self-hosting — Fase 7: Self-Compilation ✅

- [x] **7.1: Gen-1 compila el compilador** — `livac-gen1 build compiler/src/main.liva` produce Rust válido (253→0 errors, commit `01eaea3`)
- [x] **7.2: Gen-2 idempotencia generacional** — gen-1 output == gen-2 output, 2000x perf fix (commit `4cbb30a`)

### Self-hosting — Fase 8: Calidad del Rust Generado ✅

- [x] **8.1-8.4**: Liveness clone elision, Copy-type elision, println! forwarding, push_str chains
- [x] **8.5**: `&str` params for private methods — 77 params, 56 `.into()` (commit `5fa154b`)
- [x] **8.6**: `for item in &vec` borrow iteration — 138→80 clone-iterations (commit `77a6f7a`)
- [x] **8.7**: Eliminate redundant `format!("{}", x)` — 77→1 (commit `89248bd`)
- [x] **8.8**: `self.field` clone suppression in comparisons — 89→78 (commit `2f11404`)
- [x] **8.9**: Liveness-based let-binding clone elision — 1100→996 (commit `d7189bf`)
- [x] **8.10**: Benchmark suite — 6/10 within <10% of hand-written Rust (commit `45cc67c`)
- [x] 518 tests verdes
- [x] Self-hosting idempotent (gen-1 == gen-2)

### Liva Test Suite — archivos .liva que validan el lenguaje

> **Foco:** Crear suite completa de tests escritos EN Liva que validen toda la sintaxis y features documentadas.
> **Directorio:** `compiler/tests/liva/` (se promueve a `tests/liva/` con el resto del compiler)
> **Runner:** `compiler/tests/liva/run_tests.sh` con filtros por capa

**Capa 1: Syntax (`compiler/tests/liva/syntax/`)** — `livac check`, sin compilar
- [x] variables.liva — let, const, type inference, top-level const
- [x] functions.liva — block, one-liner `=>`, typed params/returns, default params
- [x] classes.liva — constructor, methods, visibility, data classes
- [x] enums.liva — unit, tuple, struct variants, recursive (auto-boxing)
- [x] generics.liva — generic functions, classes, constraints
- [x] control_flow.liva — if/else, switch (statement + expression), for, while, break/continue
- [x] error_handling.liva — fallible `!`, `or value`, `or fail`, try/catch
- [x] pattern_matching.liva — switch patterns, destructuring, guards, wildcard `_`
- [x] imports.liva — use statements, extensionless, public/private
- [x] types.liva — type aliases, optional `T?`, tuples, union types
- [x] lambdas.liva — closures, point-free refs, method references `::`
- [x] string_templates.liva — `$"..."`  interpolation
- [x] defer.liva — defer statement, defer blocks
- [x] compound_assign.liva — `+=`, `-=`, `*=`, `/=`, `%=`
- [x] rust_interop.liva — `rust { }` blocks, `use rust`

**Capa 2: Compile (`compiler/tests/liva/compile/`)** — `livac build`, cargo check
- [x] basic_program.liva — hello world, variables, functions
- [x] class_program.liva — class con métodos, constructores
- [x] enum_program.liva — enums con switch exhaustivo
- [x] generic_program.liva — funciones y clases genéricas
- [x] error_program.liva — fallible functions, or value, try/catch
- [x] collections.liva — arrays, maps, sets, iteraciones
- [x] closures.liva — lambdas como parámetros, map/filter/reduce
- [x] pattern_matching.liva — switch patterns, destructuring
- [x] multifile/ — 7 assertions (imports from sibling modules: math_utils + string_utils)

**Capa 3: E2E Runtime (`compiler/tests/liva/e2e/`)** — build + run + compare OR livac test
- [x] hello.liva + hello.expected — pipeline completo mínimo
- [x] fibonacci.liva + fibonacci.expected — recursión, expresiones
- [x] calculator.liva + calculator.expected — clases, switch, métodos
- [x] basics.test.liva — variables, constants, string templates (9 assertions) ✅ B111 fixed
- [x] functions.test.liva — params, defaults, one-liners (7 assertions)
- [x] control_flow.test.liva — if/else, switch, for, while (12 assertions)
- [x] classes.test.liva — constructors, methods, data classes (8 assertions)
- [x] enums.test.liva — unit, tuple, struct, recursive (10 assertions)
- [x] errors.test.liva — or value, or fail, try/catch (7 assertions)
- [x] error_handling.test.liva — fallible flow (13 assertions) ✅ B101/B102 fixed
- [x] collections.test.liva — arrays, maps, sets (11 assertions)
- [x] compound_assign.test.liva — +=, -=, etc. (13 assertions) ✅ B109 fixed
- [x] generics.test.liva — generic functions (6 assertions) ✅ B103/B104 fixed
- [x] lambdas.test.liva — closures, map/filter (18 assertions) ✅ B105/B106/B107 fixed
- [x] for_patterns.test.liva — for i,v; for k,v (9 assertions)
- [x] pattern_matching.test.liva — switch, destructuring (9 assertions)
- [x] defer.test.liva — defer smoke test + assignment (3 assertions) ✅ B108 fixed
- [x] smoke.test.liva — minimal sanity (1 assertion)
- [x] async_basic.test.liva — 5 assertions (par map/filter/reduce, function ref, task async + await)
- [x] string_utils.test.liva — string processing intensive (14 assertions)

**Capa 4: Stdlib (`compiler/tests/liva/stdlib/`)** — livac test, métodos stdlib
- [x] string_methods.test.liva — 26 assertions (contains, replace, split, trim, case, etc.)
- [x] array_methods.test.liva — 27 assertions (push, pop, map, filter, sort, etc.)
- [x] map_methods.test.liva — 12 assertions (get, set, has, delete, keys, values, entries)
- [x] set_methods.test.liva — 10 assertions (add, has, delete, clear, iteration, union, intersection, difference) ✅ B110 fixed
- [x] math_functions.test.liva — 13 assertions (abs, floor, ceil, pow, sqrt, min, max, PI)
- [x] type_conversions.test.liva — 12 assertions (toString, toInt, toFloat, parseInt, parseFloat)
- [x] random_functions.test.liva — 7 assertions (nextInt range, nextFloat, choice, shuffle, uuid)
- [x] regex_functions.test.liva — 8 assertions (test, findAll, split, replace) ✅ B114 fixed
- [x] date_functions.test.liva — 9 assertions (new, format, timestamp, now, toString, add, diff) ✅ B114 fixed
- [x] csv_functions.test.liva — 5 assertions (parse, stringify, roundtrip)
- [x] config_functions.test.liva — 4 assertions (load, get, getInt, getBool)
- [x] process_functions.test.liva — 5 assertions (exec, pid, or default) ✅ B113 fixed
- [x] log_functions.test.liva — 5 assertions (info, warn, error, debug, multiple args — compile-only)
- [x] crypto_functions.test.liva — 8 assertions (sha256, md5, base64Encode, base64Decode, roundtrip)

**Capa 5: Stdlib-IO (`compiler/tests/liva/stdlib-io/`)** — opt-in, requiere filesystem/red
- [x] file_operations.test.liva — 10 assertions (read, write, append, exists, delete, copy, move, size, extension, readLines)
- [x] dir_operations.test.liva — 6 assertions (create, exists, isDir, list, delete)
- [x] db_sqlite.test.liva — 5 assertions (open :memory:, exec create/insert, query, empty query)
- [x] http_server.test.liva — 1 assertion (compile-only via livac check) + http_server_fixture.liva

**Capa 6: Errors (`compiler/tests/liva/errors/`)** — `livac check`, deben fallar con error esperado
- [x] e0001_duplicate_variable.liva — variable already defined (E0001)
- [x] e0310_duplicate_param.liva — duplicate function parameter (E0310)
- [x] e0701_unhandled_fallible.liva — fallible without error handling (E0701)
- [x] e0901_bool_exhaustiveness.liva — non-exhaustive bool switch (E0901)
- [x] e0902_int_exhaustiveness.liva — non-exhaustive int switch (E0902)
- [x] e0903_string_exhaustiveness.liva — non-exhaustive string switch (E0903)
- [x] e0904_enum_exhaustiveness.liva — non-exhaustive enum switch (E0904)
- [x] e1000_lexer_error.liva — unclosed string literal (E1000)
- [x] e2000_parse_error.liva — missing closing brace (E2000)
- [x] e4004_module_not_found.liva — module file not found (E4004)
- [x] w001_unused_var.liva — unused variable warning (W001)
- [x] w002_unused_import.liva — unused import warning (W002)
- [x] w003_unreachable_code.liva — unreachable after return (W003)

**Runner:**
- [x] `compiler/tests/liva/run_tests.sh` — test runner con 6 layers + filtros
  - `./run_tests.sh` — todo menos stdlib-io
  - `./run_tests.sh --all` — incluye stdlib-io
  - `./run_tests.sh --only syntax` — solo una capa
  - Exit code 0/1 para CI

---

## Fase 10 — Optimizaciones del Rust generado (prerrequisito de v2.0)

> **v2.0 NO sale hasta cerrar Fase 10 al menos en su Tier 1.**
> **Foco:** cerrar el gap medido en `benchmarks/RESULTS.md` con cambios deterministas que preservan idempotencia gen-2≡gen-3 binaria.
> **Plan detallado:** ver `compiler/docs/PLAN.md` § Fase 10.

### Tier 1 — bloquean v2.0

#### 10.1 — Last-use numbering en `liveness.liva` ✅ DONE

> Resuelto vía aproximación pragmática: `declaredInLoop` + flag `_stmtIsLastInBlock` en codegen. Bench: Word counting 2.11x → 1.79x.

- [x] Añadir `declaredInLoop: Map<string, number>` a `LivenessContext`
- [x] `_analyzeVarDecl` marca bindings dentro de `_inLoop`
- [x] Codegen flag `_stmtIsLastInBlock` seteado en `_emitBlock`
- [x] `_entryKeyEmit`: emite move si key es Identifier declaredInLoop Y stmt es last-in-block
- [x] Idempotencia gen-2≡gen-3 binaria
- [x] 518 tests Rust + bootstrap_test 9/9 verdes

#### 10.2 — Parameter escape analysis para mutadores ✅ DONE

> Resuelto extendiendo el check `isSingleUse` para considerar move-safe a vars con uc<=1 que están `declaredInLoop`. Bench: Filter+Map 1.50x→1.00x (tras 10.2 solo), Map lookup 1.36x→0.98x.

- [x] `_emitClonedArg`: `if uc <= 1 && (not inLoop || declaredInLoop)`
- [x] `_emitForIterable` Identifier branch: misma regla
- [x] Let-binding clone elision: misma regla
- [x] Idempotencia gen-2≡gen-3 binaria
- [x] 518 tests Rust + bootstrap_test 9/9 verdes

#### 10.3 — Iterator chain fusion ✅ DONE

> Resuelto con flag `_inIterChain` en codegen + detección recursiva en `_emitIterPrefix`. `arr.filter(p).map(f)` ahora emite una única tubería sin Vec intermedio.

- [x] Flag `_inIterChain: bool` en CodeGenerator
- [x] `_emitIterPrefix` detecta obj=MethodCall(map/filter/flatMap), emite obj con `_inIterChain=true` y omite `.iter()/.cloned()`
- [x] Ramas map/filter/flatMap omiten `.collect::<Vec<_>>()` cuando `_inIterChain`
- [x] Verificado: `arr.filter(x=>x>1).map(x=>x*2)` → `arr.iter().copied().filter(...).map(...).collect::<Vec<_>>()`
- [x] Idempotencia gen-2≡gen-3 binaria
- [x] 518 tests Rust + bootstrap_test 9/9 verdes

### Gate de release v2.0

- [x] Tier 1 completo (10.1 + 10.2 + 10.3)
- [x] Tier 2 parcial (10.4 implementado — Word counting 1.79x→1.23x, CSV 1.17x→1.00x, Map 1.14x→1.09x)
- [x] **v2.0 al 100% — completado:** los 5 bloques originales cerraron antes del tag rc1 (Bloque 1 Word counting 1.79x→0.98x, Bloque 2 Box<str> analizado y descartado por bench, Bloque 3 cobertura medida con `cargo-llvm-cov` 62.81% regions / 62.36% lines, Bloque 4 E2E self-host 5/5, Bloque 5 limpieza de BACKLOG/ROADMAP/CHANGELOG). Phase 12 (21-item audit) también cerrada. Ver banner del workspace y `livac/.github/copilot-instructions.md` para resumen.

---

## Self-hosting — Phase 9: Gen-2 Parity & Hardening (2026-04-30)

> **Objetivo:** llevar gen-2 (compilador self-hosted) a paridad funcional completa con selfhost_apps + medir calidad real (cobertura, clippy, examples).
> **Punto de partida:** 16/21 selfhost_apps; ERR-UNIFY no implementado; sin medición de cobertura del gen-2; sin run sistemático contra examples/.

### 9.1 — Gen-2 parity 21/21 ✅ DONE

- [x] **GAP-007 Function types** — `Box<dyn Fn(...) -> U>` para param types `(T1,T2) => U` (commit `e3e9978`)
- [x] **ERR-UNIFY core** — `Result<T, liva_rt::Error>` + or-fail Option/Result match + `liva_rt` inline minimalista (commit `487bcfd`)
- [x] **Tier 2 final** — app16_fsm (Default-derived enums, fallible-main `Ok(())`, no double Result wrap), app17_pipeline (reduce/fold point-free wrap, comma-sep err binding), app18_template (Map param tracking, indexOf with fromIdx) (commit `d9c5de4`)
- [x] **Display vs Debug** — `print(arr)` / `println(arr)` emite `{:?}` para Vec/HashMap/HashSet (commit `525f955`)
- [x] **Validación 4-gate verde:** `rebuild_selfhost` 4/4 idempotente · `selfhost_apps/run.sh` 21/21 · `selfhost_apps/run_gen2.sh` 21/21 · `regression` 5/5 · `complex_apps` 4/4 · `e2e_selfhost` 5/5

### 9.2 — Calidad medida (2026-04-30)

**Corpus completo `tests/liva/{compile,syntax,stdlib,e2e}` (106 archivos):**
- gen-2 `check`: **105/106 pass**
- 1 diferencia: `destructuring.test.liva` — gen-2 panics donde bootstrap retorna error E2000 limpio. Ambos rechazan, pero gen-2 lo hace mal (panic vs error estructurado). Bug menor de calidad de error, no de corrección. **Aplazado a v2.x** (requiere try/catch en Liva o panic_hook codegen-level).

**Tamaño Rust generado (21 selfhost_apps):**
- bootstrap: 9962 líneas totales
- gen-2: **2175 líneas totales (-78%)** — runtime mínimo `mod liva_rt { Error{message, cause} }` vs ~350 líneas inlineadas por programa en bootstrap.

**Clippy (21 selfhost_apps):**
- **0 errors** · 222 warnings totales (~10.6/app, todo estilístico: `unneeded return`, `.clone() on Copy`, missing `Default` impl)
- gen-2 emite código **más limpio que bootstrap** (app10_stats: gen-2 5 vs bootstrap 17 warnings).

**Rendimiento runtime** (mediana 7 corridas, μs, mismo programa Liva → bootstrap-Rust vs gen-2-Rust):
```
app10_stats     bs=785   g2=861   1.10x
app21_hashmap   bs=792   g2=699   0.88x
app25_parser    bs=866   g2=702   0.81x
app17_pipeline  bs=841   g2=763   0.91x
app19_pq        bs=678   g2=813   1.20x
```
Banda 0.81x–1.20x → **paridad efectiva** (algunas mejoras por menos imports/runtime más liviano).

**Cobertura del gen-2 (llvm-cov, 25 inputs: 21 selfhost_apps + 4 e2e_progs):**

| Archivo | Lines | Functions | Notas |
|---|---|---|---|
| `liveness.rs` | 76.67% | 87.50% | mejor |
| `token.rs` | 72.73% | 50.00% | |
| `lexer.rs` | 67.16% | 86.96% | |
| `ast.rs` | 66.24% | 61.76% | |
| `semantic.rs` | 62.39% | 72.03% | |
| `parser.rs` | 54.54% | 69.03% | |
| `codegen.rs` | 47.75% | 69.09% | mayor archivo, mayor gap (stdlib paths) |
| `main.rs` | 33.05% | 54.55% | CLI subcmds (`run`/`fmt`/`test`/`lsp`) sin tests |
| `module.rs` | **0.00%** | 0.00% | **multi-file imports nunca tocados por corpus** |
| **TOTAL** | **51.47%** | **68.19%** | |

### 9.3 — Examples corpus contra gen-2

Resultado de compilar+ejecutar 5 ejemplos deterministas (con `main()`) con bootstrap y gen-2 y diff stdout:

- ✅ `calculator.liva` — match 14 lines
- ✅ `test_b39.liva` — match (después de fix Display→Debug)
- ✅ `dogfooding-v1/main.liva` — re-validado **2026-05-08** end-to-end con gen-2: cargo build OK + run OK (Student Grade Tracker prints both report cards, Honor Roll/Passing classification, formatted names, letter-grade table). Multi-file re-declaración cerrada por `0d181d1` (m5_chain) + auditoría m6_diamond (`325a059`).
- ✅ `dogfooding-v3/main.liva` — re-validado **2026-05-11** end-to-end con gen-2: cargo build OK. BUG-3 (closures HTTP movían `Arc<Mutex<Connection>>` capturado + `vec![id]` movía String) **cerrado** — ver `BUGS.md` § BUG-3 ✅ FIXED. Test 9 añadido a `cli_subcmds` (build + grep `let db = db.clone();` ≥2 shims + reject `"id".to_string().to_string()` + cargo build OK).
- (bootstrap falla en `dogfooding-v2` por motivo no relacionado con gen-2)

### 9.4 — Pendientes hacia release sólido (post-9.x)

- [x] **Multi-file imports en gen-2** — auditado y cerrado **2026-05-08**. La queja original de 9.3 (re-declaración de constantes) ya estaba resuelta tras `0d181d1` (m5_chain). Auditoría con nuevo fixture `m6_diamond` (diamond import + `MAX_GRADE.toString()` dentro de template) destapó un bug colateral: cualquier identificador uppercase pasaba por la rama de "enum variant constructor" en `_emitMethodCall` y emitía `MAX_GRADE::toString(...)`. **Fix:** añadir guard `_globalEnums.has(name) or _enumVariantFields.has(variantKey)` antes de marcar `isEnumVariant = true`. Importado const + método ahora emite `MAX_GRADE.to_string()` correctamente. m6_diamond OK; gauntlet 8/8 verde.
- [x] **HTTP `serde_json::json!` macro** — cerrado **2026-05-08**. Bug: `Response.json({ "error": "..." })` (string-literal key) parseaba como `MapLiteral` y `_emitJsonMapEntries` emitía la key vía `_emitJsonArg → _emitExpr` que añadía `.to_string()`, generando `serde_json::json!({"error".to_string(): ...})` — rustc rechazaba el macro. **Fix** en `compiler/src/codegen.liva`: nuevo helper `_emitJsonKey` que detecta `Expr.Literal(Literal.Str(s))` y emite `"s"` bare; fallback a `_emitJsonArg` para keys no-literal. Test 7 añadido a `cli_subcmds` (build-only — no se ejecuta el server).
- [x] **`Option<liva_rt::Error>` en `+` concat** — cerrado **2026-05-08**. Bug colateral surfaced en multi-file work: `let v, err = call(); print("Failed: " + err)` — gen-2 wraps `err` como `Option<liva_rt::Error>` (para trace chaining), pero la rama binary `+` de codegen emitía `format!("{}{}", "Failed: ".to_string(), err)`. `Option<T>` no impl Display → rustc E0277. Bootstrap funciona porque allí `err: String`. **Fix** en `compiler/src/codegen.liva`: nuevo helper `_emitFormatArg(expr)` que detecta `Expr.Identifier` con nombre en `_errBindings` y emite `<err>.as_ref().map(|e| format!("{}", e)).unwrap_or_default()` (espejo del unwrap ya existente en `_emitStringTemplateInner`, BACKLOG 9.5 commit `1d24ede`). Aplicado en los 5 sites que emiten `format!("{}{}", L, R)` en la rama Add. Test 8 añadido a `cli_subcmds` (build + run + assert stdout contiene "Failed: …DB.open…").
- [x] **Multi-file tests** — añadir 2-3 programas multi-file a `selfhost_apps/` o `e2e_progs/` para que `module.rs` deje de estar al 0%. **DONE 2026-05-07** — multifile_apps tiene ahora 5 fixtures (m1_basic, m2_class, m3_stdlib, m4_enum cross-module enum payloads + switch, m5_chain transitive imports a→b→c). m5_chain destapó y bloqueó un bug en `main.liva` donde gen-2 no declaraba sub-módulos transitivos en `main.rs` (rustc E0432); fix landed in commit `0d181d1`.
- [x] **CLI subcmd tests** — `livac run`, `livac check`, `livac test`, `livac init` cubiertos en gen-2. **DONE 2026-05-08** — gate `cli_subcmds` añadido a `run_all.sh` con 6 sub-tests (run+stdout, check clean, check parser-error, test runner 2/2 pass, init scaffold + scaffolded build). `fmt`/`lint`/`lsp`/`update` quedan fuera (no implementados aún en gen-2 — ver `compiler/docs/PLAN.md` Bloque B).
- [x] **destructuring.test.liva** — convertir `throw` del parser a propagación Result o instalar `panic_hook` clean en `main.liva`. **DONE 2026-05-07** — instalado `std::panic::set_hook` con bloque `rust { }` al inicio de `main()` en `compiler/src/main.liva`. Ahora panics del parser/lexer (compiled from `throw`) emiten `Error: <msg>` y exit 1, en vez del backtrace `thread 'main' panicked at src/parser.rs:N:M:`. Mejora la paridad con bootstrap en errores de sintaxis.
- [ ] **`-D warnings` en gen-2 emit** — opcional: hacer que gen-2 emita `#![deny(...)]` selectivo si así lo quiere el usuario

### 9.5 — Polish landed during v2.0 stabilization (Phase 10 epilog)

Self-host codegen polish committed on `feat/self-hosting-v2` after the
v2.0 release-ready freeze. All five validation gates remain green
(rebuild_selfhost idempotente gen-2≡gen-3 src+bin, selfhost_apps
21/21, regression 5/5, complex_apps 4/4, e2e_selfhost 5/5,
cargo test --release 528+).

- [x] **Cross-module enum registry for Default-derive** (`1d24ede`) — when a class field's type is an enum declared in another module, suppress `#[derive(Default)]` (enums don't impl Default).
- [x] **Option<Error> template unwrap** (`1d24ede`) — `${err}` in string template auto-unwraps `Option<liva_rt::Error>` via `.as_ref().map(...).unwrap_or_default()`.
- [x] **`array.filter()` non-Copy lowering** (`8487bc7`) — emits `.iter().filter().cloned().collect()` for non-Copy element types (was producing `cannot move out of dereference` on String/struct arrays).
- [x] **`Math.min/max/clamp` no-cast emission** (`dc103a9` + revert) — emits native `.min()/.max()` without `as f64` coercion, preserving integer return types.
- [x] **Per-class transitive mut-self analysis** (`7695c26`) — replaces always-`&mut self` heuristic with bootstrap-parity fixpoint over (a) direct field assignments + setter heuristic + known-mutating method calls (push/pop/insert/remove/clear/sort/reverse/extend/retain/truncate/set/add/delete) on `this`/`this.field`, then (b) iterates: any method calling another mut-self method joins the set. Stored per-class in `_classMutMethods: Map<string, bool>`. Effect: dogfooding-v1 GradeBook emits `&self` for read-only methods (`display`, `getSummary`, `getPassing/FailingGrades`) and `&mut self` only for `addGrade`/`sort`. Compiles + runs end-to-end (only cosmetic diff vs bootstrap is the Error-trace box renderer in `liva_rt::Display`).
- [x] **Transitive Default-derive detection** (`590238e`) — `_buildNoDefaultClasses(program)` runs as program-wide pre-pass: seeds with classes containing direct enum fields, then fixpoints to mark any class whose field-graph reaches an enum. `_emitClassStruct` consults the precomputed set. Handles arbitrary-depth chains like `A { x: B }`, `B { x: SomeEnum }` — both correctly skip `Default` derive.

> **Status post-9.5:** v2.0 still RELEASE READY. Pendientes 9.4 (`HTTP routes`, `multi-file imports`, `module.rs coverage`, `CLI subcmd tests`) siguen abiertos como **post-v2.0** — no son bloqueantes para el release.

---

## 🏛️ Fase 11 — Hardening estructural pre-v2.0 (in progress)

> **Decisión 2026-04-30:** antes de etiquetar v2.0 vamos a saldar la deuda
> arquitectónica detectada en la auditoría general (compilador, stdlib,
> tests, examples, docs). Objetivo: que v2.x pueda crecer 3× sin
> volverse inmantenible. Ningún cambio toca semántica del lenguaje;
> todos preservan los 5 gates verdes.

### Tier A — Refactor crítico del compilador self-hosted

- [x] **A3.** Extraer snippets Rust embebidos a constantes top-level
      (`CSV_PARSE_LINE`, `DB_ROW_TO_MAP`, `DB_PARAM_BINDING(_TAIL)`,
      `CSV_ESCAPE_FIELD`). Eliminada la duplicación de `DB.query` y de
      las dos rutas de parse de CSV. Commit `654127f`.

- [x] **A0.** **Auto-`&mut`/`&` inference para colecciones** (Map/Vec/Set).
      Extiende el mecanismo existente de auto-`&str` (Phase 8.5) a tipos
      no-`Copy` de colecciones. Pre-requisito para A1/A2 y unblocker de
      Word Counting bench. Plan incremental:
      - [x] **Cycle 38** — `Map<K,V>` ReadOnly. Free functions y métodos
        privados cuyos params Map se detectan no-escapantes emiten
        `&HashMap<K,V>` en la firma; call-sites añaden `&`. Liveness
        refinada: receptor de método mutante (`push/set/insert/...`) ya
        no es marcado escape sino mutated (separación nueva). Commit
        `6e2ee0d` + auditoría ai/* 8/8 GREEN + gauntlet 7/7 GREEN.
      - [x] **Cycle 39** — `Map<K,V>` Mutated. Nuevo flag
        `paramMutated: Map<string, number>` en `LivenessContext`. Cuando
        un param Map es mutated pero NO escapa, la firma emite
        `&mut HashMap<K,V>` y los call-sites emiten `&mut arg`. Nuevo
        registro `_borrowedParamMutIndices` + helper `_emitMutBorrowedArg`
        + helper público `primeBorrowedParamMut`. Commit `09f8844` +
        gauntlet 7/7 GREEN + ai/* 9/9 GREEN. Spot-check `addItem(counts:
        Map<string, number>, key)` ahora emite
        `add_item(counts: &mut HashMap<String, i32>, key: &str)`.
      - [x] **Cycle 40** — `[T]` (Vec): ReadOnly + Mutated. Misma puerta.
        Refactor for-loop: `for x in &vec` ahora emite
        `for x in vec.iter()` para funcionar tanto con `Vec<T>` como con
        `&Vec<T>`. Phase 9.4 Copy-deref path también actualizado. Nuevo
        tracker `_collRefParams` que fuerza `.clone()` cuando un nombre
        marcado como borrow se usa en arg position (impide move de `&T`).
        Fix colateral: `_emitConstructor` ahora setea
        `_currentFunc = "{ClassName}.constructor"` antes de
        `_buildParamList` para que las escape lookups de liveness
        encuentren la key correcta. 48 `&Vec<...>` borrows aparecen en
        `codegen.rs` self-host generado (vs 0 antes). Commit `ad1ed52`.
      - [x] **Cycle 41** — `Set<T>` (HashSet). Integrado en el mismo commit
        que Cycle 40 (`ad1ed52`) — sólo añadir `startsWith("HashSet<")`
        + `TypeRef.SetType` al collector. Self-host no usa HashSet pero
        la inferencia está lista. Aplicación al self-host (eliminar
        `.clone()` defensivos en `compiler/src/*.liva`) **diferida** —
        ya no es necesaria: con borrow inference activa, no se generan
        clones extras a eliminar.
      - [ ] **Cycle 42** — desbloquea **A1** (modularizar codegen.liva) sin
        necesidad de `partial class`: free functions toman `e: RustEmitter`
        (instance) y los Maps internos siguen accesibles vía `e._field`.
      Acceptance: gauntlet 7/7 GREEN tras cada cycle, ai/* sigue limpio.

- [ ] **A1.** ~~Modularizar `compiler/src/codegen.liva` en 7 archivos.~~
      **Diferido a v2.1.** Requiere soporte del lenguaje para *partial
      classes* o *extension methods*. Liva actualmente exige que toda
      la clase `RustEmitter` viva en un único archivo (la sintaxis
      `RustEmitter { … }` declara la clase entera). Las alternativas
      (free functions + `EmitContext` struct pasado por referencia
      mutable) chocan con el known-issue de `Map<K,V>` que se mueve al
      pasar como parámetro (E0382 documentado en `conversation
      summary § 2`). Plan v2.1: añadir `partial` keyword o pivotar a
      arquitectura free-function una vez Liva soporte mut-borrow de Map.
      **Update 2026-05-13:** desbloqueado por A0 (Cycle 42). Se hará
      después de Cycles 38-41.

- [ ] **A2.** ~~Consolidar los 25+ `Map<string, …>` dispersos en
      `EmitContext`.~~ **Diferido a v2.1** por el mismo bloqueo que A1
      — un `EmitContext` requeriría pasarlo por mut-ref a docenas de
      free functions, que Liva aún no soporta sin clonar.

### Tier B — Higiene del repo

- [x] **B4.** Borrar `compiler/src/main.liva.bak`, mover o eliminar
      `compiler/test_concat.liva` y `compiler/test_suite.liva` (no son
      ejercitados por ningún gate). ✅
- [x] **B5.** Resincronizar `compiler/PARITY.md` con la realidad
      (baseline 21/21, items Tier 1+2+3 completados marcados ✅). ✅
- [x] **B6.** Unificar los 5 gates en `compiler/tests/run_all.sh` +
      target `make test-full` que los lance en orden. ✅
- [x] **B7.** Quitar la promesa "Jest-like" del README + QUICK_REFERENCE
      § 12 — alineada con realidad (`test_*` runner hoy, `liva/test`
      planificado v2.x). Implementación completa diferida a v2.x. ✅

### Tier C — Escalabilidad

- [x] **C8.** Scaffold `lib/std/` creado con primer módulo `.liva`
      reutilizable: `lib/std/validators.liva` (`isBlank`, `isNumeric`,
      `isEmail`, `isUrl`). README explica la diferencia entre stdlib
      Liva-side (`.liva` puro) y FFI stdlib (en compilador). Smoke
      test: `compiler/tests/multifile_apps/m3_stdlib/` lo importa
      y valida con gen-2. ✅
- [x] **C9.** Tests unitarios del codegen para los snippets extraídos
      en A3 — `tests/codegen_tests.rs` añade 3 invariant-tests
      (`test_csv_parse_line_invariants`, `test_db_param_binding_invariants`,
      `test_db_row_to_map_invariants`) que verifican estructura sin
      lock formatting; complementan los 340 snapshots existentes.
      Gen-2 idempotence (gen-2 ≡ gen-3) sigue locking el output
      self-host en `rebuild_selfhost.sh`. ✅
- [x] **C10.** Cubrir multi-file imports en gen-2 — nueva gate
      `compiler/tests/multifile_apps/run.sh` (2 fixtures: m1_basic =
      3 archivos con functions, m2_class = 2 archivos con clases).
      Confirmado que gen-2 resuelve `import { ... } from "./mod"`
      correctamente. CLI subcmds (`main.rs` coverage) diferido a v2.1
      junto con A1/A2. ✅

### Tier D — Nice to have

- [x] **D11.** Eliminar duplicación de `examples/ai/*/.copilot/skills/`
      → `scripts/hydrate-ai-skills.sh` reconstruye desde
      `skills/liva-lang/` (canónico) + `docs/` (references). Removidas
      216 entradas de `git ls-tree`, ~91 580 líneas. `.gitignore`
      añadido. ✅
- [x] **D12.** Phase 10 benchmarks (Line / CSV / Word / Map) verificados:
      ya viven en `benchmarks/liva/bench_strings.liva` (Line, CSV, Word)
      y `benchmarks/liva/bench_collections.liva` (Map), con sus pares
      Rust en `benchmarks/rust/`. `RESULTS.md` recoge la última corrida
      (2026-04-29) — 1.03x · 0.99x · 0.98x · 1.09x. Ningún archivo
      perdido; el item del backlog era impreciso. ✅
- [x] **D13.** Cabeceras de `BACKLOG.md`/`ROADMAP.md`/`CHANGELOG.md`
      reescritas con bloque "Source of truth for: …" + "Companion docs"
      explícitos. Cada documento ahora declara su propósito sin
      ambigüedad. (Refactor profundo de contenido aplazado a v2.1.) ✅

> **Gates de aceptación de Fase 11:** los 5 originales (rebuild_selfhost
> idempotente, selfhost_apps 21/21, regression 5/5, complex_apps 4/4,
> e2e_selfhost 5/5, cargo test 528+) **+** `compiler/tests/run_all.sh`
> verde en una sola invocación + `compiler/src/codegen.liva` ≤ 1 500 LOC.

---

## Post-v2.0-rc1 — Codegen bug-fix cycles (2026-05-08 → 2026-05-11)

> **Origen:** auditoría post-rc1 de ejemplos reales (http-api, http-crud, github-dashboard-real, crypto-tracker, dogfooding-v*, tests/*). Cada ciclo: bounded fix → rebuild self-host (gen1→gen2→gen3 idempotente) → 8-gate gauntlet GREEN → commit local.
> **Estado:** EN CURSO. 10 commits acumulados, gauntlet 8/8 verde, push pendiente de autorización.

### Cerrado ✅

- [x] **Cycle 6** (`1460e4d`) — `JsonValueExt` trait sobre `serde_json::Value` (as_int/as_float/as_string/as_bool/as_array_owned/length). Fixes http-api.
- [x] **Cycle 7** (`243540f`) — Rename `as_array` → `as_array_owned` para evitar shadow + rewrite en codegen. Fixes http-crud.
- [x] **Cycle 8** (`7bc1931`) — `_jsonValueVars` tracking + rewrite `obj.forEach(...)` → `obj.as_array_owned().into_iter().for_each(...)` cuando obj es JSON Value. Fixes github-dashboard-real.
- [x] **Cycle 9** (`de0c48f`) — JSON tracking propagación a través de `Expr.Index` / `Expr.MemberAccess` + `obj[strKey]` → `obj[&(k)[..]].clone()` + `!= null` sobre JSON → `!v.is_null()`. Fixes crypto-tracker.
- [x] **Cycle 10** (`6845489`) — `[a, b]` (array + array) en `_emitBinaryWithRight` → `[a, b].concat()`. Fixes dogfooding-v2.
- [x] **Cycle 11** (`4741438`) — print(literal) fast path: escape `\n`/`\t`/`\\` etc. ANTES de escape de llaves. Fixes parser-error en parallel-search.
- [x] **Cycle 12** (`5c1b78f`) — Auto `impl std::fmt::Display for <Enum>` delegando a Debug. Fixes enum_test.
- [x] **Cycle 13** (`5a238b7`) — `arr.sortBy(fn)` + `arr.groupBy(fn)` codegen (closure estable + `HashMap<K, Vec<V>>` aggregation). Fixes test_sort_group.
- [x] **Cycle 14** (`91367a1`) — `.length` sobre user class con campo `length` declarado emite field access (no `.len() as i32`). Fixes test_bug90_94.
- [x] **Cycle 15** (`2713e71`) — Paréntesis alrededor de lambdas inline en `findIndex`/`count`/`flatMap` + cast f64 explícito en `Math.clamp` args. Fixes test_stdlib_p0.
- [x] **Cycle 16** (`1b8db03`) — Async runtime: `task async f(args)` / `task par f(args)` → `tokio::spawn(async move { f(...) })`; `await taskHandle` → `.await.unwrap()` (JoinHandle yields `Result<T, JoinError>`). Recursive `_scanStmtForAsync`/`_exprIsAsyncTrigger` covers VarDecl/Assign/Return/Throw/While/For/TryCatch/Block + Unary(Await)/Binary/Call(Async|Par|Task*)/MethodCall(Server.listen)/Member/Index/Ternary/RustBlock(.await). Validated end-to-end with `examples/concurrency/main.liva`.
- [x] **Cycle 17** (`6a46098`) — `Stmt.Fail(identifier)` where identifier ∈ `_errBindings` (Option<liva_rt::Error>) → `Error::from(err.as_ref().unwrap().message.clone())` instead of `format!("{}", err)` (which doesn't impl Display). Reduces `ai/calculator` self-host errors 29 → 19.
- [x] **Cycle 18** (`457d76a`) — Stdlib: `Sys.input(prompt): string`, `Console.log/info/error/warn`, `Console.input/prompt` (stdin). Generates `print!()` + `stdout().flush()` + `stdin().lock().read_line()` + trim '\n'/'\r'. Probe `compiler/tests/probe/stdin_probe.liva` validated. Unblocks REPL-style examples (calculator, todo-list).
- [x] **Cycle 19** (`946f6aa`) — `Expr.RustBlock(code)` wraps emitted code in `{ ... }` so embedded `use` statements + trailing expression are valid in expression-position (e.g. inside `return`). Fixes `ai/web-scraper` `return rust { use std::time::...; expr }`.
- [x] **Cycle 20** (`b32edac`) — Non-main async inference for block-bodied fns: scanner snapshots `_mainNeedsAsync`, walks the fn body via `_scanStmtsForAsync`, marks `async` if `await`/server.listen/HTTP.* triggers found. One-liner inference NOT applied (bootstrap can't auto-clone non-Copy `Expr` params → E0382).
- [x] **Cycle 21** (`66bb1b3`) — Rewrite `examples/ai/csv-reader/main.liva` to valid Liva: if/else-if chain on menu option, `Console.error`, `Sys.input`, fallible `let content, readErr = File.read(...)`. Workarounds for self-host gaps: manual f64 count loop (no `as float` cast inside arithmetic) + `printEmployees(allEmployees) + parseCSV(content)` re-parse (bootstrap doesn't auto-clone `currentData = allEmployees` when LHS reused).
- [x] **Cycle 22** (`7e065a9`) — Self-host codegen fix: `JSON.stringify` was emitting bare `String` but `JSON` is registered in `_isStdlibTupleCall`, so destructuring wrapped it as if it were `(Option, String)`. Now emits `{ match serde_json::to_string(&x) { Ok(s) => (Some(s), String::new()), Err(e) => (None, ...) } }` to match `JSON.parse` tuple shape. + Rewrite `examples/ai/json-parser/main.liva` to focused builtin-JSON demo (preserve `*.bak` originals).
- [x] **Cycle 23** (`003c550`) — `liveness.liva` RustBlock arm of `_analyzeExpr` now marks ALL params of the current fn as escaping. Previously the analyzer saw zero non-escaping reads (opaque rust code) and emitted params as `&str`, but the embedded Rust typically expects owned `String` → E0308 on field inits like `str_value: value`. Drops `ai/mini-interpreter/fixed/main.liva` self-host errors 16 → 5.
- [x] **Cycle 24** — Promote `examples/ai/mini-interpreter/fixed/` to `examples/ai/mini-interpreter/` (replacing broken `src/`, preserved as `src_old_broken.bak`). Resolves Result-shape `or fail` for tokenize/evaluate, plain call for `parseTokens`. String concat in evaluator: `$"{ls}{rs}"` template instead of `+`.
- [x] **Cycle 25** (`fe3324b`) — Rewrite `examples/ai/text-search/search.liva`: wrap top-level body in `main()`, replace `.filter(closure)` (closure → `&String == String` mismatch) with manual loops, replace `.reversed()` (emits broken `.reverse()` returning `()`) with manual reverse loop.
- [x] **Cycle 26** (`c6c3d0e`) — `_buildParam` self-host now emits `mut <name>: <type>` for non-`&str` params so methods needing `&mut self` (e.g. `Vec::push`, `HashMap::insert`) don't fail with E0596 on the immutable param binding. + Rewrite `examples/ai/todo-list/{main,manager}.liva`: `task` reserved word → `newTask`/`t`, replace C-style `switch` with if/else-if chain, replace `console.*` with `Sys.input`/`println`.
- [x] **Cycle 27** (`bb2031a`) — Self-host now propagates `use rust "<crate>"` decls to Cargo.toml (`renderUseRustLine` + `collectUseRustLines` + `appendUseRustDeps` in `compiler/src/main.liva`, wired in both single- and multi-file paths). Bootstrap codegen.rs already did this; the self-host was discarding `UseRust` decls after parse, so ai/snake-game (crossterm) and ai/rest-api (actix-web) failed at cargo build with E0432/E0433. Unlocks 8/10 ai/* examples building clean (was 7/10).

### Pendiente — ciclos bounded (probablemente abordables)

- [x] **Cycle 28** — `switch` como sentencia (sin `let _ =` y sin `0` filler) — Hoy en self-host `switch` solo existía como expresión, así que para usarlo por sus efectos secundarios el código auto-hosted estaba plagado de `let _ = switch x { Arm => { ...; 0 }, _ => { 0 } }`. **Implementado**:
  1. `parser.liva` + `src/parser.rs`: en posición de statement, tras `switch x {` se hace peek: si el primer token interior es `case` o `default`, se mantiene la forma legacy; si es otra cosa, se parsean los arms (`pat => body`, body = expr | `{ stmts }` | simple statement) y se envuelve en `Stmt.ExprStmt(Expr.SwitchExpr(...))`.
  2. `codegen.liva`: nueva flag `_switchInStmtPosition`; en `Stmt.ExprStmt` se detecta `Expr.SwitchExpr` y se activa, escribiendo `;` final. En `_emitSwitchArm` los bodies se envuelven como `{ expr; }` (Expr arm) o `{ stmts;... }` (Block arm), lo cual fuerza tipo `()` uniforme y elimina la necesidad de `0` filler.
  3. `src/codegen.rs`: nuevo helper `generate_switch_stmt` con la misma semántica (emite `match { ... };` con arms `()`-typed).
  4. La forma de expresión (`let x = switch ... { ... }`) sigue funcionando idéntica.
  5. Docs actualizados: `docs/language-reference/syntax-overview.md` muestra la forma moderna; `docs/language-reference/control-flow.md` documenta ambas posiciones (statement / expression) y marca el `case/default:` como legacy.
  6. Validación: gauntlet 8/8 ✅ (rebuild_selfhost 70s, selfhost_apps 71s, multifile 77s, cli 27s, regression 291s, complex 108s, e2e 111s, cargo test 14s) + idempotencia gen-2 ≡ gen-3 (source + binary).
  7. **Pendiente como follow-up** (opcional): codemod sed-asistido sobre `compiler/src/*.liva` para colapsar `let _ = switch ... { ...; 0 }, _ => { 0 } }` → `switch ... { ... }, _ => {} }`. La feature ya está disponible; el codemod limpiará la deuda histórica.

- [x] **Async/await runtime** — implementado en Cycle 16. `examples/concurrency/main.liva` builds + runs end-to-end. (Antes bloqueaba 4 ejemplos.)
- [x] **`rust { use ... }` inside function body** — verificado 2026-05-12: web-scraper emite correctamente el bloque (`{ use std::time::...; <expr> }`), bug original ya resuelto. Marcado como done.
- [x] **Examples con parser errors** (sintaxis no-Liva, generada por IA) — verificado 2026-05-12 (Cycle 31 audit): `ai/csv-reader`, `ai/mini-interpreter`, `ai/text-search`, `ai/todo-list`, `ai/json-parser` ahora todos parsean limpios y compilan a Rust válido (audit con `gen-2` + `cargo build --release`). El compilador self-host evolucionó hasta cubrir las construcciones que antes fallaban; estos casos ya no son bloqueantes.
- [x] **Examples con errors estructurales (self-host codegen gaps)** — verificado 2026-05-12 (audit completo):
    - `ai/calculator` ✅ — cargo build --release clean (Cycles 17+18+29 lo arreglaron por completo).
    - `ai/snake-game` ✅ — cargo build --release clean (Cycle 27 propagó `use rust "crossterm"` a Cargo.toml).
    - `ai/json-parser` ✅ — cargo build --release clean.
    - `ai/web-scraper` — sigue 6 errores residuales (codegen `tokio::spawn(async move { fn() })` con `fn` async produce `JoinHandle<Future<...>>` en lugar de unwrap interno + `for t in &tasks` clona JoinHandle no-Clone). Fix requiere tracking de async-fn-callees en codegen + override del auto-clone pattern para `JoinHandle`. **Out-of-cycle**: documentado como deuda post-2.0.
    - `ai/rest-api` — sigue ~58 errores. Causa raíz: actix-web requiere `Serialize`/`Deserialize` derives en structs cliente, pero las clases self-host emiten `derive(Clone, Debug)` solamente. Requiere atributo `#[serde]` o anotación equivalente. **Out-of-cycle**.
- [x] **Cycle 29** — Arrow-method return type inference (self-host) — Métodos `_name() => expr` sin tipo de retorno explícito ahora infieren `-> i32` / `-> bool` / `-> String` desde Literal, Binary (Lt/Le/Gt/Ge/Eq/Ne/And/Or → bool), Unary, StringTemplate. Además, `_get() => this.field` consulta `_currentClassFieldRetSuffix` (Map<string, string> de field-name → " -> T") que `_emitClassImpl` construye desde las declaraciones de campo de la clase actual antes de emitir cada método. Helper `_inferArrowReturnType` extraído (también usado por `_emitFunction`). Cuando la inferencia no produce nada en `_emitMethod`, se mantiene `-> ()` (importante: métodos arrow side-effect-only como `_log() => print(x)` siguen funcionando). Gauntlet 8/8 verde + idempotencia gen-2 ≡ gen-3.

- [x] **Cycle 30** (`81d2285`) — Codemod 355 `let _ = switch X { ...; 0 }, _ => { 0 } }` → bare statement-position `switch` en los 6 archivos de `compiler/src/` (net **-391 LOC**). Counts: module 2, main 4, parser 5, liveness 14, semantic 22, codegen 310. Herramienta nueva: `scripts/codemod_switch.py` (brace-matching consciente de strings + `//` comentarios, iteración a fixpoint para patrones anidados). Además se arregló un bug latente en codegen self-host expuesto por el cleanup: `_emitExpr -> Expr.SwitchExpr` no salvaba/restauraba `_switchInStmtPosition`, así que un `switch` en posición de valor anidado dentro del body de un arm en posición de statement heredaba la flag y emitía `{ expr; }` (tipo `()`) en lugar de `{ expr }`, produciendo `error[E0308]` en Rust. Fix: save/restore + set `false` antes del `_emitSwitchExpr` anidado. Gauntlet 8/8 verde + idempotencia gen-2 ≡ gen-3.

- [x] **Cycle 31** (audit 2026-05-12) — AI examples audit con `gen-2` + `cargo build --release`. Resultado: **8/10 limpios** (calculator, chat-server, csv-reader, json-parser, mini-interpreter, snake-game, text-search, todo-list). Residuales: `web-scraper` (6 errors — async JoinHandle codegen) y `rest-api` (~58 errors — actix-web requiere Serialize derives). Ambos documentados como post-v2.0 con causa raíz identificada.

- [x] **Cycle 32** (`c097bfd`) — Auto-`.await` para spawned async user fns. Pre-pass `_collectAsyncFns` (nuevo) recorre `program.items` y registra cada `TopLevel.Function` cuyo cuerpo dispara async (`await`, `Server.listen`, `task async/par`) escaneando con `_scanStmtsForAsync`. Nuevo Map<string, bool> `_asyncFnNames` poblado por la pre-pass. En la emisión de `task async f(args)`, si `f` está en `_asyncFnNames` se emite `tokio::spawn(async move { f(args).await })` (antes: `tokio::spawn(async move { f(args) })`, que producía `JoinHandle<impl Future<...>>` con un Future anidado sin awaitar — error E0277 al await el handle). Gauntlet 8/8 verde + idempotencia gen-2 ≡ gen-3. Web-scraper sigue con errores residuales no-async (main no se promociona a `#[tokio::main]` cuando solo hay `task async` indirectos + `for t in &tasks` clona JoinHandle no-Clone) — documentados como follow-up.

- [x] **Cycle 33** (`3e24466`) — `_ => {}` arm ahora **opcional** en stmt-position switches. Bootstrap: `src/codegen.rs::generate_switch_stmt` sintetiza `_ => {},` cuando el usuario no escribe `Pattern::Wildcard` (mantiene match exhaustivo para Rust). `src/semantic.rs` añade flag `in_stmt_switch` (set por `Stmt::Expr`, consumido por `Expr::Switch`); cuando está activo se omite `check_switch_exhaustiveness`. Self-host: `_emitSwitchExpr` escanea arms por `Pattern.Wildcard` cuando `_switchInStmtPosition` true y emite `_ => {}, ` si no existe. Expression-position sigue exigiendo exhaustividad (E0904). Probes: `switch_no_default.liva` (ok) + `switch_expr_must_be_exhaustive.liva` (E0904). Gauntlet 8/8.

- [x] **Cycle 34** (`11fa24c`) — Codemod elimina 297 `_ => {}` redundantes en `compiler/src/*.liva` ahora que son opcionales. Tool: `scripts/codemod_strip_wildcard.py` (quita la línea wildcard + coma de cierre del arm anterior; conserva sole-arm). Counts: codegen 266, semantic 13, liveness 8, parser 4, main 4, module 2. 4 wildcards sole-arm conservados. Net -226 LOC. Gauntlet 8/8.

- [x] **Cycle 35** (`a9e5abf`) — Async transitivo end-to-end (cierra los 2 residuales no-async de web-scraper). Tres cambios en `compiler/src/codegen.liva`:
    1. **Fixpoint** de descubrimiento de async fns: el collector de Cycle 32 era single-pass, ahora itera hasta estabilizar `_asyncFnNames`. Se ejecuta **antes** de `_detectMainAsync` para que main se promocione cuando llama transitivamente a una async fn.
    2. **Auto-`.await`** en sitios de llamada. `Expr.Call(Expr.Identifier(fn), args)` añade `.await` cuando (a) `fn` está en `_asyncFnNames` y (b) estamos en contexto async (`_inAsyncContext()` consulta `_currentFunc` contra `_asyncFnNames`, o `main` contra `_mainNeedsAsync`). `_exprIsAsyncTrigger` también detecta estas llamadas para que main se promocione a `#[tokio::main] async fn main()`.
    3. **Iteración by-value** de JoinHandle. Nuevos helpers `_blockAwaitsIdent` / `_stmtAwaitsIdent` / `_exprAwaitsIdent` recorren el for-body buscando `Expr.Unary(UnOp.Await, Expr.Identifier(loopVar))`. Si lo encuentran, `_emitFor` consume el iterable by value (sin `&`, sin `let t = t.clone()` que rompía con JoinHandle no-Clone) y registra el loop var en `_taskHandleVars` para que el await emita `t.await.unwrap()` (JoinHandle::await yields Result). Gauntlet 8/8 (rebuild 96s, regression 318s, cargo test 13s) + gen-2 ≡ gen-3 + web-scraper genera Rust limpio que compila sin errores.

### Pendiente — out-of-scope estructural

- [ ] Test framework Jest-style completo: `async.test.liva`, `lifecycle.test.liva` (uso de `beforeEach` top-level), `math_jest.test.liva`, `stdlib_*.test.liva` (sin `main fn` — test runner debe ejecutar `test_*` funciones).

### Push pendiente

- [x] **209 commits locales** pusheados a `origin/feat/self-hosting-v2` (2026-05-12, autorización explícita del usuario).

---

## Fase 12 — Pre-tag v2.0 (21 ítems) — ✅ DONE (2026-05-04)

> **Origen:** auditoría externa `compiler/docs/SELF_HOSTED_V2_AUDIT_2026-05-04.md`
> + matices propios (BUG-1 exit code, REL-2 Cargo.lock).
> **Objetivo:** todo lo que el informe identifica como bloqueante o
> deuda barata se cierra ANTES del tag. Nada se posterga a post-2.0
> excepto items con rationale técnico explícito.
> **Última actualización:** 2026-05-04

### Grupo A — Release hygiene (versión y narrativa)

- [x] **REL-1.** Bump `Cargo.toml` 1.5.0 → 2.0.0-rc1.
- [x] **REL-2.** `Cargo.lock` sincronizado.
- [x] **REL-3.** `README.md` badge → "531 tests, 7 gates".
- [x] **REL-4.** `README.md` sección self-hosted con narrativa gen-2 + bootstrap congelado.
- [x] **REL-5.** `vscode-extension` mantiene 0.14.0 (compatible).

### Grupo B — CI / hermeticidad

- [x] **CI-1.** `.cargo/config.toml` con `NO_COLOR=1` + `CLICOLOR=0`.
- [x] **CI-2.** Job `selfhost-quick` para PRs en `.github/workflows/ci.yml`.
- [x] **CI-3.** Job nightly + manual `selfhost-full` con `run_all.sh` completo.
- [x] **CI-4.** 2 tests `#[ignore]` resueltos: `test_imports` (fixture migrada a sintaxis actual de imports), `test_length_misuse` (semantic phase ahora rechaza `.length` en identifier con tipo conocido distinto a array/string).

### Grupo C — Bugs funcionales reales

- [x] **BUG-1.** Fix Process.exec en `compiler/src/codegen.liva` línea 6014:
      ya no trata stderr no vacío como error; combina stdout+stderr
      como bootstrap (`src/codegen.rs` línea 15265). Verificado:
      `livac build` ahora reporta "Build successful" correctamente.
- [x] **BUG-2.** Causa raíz confirmada: binario global `~/.liva/bin/livac`
      era 1.5.0; reemplazado por 2.0.0-rc1 desde HEAD. LSP ahora corre
      gen-2 actualizado.
- [x] **BUG-3.** No necesario — BUG-2 resuelto vía reinstall.

### Grupo D — Documentación honesta

- [x] **DOC-1.** Disclaimer en `compiler/docs/PLAN.md`.
- [x] **DOC-2.** Disclaimer en `compiler/docs/ISSUES.md`.
- [x] **DOC-3.** Política LSP/v2.0 en `README.md` (sección self-hosted).
- [x] **DOC-4.** Sección "Gate oficial v2.0" en `benchmarks/RESULTS.md`.
- [x] **DOC-5.** Sort/Filter+Map/classes 0ms resueltos: benches reescritos con checksums laterales + workloads más grandes + input adversarial para Sort. Resultado: 10/10 benchmarks bajo 1.15x.

### Grupo E — Validación final + tag

- [x] **TAG-1.** `run_all.sh` 7/7 verde (rebuild 63s · bootstrap 5s · multifile 26s · regression 43s · complex 33s · e2e 71s · cargo 31s).
- [x] **TAG-2.** `run_official.sh` ejecutado; gate <1.15x confirmado (Line 1.08x · CSV 0.99x · Word 0.98x).
- [x] **TAG-3.** `hydrate-ai-skills.sh` ejecutado; 10 proyectos hidratados sin huérfanos.
- [x] **TAG-4.** Commit + tag `v2.0.0-rc1` (este commit).

> **Gates de aceptación de Fase 12 (= release gate v2.0):**
> 21/21 ítems cerrados; `run_all.sh` completo verde; `Cargo.toml` y
> `livac --version` reportan 2.0.0(-rc1); CI Linux protege gen-2 en PRs.

---



> **Objetivo:** cerrar v2.0 al 100% en compilación, tests, cobertura y bench.
> **Estado actual:** 518 cargo tests + 135 archivos `.liva` (e2e 61, errors 28, syntax 18, stdlib 19, compile 9) + bootstrap 9/9 + idempotencia gen-2≡gen-3. Bench 4/5 en gate; Word counting 1.23x.

### Bloque 1 — Cross-module `&str` (cierra Word counting <1.15x) ✅ DONE

> Causa raíz: `text.split(" ")` en Liva produce `[string]` (Vec<String>) por la signatura owned actual. Si `count_words(text)` aceptara `text: &str` y propagara `&str` al `for word in text.split(" ")`, eliminamos la alocación por palabra.
> Bloqueo histórico: cada módulo se compila con su propio `RustEmitter`; `_borrowedParamIndices` no se comparte.

- [x] Refactor `main.liva`: pre-pass que recolecta signaturas de todas las funciones libres ANTES de codegen-por-módulo
- [x] Pasar `globalBorrowRegistry: Map<string, bool>` (clave `funcSan:idx`) a cada `RustEmitter`
- [x] `_buildParam` y call-site usar el registry global cuando el callee es función libre cross-module
- [x] Verificar idempotencia gen-2≡gen-3 (binario+src) + 518 tests + bootstrap 9/9
- [x] Bench: Word counting 1.23x → 0.98x (✅ <1.15x — Liva más rápido que Rust)
- [x] Commit: `b6c4aa4`

### Bloque 2 — 10.5 Box<str> para Map<K, String> values ✅ ANALYSIS CLOSED (not shipped in v2.0)

> Análisis técnico realizado 2026-04-29. Conclusión: la optimización no aporta beneficio medible bajo el API actual y no hay hotpath en el bench que la justifique. **No se implementa en v2.0**.

**Hallazgos del análisis:**

1. **Bench Map (1.09x) usa `Map<string, number>`**, no `Map<K, String>`. La optimización no aplicaría a la métrica medida. El gap viene del overhead de `entry()` API + hashing, no de la memoria de los valores.

2. **El idiom Liva `m.get(k) or default` siempre clona.** Box<str>::clone() asigna nuevo slice (igual coste que String::clone()). Sin rediseño del API para devolver `&str` (incompatible con el lowering `or default` que necesita owned `String`), no hay ahorro de CPU.

3. **Beneficio teórico solo de memoria** (16B vs 24B por valor, ~33%). En el bench (1000 entries) la diferencia (8KB) cabe holgada en L2 cache, sin impacto de localidad observable.

4. **Coste de implementación:** `_localMapEscape` analysis en liveness.liva, dispatch en codegen.liva para insert/get/iter, manejo de tipos en pattern matching, tests de idempotencia. Riesgo no trivial de romper `gen-2 ≡ gen-3`.

**Decisión:** Cerrar Bloque 2 como analysis-only. Si en el futuro se identifica un hotpath con `Map<K, String>` (p.ej. config parsing, JSON loading) o se rediseña el API de `.get()` para devolver `&str`, reabrir como tarea v2.x.

- [x] Análisis técnico completo (este bloque)
- [x] Bench actual confirma 4/4 métricas <1.15x sin esta optimización
- [ ] (post-v2.0, condicional) Reabrir si nuevo hotpath con Map<K,String>

### Bloque 3 — Cobertura medida (cargo-llvm-cov) ✅ DONE (baseline)

- [x] Instalar `cargo-llvm-cov` (`cargo install cargo-llvm-cov --locked` + `rustup component add llvm-tools-preview`)
- [x] Generar reporte baseline: `make coverage` — **62.81% regions / 62.36% lines** (518 tests)
- [x] Identificar zonas <90% en `src/` — documentado en `docs/PROJECT_STRUCTURE.md`
- [x] `make coverage` y `make coverage-html` targets añadidos a `Makefile`
- [x] Documentar baseline en `docs/PROJECT_STRUCTURE.md` con tabla por módulo
- [ ] (post-v2.0, low-priority) Añadir tests para subir módulos core a ≥90%: `parser` 77→90%, `codegen` 67→90%, `semantic` 48→90%. Ámbito grande — trackeado para v2.x.

**Nota:** `liva_rt.rs` (0%), `main.rs` (19%) y `lsp/*` (0–59%) son intencionalmente bajos:
se cubren vía E2E (`compiler/tests/e2e_selfhost.sh`), test suite Liva (`compiler/tests/liva/`)
y tests LSP manuales — no representan gap real.

### Bloque 4 — E2E self-hosted bench ✅ DONE

- [x] Script `compiler/tests/e2e_selfhost.sh`: compila cada test con bootstrap **y** gen-2, ejecuta ambos binarios y compara stdout
- [x] Programs deterministas en `compiler/tests/e2e_progs/` (basics, enums_match, errors, stdlib) + ejemplo `calculator.liva`
- [x] Helper `compiler/tests/rebuild_selfhost.sh`: reconstruye gen-1→gen-2→gen-3 y verifica idempotencia (src+binario)
- [x] **Bug fix descubierto:** `Map.get(k) or default` self-host emitía pattern de tupla inválido — fix en `_emitOptionGetWithDefault`
- [x] **Bug fix descubierto:** `userFunc() or default` self-host emitía pattern de tupla pero las fns retornan `Result<T, Error>` — fix con switch en `isFreeCall`
- [x] 5/5 tests E2E PASS, idempotencia gen-2≡gen-3 preservada, 518 cargo tests, bench bajo gate
- [ ] (opcional) Integrar en `scripts/run_tests.sh` y CI

### Bloque 5 — Limpieza BACKLOG ✅ DONE

- [x] L478 (Implementar codegen self-host) → marcado completo (codegen.liva ~7000 líneas, idempotente)
- [x] L690-696 (validación Fase 10) → marcado completo
- [x] Bloque 2 (Box<str> Map values) → cerrado como analysis-only con rationale técnico documentado
- [x] REPL listado en v2.x section (post-v2.0) — ya estaba
- [x] Sincronizar `ROADMAP.md` con v2.0 final
- [x] Sincronizar `CHANGELOG.md` con v2.0 final

### Tier 2 — solo si Tier 1 no alcanza <1.15x

#### 10.4 — `&str` deref directo en Map APIs + sort/reverse in-place + split→for fusion

> Bench: Word counting 1.79x → 1.23x (-46% gap), CSV building 1.17x → 1.00x, Sort/Reverse statement-position elide `__v.clone()` wrapper.

- [x] `_emitMapKeyArg`: emitir `key.as_str()` cuando key es Identifier de tipo `String` (no `strRefParams`)
- [x] `_inExprStmt` flag: `arr.sort()` / `arr.reverse()` / `arr.reversed()` en posición de statement emiten directo (sin `{ let mut __v = obj.clone(); __v.sort(); __v }`)
- [x] `_canMoveIdent` helper + sort/reversed move-on-last-use cuando obj es Identifier single-use+declaredInLoop
- [x] Peephole `_emitBlock`: fusiona `let X = e.split(s); for Y in X { ... }` → `for Y in e.split(s).map(|s| s.to_string()) { ... }` (skip Vec<String>)
- [x] `_emitForIterable` MethodCall("split"): omite `.collect::<Vec<_>>()` para iteración lazy
- [x] `_emitBinary` push_str chain: omite `.to_string()` cuando RHS es ya un `String` (CSV building 1.17x → 1.00x)
- [x] Idempotencia gen-2≡gen-3 binaria + 518 tests + bootstrap 9/9

#### 10.5 — `Box<str>` para Map values nunca mutados ✅ ANALYSIS CLOSED (post-v2.0)

> Cerrado como Bloque 2 de "v2.0 al 100%" tras análisis técnico. **No se implementa en v2.0**. Ver § Bloque 2 arriba para rationale completo.

- [x] Análisis técnico realizado (no hay hotpath con `Map<K, String>` en bench actual; idiom `.get() or default` clona en cualquier caso → sin ahorro de CPU; ahorro de memoria 24B→16B no cambia bench)
- [ ] (post-v2.0) Reabrir si surge un hotpath con Map<K,String> o se rediseña `.get()` para devolver `&str`

### Validación obligatoria por cada item de Fase 10

- [x] `cargo test --release` 100% verde (518 tests)
- [x] `bootstrap_test.sh` 9/9
- [x] `compiler/tests/liva` sin regresiones (135 archivos: e2e 61, errors 28, syntax 18, stdlib 19, compile 9)
- [x] gen-2 source ≡ gen-3 source (`diff -r = 0`)
- [x] gen-2 release binary ≡ gen-3 release binary (`cmp = 0`)
- [x] `benchmarks/run_official.sh` mejora la métrica objetivo, ninguna otra regresa >5%
- [x] `benchmarks/RESULTS.md` actualizado y commiteado

---

## Post-v2.0 — Borrow-tracking IR completo (Tier 3, rediseño)

> **NO bloquea v2.0.** Solo si tras Fase 10 los datos justifican un rediseño mayor para acercar todos los benches a 1.05x. Estimación: 3–6 semanas.

- [ ] Nuevo IR `liva-AST → liva-IR` con anotaciones `Owned | Borrowed | MutBorrowed` por uso
- [ ] Pase de inferencia de borrow modes (combina liveness + mutabilidad efectiva + escape)
- [ ] Codegen `IR → Rust` que solo emita `.clone()` cuando dos usos `Owned` consumen la misma variable
- [ ] Migración incremental: feature flag `--ir`, comparar output con codegen actual hasta paridad
- [ ] Retirar codegen legacy

---

## v2.1 — Self-Hosted Migration (eliminar bootstrap Rust)

> **Objetivo:** Cerrar GAP-005 al completo. El compilador escrito en Liva (`livac/compiler/src/*.liva`) reemplaza al bootstrap Rust (`livac/src/*.rs`). Después de esto, sólo queda `liva_rt` como crate Rust.
> **Estado:** 🚧 EN CURSO desde 2026-04-30.
> **Razón:** Cada feature añadida al bootstrap sin portar agranda GAP-005. Para v2.0 self-host real hay que congelar bootstrap, portar y rediseñar gen-2.

### Fase A — Spec freeze (HACER YA) ⚡
- [x] Marcar bootstrap Rust como CONGELADO post-`ba7f263` (GAP-007).
- [x] No se ampliará el lenguaje en `livac/src/*.rs` hasta que gen-2 alcance paridad.
- [x] Actualizar BACKLOG y ROADMAP con la decisión.

### Fase B — Inventario de paridad
- [ ] Listar cada feature/bug del bootstrap NO portado a gen-2.
- [ ] Tabla en `compiler/PARITY.md`: ID, descripción, archivo origen (`.rs`), archivo destino (`.liva`), test que lo cubre.
- [ ] Priorizar por: bloqueante → frecuencia de uso → simpleza.

### Fase C — Rediseño gen-2 (escalable y mantenible)
> `codegen.liva` tiene 7463 líneas — está convirtiéndose en monolito.
- [ ] Dividir `codegen.liva` en módulos:
  - `codegen/expr.liva` — expresiones
  - `codegen/stmt.liva` — statements
  - `codegen/types.liva` — TypeRef → Rust type
  - `codegen/class.liva` — impls, Display, Debug
  - `codegen/method.liva` — method dispatch (Array/Map/Set/String/User)
  - `codegen/runtime.liva` — literales, strings, collections
  - `codegen/error.liva` — fail / Result / Error::chain
- [ ] Introducir abstracción `Emitter` (push, pushIndent, scope) para reemplazar la concatenación manual de strings.
- [ ] `TypeContext` centralizado (un solo struct con var_types, map_vars, array_vars, etc.) en lugar de HashMaps dispersos.
- [ ] Tests unitarios por módulo en `compiler/tests/codegen_modules/`.

### Fase D — Portar fixes (orden recomendado, fáciles primero)
- [x] **B151** — string escape `\"` dentro de `${...}` (gen-2 parser ya maneja `\"`, `\\`, `\n`, `\r`, `\t` en placeholder; verificado 2026-05-07 con `print($"a:{m.get(\"apple\")}")` → `a:1`)
- [x] **B152** — `Display` impl con `{:?}` añade `Debug` bound. **DONE 2026-05-07** — añadido `_emitClassDisplay` en gen-2 que auto-emite `impl Display for ClassName` para toda clase con campos (mirroring bootstrap BUG-004), usando `{:?}` para Vec/Map/Set/Optional/Tuple/enum. Type params reciben `Display` bound (y `Debug` ya estaba). También fix collateral en bootstrap: `_emit_display_for_class` emitía `}}}}` (doble cierre literal) en lugar de `}}`. Test: `compiler/tests/regression/b152_class_display.liva` (Point/Bag/Dict).
- [x] **B153** — free generic functions auto `Clone + Display` (gen-2 emite `<T: Clone + std::fmt::Debug + PartialEq>` en función libre genérica; verificado 2026-05-07 con `firstOf<T>` retornando `items[0]`)
- [x] **B141–B147** — fn-ref reduce, nested [[T]], toInt or fail, Map/Set params, indexOf 2-arg, user pop, arr.reverse on [T] — todos verificados en gen-2 con `compiler/tests/regression/b141_b147_gen2.liva` (2026-05-07; ya funcionaban, solo se pinearon)
- [x] **B148–B150** — patrones de constructor (`this.X` reads, mut locals, literal-string args) — verificados en gen-2 con `compiler/tests/regression/b148_b150_gen2.liva` (2026-05-07)
- [x] **GAP-007** — function types `(T) => U` → `Box<dyn Fn>` para inline lambda args en gen-2 (verificado 2026-05-07 con `compiler/tests/regression/gap007_fn_types.liva`. Caso let-bound closure pasado por identifier sigue OPEN: bootstrap está FROZEN y gen-2 mirrors that behavior; documentado.)
- [x] **B134–B137** — Map for-loop typing, switch-arm if-tail, Set.size, user `method.count(literal)` — verificados en gen-2 con `compiler/tests/regression/b134_b137_gen2.liva` (2026-05-07)
- [x] **B138** — `fail` en posición de expresión — verificado en gen-2 vía PARITY.md Tier 2 (probe + selfhost_apps 21/21 verde 2026-05-07). Bootstrap re-wrap bug en ternary-with-fail dentro de `T!` queda como deuda menor; bootstrap está FROZEN.
- [x] **B139** — switch arms en `T!` auto-wrap `Ok(...)` — verificado en gen-2 vía PARITY.md Tier 2 (probe `classify(n): string!`).
- [x] **B127–B133** — error handling completo — verificados en gen-2 vía PARITY.md Tier 2 (`err_unify_gen2.test.liva` 5/5, probes B129/B130/B132/B133, selfhost_apps 21/21 verde 2026-05-07).
- [x] **B116, B117, B120, B124** — indexed self-field assign, self.field.concat, .len() cast, m.set(p.field, p) partial-move — verificados en gen-2 con `compiler/tests/regression/b116_b124_gen2.liva` (2026-05-07; B118/B121/B122/B123/B125 ya estaban pineados)

> **Fase D status (2026-05-07): COMPLETA.** Todos los items B116–B153 y GAP-007 cerrados.
> Bootstrap_apps: 21/21 verde con gen-2. Regression suite: 18/18 verde. Gauntlet 7/7 verde.
> Pre-condición para `src/FROZEN.md` unfreeze (eliminar bootstrap) cumplida.

### Fase E — Promover apps a self-host
- [x] `selfhost_apps/*.liva` (21 apps) deben pasar también con gen-2 — verificado 2026-05-07 (`compiler/tests/selfhost_apps/run_gen2.sh` 21/21).
- [x] Renombrar a `selfhost_apps/` cuando todas pasen. ✅ 2026-05-07: directorio renombrado, scripts y docs actualizados, gauntlet 7/7 verde.
- [ ] CI: ejecutar la suite contra ambos compiladores hasta el corte final.

### Fase F — Cortar la cuerda
- [ ] Construir `livac` final con gen-N (Liva).
- [ ] Reemplazar `target/release/livac` (Rust) por el binario gen-N en CI.
- [ ] Eliminar `livac/src/*.rs` salvo `liva_rt` (que se queda como crate de runtime).
- [ ] Actualizar `Cargo.toml` para que `liva_rt` sea standalone.
- [ ] **v2.1 Release: Liva is fully self-hosted.**

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
