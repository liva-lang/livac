# Audit Report: snake-game

## Resumen
- Total errores: 1
- BUG: 1 | SKILL: 0 | GAP: 0 | AI: 0 | DESIGN: 0
- **Bloques `rust { }`:** 8 (~90 líneas Rust de 271 totales)
- **GAPs de stdlib identificados:** 6 (4 triviales, 1 fácil, 1 complejo)

## Información del proyecto
- **Archivos:** 1 (main.liva)
- **Líneas:** 271
- **Skill local:** Sí
- **Tema:** Juego Snake en terminal con crossterm (raw mode, input no-bloqueante)
- **Calidad del código:** Excelente — uso sofisticado de `rust { }` interop, lógica de juego limpia

## Errores encontrados

| # | Archivo | Línea | Error | Categoría | Fix |
|---|---------|-------|-------|-----------|-----|
| 1 | main.liva | 78 (flushOutput) | Rust E0252: `stdout` reimported — `use std::io::stdout` (hoisted de setupTerminal/restoreTerminal/clearAndHome) colisiona con `use std::io::{Write, stdout}` (hoisted de flushOutput) | BUG | Cambiado a `use std::io::Write; std::io::stdout().flush().unwrap();` para evitar import duplicado |

## Detalle del error

### Error 1: Duplicate hoisted `use` imports (BUG)

**Causa raíz:** El compilador hoistea todos los `use` statements de bloques `rust { }` al top del archivo Rust generado. Cuando múltiples bloques importan el mismo símbolo (`stdout`) de formas diferentes (`use std::io::stdout` vs `use std::io::{Write, stdout}`), Rust rechaza la doble importación.

**Código original (funciones separadas):**
```liva
setupTerminal() {
    rust {
        use std::io::stdout;     // ← hoisted
        stdout().execute(...);
    }
}

flushOutput() {
    rust {
        use std::io::{Write, stdout};  // ← hoisted → conflicto con stdout anterior
        stdout().flush().unwrap();
    }
}
```

**Rust generado (conflicto):**
```rust
use std::io::stdout;           // de setupTerminal
use std::io::{Write, stdout};  // de flushOutput → ERROR: stdout ya importado
```

**Fix aplicado:**
```liva
flushOutput() {
    rust {
        use std::io::Write;
        std::io::stdout().flush().unwrap();  // stdout() inline, sin import
    }
}
```

**Acción recomendada para el compilador:** Implementar deduplicación/merge de imports hoisted. Cuando se encuentran `use std::io::stdout` y `use std::io::{Write, stdout}`, debería generar `use std::io::{Write, stdout}` (merge).

## Patrones problemáticos
- **Hoisting de `use` sin deduplicación:** Este es un bug conocido del compilador (ya visto en text-search). Cuando un proyecto usa muchos bloques `rust { }` que importan los mismos módulos, los `use` hoisted generan conflictos. Este patrón será cada vez más común a medida que los usuarios hagan más uso de `rust { }` interop.

## Observaciones positivas
- El código generado por la IA es de muy alta calidad — bien estructurado, con buenas prácticas (separation of concerns, polling loop, collision detection)
- Uso correcto de la skill: `use rust "crossterm" version "0.28"`, `rust { }` blocks, funciones sin keyword, `Math.random()`, arrays con `take()`, string templates, etc.
- La IA acertó completamente el estilo y las capacidades del lenguaje
- Solo 1 error, y es del compilador, no de la IA

## Verificación de ejecución
- ✅ Compila exitosamente tras el fix
- ✅ Se ejecuta sin crash (3 segundos con timeout, exit code 124 = timeout, no error)
- ⚠️ No se pudo verificar gameplay interactivo automáticamente (raw mode terminal — crossterm captura input directo del TTY, no de stdin)

## Análisis de dependencia de `rust { }` — Hacia Liva puro

Este proyecto usa **8 bloques `rust { }`** de 271 líneas totales (~90 líneas son Rust puro). El objetivo de Liva es que los usuarios no necesiten escribir Rust directamente. Cada uso de `rust { }` señala un **GAP** del lenguaje/stdlib.

| # | Función | Propósito del `rust { }` | Feature necesaria en Liva | Prioridad | Notas |
|---|---------|--------------------------|--------------------------|-----------|-------|
| 1 | `setupTerminal()` | Raw mode, alternate screen, hide cursor (crossterm) | `Terminal.enableRaw()`, `Terminal.alternateScreen(true)`, `Terminal.hideCursor()` | Baja | Requiere módulo `Terminal` con bindings a crossterm |
| 2 | `restoreTerminal()` | Disable raw mode, leave alt screen, show cursor | `Terminal.disableRaw()`, `Terminal.alternateScreen(false)`, `Terminal.showCursor()` | Baja | Complemento de setupTerminal |
| 3 | `pollKey()` | Non-blocking keyboard polling con crossterm events | `Terminal.pollKey(timeout)` devolviendo enum `Key` | Baja | Input no-bloqueante es complejo — necesita event loop |
| 4 | `clearAndHome()` | Clear screen + move cursor a (0,0) | `console.clear()` o `Terminal.clear()` + `Terminal.moveTo(0, 0)` | Media | `console.clear()` es muy común en otros lenguajes |
| 5 | `sleepMs(ms)` | Thread sleep | **`Sys.sleep(ms)`** | **Alta** | Trivial de implementar. Función básica que todo lenguaje tiene |
| 6 | `flushOutput()` | Flush stdout | **`console.flush()`** o flush automático | **Alta** | Trivial. Idealmente el runtime podría auto-flush |
| 7 | `randomPos(max)` | `(r * range as f64) as i32 + 1` — cast float→int | **`Math.randomInt(min, max)`** | **Alta** | Ya existe `Math.random()` (float). Falta versión int con rango |
| 8 | (dentro de `randomPos`) | `as` cast entre tipos numéricos | Cast explícito: `toInt()`, `toFloat()` o `as number` | Media | Conversión entre tipos numéricos sin `rust { }` |

### Versión ideal sin `rust { }` (pseudocódigo Liva)

Si Liva tuviera las features necesarias, el código sería:

```liva
// En vez de 8 bloques rust { }, se usaría:
setupTerminal() {
    Terminal.enableRaw()
    Terminal.alternateScreen(true)
    Terminal.hideCursor()
}

restoreTerminal() {
    Terminal.disableRaw()
    Terminal.alternateScreen(false)
    Terminal.showCursor()
}

pollKey(): number {
    let key = Terminal.pollKey(0)   // 0ms timeout
    // ... mapping logic en Liva puro
}

clearAndHome() {
    console.clear()
}

sleepMs(ms: number) {
    Sys.sleep(ms)
}

randomPos(max: number): number {
    return Math.randomInt(1, max - 2)
}
```

### Resumen de GAPs identificados

| Feature | Impacto | Dificultad | Candidato a versión |
|---------|---------|------------|--------------------|
| `Sys.sleep(ms)` | Alto — usado en cualquier programa con timing | Trivial | v1.6 (Stdlib P1) |
| `Math.randomInt(min, max)` | Alto — evita cast manual float→int | Trivial | v1.6 (Stdlib P1) |
| `console.clear()` | Medio — útil para TUI/CLI interactivas | Trivial | v1.6 (Stdlib P1) |
| `console.flush()` | Medio — necesario cuando se mezcla print con cursor | Trivial | v1.6 (Stdlib P1) |
| Cast numérico (`toInt()`, `toFloat()`) | Medio — conversión entre tipos sin rust | Fácil | v1.6 |
| Módulo `Terminal` (raw mode, key polling, cursor) | Bajo (nicho) — solo para TUI games/apps | Complejo | v2.0+ |

**Con solo 4 features triviales** (`Sys.sleep`, `Math.randomInt`, `console.clear`, `console.flush`), este proyecto reduciría los bloques `rust { }` de 8 a 3 (solo los de crossterm terminal). El módulo `Terminal` completo es más ambicioso y puede esperar.

## Conclusiones
- Este es el proyecto con **menos errores de compilación** del audit hasta ahora (1 error, categoría BUG)
- Sin embargo, la **alta dependencia de `rust { }`** (8 bloques, ~90 líneas Rust) revela gaps importantes en la stdlib
- Las features más urgentes (`Sys.sleep`, `Math.randomInt`, `console.clear/flush`) son triviales de implementar y eliminarían la mayoría de los usos de `rust { }` en este tipo de proyectos
- La calidad del código de la IA es excelente — el problema no es la IA ni la skill, sino que Liva aún no tiene stdlib suficiente para este tipo de aplicación
