# 🔧 Liva Compiler Context

> **Proyecto:** livac - El compilador de Liva  
> **Versión:** v1.2.0-dev (tag: v1.0.2)  
> **Lenguaje:** Rust  

---

## 📦 Qué es este proyecto

**livac** es el compilador del lenguaje Liva. Transforma código `.liva` en código Rust, que luego se compila a binario nativo.

```
Liva Source → Lexer → Parser → AST → Semantic → IR → Codegen → Rust → Binary
```

---

## 🏗️ Arquitectura

```
src/
├── main.rs           # CLI (clap) - punto de entrada
├── lib.rs            # API pública del compilador
├── lexer.rs          # Tokenización (logos)
├── parser.rs         # Parser (chumsky) → AST
├── ast.rs            # Definición del AST
├── semantic.rs       # Análisis semántico y tipos
├── desugaring.rs     # Transformaciones sintácticas
├── ir.rs             # Representación intermedia
├── lowering.rs       # AST → IR
├── codegen.rs        # IR → Código Rust (~400KB, ~11300 líneas)
├── formatter.rs      # Code formatter (--fmt)
├── module.rs         # Sistema de módulos e imports
├── traits.rs         # Sistema de traits/interfaces
├── error.rs          # Sistema de errores
├── error_codes.rs    # Códigos E0xxx
├── hints.rs          # Sugerencias de errores
├── suggestions.rs    # "Did you mean?" (Levenshtein)
├── span.rs           # Ubicaciones en código fuente
├── liva_rt.rs        # Runtime embebido
└── lsp/              # Language Server Protocol
    ├── server.rs     # Servidor LSP (tower-lsp)
    ├── document.rs   # Gestión de documentos
    ├── symbols.rs    # Tabla de símbolos
    ├── diagnostics.rs
    ├── imports.rs
    └── workspace.rs
```

---

## 🛠️ Comandos

```bash
# Build
cargo build --release

# Test
cargo test

# Compilar archivo Liva
livac archivo.liva

# Compilar y ejecutar
livac archivo.liva --run

# Verificar sintaxis
livac archivo.liva --check

# Formatear código
livac archivo.liva --fmt

# Ejecutar tests (archivos .test.liva)
livac archivo.test.liva --test

# Iniciar LSP
livac --lsp
```

---

## 📖 Sintaxis del Lenguaje (Quick Reference)

### Variables
```liva
let x = 10              // Mutable
const PI = 3.14159      // Inmutable
let name: string = "Alice"
```

### Funciones
```liva
greet() => print("Hello!")
add(a: number, b: number): number => a + b

calculate(x: number): number {
    let result = x * 2
    return result
}
```

### Control de Flujo
```liva
// Block syntax
if condition { } else { }
for item in items { }
while running { }

// One-liner => syntax (v1.1.0)
if age >= 18 => print("Adult")
for item in items => print(item)
while running => tick()
```

### Error Handling
```liva
// Error binding
let result, err = divide(10, 0)
if err { fail "Error occurred" }

// or fail (v1.1.0)
let data = File.read("config.json") or fail "Cannot read"
```

### Point-Free Function References (v1.1.0)
```liva
// Pass function names directly as callbacks
items.forEach(print)           // instead of: items.forEach(x => print(x))
nums.map(toString)             // instead of: nums.map(n => toString(n))
names.filter(isValid)          // instead of: names.filter(n => isValid(n))

// Also works with for => loops
for item in items => print     // instead of: for item in items => print(item)
```

### Method References with `::` (v1.1.0)
```liva
// Reference an instance method as a callback
let fmt = Formatter("Hello")
let greetings = names.map(fmt::format)    // ["Hello: Alice", ...]
names.forEach(fmt::format)

// object::method binds the method to the specific instance
// Works with: map, filter, forEach, find, some, every
```

### Clases
```liva
Person {
    name: string
    age: number
    
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    
    greet() => $"Hi, I'm {this.name}"
}
```

### Test Framework (v1.2.0) — Jest-like API
```liva
import { describe, test, expect, beforeEach, afterEach, beforeAll, afterAll } from "liva/test"

describe("Calculator", () => {
    let calc = 0

    beforeEach(() => {
        calc = 0
    })

    test("adds numbers", () => {
        calc = 2 + 3
        expect(calc).toBe(5)
    })

    test("async fetch", () => {
        let data = async fetchData()
        expect(data).toBeDefined()
    })
})
```

**Matchers disponibles:** `toBe`, `toEqual`, `toBeGreaterThan`, `toBeLessThan`,
`toContain`, `toHaveLength`, `toBeTruthy`, `toBeFalsy`, `toBeDefined`, `toBeNull`
(+ variantes `.not.`)

**Nota:** Los tests async se detectan automáticamente y generan `#[tokio::test]`.

---

## 📚 Documentación

| Archivo | Contenido |
|---------|-----------|
| `docs/QUICK_REFERENCE.md` | Cheat sheet completo de sintaxis |
| `docs/README.md` | Índice de toda la documentación |
| `docs/language-reference/` | Referencia detallada por tema |
| `docs/guides/` | Tutoriales y best practices |
| `ROADMAP.md` | Plan del proyecto y estado |
| `CHANGELOG.md` | Historial de versiones |
| `BUGS.md` | Bugs encontrados en dogfooding |

---

## 🔄 Estado Actual

- **54/54 bugs** del dogfooding corregidos
- **Phase 10** (Formatter): ✅ Completado
- **Phase 11.1** (`or fail`): ✅ Completado  
- **Phase 11.2** (`=>` one-liners): ✅ Completado
- **Phase 11.3** (Point-free): ✅ Completado
- **Phase 11.4** (Method refs `::`): ✅ Completado
- **Phase 12.1** (Test Runner): ✅ Completado
- **Phase 12.2** (Test Library): ✅ Completado
- **Phase 12.3** (Lifecycle Hooks): ✅ Completado
- **Phase 12.4** (Async Test Support): ✅ Completado

---

## ⚠️ Notas para Desarrollo

1. **codegen.rs** es el archivo más grande (~11300 líneas) - contiene toda la generación de Rust
2. **formatter.rs** maneja el formateo de código
3. Los tests están en `tests/` y se ejecutan con `cargo test`
4. El LSP se comunica por stdio con la extensión VS Code
5. Los archivos `.test.liva` se ejecutan con `livac --test` y generan tests Rust nativos

---

## 🔁 Regla: Actualizar Contextos

**Al terminar cada tarea o fase, SIEMPRE actualizar estos archivos de contexto:**
- `livac/.github/copilot-instructions.md` — versión, estado, features, arquitectura
- `.github/copilot-instructions.md` (workspace) — versión, estado, features recientes
- `WORKSPACE_CONTEXT.md` — igual que el workspace copilot-instructions
- `ROADMAP.md` y `CHANGELOG.md` — progreso y changelog

Esto asegura que el asistente AI siempre tenga contexto actualizado del proyecto.
