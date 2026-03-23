# 🚀 Plan: Production Readiness

> **Objetivo:** Llevar Liva a un estado donde se puedan hacer proyectos serios en producción  
> **Estado:** v1.4 P0 completado ✅  
> **Creado:** 2026-03-11  
> **Última actualización:** 2026-03-12

---

## 📋 Resumen ejecutivo

Dos líneas de trabajo paralelas para desbloquear producción:

1. **Stdlib ampliada** — Inspirada en Apache Commons para Java. No se crean librerías separadas: se amplían los tipos y módulos que ya existen. Solo se crean módulos nuevos cuando no hay tipo base.
2. **Rust Interop** — Bloques `rust { }` inline que permiten escribir Rust puro dentro de Liva, accediendo a todo el ecosistema de crates.

---

## 🗂️ Línea 1: Stdlib "Apache Commons"

### Estrategia

Inspirada en Apache Commons para Java, pero adaptada a Liva:

- **`StringUtils`** → NO es librería. Son **métodos nativos de `string`** (ampliar `generate_string_method_call()`)
- **`CollectionUtils`** → NO es librería. Son **métodos nativos de arrays/maps/sets** (ampliar generación de métodos)
- **`NumberUtils`** → NO es librería. Se amplía **`Math.*`** (ampliar `generate_math_function_call()`)
- **`FileUtils`/`IOUtils`** → NO es librería nueva. Se amplía **`File.*`** y **`Dir.*`** existentes
- **`DateUtils`**, **`RegexUtils`**, **`DigestUtils`** → SÍ son **módulos nuevos** (`Date.*`, `Regex.*`, `Crypto.*`) porque no hay tipo base

Cero dependencias externas para P0. Crates auto-inyectados para P1 (chrono, regex).

### Lo que ya existe hoy

**String (11 métodos):** `toUpperCase`, `toLowerCase`, `trim`, `trimStart`, `trimEnd`, `split`, `replace`, `contains`, `startsWith`, `endsWith`, `substring`, `charAt`, `indexOf`

**Array (11 métodos):** `map`, `filter`, `reduce`, `forEach`, `find`, `some`, `every`, `includes`, `indexOf`, `join`, `length`

**Math (11):** `sqrt`, `pow`, `abs`, `floor`, `ceil`, `round`, `min`, `max`, `random`, `PI`, `E`

**File (5):** `read`, `write`, `append`, `exists`, `delete`

**Dir (2):** `list`, `isDir`

**Sys (3):** `args`, `env`, `exit`

**HTTP (4):** `get`, `post`, `put`, `delete`

---

### P0 — String: ampliar métodos nativos (+15) ✅

> **Completado:** v1.4.0-dev — 15 nuevos métodos implementados y testeados.  
> **Nota:** Se usaron `padStart`/`padEnd` (estilo JS) en lugar de `padLeft`/`padRight`.

No se crea librería. Se añaden ramas al `match` en `generate_string_method_call()`:

| Método | Descripción | Rust equivalente |
|--------|-------------|-----------------|
| `str.padLeft(len, char)` | Rellena por la izquierda | `format!` con padding |
| `str.padRight(len, char)` | Rellena por la derecha | `format!` con padding |
| `str.repeat(n)` | Repite n veces | `.repeat(n)` |
| `str.capitalize()` | Primera letra mayúscula | `first.to_uppercase() + rest` |
| `str.isBlank()` | Solo whitespace o vacío | `.trim().is_empty()` |
| `str.isEmpty()` | Longitud cero | `.is_empty()` |
| `str.reverse()` | Invierte la cadena | `.chars().rev().collect()` |
| `str.truncate(len)` | Corta a longitud máxima | `&s[..len.min(s.len())]` (UTF-8 safe) |
| `str.countMatches(sub)` | Cuenta ocurrencias | `.matches(sub).count()` |
| `str.removePrefix(pre)` | Quita prefijo si existe | `.strip_prefix().unwrap_or(s)` |
| `str.removeSuffix(suf)` | Quita sufijo si existe | `.strip_suffix().unwrap_or(s)` |
| `str.chars()` | Array de caracteres | `.chars().map(\|c\| c.to_string()).collect()` |

### P0 — Array: ampliar métodos nativos (+20) ✅

> **Completado:** v1.4.0-dev — 20 nuevos métodos implementados y testeados.  
> **Nota:** `chunk()` renombrado a `chunks()` (conflicto con keyword `chunk` de parallel adapter). `flatten()` implementado como `flat()` (estilo JS). `sortBy(fn)` y `groupBy(fn)` pospuestos por complejidad.

No se crea librería. Se añaden al pipeline de generación de method calls de arrays:

| Método | Descripción | Rust equivalente |
|--------|-------------|-----------------|
| `arr.chunk(size)` | Divide en sub-arrays | `.chunks(size).map(\|c\| c.to_vec()).collect()` |
| `arr.zip(other)` | Combina dos arrays | `.iter().zip(other.iter()).collect()` |
| `arr.flatten()` | Aplana array de arrays | `.into_iter().flatten().collect()` |
| `arr.distinct()` | Elimina duplicados | `HashSet` dedup |
| `arr.sortBy(fn)` | Ordena por criterio | `.sort_by_key()` |
| `arr.groupBy(fn)` | Agrupa en Map | `HashMap` manual |
| `arr.reversed()` | Invierte el array | `.iter().rev().collect()` |
| `arr.slice(start, end)` | Sub-array | `&arr[start..end].to_vec()` |
| `arr.first()` | Primer elemento (optional) | `.first().cloned()` |
| `arr.last()` | Último elemento (optional) | `.last().cloned()` |
| `arr.isEmpty()` | Sin elementos | `.is_empty()` |
| `arr.count(fn)` | Cuenta que cumplan predicado | `.iter().filter(fn).count()` |
| `arr.sum()` | Suma (arrays numéricos) | `.iter().sum()` |
| `arr.min()` / `arr.max()` | Mínimo/máximo | `.iter().min()` / `.max()` |
| `arr.take(n)` / `arr.drop(n)` | Primeros n / sin primeros n | `&arr[..n]` / `&arr[n..]` |

### P0 — Math: ampliar funciones (+3) ✅

> **Completado:** v1.4.0-dev — 3 nuevas funciones implementadas y testeadas.  
> **Nota:** Se añadió `Math.log(x)` además de lo planificado originalmente.

| Función | Descripción | Rust equivalente |
|---------|-------------|-----------------|
| `Math.clamp(val, min, max)` | Limitar valor a rango | `val.max(min).min(max)` |
| `Math.log(x)` | Logaritmo natural | `(x as f64).ln()` |

### P1 — File: ampliar funciones existentes (+6) ✅

> **Completado:** v1.6.0-dev — 6 nuevas funciones (copy, move, size, extension, readLines, writeLines) + parser fix para `move` keyword.

Se añaden ramas al `match` en `generate_file_function_call()`:

| Función | Descripción | Rust equivalente |
|---------|-------------|-----------------|
| `File.readLines(path)` | Leer como array de líneas (fallible) | `read_to_string().lines().collect()` |
| `File.writeLines(path, lines)` | Escribir array como líneas (fallible) | `.join("\n")` + `write` |
| `File.copy(src, dst)` | Copiar archivo (fallible) | `fs::copy()` |
| `File.move(src, dst)` | Mover/renombrar (fallible) | `fs::rename()` |
| `File.size(path)` | Tamaño en bytes (fallible) | `.metadata()?.len()` |
| `File.extension(path)` | Extensión del archivo (no-fail) | `Path::extension()` |

### P1 — Dir: ampliar funciones existentes (+5) ✅

> **Completado:** v1.6.0-dev — 5 nuevas funciones (exists, create, delete, listRecursive, walk).

Se añaden ramas al `match` en `generate_dir_function_call()`:

| Función | Descripción | Rust equivalente |
|---------|-------------|-----------------|
| `Dir.create(path)` | Crear directorio recursivo (fallible) | `fs::create_dir_all()` |
| `Dir.delete(path)` | Eliminar directorio recursivo (fallible) | `fs::remove_dir_all()` |
| `Dir.walk(path)` | Listar recursivamente (fallible) | Recursivo manual → `[string]` |
| `Dir.exists(path)` | Comprobar si directorio existe (no-fail) | `Path::exists() && Path::is_dir()` |
| `Dir.listRecursive(path)` | Alias de walk | Recursivo manual → `[string]` |

### P1 — Date (tipo NUEVO, first-class)

Tipo `Date` first-class con propiedades y métodos. Internamente es un wrapper sobre `chrono::NaiveDateTime` (crate auto-inyectado). Se necesita:
- Nuevo tipo en `ast.rs` / `semantic.rs`
- `generate_date_function_call()` para constructores estáticos
- `generate_date_method_call()` para métodos de instancia

**Constructores estáticos (`Date.xyz()`):**

| Función | Descripción |
|---------|-------------|
| `Date.now()` | Fecha/hora actual → `Date` |
| `Date.new(year, month, day)` | Crear fecha → `Date` |
| `Date.parse(str, pattern)` | Parsear string → `Date, error` (fallible) |
| `Date.timestamp()` | Unix epoch actual (millis) → `int` |

**Propiedades de instancia (`d.xyz`):**

| Propiedad | Tipo | Descripción |
|-----------|------|-------------|
| `d.year` | `int` | Año (2026) |
| `d.month` | `int` | Mes (1-12) |
| `d.day` | `int` | Día (1-31) |
| `d.hour` | `int` | Hora (0-23) |
| `d.minute` | `int` | Minuto (0-59) |
| `d.second` | `int` | Segundo (0-59) |

**Métodos de instancia (`d.xyz()`):**

| Método | Descripción |
|--------|-------------|
| `d.format(pattern)` | Formatear → `string` ("DD/MM/YYYY", etc.) |
| `d.add(n, unit)` | Sumar tiempo → `Date` (unit: "days", "hours", etc.) |
| `d.diff(other, unit)` | Diferencia → `int` |
| `d.toString()` | ISO 8601 → `string` |

**Comparación:** soporta `>`, `<`, `>=`, `<=`, `==`, `!=` entre dos `Date`.

**Interpolación:** `$"Hoy es {now}"` usa `.toString()` automáticamente.

```liva
// Ejemplo de uso
let now: Date = Date.now()
let birthday = Date.new(1990, 6, 15)
let parsed, err = Date.parse("2026-03-11", "YYYY-MM-DD")

print(now.year)                    // 2026
print(now.format("DD/MM/YYYY"))    // 11/03/2026

let nextWeek = now.add(7, "days")
let age = now.diff(birthday, "years")  // 35

if nextWeek > now {
    print("El tiempo avanza")
}
print($"Hoy es {now}")  // → "Hoy es 2026-03-11T10:30:00"
```

### P1 — Regex (módulo NUEVO) ✅

> **Completado:** v1.6.0-dev — 5 funciones, crate `regex` auto-inyectado, parser fix para `test` keyword.

Requiere crate `regex` auto-inyectado. Se crea `generate_regex_function_call()`:

| Función | Descripción |
|---------|-------------|
| `Regex.test(pattern, text)` | ¿Coincide? → `bool` |
| `Regex.match(pattern, text)` | Primera coincidencia → `string?` |
| `Regex.findAll(pattern, text)` | Todas las coincidencias → `[string]` |
| `Regex.replace(pattern, text, repl)` | Reemplazar → `string` |
| `Regex.split(pattern, text)` | Dividir por patrón → `[string]` |

### P1 — CSV (módulo NUEVO)

Sin crates extra para CSV básico (split por comas + manejo de comillas con `std`). Se crea `generate_csv_function_call()`:

| Función | Descripción |
|---------|-------------|
| `CSV.read(path)` | Leer CSV → `[[string]]` (fallible) |
| `CSV.read(path, separator: "\t")` | Leer con separador custom (TSV, etc.) |
| `CSV.readTable(path)` | Leer con headers → `Table` (fallible). Ver sección Data |
| `CSV.write(path, rows)` | Escribir `[[string]]` → archivo (fallible) |
| `CSV.writeTable(path, table)` | Escribir `Table` con headers (fallible) |
| `CSV.parse(text)` | Parsear string → `[[string]]` |
| `CSV.stringify(rows)` | `[[string]]` → string CSV |

#### Tipo `Table`

`Table` es un tipo nuevo que representa datos tabulares con headers. Internamente es `[Map<string, string>]` con métodos extra:

```liva
let table, err = CSV.readTable("ventas.csv")

// Acceso a estructura
table.headers()                          // → ["producto", "region", "ventas"]
table.rows()                             // → [Map<string, string>]
table.column("ventas")                   // → [string]
table.length                             // → número de filas

// Operaciones (devuelven Table nueva)
let filtrado = table.filter(row => row.get("region") == "Europa")
let ordenado = table.sortBy("ventas", descending: true)
let agrupado = table.groupBy("producto")  // → Map<string, Table>

// Conversión numérica + cálculos con arrays existentes
let ventas = table.column("ventas").map(x => parseInt(x) or 0)
let total = ventas.sum()
print($"Total ventas: {total}")

// Escribir resultado
CSV.writeTable("resultado.csv", filtrado)
```

### P2 — Random (módulo NUEVO, `Math.random()` se mantiene)

Se crea `generate_random_function_call()`. Crate `rand` ya está disponible:

| Función | Descripción |
|---------|-------------|
| `Random.nextInt(min, max)` | Entero en rango → `number` |
| `Random.nextFloat(min, max)` | Float en rango → `float` |
| `Random.choice(array)` | Elemento aleatorio → `T?` |
| `Random.shuffle(array)` | Mezclar array → `[T]` |
| `Random.uuid()` | UUID v4 → `string` (requiere crate `uuid`) |

### P2 — Crypto (módulo NUEVO)

Requiere crates `sha2`, `base64` auto-inyectados. Se crea `generate_crypto_function_call()`:

| Función | Descripción |
|---------|-------------|
| `Crypto.sha256(input)` | Hash SHA-256 hex → `string` |
| `Crypto.md5(input)` | Hash MD5 hex → `string` |
| `Crypto.base64Encode(input)` | Codificar → `string` |
| `Crypto.base64Decode(input)` | Decodificar (fallible) → `string` |

### P2 — Process (módulo NUEVO)

Se crea `generate_process_function_call()`. Sin crates extra (usa `std::process`):

| Función | Descripción |
|---------|-------------|
| `Process.exec(cmd)` | Ejecutar comando (fallible) → `string` |
| `Process.spawn(cmd)` | Ejecutar en background → `number` (pid) |
| `Process.pid()` | PID del proceso actual → `number` |

---

### Resumen: qué se amplía vs qué se crea

```
AMPLIAR lo existente (no crear nada nuevo):
─────────────────────────────────────────
• string.xyz()  → +12 métodos (padLeft, repeat, capitalize, etc.)
• array.xyz()   → +16 métodos (chunk, zip, flatten, etc.)
• Math.xyz()    → +2 funciones (clamp, log)
• File.xyz()    → +5 funciones (readLines, copy, move, etc.)
• Dir.xyz()     → +3 funciones (create, delete, walk)

CREAR módulos nuevos:
────────────────────
• Date (tipo)   → tipo first-class con propiedades y métodos [P1, crate: chrono]
• Regex.xyz()   → 5 funciones (test, match, findAll, etc.)  [P1, crate: regex]
• CSV.xyz()     → 7 funciones + tipo Table (read, write...)  [P1, std puro]
• Random.xyz()  → 5 funciones (nextInt, choice, etc.)       [P2, crate: rand]
• Crypto.xyz()  → 4 funciones (sha256, base64, etc.)        [P2, crates: sha2, base64]
• Process.xyz() → 3 funciones (exec, spawn, pid)            [P2, std::process]
```

---

## 🗂️ Línea 2: Rust Interop — Bloques `rust { }`

### Sintaxis

```liva
// Dentro de una función Liva
calculate(): number {
    let x = 10
    let result = rust {
        let y: i32 = x * 2;
        y + 100
    }
    print(result)  // 120
}

// Función entera en Rust
fastHash(input: string): string {
    rust {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

// Con crate externo
use rust "uuid" version "1.0"

generateId(): string {
    rust {
        uuid::Uuid::new_v4().to_string()
    }
}
```

### Reglas semánticas

1. **`rust { }` es una expresión** — tiene un valor (última expresión Rust sin `;`)
2. **Variables del scope Liva son visibles** dentro del bloque con sus nombres en snake_case
3. **El tipo de retorno se infiere** del contexto Liva (variable destino o return type)
4. **`use` dentro del bloque** se hoistean al top del archivo Rust generado
5. **No hay validación semántica** del contenido — errores de `rustc` se reenvían al usuario
6. **El formatter no toca** el interior del bloque

### `use rust` ampliado

```liva
// Actual (ya funciona)
use rust "serde"

// Propuesta: con versión explícita
use rust "sha2" version "0.10"

// Propuesta: con features
use rust "tokio" features ["net", "io-util"]

// Propuesta: con version + features
use rust "reqwest" version "0.12" features ["json", "rustls-tls"]
```

Genera en `Cargo.toml`:
```toml
sha2 = "0.10"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
```

### Crates internos — Protección contra conflictos

Liva usa internamente estos crates con versiones fijas:

| Crate | Versión | Uso interno |
|-------|---------|-------------|
| `tokio` | 1.x | Runtime async, `liva_rt` |
| `serde` | 1.0 | JSON parse/stringify |
| `serde_json` | 1.0 | JSON parse/stringify |
| `reqwest` | 0.11 | HTTP client |
| `rayon` | 1.11 | Parallel execution |
| `rand` | 0.8 | `Math.random()` |

**Reglas:**

1. **No se puede hacer override de versión** de un crate interno:
   ```liva
   use rust "tokio" version "2.0"
   // ❌ Error[L9002]: Cannot override internal crate "tokio" (v1.x)
   //    Liva uses tokio 1.x for its async runtime.
   //    help: tokio is already available inside rust { } blocks
   //    help: to add features: use rust "tokio" features ["net"]
   //    note: internal crates: tokio, serde, serde_json, reqwest, rayon, rand
   ```

2. **Sí se pueden añadir features** a crates internos:
   ```liva
   use rust "tokio" features ["net"]  // ✅ Merge de features
   ```

3. **Los crates internos son auto-disponibles** en `rust { }` blocks sin declarar nada:
   ```liva
   myFunc() {
       rust {
           // tokio, serde, serde_json ya están en scope
           let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
       }
   }
   ```

### Impacto en el compilador

| Componente | Cambio | Esfuerzo |
|-----------|--------|----------|
| **Lexer** | Modo "raw block": contar `{`/`}` balanceadas, capturar como string. Ignorar `{`/`}` dentro de strings/comentarios Rust | ~1h |
| **AST** | Nuevo nodo: `RustBlock { code: String, span: Span }` como `Expr` o `Stmt` | ~30min |
| **Parser** | Parsear `rust { ... }` como expresión. Parsear `version`/`features` en `use rust` | ~1.5h |
| **Semantic** | Skip del contenido. Validar crates internos (error L9002) | ~1h |
| **IR** | Pass-through: `IRExpr::RustBlock(String)` | ~30min |
| **Codegen** | Emitir bloque verbatim. Hoistear `use`. Generar `Cargo.toml` con version/features | ~2h |
| **Formatter** | No formatear el interior del bloque | ~30min |
| **LSP** | Marcar bloque como lenguaje "rust" para syntax highlighting embebido | ~1h |
| **Error mapping** | Mapear errores de `rustc` a líneas del `.liva` | ~2h |
| **Tests** | Snapshot tests, integración | ~2h |
| **Total** | | **~12h** |

### Bloques futuros (no implementar aún)

| Bloque | Dificultad | Utilidad | Notas |
|--------|-----------|----------|-------|
| `asm { }` | Baja | Nicho (cripto, SIMD) | Mapear a macro `asm!` de Rust |
| `c { }` | Media | Alta (legacy libs) | Generar `.c` + `build.rs` + `extern "C"` |
| `cpp { }` | Alta | Media | Requiere bridge `cxx`. Diferir. |

---

##  Decisiones tomadas

1. **Stdlib nativa (Nivel C)** para funciones de uso universal — no requiere que el usuario sepa Rust
2. **`rust { }` blocks (Nivel A)** como escape hatch para acceder a todo el ecosistema Rust
3. **Crates internos protegidos** — no se puede hacer override de versión, sí añadir features
4. **Crates internos auto-disponibles** — no necesitan `use rust` para usarlos en bloques `rust { }`
5. **Sintaxis `rust { }`** preferida sobre `native("rust") { }` por legibilidad
6. **`asm { }` y `c { }` diferidos** — implementar solo si hay demanda real

---

## 🗂️ Línea 3: Estrategia de Datos — "Liva como alternativa a Python"

### Visión

Python domina big data/scripting por pandas, pero tiene un problema fundamental: es lento. Liva compila a Rust nativo, eliminando el cuello de botella. La concurrencia híbrida (`.par().map(fn)`) da paralelismo real sin GIL.

### Progresión por fases

```
Fase 1 (v1.5):  CSV básico nativo
─────────────────────────────────
CSV.read / CSV.write / CSV.parse / CSV.stringify
→ Cubre scripts simples y ETL básico
→ Cero dependencias, puro Rust std

Fase 2 (v1.5):  Table (CSV con headers + operaciones)
─────────────────────────────────────────────────────
CSV.readTable → Table con filter/sortBy/groupBy/column
→ Cubre el 80% de análisis de datos simples
→ Tipo Table = [Map<string, string>] con métodos
→ Sigue sin dependencias externas

Fase 3 (v1.6+): rust { } + Polars para heavy lifting
─────────────────────────────────────────────────────
use rust "polars" → DataFrames reales vía rust { }
→ Para millones de filas, lazy eval, joins, pivots
→ El usuario que necesite esto ya sabe lo que hace

Fase 4 (v2.x+): DataFrame nativo (si hay demanda)
──────────────────────────────────────────────────
Wrapper de Polars con sintaxis Liva pura
→ Solo si la comunidad lo pide y hay adopción
→ Sería librería separada, no stdlib
```

### Por qué Liva puede competir con Python

```
Python + pandas:   Python (lento) → C/Fortran (rápido) → Python (lento)
Liva:              Todo Rust nativo (rápido siempre)

Python: CSV 1GB → ~30s en pandas, for loop 1M filas → ~45s, paralelizar → dolor (GIL)
Liva:   CSV 1GB → ~3s nativo, for loop 1M filas → ~0.5s, paralelizar → arr.par().map(fn)
```

### Ejemplo comparativo

```liva
// Liva — análisis de ventas (nativo, sin dependencias)
let table, err = CSV.readTable("ventas.csv")
if err { fail err }

let europa = table.filter(row => row.get("region") == "Europa")
let ventas = europa.column("importe").map(x => parseInt(x) or 0)

print($"Filas Europa: {europa.length}")
print($"Total: {ventas.sum()}")
print($"Promedio: {ventas.sum() / ventas.length}")
print($"Máximo: {ventas.max()}")

// Paralelizar cálculos pesados sobre los datos (una línea)
let procesado = ventas.par().map(x => heavyCalc(x))

CSV.writeTable("resultado.csv", europa.sortBy("importe", descending: true))
```

---

## 🗂️ Línea 4: Infraestructura de Producción

> Features adicionales necesarias para proyectos reales en producción. Pendiente de priorización.

### Logging estructurado (módulo `Log`)

`print()` no sirve en producción. Se necesitan niveles, timestamps y contexto.

```liva
Log.info("User login", userId: 42)
// → 2026-03-11T10:30:00 [INFO] User login {userId: 42}

Log.warn("Rate limit close", current: 95, max: 100)
Log.error("Connection failed", host: "db.prod", err: err)
Log.debug("Query result", rows: items.length)
```

| Función | Descripción |
|---------|-------------|
| `Log.info(msg, ...context)` | Nivel informativo |
| `Log.warn(msg, ...context)` | Nivel advertencia |
| `Log.error(msg, ...context)` | Nivel error |
| `Log.debug(msg, ...context)` | Nivel debug (solo si `--verbose`) |
| `Log.setLevel(level)` | Cambiar nivel mínimo en runtime |

Implementación: Rust `eprintln!` con formato, o crate `tracing` para producción seria.

### HTTP Server (módulo `Server` o `Router`)

Liva ya tiene HTTP client pero no puede *servir* HTTP. Sin esto no se pueden hacer APIs.

```liva
import { Router, Request, Response } from "liva/http"

let app = Router()

app.get("/", (req: Request): Response => {
    Response.json({ message: "Hello from Liva!" })
})

app.get("/users/:id", (req: Request): Response => {
    let id = req.params.get("id") or fail "Missing id"
    Response.json({ id: id, name: "Alice" })
})

app.post("/users", (req: Request): Response => {
    let body = req.json()
    let name = body.get("name") or fail "Missing name"
    Response.json({ created: true }, status: 201)
})

app.listen(3000)
print("Server running on :3000")
```

Implementación: genera código con `tokio` + `hyper` o wrapper simple sobre `axum`.

### Base de datos (módulo `DB`)

No se puede construir un backend sin persistencia. Empezar con SQLite (sin servidor externo):

```liva
let db, err = DB.open("app.db")
if err { fail err }

DB.exec(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
DB.exec(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])

let rows, err = DB.query(db, "SELECT * FROM users WHERE name = ?", ["Alice"])
for row in rows {
    print($"User: {row.get("name")}")
}

DB.close(db)
```

Implementación: crate `rusqlite` auto-inyectado. Futuro: `sqlx` para PostgreSQL/MySQL.

### Project scaffolding (`livac init`)

Crear estructura de proyecto, no archivos sueltos:

```bash
livac init my-api
# Crea:
# my-api/
# ├── main.liva
# ├── config.liva
# ├── tests/
# │   └── main.test.liva
# └── .gitignore

livac init my-cli --template cli
livac init my-data --template data
```

Implementación: puro en el binario `livac`, templates hardcodeados. Sin dependencias.

### Config / Variables de entorno (módulo `Config`)

Más allá de `Sys.env()` — archivos `.env`, config por entorno:

```liva
Config.load(".env")                        // Cargar archivo .env
let dbUrl = Config.get("DATABASE_URL") or fail "Missing DATABASE_URL"
let port = Config.getInt("PORT") or 3000   // Con fallback y tipo
let debug = Config.getBool("DEBUG") or false

// Config.getAll() → Map<string, string>
```

Implementación: parser `.env` simple en Rust puro (formato `KEY=VALUE`).

### REPL interactivo (`livac repl`)

El killer feature de Python. Probar código sin crear archivo:

```
$ livac repl
Liva v1.5.0 REPL — Type .help for commands
>> let x = 42
>> x * 2
84
>> let names = ["Alice", "Bob"]
>> names.map(n => n.toUpperCase())
["ALICE", "BOB"]
>> .exit
```

Implementación: loop read-eval, compilar cada línea como snippet, mantener estado entre líneas. Medio-alto esfuerzo.

### WebSockets (ampliar `HTTP` o módulo `WS`)

Comunicación en tiempo real:

```liva
// Server
let ws = WS.serve(8080, (conn) => {
    conn.onMessage((msg) => {
        conn.send($"Echo: {msg}")
    })
})

// Client
let conn, err = async WS.connect("ws://localhost:8080")
conn.send("Hello")
let reply = async conn.receive()
```

Implementación: `tokio-tungstenite` auto-inyectado.

### Benchmarking (`livac bench`)

Medir rendimiento, mostrar la ventaja sobre Python:

```liva
import { bench, report } from "liva/bench"

bench("sort 10k numbers", () => {
    let nums = Random.shuffle([0..10000])
    nums.sortBy(x => x)
})

bench("filter + map", () => {
    let result = bigArray.filter(x => x > 50).map(x => x * 2)
})

report()
// sort 10k numbers:  avg 1.2ms  (min 0.9ms, max 1.8ms, 100 runs)
// filter + map:      avg 0.3ms  (min 0.2ms, max 0.5ms, 100 runs)
```

Implementación: similar al test runner existente. Medir con `std::time::Instant`.

### Linter / Warnings

Detectar code smells en tiempo de compilación:

```
warning[W001]: Variable 'x' declared but never used
  --> main.liva:5:5
   |
5  | let x = 42
   |     ^ unused variable (prefix with _ to suppress)

warning[W002]: Import 'math' is unused
  --> main.liva:1:1
   |
1  | import { sqrt } from "./math.liva"
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unused import
```

Implementación: análisis en `semantic.rs`, emitir warnings además de errores.

### Doc generator (`livac doc`)

Generar documentación desde comentarios:

```liva
/// Calculates the area of a circle
/// @param radius The radius of the circle
/// @returns The area as a float
circleArea(radius: float): float => Math.PI * Math.pow(radius, 2.0)
```

```bash
livac doc src/          # Genera docs/api/ con HTML o markdown
```

Implementación: parsear `///` comments, generar markdown o HTML. Medio esfuerzo.

### Test coverage (`livac test --coverage`)

```bash
livac test --coverage
# ────────────────────────────
# File              Lines   Covered   %
# src/math.liva     45      42        93.3%
# src/utils.liva    120     87        72.5%
# Total             165     129       78.2%
```

Implementación: instrumentación del código generado + recolección de datos. Alto esfuerzo.

### Serialización: YAML / TOML

Formatos de config estándar (Kubernetes, Docker, CI/CD):

```liva
let config, err = YAML.parse(File.read("config.yaml"))
let settings, err = TOML.parse(File.read("settings.toml"))

let yamlStr = YAML.stringify(data)
```

Implementación: crates `serde_yaml` y `toml` auto-inyectados.

### Package manager (`livac install`)

Compartir y reutilizar código entre proyectos:

```bash
livac install http-router          # Instala paquete
livac install --save csv-parser    # Añade a liva.toml
```

```liva
import { Router } from "http-router"
```

Implementación: ALTO esfuerzo (registry, resolución de dependencias, versionado, lock files). Diferir hasta que haya comunidad.

### Debugging (breakpoints, step-through)

Inspección interactiva del código en ejecución:

```
Breakpoint hit at main.liva:15
>> inspect users
[User { name: "Alice", age: 30 }, User { name: "Bob", age: 25 }]
>> step
```

Implementación: ALTO esfuerzo (DWARF debug info, DAP protocol para VS Code). Diferir.

### Profiler

Encontrar cuellos de botella:

```bash
livac run --profile main.liva
# ────────────────────────────
# Function          Calls    Total ms    Avg ms
# processData       1        450.2       450.2
# parseRow          10000    320.1       0.03
# heavyCalc         10000    125.8       0.01
```

Implementación: instrumentación de funciones + `std::time`. Medio esfuerzo.

---

### Resumen Línea 4: Infraestructura de Producción

```
TIER 1 — Sin esto no hay producción real:
──────────────────────────────────────────
• Log.xyz()        → Logging estructurado con niveles y contexto
• HTTP Server      → Router + handlers (construir APIs)
• livac init       → Scaffolding de proyectos
• Config / .env    → Variables de entorno y config por archivo
• DB (SQLite)      → Persistencia básica

TIER 2 — Diferenciadores competitivos:
──────────────────────────────────────────
• livac repl       → REPL interactivo (killer feature vs Python)
• livac bench      → Benchmarking built-in
• Linter/Warnings  → Detección de code smells

TIER 3 — Ecosistema maduro:
──────────────────────────────────────────
• livac doc        → Generación de documentación
• livac test --coverage → Cobertura de tests
• WebSockets       → Comunicación en tiempo real
• YAML/TOML        → Formatos de config estándar

TIER 4 — Largo plazo (alto esfuerzo):
──────────────────────────────────────────
• Package manager  → livac install (necesita comunidad primero)
• Debugging        → Breakpoints + DAP protocol
• Profiler         → Instrumentación de funciones
```

---

## 🗓️ Roadmap propuesto (actualizado)

| Versión | Contenido | Tipo |
|---------|-----------|------|
| **v1.4** | Stdlib P0 — String (+12), Array (+16), Math (+2) | Nativo |
| **v1.5** | Stdlib P1 — File, Dir, Date, Regex, CSV/Table + Logging + Config | Nativo + crates |
| **v1.6** | Rust Interop — `rust { }` blocks + `use rust` ampliado | Interop |
| **v1.7** | Stdlib P2 — Random, Crypto, Process + `livac init` | Nativo + tooling |
| **v1.8** | HTTP Server + DB (SQLite) | Networking + persistencia |
| **v1.9** | REPL + Benchmarking + Linter/Warnings | Developer experience |
| **v2.0** | Dogfooding — API REST real con DB + validación producción | Validación |
| **v2.x** | Doc generator, Coverage, WebSockets, YAML/TOML | Ecosistema maduro |
| **v3.x** | Package manager, Debugging, Profiler | Largo plazo |

---

## ❓ Decisiones pendientes

- [ ] ¿Cómo manejar el return de `rust { }` cuando la función Liva es fallible (`fail`)?
- [ ] ¿Permitir `rust { }` a nivel top-level (para definir structs/impls Rust auxiliares)?
- [ ] ¿Syntax highlighting del bloque `rust { }` en VS Code — language embedding o solo "string"?
- [ ] ¿Qué pasa con `async` dentro de `rust { }` — el usuario gestiona tokio directamente?
- [ ] ¿Dogfooding v2.0: qué proyecto concreto? (API REST con DB, CLI tool complejo, ETL pipeline?)
- [ ] ¿`Table` es un tipo first-class del lenguaje o un alias/wrapper sobre `[Map<string, string>]`?
- [ ] ¿`table.sortBy` ordena numéricamente strings que parecen números, o siempre alfabético?
- [ ] ¿HTTP Server: API propia de Liva o wrapper sobre `axum`/`hyper`?
- [ ] ¿DB: solo SQLite para empezar, o soporte multi-driver desde el inicio?
- [ ] ¿REPL: compila cada línea individualmente o mantiene un programa incremental?
- [ ] ¿Logging: formato propio o compatible con estándares (JSON structured, OpenTelemetry)?
- [ ] ¿Linter: integrado en `livac` o flag separado (`livac lint`)?
