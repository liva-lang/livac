# 📋 Backlog — Production Readiness

> **Objetivo:** Llevar Liva a producción real  
> **Plan de diseño:** `docs/plans/PLAN_PRODUCTION_READINESS.md`  
> **Prioridad:** Orden por versión = orden de implementación  
> **Última actualización:** 2026-03-31

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
- [ ] Implementar codegen (subset) en Liva (pendiente — reiniciar tras fixes)
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

### Liva Test Suite — archivos .liva que validan el lenguaje

> **Foco:** Crear suite completa de tests escritos EN Liva que validen toda la sintaxis y features documentadas.
> **Directorio:** `compiler/tests/liva/` (se promueve a `tests/liva/` con el resto del compiler)
> **Runner:** `compiler/tests/liva/run_tests.sh` con filtros por capa

**Capa 1: Syntax (`compiler/tests/liva/syntax/`)** — `livac check`, sin compilar
- [ ] variables.liva — let, const, type inference, top-level const
- [ ] functions.liva — block, one-liner `=>`, typed params/returns, default params
- [ ] classes.liva — constructor, methods, visibility, data classes
- [ ] enums.liva — unit, tuple, struct variants, recursive (auto-boxing)
- [ ] generics.liva — generic functions, classes, constraints
- [ ] control_flow.liva — if/else, switch (statement + expression), for, while, break/continue
- [ ] error_handling.liva — fallible `!`, `or value`, `or fail`, try/catch
- [ ] pattern_matching.liva — switch patterns, destructuring, guards, wildcard `_`
- [ ] imports.liva — use statements, extensionless, public/private
- [ ] types.liva — type aliases, optional `T?`, tuples, union types
- [ ] lambdas.liva — closures, point-free refs, method references `::`
- [ ] string_templates.liva — `$"..."`  interpolation
- [ ] defer.liva — defer statement, defer blocks
- [ ] compound_assign.liva — `+=`, `-=`, `*=`, `/=`, `%=`
- [ ] rust_interop.liva — `rust { }` blocks, `use rust`

**Capa 2: Compile (`compiler/tests/liva/compile/`)** — `livac build`, cargo check
- [ ] basic_program.liva — hello world, variables, functions
- [ ] class_program.liva — class con métodos, constructores
- [ ] enum_program.liva — enums con switch exhaustivo
- [ ] generic_program.liva — funciones y clases genéricas
- [ ] error_program.liva — fallible functions, or value, try/catch
- [ ] collections.liva — arrays, maps, sets, iteraciones
- [ ] closures.liva — lambdas como parámetros, map/filter/reduce
- [ ] multifile/ — proyecto multi-archivo con imports

**Capa 3: E2E Runtime (`compiler/tests/liva/e2e/`)** — build + run + comparar output
- [ ] hello.liva + hello.expected — pipeline completo mínimo
- [ ] fibonacci.liva + fibonacci.expected — recursión, expresiones
- [ ] calculator.liva + calculator.expected — clases, switch, métodos
- [ ] linked_list.liva + linked_list.expected — enums recursivos, pattern matching
- [ ] grade_tracker.liva + grade_tracker.expected — arrays, map, filter, sort
- [ ] key_value_store.liva + key_value_store.expected — Map, Set, iteración
- [ ] error_chain.liva + error_chain.expected — fallible, or fail, error trace
- [ ] async_basic.liva + async_basic.expected — async/await básico
- [ ] string_utils.liva + string_utils.expected — string processing intensivo
- [ ] for_patterns.liva + for_patterns.expected — for i,v in array, for k,v in map, ranges

**Capa 4: Stdlib (`compiler/tests/liva/stdlib/`)** — build + run, métodos stdlib
- [ ] string_methods_1.liva — contains, replace, split, trim, case, indexOf, startsWith/endsWith
- [ ] string_methods_2.liva — padStart/End, repeat, slice, chars, capitalize, removePrefix/Suffix
- [ ] string_methods_3.liva — countMatches, isBlank, isEmpty, reverse, truncate, lastIndexOf
- [ ] array_methods_1.liva — push, pop, length, sort, reversed, includes, indexOf, join
- [ ] array_methods_2.liva — map, filter, find, forEach, some, every, reduce, flatMap
- [ ] array_methods_3.liva — distinct, chunks, zip, take, drop, first, last, flat, findIndex, slice
- [ ] map_methods.liva — get, set, has, delete, keys, values, entries, clear, forEach, isEmpty
- [ ] set_methods.liva — add, has, delete, clear, values, forEach, union, intersection, difference
- [ ] math_functions.liva — abs, floor, ceil, round, pow, sqrt, min, max, PI, E, log, sin, cos, tan
- [ ] random_functions.liva — Random.int, Random.float, Random.bool, Random.choice, Random.shuffle
- [ ] regex_functions.liva — Regex.match, test, findAll, replace, split
- [ ] date_functions.liva — Date.now, format, parse, diff, add, year/month/day/hour/minute/second
- [ ] csv_functions.liva — CSV.parse, stringify, parseFile, writeFile
- [ ] config_functions.liva — Config.load, get, getInt, getBool, getAll
- [ ] process_functions.liva — Process.exec, Process.exit, Sys.args, Sys.env
- [ ] log_functions.liva — Log.info, Log.warn, Log.error, Log.debug
- [ ] crypto_functions.liva — Crypto.hash, hmac, randomBytes, uuid
- [ ] type_conversions.liva — toString, toInt, toFloat, parseInt, parseFloat

**Capa 5: Stdlib-IO (`compiler/tests/liva/stdlib-io/`)** — opt-in, requiere filesystem/red
- [ ] file_operations.liva — File.read, write, append, exists, delete, copy, move, size, ext, name, lines
- [ ] dir_operations.liva — Dir.list, create, exists, remove, current, files, dirs
- [ ] db_sqlite.liva — DB.open, exec, query, close
- [ ] http_server.liva — Server.create, routes, Response helpers

**Capa 6: Errors (`compiler/tests/liva/errors/`)** — `livac check`, deben fallar con error esperado
- [ ] E0101_undefined_var.liva — variable not defined
- [ ] E0201_type_mismatch.liva — type mismatch assignment
- [ ] E0301_undefined_function.liva — calling undefined function
- [ ] E0401_missing_return.liva — missing return type
- [ ] E0501_duplicate_definition.liva — duplicate name
- [ ] E0601_invalid_import.liva — importing from nonexistent module
- [ ] E0904_non_exhaustive_switch.liva — enum switch missing variant
- [ ] W001_unused_var.liva — unused variable warning
- [ ] W002_unused_import.liva — unused import warning
- [ ] W003_unreachable_code.liva — unreachable after return

**Runner:**
- [ ] `compiler/tests/liva/run_tests.sh` — test runner con filtros
  - `./run_tests.sh` — todo menos stdlib-io
  - `./run_tests.sh --all` — incluye stdlib-io
  - `./run_tests.sh --only syntax` — solo una capa
  - `./run_tests.sh --only stdlib` — solo stdlib
  - Exit code 0/1 para CI

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
