# 📋 Backlog — Production Readiness

> **Objetivo:** Llevar Liva a producción real  
> **Plan de diseño:** `docs/plans/PLAN_PRODUCTION_READINESS.md`  
> **Prioridad:** Orden por versión = orden de implementación  
> **Última actualización:** 2026-03-12

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

**Pendientes (pospuestos — complejidad alta):**
- [ ] `arr.sortBy(fn)` → `[T]`
- [ ] `arr.groupBy(fn)` → `Map<K, [T]>`

### Math — ampliar `generate_math_function_call()` ✅

- [x] `Math.clamp(val, min, max)` → `number`
- [x] `Math.sign(val)` → `int` (-1, 0, 1)
- [x] `Math.log(x)` → `float` (logaritmo natural)

---

## v1.5 — Rust Interop + Logging + Config + `livac init`

> **Foco:** `rust { }` desbloquea todo el ecosistema. Logging/Config/init hacen proyectos "reales".  
> **Esfuerzo estimado:** ~18h (12h rust interop + 2h×3 tooling)  
> **Impacto:** Force multiplier — de "usable" a "viable".

### `rust { }` interop — Ver plan §Línea 2

- [ ] Parser: reconocer `rust { ... }` como expresión
- [ ] Parser: reconocer `use rust "crate" version "x.y"` con features opcionales
- [ ] Desugaring: registrar crates del `rust { }` y `use rust`
- [ ] Codegen: emitir bloque Rust inline tal cual
- [ ] Codegen: inferir crate de uso (`use xxx::yyy` → detectar crate)
- [ ] Codegen: `generate_cargo_toml()` — inyectar crates de usuario + internos
- [ ] Protección: error L9001 si `use rust` usa crate interno sin versión diferente
- [ ] Protección: error L9002 si intenta override de versión de crate interno
- [ ] Tests: bloque rust básico, con crate, con crate interno, errores
- [ ] Docs: documentar sintaxis y limitaciones

### Logging — módulo `Log`

- [ ] `Log.info(msg, ...context)` → stderr con timestamp + nivel + contexto
- [ ] `Log.warn(msg, ...context)`
- [ ] `Log.error(msg, ...context)`
- [ ] `Log.debug(msg, ...context)` — solo con `--verbose`
- [ ] `Log.setLevel(level)` — cambiar nivel en runtime
- [ ] Tests

### Config / .env — módulo `Config`

- [ ] `Config.load(path)` — parsear archivo `.env` (KEY=VALUE)
- [ ] `Config.get(key)` → `string, error`
- [ ] `Config.getInt(key)` → `int, error`
- [ ] `Config.getBool(key)` → `bool, error`
- [ ] `Config.getAll()` → `Map<string, string>`
- [ ] Tests

### `livac init` — scaffolding

- [ ] `livac init <name>` — crea directorio con main.liva + tests/ + .gitignore
- [ ] `livac init <name> --template cli` — template CLI
- [ ] `livac init <name> --template data` — template data processing
- [ ] Tests

---

## v1.6 — Stdlib P1: File, Dir, Date, Regex, CSV/Table

> **Foco:** Módulos para scripts y procesamiento de datos.  
> **Esfuerzo estimado:** ~16h  
> **Impacto:** Scripts reales, posicionamiento vs Python.

### File — ampliar `generate_file_function_call()`

- [ ] `File.exists(path)` → `bool`
- [ ] `File.copy(src, dest)` → `error?`
- [ ] `File.move(src, dest)` → `error?`
- [ ] `File.size(path)` → `int, error`
- [ ] `File.extension(path)` → `string`

### Dir — ampliar `generate_dir_function_call()`

- [ ] `Dir.exists(path)` → `bool`
- [ ] `Dir.list(path)` → `[string], error`
- [ ] `Dir.listRecursive(path)` → `[string], error`

### Date — tipo nuevo (first-class)

**Tipo en compilador:**
- [ ] Añadir tipo `Date` en `ast.rs` (Type::Date)
- [ ] Soporte en `semantic.rs` (type checking, comparaciones)
- [ ] `generate_date_function_call()` para constructores estáticos
- [ ] `generate_date_method_call()` para métodos de instancia
- [ ] Soporte en interpolación de strings (`$"{date}"` → `.toString()`)

**Constructores estáticos:**
- [ ] `Date.now()` → `Date`
- [ ] `Date.new(year, month, day)` → `Date`
- [ ] `Date.parse(str, pattern)` → `Date, error`
- [ ] `Date.timestamp()` → `int` (unix epoch ms)

**Propiedades:** `.year`, `.month`, `.day`, `.hour`, `.minute`, `.second`
- [ ] Acceso a propiedades de instancia → `int`

**Métodos de instancia:**
- [ ] `d.format(pattern)` → `string`
- [ ] `d.add(n, unit)` → `Date`
- [ ] `d.diff(other, unit)` → `int`
- [ ] `d.toString()` → `string` (ISO 8601)

**Operadores:** `>`, `<`, `>=`, `<=`, `==`, `!=`
- [ ] Comparación entre dos `Date`

- [ ] Tests

### Regex — módulo nuevo (crate `regex` auto-inyectado)

- [ ] `Regex.match(pattern, str)` → `bool`
- [ ] `Regex.find(pattern, str)` → `string, error`
- [ ] `Regex.findAll(pattern, str)` → `[string]`
- [ ] `Regex.replace(pattern, str, replacement)` → `string`
- [ ] `Regex.split(pattern, str)` → `[string]`
- [ ] Tests

### CSV — módulo nuevo

- [ ] `CSV.read(path)` → `[[string]], error`
- [ ] `CSV.write(path, data)` → `error?`
- [ ] `CSV.parse(str)` → `[[string]]`
- [ ] `CSV.stringify(data)` → `string`
- [ ] `CSV.readTable(path)` → `Table, error` (con headers)
- [ ] `Table.filter(fn)`, `Table.sortBy(col)`, `Table.groupBy(col)`, `Table.column(name)`
- [ ] Tests

---

## v1.7 — Stdlib P2: Random, Crypto, Process + HTTP Server

> **Foco:** Completar stdlib + poder servir HTTP.  
> **Esfuerzo estimado:** ~16h

### Random — módulo nuevo

- [ ] `Random.int(min, max)` → `int`
- [ ] `Random.float()` → `float`
- [ ] `Random.choice(arr)` → `T`
- [ ] `Random.shuffle(arr)` → `[T]`
- [ ] `Random.uuid()` → `string`
- [ ] Tests

### Crypto — módulo nuevo (crate `sha2`/`hmac` auto-inyectado)

- [ ] `Crypto.sha256(data)` → `string`
- [ ] `Crypto.md5(data)` → `string`
- [ ] `Crypto.hmac(key, data)` → `string`
- [ ] `Crypto.randomBytes(n)` → `string` (hex)
- [ ] Tests

### Process — módulo nuevo

- [ ] `Process.exec(cmd, args?)` → `string, error`
- [ ] `Process.spawn(cmd, args?)` → `int, error` (PID)
- [ ] `Process.exit(code)`
- [ ] Tests

### HTTP Server — Ver plan §Línea 4

- [ ] `Router()` — crear router
- [ ] `app.get(path, handler)`, `app.post(...)`, `app.put(...)`, `app.delete(...)`
- [ ] `app.listen(port)` — arrancar servidor
- [ ] `Request` type: params, query, body, headers
- [ ] `Response` type: json(), text(), status()
- [ ] Codegen: genera código con tokio + hyper/axum
- [ ] Tests

---

## v1.8 — DB + REPL + Linter

> **Foco:** Persistencia, developer experience, calidad de código.  
> **Esfuerzo estimado:** ~20h

### DB — módulo nuevo (crate `rusqlite` auto-inyectado)

- [ ] `DB.open(path)` → `DB, error`
- [ ] `DB.exec(db, sql, params?)` → `error?`
- [ ] `DB.query(db, sql, params?)` → `[Map<string, string>], error`
- [ ] `DB.close(db)`
- [ ] Tests

### REPL — `livac repl`

- [ ] Loop read-eval-print básico
- [ ] Mantener estado entre líneas (variables persisten)
- [ ] Mostrar resultado de expresiones
- [ ] Comandos `.help`, `.exit`, `.clear`
- [ ] Historial con readline
- [ ] Tests

### Linter / Warnings

- [ ] W001: Variable declarada pero no usada
- [ ] W002: Import no usado
- [ ] W003: Código inalcanzable después de `return`/`fail`
- [ ] W004: Comparación siempre true/false
- [ ] Flag `--lint` o integrado en compilación normal
- [ ] Tests

---

## v2.0 — Dogfooding: API REST real

> **Foco:** Validación real construyendo un proyecto completo.  
> **Esfuerzo estimado:** ~12h

- [ ] Definir proyecto de dogfooding (API REST con DB)
- [ ] Implementar proyecto completo en Liva
- [ ] Documentar bugs encontrados en BUGS.md
- [ ] Corregir todos los bugs
- [ ] Escribir regression tests
- [ ] Post-mortem: qué falta, qué mejorar

---

## v2.x — Ecosistema maduro (futuro)

> **Priorizar según demanda de usuarios.**

- [ ] `livac --doc` — generación de documentación desde `///` comments
- [ ] `--test --coverage` — cobertura de tests
- [ ] WebSockets — módulo `WS` (tokio-tungstenite)
- [ ] YAML/TOML parsing — módulos nuevos (crates serde_yaml/toml)
- [ ] `livac --bench` — benchmarking built-in

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
