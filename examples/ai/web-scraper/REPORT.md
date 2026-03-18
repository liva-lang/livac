# Audit Report: web-scraper

> **Fecha:** 2026-03-18  
> **Archivos:** 1 (main.liva, 115 líneas)  
> **Skill local:** No  
> **Resultado:** ✅ Compilado y funcionando tras 4 correcciones

---

## Resumen

- **Total errores: 5**
- BUG: 3 | SKILL: 0 | GAP: 1 | AI: 0 | DESIGN: 1

---

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix aplicado |
|---|---------|-------|-------|-----------|--------------|
| 1 | main.liva | 58-68 | `E0603: expression 't' is not awaitable` — El código pushea `task async fetchPage(url)` a un array y luego itera con `await t`. El compilador pierde el tipo "task handle" al meterlo en un array. | **GAP** | Reestructurar concurrencia: eliminar `processBatch()` y hacer las llamadas secuenciales por batch |
| 2 | main.liva | 44 | `err.is_some()` on String — La error binding `let resp, err = async HTTP.get(url)` con el prefijo `async` no registra `err` en `string_error_vars`. El codegen genera `err.is_some()` pero `err` es un String. | **BUG** | Quitar `async` explícito de `HTTP.get(url)` — el runtime ya es async internamente |
| 3 | main.liva | 47 | `err.message` en String error var — El codegen compila `err.message` como `err.as_ref().map(\|e\| e.message.as_str())` pero `err` es String, no tiene `.message`. | **BUG** | Usar `err` directamente en vez de `err.message` ya que el string ES el mensaje |
| 4 | main.liva | 50 | `resp.get_field("body")` — El codegen genera `.get_field("body")` para acceder a `resp.body`, pero `LivaHttpResponse` tiene `body` como campo público directo. | **BUG** | Quitar `async` explícito resolvió también este problema — sin `async`, el codegen accede a `resp.body` directamente |
| 5 | main.liva | 62 | Future no awaited en spawn — `async fetchOne(url)` genera `spawn_async(async move { fetch_one(url) })` sin `.await` dentro del bloque. La función async del usuario no se awaita dentro del spawn, resultando en `Future<Future<String>>` en vez de `Future<String>`. | **BUG** → **DESIGN** | Eliminar la indirección async y llamar `fetchPage` directamente (sequential) |

---

## Patrones problemáticos

### 1. Task handles en arrays (GAP)
La IA intentó un patrón razonable y común en otros lenguajes: crear task handles, meterlos en un array, y luego iterar para await. Liva no soporta esto actualmente. El compilador debería poder trackear que un array contiene task handles y permitir `await` sobre sus elementos.

### 2. `async` explícito en HTTP.get rompe 3 codegen features (BUG)
Cuando se usa `async HTTP.get(url)` en vez de `HTTP.get(url)` (sin prefijo):
- `string_error_vars` no se registra → `if err { }` genera `err.is_some()` incorrecto
- `err.message` se compila como Option access en vez de acceso directo al String
- `resp.body` se compila como `resp.get_field("body")` en vez de campo directo

El path **sin** `async` funciona correctamente. El path **con** `async` tiene el codegen roto para error binding de HTTP. Esto es grave porque la skill enseña `async HTTP.get(url)`.

### 3. spawn_async no awaita funciones async del usuario (BUG)
`async myFunc(args)` genera `spawn_async(async move { my_func(args) })` — pero `my_func` es async, así que debería ser `spawn_async(async move { my_func(args).await })`. El `.await` interno sí se genera para calls del runtime (HTTP, etc.) pero no para funciones del usuario.

---

## Cambios realizados en fixed/

| Cambio | Líneas originales | Motivo |
|--------|-------------------|--------|
| Eliminar `processBatch()` y restructurar como loop secuencial | 54-68 | Task handles en arrays no soportado |
| `HTTP.get(url)` sin `async` explícito | 44 | Bug: el prefijo async rompe error binding |
| `err` en vez de `err.message` | 47 | Bug: err es String, no tiene .message |
| Añadir type hints `[string]` a arrays vacíos | 80, 86 | Ayudar al type inference |
| Eliminar wrapper `fetchOne()` | 55-57 | No necesario sin spawn pattern |

---

## Conclusiones

### Acciones para el compilador (Bugs)
1. **[BUG-HIGH]** Fix codegen: `async HTTP.get(url)` debe registrar `string_error_vars` igual que `HTTP.get(url)` sin async
2. **[BUG-HIGH]** Fix codegen: `async userFunc(args)` debe generar `.await` dentro del `spawn_async` block para funciones async del usuario
3. **[BUG-MED]** Fix codegen: `resp.body` con `async HTTP.get` no debe generar `get_field("body")` sino acceso directo al campo

### Acciones para la Skill
4. **[SKILL]** Considerar documentar que `HTTP.get()` ya es async internamente y no necesita prefijo `async` — o bien, si se quiere mantener el patrón `async HTTP.get()`, arreglar los bugs primero
5. **[SKILL]** Documentar que `err.message` vs `err` depende del tipo de error binding (actualmente confuso)

### Acciones para el lenguaje (Features)
6. **[GAP]** Soportar task handles en arrays: `tasks.push(task async fn())` → `await t` iterando — patrón fundamental para concurrencia dinámica

### Lo que la IA hizo bien
- Estructura general del programa correcta
- Uso idiomático de `rust { }` para timestamp
- String interpolation con `$"..."` correcto
- Métodos de string (`toLowerCase`, `indexOf`, `substring`, `trim`) correctos
- `and` como operador lógico correcto
- Batching manual con `while` loop correcto
- Error handling con `if err { ... }` correcto (el patrón, no la implementación)
