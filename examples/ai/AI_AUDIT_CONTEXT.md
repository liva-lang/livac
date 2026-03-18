# AI-Generated Code Audit — Contexto de Trabajo

> **Inicio:** 2026-03-18  
> **Objetivo:** Analizar 10 proyectos generados por IA con la skill `liva-lang` para identificar fallos y oportunidades de mejora  
> **Estado:** ✅ Completado (10/10 proyectos)

---

## Qué estamos haciendo

Tenemos 10 proyectos en `examples/ai/` generados por distintas IAs usando la skill `liva-lang` como guía. **Ninguno compila.** El objetivo es:

1. Compilar cada proyecto tal cual (código original intacto)
2. Crear una carpeta `fixed/` con la versión corregida del proyecto completo
3. Clasificar cada error encontrado
4. Generar un `REPORT.md` por proyecto
5. Al final, generar un informe consolidado con conclusiones

---

## Categorías de errores

Cada error encontrado se clasifica en una de estas categorías:

| Categoría | Código | Descripción |
|-----------|--------|-------------|
| **Bug del compilador** | `BUG` | El código es válido según la spec pero livac lo rechaza o genera Rust incorrecto |
| **Error de la Skill** | `SKILL` | La IA usó sintaxis que parece razonable porque la skill la indujo a creerlo o no lo documentó bien |
| **Feature gap** | `GAP` | El código usa algo que Liva debería soportar pero aún no tiene (candidato a backlog) |
| **Error de la IA** | `AI` | La IA inventó sintaxis sin base en la skill ni en ningún lenguaje razonable |
| **Diseño del lenguaje** | `DESIGN` | El error revela un problema de diseño o una oportunidad de mejora del lenguaje |

---

## Inventario de proyectos

| # | Proyecto | Archivos | Líneas | Skill local | Estado |
|---|----------|----------|--------|-------------|--------|
| 1 | web-scraper | 1 | 115 | No | ✅ Completado |
| 2 | todo-list | 3 | 251 | No | ✅ Completado |
| 3 | text-search | 1 | 253 | No | ✅ Completado |
| 4 | snake-game | 1 | 270 | Sí | ✅ Completado |
| 5 | calculator | 3 | 274 | No | ✅ Completado |
| 6 | csv-reader | 1 | 283 | No | ✅ Completado |
| 7 | chat-server | 1 | 195 | No | ✅ Completado |
| 8 | json-parser | 5 | 882 | Sí | ✅ Completado |
| 9 | rest-api | 4 | 973 | Sí | ✅ Completado |
| 10 | mini-interpreter | 6 | 1494 | Sí | ✅ Completado |

**Total: 26 archivos, ~4990 líneas de Liva**

---

## Metodología por proyecto

### Paso 1: Compilar original
```bash
cd examples/ai/<proyecto>/
livac check *.liva    # o livac build main.liva
```
Capturar todos los errores del compilador.

### Paso 2: Clasificar errores
Para cada error, determinar la categoría (BUG / SKILL / GAP / AI / DESIGN) y anotar:
- Línea y archivo
- Error reportado por livac
- Categoría
- Explicación de por qué ocurrió
- Fix aplicado

### Paso 3: Crear versión corregida
Crear carpeta `fixed/` con el proyecto completo corregido. Esto permite que los imports entre archivos funcionen correctamente.

```
examples/ai/<proyecto>/
  main.liva              # original (intacto)
  other.liva             # original (intacto)
  original_prompt.md
  fixed/                 # versión corregida completa
    main.liva
    other.liva
  REPORT.md              # informe del audit
```

### Paso 4: Ejecutar y verificar
No basta con que compile — hay que **ejecutar el binario** y verificar que funciona correctamente:
- Probar el flujo principal (happy path)
- Probar edge cases si aplica (input vacío, valores inválidos, etc.)
- Verificar que la salida es coherente con el propósito del programa
- Para programas interactivos: usar `echo -e "input1\ninput2\n..." | ./binary` para simular input
- Para programas no interactivos: ejecutar directamente y revisar output

Si el programa compila pero falla en ejecución, iterar sobre la versión `fixed/` hasta que funcione.

### Paso 5: Analizar dependencia de `rust { }`
Para cada bloque `rust { }` en el proyecto, documentar:
- **Qué hace** el bloque (propósito)
- **Qué feature de Liva** lo haría innecesario (stdlib function, módulo, operador)
- **Prioridad** (Alta/Media/Baja) — qué tan común y fácil de implementar es
- **Versión candidata** para implementarlo

El objetivo es que **todas las aplicaciones se puedan escribir sin `rust { }`**. Cada uso de `rust { }` que no sea para interop con crates específicos es un GAP del lenguaje.

### Paso 6: Generar REPORT.md
Crear `REPORT.md` en la carpeta del proyecto con:
- Tabla de errores (línea, error, categoría, fix)
- **Análisis de `rust { }` con tabla de features necesarias** (si aplica)
- Resumen de patrones problemáticos
- Conclusiones específicas
- **Confirmación de que el programa fue ejecutado y funciona**

---

## Formato del REPORT.md de cada proyecto

```markdown
# Audit Report: <nombre-proyecto>

## Resumen
- Total errores: X
- BUG: X | SKILL: X | GAP: X | AI: X | DESIGN: X

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix |
|---|---------|-------|-------|-----------|-----|
| 1 | main.liva | 12 | ... | SKILL | ... |

## Análisis de dependencia de `rust { }` (si aplica)

| # | Función | Propósito del `rust { }` | Feature necesaria en Liva | Prioridad |
|---|---------|--------------------------|--------------------------|-----------|
| 1 | fn_name | descripción | `Stdlib.method()` | Alta/Media/Baja |

## Patrones problemáticos
- ...

## Conclusiones
- ...
```

---

## Informe consolidado (al final)

Cuando se completen los 10 proyectos, generar `REPORT_SUMMARY.md` con:
- Estadísticas globales por categoría
- Top N errores más frecuentes
- Patrones que la IA repite
- Acciones concretas:
  - Fixes para la skill
  - Bugs para el backlog del compilador
  - Features a considerar para el lenguaje

---

## Progreso

| Fecha | Proyecto | Estado | Notas |
|-------|----------|--------|-------|
| 2026-03-18 | web-scraper | ✅ Completado | 5 errores (3 BUG, 1 GAP, 1 DESIGN). Compila y funciona. |
| 2026-03-18 | todo-list | ✅ Completado | 12 errores (8 BUG, 2 SKILL, 1 AI, 1 DESIGN). Merged single-file. Ejecutado y verificado: add/list/complete/delete/quit OK. |
| 2026-03-18 | text-search | ✅ Completado | 9 errores (5 BUG, 3 SKILL, 1 DESIGN). Reestructurado a clase SearchEngine para evitar ownership moves de Map. Ejecutado y verificado: single/AND/OR search OK. |
| 2026-03-18 | snake-game | ✅ Completado | 1 error (1 BUG). Duplicate hoisted `use` imports en Rust generado. Código de muy alta calidad. Compila y ejecuta OK. |
| 2026-03-18 | calculator | ✅ Completado | 7 errores (7 BUG). Todos codegen: method error binding, arrow return type, scope leak, mutability, ownership. Merged single-file + rust {} para parser. Ejecutado y verificado: 16 tests OK. |
| 2026-03-18 | csv-reader | ✅ Completado | 9 errores (7 BUG, 1 SKILL, 1 DESIGN). Principales: `_` en error binding, string en interpolación, ownership/move en loops, f64/i32 division. 1 rust {} para sort. Ejecutado y verificado: todos los menús OK. |
| 2026-03-18 | chat-server | ✅ Completado | 1 error (1 BUG). `main()` no se genera async cuando `rust { }` contiene `.await`. 100% del código es rust {} (networking). Compilado y verificado: TCP multi-cliente, broadcast, DM, /list, /help, /quit OK. |
| 2026-03-18 | json-parser | ✅ Completado | 16 errores (13 BUG, 1 AI, 2 DESIGN). El proyecto con más bugs de codegen: charAt, char escapes, enum destructuring, or fail, error binding, string +, template {:?}, ownership/move. Merged 5→1 archivo. Reescrito encoding (length-prefixed). 0 rust {}. Ejecutado y verificado: 4 demos OK (build, parse, round-trip, all-types). |
| 2026-03-18 | rest-api | ✅ Completado | 17 errores (14 BUG, 1 SKILL, 1 GAP, 1 AI). Proyecto más complejo: REST API con actix-web. 80% bugs de codegen (ownership/move en array concat y for loops). Fix aplicado al compilador: hyphen→underscore en `use` crate names (388 tests pass). Merged 4→1. rust {} masivo (~450 líneas) para HTTP server. 18 tests ejecutados: CRUD completo, búsqueda, paginación, validación, persistencia JSON OK. |
| 2026-03-18 | mini-interpreter | ✅ Completado | 15 errores (11 BUG, 3 AI, 1 DESIGN). Proyecto más grande: 1494 líneas, 6 archivos. 11 bugs de codegen (mayor cantidad). 2 bugs nuevos: find_rust_blocks matchea comments, find_balanced_brace confunde lifetimes/apostrophes con char literals. Merged 6→1 (1590 líneas). 92 rust {} blocks (48.9%). Ejecutado y verificado: demo.mini completo (variables, aritmética, strings, comparaciones, condicionales, while, funciones recursivas, factorial, fibonacci, fizzbuzz, max). |

---

## Referencia rápida

- **Skill:** `livac/skills/liva-lang/SKILL.md` (423 líneas)
- **Quick Reference:** `livac/docs/QUICK_REFERENCE.md` (1548 líneas)
- **Backlog:** `livac/BACKLOG.md`
- **Error Codes:** `livac/docs/ERROR_CODES.md`
- **Compilador:** `livac check <file>` para solo verificar, `livac build <file>` para compilar

---

## Regla: Actualizar este archivo

**Después de completar cada proyecto, SIEMPRE actualizar:**
1. La tabla de **Inventario de proyectos** — cambiar estado a ✅ Completado
2. La tabla de **Progreso** — añadir entrada con fecha, estado y notas
3. Si se descubren patrones nuevos, añadirlos a la sección de hallazgos globales (se creará con el informe consolidado)
