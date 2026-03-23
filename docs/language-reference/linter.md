# Linter — Static Analysis Warnings

> **Subcommand:** `livac lint <file>`  
> **Versión:** v1.8  
> **Propósito:** Detección de code smells sin bloquear la compilación

---

## Uso

```bash
# Lint a file (human-readable output)
livac lint main.liva

# JSON output (for IDE integration)
livac lint main.liva --json
```

### Salida ejemplo

```
warning [W001]: Unused variable
  --> main.liva:5
      5 |     let y = 10
   = Variable 'y' is declared but never used
   help: Prefix with underscore to suppress: _y

warning [W003]: Unreachable code
  --> main.liva:14
     14 |     console.log("unreachable")
   = Code after 'return' will never be executed
   help: Remove unreachable code or restructure the logic

2 warnings emitted
```

---

## Warning Codes

### W001 — Variable declarada pero no usada

Se emite cuando una variable local (let/const) o loop variable (for) se declara pero nunca se referencia.

```liva
main() {
    let x = 42        // W001: 'x' never used
    console.log("hi")
}
```

**Suprimir:** Prefija con `_`:
```liva
main() {
    let _x = 42       // ✅ No warning
    console.log("hi")
}
```

**No se emite para:**
- Parámetros de función (son parte de la interfaz pública)
- Variables prefijadas con `_`
- El wildcard `_` en destructuring

---

### W002 — Import no usado

Se emite cuando un símbolo importado no se usa en ningún lugar del archivo.

```liva
import { add, subtract } from "./math.liva"   // W002: 'subtract' unused

main() {
    console.log(add(1, 2))
}
```

**Solución:** Eliminar el import no usado:
```liva
import { add } from "./math.liva"   // ✅
```

**No se emite para:**
- Imports wildcard (`import * from "..."`)
- Tipos referenciados en type annotations

---

### W003 — Código inalcanzable

Se emite cuando hay statements después de `return`, `fail`, `break` o `continue`.

```liva
main() {
    return "done"
    console.log("never executed")   // W003
}
```

```liva
process(): string {
    fail "error"
    return "never"   // W003
}
```

**Nota:** Solo reporta el primer statement inalcanzable por bloque. No se emite dentro de branches `if/else` separados (solo en el mismo nivel de bloque).

---

### W004 — Comparación siempre true/false

Se emite cuando una comparación puede evaluarse estáticamente:

#### Caso 1: Variable comparada consigo misma
```liva
if x == x { ... }   // W004: always true
if x != x { ... }   // W004: always false
```

#### Caso 2: Literales diferentes comparados
```liva
if 42 == 99 { ... }   // W004: always false
if "a" != "b" { ... } // W004: always true
```

#### Caso 3: Literales iguales comparados
```liva
if true == true { ... }   // W004: always true
if 42 == 42 { ... }       // W004: always true
```

---

## JSON Output

Para integración con IDEs, usa `--json`:

```bash
livac lint main.liva --json
```

```json
[
  {
    "code": "W001",
    "title": "Unused variable",
    "message": "Variable 'y' is declared but never used",
    "file": "main.liva",
    "line": 5,
    "column": null,
    "source_line": "    let y = 10",
    "help": "Prefix with underscore to suppress: _y"
  }
]
```

---

## Comportamiento

- **No bloquea compilación:** Las warnings son informativas, `livac build/run` sigue funcionando sin cambios.
- **Exit code:** `livac lint` siempre retorna 0 (éxito) incluso con warnings. Solo retorna 1 si el archivo no parse.
- **`_` suprime W001:** Convención de Rust/Liva — variables prefijadas con `_` son intencionalmente ignoradas.
- **Un warning por bloque para W003:** Solo reporta el primer statement inalcanzable, no todos los siguientes.
