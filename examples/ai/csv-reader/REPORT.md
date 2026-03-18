# Audit Report: csv-reader

## Resumen
- Total errores: 9
- BUG: 7 | SKILL: 1 | GAP: 0 | AI: 0 | DESIGN: 1

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix |
|---|---------|-------|-------|-----------|-----|
| 1 | main.liva | 82+ | `let bestSal, _ = parseFloat(...)` — `_` no aceptado como identificador en error binding. El lexer emite `Token::Underscore` pero `parse_identifier()` no lo reconoce. El propio `hints.rs` sugiere este patrón. | BUG | Reemplazar `_` con nombre de variable (`err1`, `err2`, etc.) |
| 2 | main.liva | 247 | `$"...{depts.join(", ")}..."` — string literal `", "` dentro de interpolación cierra prematuramente el template string. Produce E2001. | BUG | Extraer `depts.join(", ")` a variable local antes del template |
| 3 | main.liva | 283 | `main()` standalone al final del archivo — parser lo interpreta como definición de función sin cuerpo ("Expected LBrace"). Liva auto-invoca `main()`. | DESIGN | Eliminar la llamada `main()` al final |
| 4 | main.liva | 56 | `.filter(emp => ...)` sobre `[[string]]` genera `.copied()` en Rust, pero `Vec<String>` no implementa `Copy`. | BUG | Reemplazar `.filter()` con loop manual |
| 5 | main.liva | 128,165 | `totalSalary / employees.length` — codegen genera `f64 / i32` sin cast. Rust no permite división de tipos mixtos. | BUG | Usar contador float manual en vez de `.length` para la división |
| 6 | main.liva | 179 | `let writeErr = File.write(...)` — binding de una sola variable para función fallible genera tupla `(Option<bool>, String)` incompatible con `if writeErr {`. | BUG | Usar binding de dos variables: `let ok, writeErr = File.write(...)` |
| 7 | main.liva | 82-96 | Variables de error binding (`bestSal`) no se declaran como `mut` en Rust generado cuando se reasignan. Produce E0384. | BUG | Copiar valor de error binding a variable separada: `let bestSal = bestSalParsed` |
| 8 | main.liva | 97,103 | `remaining[bestIdx]` y `remaining[i]` en sort — acceso por índice a `Vec<Vec<String>>` genera movimiento (move) en vez de clone. E0507. | BUG | Usar `rust { }` block con sort_by para implementar el ordenamiento |
| 9 | main.liva | 236-277 | `allEmployees` y `currentData` pasados a funciones dentro de `while` loop — cada paso mueve el valor. No se genera `.clone()` para llamadas a funciones en loops. E0382. | BUG | Usar patrón de copia inline: `let copy = []; for e in original { copy.push(e) }` — el for-in genera `.clone()` sobre la fuente |

## Análisis de dependencia de `rust { }`

| # | Función | Propósito del `rust { }` | Feature necesaria en Liva | Prioridad | Versión |
|---|---------|--------------------------|--------------------------|-----------|---------|
| 1 | sortBySalary | Sort con comparador custom (workaround para acceso por índice moviendo valores y error binding inmutable) | `array.sortBy(fn)` — método de ordenamiento con comparador personalizado | Alta | v1.6+ |

## Patrones problemáticos

### 1. Ownership / Move semantics (errores 8, 9)
El patrón más problemático del proyecto. Todo valor pasado a una función es movido (consumed). En un loop donde la misma variable se pasa múltiples veces en distintas iteraciones, el compilador Rust rechaza el código. La clave del workaround es que `for x in array { ... }` genera `array.clone()` automáticamente, así que iterar y reconstruir es la única forma de "clonar" sin `rust { }`.

**Impacto**: Obliga a patrones verbosos y no idiomáticos. Agregar `.clone()` implícito en llamadas a funciones dentro de loops sería la solución en codegen.

### 2. Error binding no mutable (error 7)
Cuando el resultado de un error binding (`let val, err = fallible()`) se reasigna más tarde, el codegen no detecta que necesita `mut`. El workaround de copiar `let newVar = val` funciona pero es antinatural.

### 3. `_` como discard en error binding (error 1)
El hint del compilador (`hints.rs:97`) sugiere `let result, _ = func(...)` pero el parser no acepta `_` como identificador. Inconsistencia interna.

### 4. String literals en interpolación (error 2)
`$"text {fn("arg")}"` — las comillas del argumento cierran la template string. Esto es un edge case del lexer que debería parsear correctamente (como lo hace JavaScript con template literals).

### 5. Filter sobre arrays anidados (error 4)
`.filter()` en `[[string]]` genera `.copied()` que falla porque `Vec<String>` no es `Copy`. La alternativa idiomática sería `.cloned()` en el codegen.

## Verificación de ejecución

**Programa ejecutado y verificado.** Todos los flujos probados:
- ✅ Cargar CSV (15 empleados desde employees.csv)
- ✅ Mostrar todos los empleados (opción 1)
- ✅ Filtrar por departamento (opción 2 — Ingeniería: 5 empleados)
- ✅ Ordenar ascendente por salario (opción 3)
- ✅ Ordenar descendente por salario (opción 4)
- ✅ Estadísticas completas (opción 5 — promedio, min, max, por departamento)
- ✅ Exportar a CSV (opción 6 — verificado contenido del archivo)
- ✅ Salir (opción 7)

## Conclusiones

- **Calidad del código original**: Alta. Estructura clara, funciones bien separadas, buen uso de tipos, menú interactivo funcional. Solo 9 errores para 283 líneas (3.2%).
- **9 de 9 errores son bugs del compilador o diseño** — la IA no cometió inventos ni usó sintaxis incorrecta.
- **El error 1 (discard con `_`)** es el más impactante: el compilador sugiere un patrón en sus propios hints que no funciona.
- **Los errores de ownership (8, 9)** son los más costosos de workaround: requieren reestructuración significativa del código y patrones verbosos antiidiomáticos.
- **Un solo `rust { }` block** fue necesario para el sort con comparador. Un `array.sortBy(fn)` en la stdlib lo eliminaría.
- **No se usó skill local** — el código fue generado sin referencia a la skill, lo que explica que `console.prompt` (válido pero no documentado en la skill) se usó en vez de `console.input`.
