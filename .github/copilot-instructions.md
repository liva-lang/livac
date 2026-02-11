# 🏗️ Liva Project Workspace Context

> **Propósito**: Este archivo proporciona contexto completo sobre el workspace para mantener consistencia en todas las interacciones de desarrollo.

---

## 📦 Resumen del Proyecto

**Liva** es un lenguaje de programación moderno que compila a Rust, combinando:
- Simplicidad de Python/TypeScript
- Rendimiento de Rust
- Seguridad con manejo explícito de errores
- Concurrencia híbrida (async + parallel)

### Estado Actual (Dogfooding Session - 2026-02-04)

Se han realizado múltiples sesiones de dogfooding donde se construyeron herramientas CLI reales usando Liva:

**Herramientas CLI construidas:**
- 🔧 **GitHub CLI** - HTTP + JSON + Arrays (user/repos/issues/search)
- 🔧 **Config Tool** - File I/O + JSON + Dynamic keys
- 🔧 **Task Manager** - File I/O + JSON + String handling
- 🔧 **Notes App** - Classes + Methods + Arrays + File I/O + JSON
- 🔧 **Weather CLI** - HTTP async + Real APIs + Nested JSON parsing
- 🔧 **Crypto Tracker** - CoinGecko API + JSON arrays + null checking
- 🔧 **Todo API** - HTTP POST/PUT/DELETE CRUD operations
- 🔧 **Modular App** - Multi-file imports + string indexing
- 🔧 **Log Analyzer** - Switch/match patterns + File.exists + for loops
- 🔧 **Generics Tests** - Box<T>, Pair<A,B>, Stack<T>, generic imports

**Bugs corregidos (54 de 54 - ¡100% ARREGLADOS! 🎉):**

*v0.11.4 - Fixes iniciales:*
- ✅ Private field underscore prefix preservado en snake_case
- ✅ `.length` genera `.len() as i32` para strings/arrays
- ✅ Métodos que modifican `this.field` generan `&mut self`
- ✅ Asignación de `this.field` auto-clona
- ✅ `.filter()`/`.find()` usan `.cloned()` para tipos non-Copy

*v0.11.5 - JSON/HTTP fixes:*
- ✅ Bug #10-13: JsonValue improvements (as_str, Display, Index, PartialEq)

*v0.11.6 - Sys module + JSON access:*
- ✅ Bug #14: Nested JSON access (`issue["user"]["login"]`)
- ✅ Bug #15: JSON indexed variables tracking
- ✅ Bug #16: JSON access with string variable

*v0.11.7 - Classes + String handling:*
- ✅ Bug #17: String literals generate `.to_string()`
- ✅ Bug #18: String vars detected in concatenations
- ✅ Bug #19: Constructor body field assignment parsing
- ✅ Bug #20: Mutating methods trigger `&mut self`
- ✅ Bug #22: forEach lambda for non-Copy types

*v0.11.8 - HTTP + JSON array access:*
- ✅ Bug #23: `Http.get()` not recognized (case-insensitive now)
- ✅ Bug #24: `as_array()` returns Vec directly, array indexing with `.cloned()`

*v0.11.9 - JsonValue null and float:*
- ✅ Bug #25: JsonValue null comparison uses `.is_null()`
- ✅ Bug #26: Added `as_float()` returning f64 directly
- ✅ Bug #27: Vec<JsonValue> uses `.len()` not `.length()`

*v0.11.10 - String indexing:*
- ✅ Bug #28: `s[i]` uses `.chars().nth(i)` for UTF-8 safety

*v0.11.11 - Switch patterns:*
- ✅ Bug #29: Switch with string literals adds `.as_str()` to discriminant

*v0.11.12 - Pattern matching:*
- ✅ Bug #30: Pattern matching exhaustiveness check with int/string literals

*v0.11.13-v0.11.19 - Session 10 Dogfooding:*
- ✅ Bug #31: `array.length.toString()` wraps cast in parens: `(len as i32).to_string()`
- ✅ Bug #32: String variables cloned when passed to constructors
- ✅ Bug #34: Array indexing with int variables adds `as usize` + `.clone()` for strings
- ✅ Bug #35: forEach on `[string]` uses `|p|` not `|&p|` - track string array types
- ✅ Bug #36: Method calls on binary expressions wrap in parens: `(a + b).method()`
- ✅ Bug #37: `join()` keeps `&str` argument, no `.to_string()`
- ✅ Bug #38: JSON `asString()`, `asBool()` add `.unwrap_or_default()` for direct values

*v0.11.22 - Wildcard imports:*
- ✅ Bug #40: `import * as alias` genera `module::function()` correctamente

*v0.11.23 - Parallel & Filter fixes:*
- ✅ Bug #43: `mut` inference para métodos como `push`/`pop` en instancias
- ✅ Bug #47-49: Parallel `filter`/`reduce` con patrones de referencia correctos
- ✅ Bug #50: Regular `filter()` con primitivos y dereference
- ✅ Bug #51: Array indexing + field access (`results[0].value`)

*v0.11.24 - Division & Template fixes:*
- ✅ Bug #52: División int/int con retorno float ahora castea correctamente
- ✅ Bug #53: Field access en string templates con arrays

*v0.11.25 - Generic Bounds Inference:*
- ✅ Bug #41: `pop()` añade `.expect()` automáticamente
- ✅ Bug #42: Generic array indexing envuelve en paréntesis: `(len-1) as usize`
- ✅ Bug #44: Trait `Eq` usa `Clone` en vez de `Copy`
- ✅ Bug #45-46: Inferencia automática de `Clone` bound para métodos que retornan T
- ✅ Bug #54: Inferencia automática de `Display` bound para templates con campos genéricos

---

## 📁 Estructura del Workspace

```
livac-project/
├── livac/                    # 🔧 Compilador de Liva (Rust)
├── vscode-extension/         # 🎨 Extensión VS Code/Cursor (TypeScript)
├── github-dashboard/         # 📊 Demo project (escrito en Liva)
└── github-dashboard-real/    # 📊 Real API version
```

---

## 🔧 Proyecto: livac (Compilador)

### Información General
| Campo | Valor |
|-------|-------|
| **Lenguaje** | Rust |
| **Versión actual** | v1.1.0-dev (tag: v1.0.2) |
| **Autor** | Fran Nadal |
| **Build** | `cargo build --release` |
| **Test** | `cargo test` |
| **Ejecutar** | `livac archivo.liva --run` |
| **Formatear** | `livac archivo.liva --fmt` |

### Arquitectura del Compilador

```
src/
├── main.rs           # CLI y punto de entrada
├── lib.rs            # API pública del compilador
├── lexer.rs          # Tokenización (logos)
├── parser.rs         # Parser (chumsky) → AST
├── ast.rs            # Definición del AST
├── semantic.rs       # Análisis semántico y tipos
├── desugaring.rs     # Transformaciones sintácticas
├── ir.rs             # Representación intermedia
├── lowering.rs       # AST → IR
├── codegen.rs        # IR → Código Rust (~400KB, archivo principal)
├── formatter.rs      # Code formatter (Phase 10) 🆕
├── module.rs         # Sistema de módulos e imports
├── traits.rs         # Sistema de traits/interfaces
├── error.rs          # Sistema de errores
├── error_codes.rs    # Códigos de error (E0xxx)
├── hints.rs          # Sugerencias de errores
├── suggestions.rs    # "Did you mean?" (Levenshtein)
├── span.rs           # Ubicaciones en código fuente
├── liva_rt.rs        # Runtime de Liva para código generado
└── lsp/              # Language Server Protocol
    ├── mod.rs        # Declaraciones del módulo LSP
    ├── server.rs     # Servidor LSP principal (tower-lsp)
    ├── document.rs   # Gestión de documentos
    ├── symbols.rs    # Tabla de símbolos
    ├── diagnostics.rs # Conversión de errores → diagnósticos
    ├── imports.rs    # Resolución de imports
    └── workspace.rs  # Gestión del workspace
```

### Dependencias Principales
- `logos` - Lexer
- `chumsky` - Parser combinators
- `tower-lsp` - Servidor LSP
- `tokio` - Runtime async
- `serde/serde_json` - Serialización
- `clap` - CLI

### Features del Lenguaje Implementados
- ✅ Variables (`let`, `const`) con inferencia de tipos
- ✅ Top-level `const` declarations 🆕
- ✅ Funciones (one-liner `=>`, bloques, tipadas)
- ✅ Clases (constructores, campos, métodos)
- ✅ Interfaces (firmas, implementación múltiple)
- ✅ Control de flujo (`if`, `while`, `for`, `switch`)
- ✅ **One-liner `=>` para if/for/while** (v1.1.0) 🆕
- ✅ Templates de strings con interpolación
- ✅ Generics (`Box<T>`, `Pair<T,U>`)
- ✅ Tuples con destructuring `(a, b)`
- ✅ Type aliases `type UserId = int`
- ✅ Union types `int | string` con pattern matching
- ✅ Destructuring (arrays, objetos, parámetros)
- ✅ Async/await para I/O
- ✅ Parallel execution para CPU
- ✅ Manejo de errores (`fail`, error binding)
- ✅ **`or fail` operator** (v1.1.0) 🆕
- ✅ Sistema de módulos (`import/from/export`)
- ✅ HTTP Client, File I/O, JSON
- ✅ LSP completo (v0.12.0)
- ✅ **Code Formatter** (Phase 10) 🆕

### 📖 Sintaxis del Lenguaje (Quick Reference)

> **Referencia completa**: [`livac/docs/QUICK_REFERENCE.md`](livac/docs/QUICK_REFERENCE.md)

#### Variables
```liva
let x = 10              // Mutable
const PI = 3.14159      // Inmutable
let name: string = "Alice"  // Con tipo explícito
let data = null         // Valor nulo
```

#### Funciones
```liva
// One-liner
greet() => print("Hello!")
add(a, b) => a + b
square(x: number): number => x * x

// Bloque
calculate(a: number, b: number): number {
    let result = a + b * 2
    return result
}
```

#### Control de Flujo
```liva
// If/else
if condition { } else if other { } else { }

// Ternario
let status = age >= 18 ? "adult" : "minor"

// Switch (pattern matching)
let result = switch value {
    0 => "zero",
    1 | 2 | 3 => "small",     // Or-pattern
    4..=10 => "medium",       // Range
    n if n > 100 => "huge",   // Guard
    _ => "other"              // Wildcard
}
```

#### Loops
```liva
while i < 10 { i = i + 1 }
for i in 0..10 { print(i) }      // 0 a 9
for i in 1..=10 { print(i) }     // 1 a 10 (inclusive)
for item in array { print(item) }
break    // Salir del loop
continue // Siguiente iteración

// One-liner => syntax (v1.1.0) 🆕
if age >= 18 => print("Adult")
for item in items => print(item)
while running => tick()
```

#### Clases
```liva
Person {
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    
    name: string       // Campo público
    _secret: string    // Campo privado (prefijo _)
    
    greet() => print($"Hi, I'm {this.name}")
    isAdult(): bool => this.age >= 18
}

let person = Person("Alice", 25)
person.greet()
```

#### Interfaces
```liva
Animal {
    makeSound(): string
    getName(): string
}

Dog : Animal {
    constructor(name: string) { this.name = name }
    name: string
    makeSound() => "Woof!"
    getName() => this.name
}
```

#### Error Handling
```liva
// Fail para errores
divide(a, b) {
    if b == 0 { fail "Cannot divide by zero" }
    return a / b
}

// Error binding
let result, err = divide(10, 0)
if err { print($"Error: {err}") }

// or fail operator (v1.1.0) 🆕 — shorthand error propagation
let response = HTTP.get(url) or fail "Connection error"
let content = File.read("config.json") or fail "Cannot read config"
let data = JSON.parse(text) or fail "Invalid JSON"
```

#### Concurrencia
```liva
let data = async fetchAPI()     // I/O async (auto-await)
let result = par heavyCalc()    // CPU parallel (auto-join)
let t = task async operation()  // Task handle
let value = await t             // Await explícito
fire async logEvent()           // Fire-and-forget
```

#### Arrays y Funcionales
```liva
let nums = [1, 2, 3, 4, 5]
nums.map(x => x * 2)              // [2, 4, 6, 8, 10]
nums.filter(x => x % 2 == 0)      // [2, 4]
nums.reduce((acc, x) => acc + x, 0) // 15
nums.find(x => x > 3)             // Some(4)
nums.some(x => x > 3)             // true
nums.every(x => x > 0)            // true
```

#### String Templates
```liva
let name = "Alice"
let greeting = $"Hello, {name}!"
let calc = $"Result: {a + b}"
```

#### Módulos
```liva
// math.liva
add(a, b) => a + b       // Público (exportado)
_helper(x) => x * 2      // Privado (prefijo _)

// main.liva
import { add } from "./math.liva"
import * as math from "./utils.liva"
```

#### Standard Library
```liva
// Console
print("Hello")
console.log(data)
console.error("Error!")
let input = console.input("Name: ")

// Math
Math.sqrt(16.0)  Math.pow(2.0, 3.0)  Math.abs(-5)
Math.floor(3.7)  Math.ceil(3.2)       Math.random()

// File I/O
let content, err = File.read("file.txt")
let ok, err = File.write("out.txt", "data")
let exists = File.exists("file.txt")

// JSON
let data: [number], err = JSON.parse("[1,2,3]")
let json = JSON.stringify(data)

// HTTP
let resp, err = async HTTP.get("https://api.example.com")
let data, err = resp.json()
```

#### Tipos
| Liva | Rust | Descripción |
|------|------|-------------|
| `number` | `i32` | Entero 32-bit |
| `float` | `f64` | Flotante 64-bit |
| `bool` | `bool` | Booleano |
| `string` | `String` | Cadena UTF-8 |
| `char` | `char` | Carácter Unicode |
| `[T]` | `Vec<T>` | Array/Vector |

#### Operadores
```
Aritméticos:  + - * / %
Comparación:  == != < > <= >=
Lógicos:      and or not (también && || !)
Rangos:       .. (exclusivo)  ..= (inclusivo)
Arrow:        =>
Ternario:     ? :
```

### Comandos Útiles
```bash
# Compilar
cargo build --release

# Ejecutar tests
cargo test

# Compilar un archivo .liva
./target/release/livac archivo.liva

# Compilar y ejecutar
./target/release/livac archivo.liva --run

# Verificar sintaxis
./target/release/livac archivo.liva --check

# Iniciar servidor LSP
./target/release/livac --lsp
```

---

## 🎨 Proyecto: vscode-extension

### Información General
| Campo | Valor |
|-------|-------|
| **Lenguaje** | TypeScript |
| **Versión actual** | 0.12.0 |
| **Publisher** | liva-lang |
| **Build** | `npm run compile` |
| **Package** | `vsce package` |
| **Instalar local** | `code --install-extension liva-vscode-*.vsix` |

### Arquitectura de la Extensión

```
src/
├── extension.ts          # Punto de entrada, activación
├── lspClient.ts          # Cliente LSP (vscode-languageclient)
└── providers/
    ├── completionProvider.ts    # Autocompletado
    ├── definitionProvider.ts    # Go to Definition + References
    ├── hoverProvider.ts         # Hover information
    ├── signatureHelpProvider.ts # Firma de funciones
    ├── symbolProvider.ts        # Document symbols
    ├── interfaceValidator.ts    # Validación de interfaces
    └── fallibleValidator.ts     # Validación de funciones fallibles
```

### Archivos de Configuración
```
├── package.json              # Manifiesto de la extensión
├── language-configuration.json # Brackets, comments, etc.
├── syntaxes/
│   └── liva.tmLanguage.json  # Syntax highlighting (TextMate)
└── snippets/
    └── liva.json             # Code snippets
```

### Características
- ✅ Syntax highlighting completo
- ✅ 60+ snippets de código
- ✅ Cliente LSP integrado
- ✅ Autocompletado inteligente
- ✅ Go to Definition (F12)
- ✅ Find References (Shift+F12)
- ✅ Hover con tipos
- ✅ Diagnósticos en tiempo real
- ✅ Quick fixes ("Did you mean?")
- ✅ Validación de interfaces
- ✅ Validación de funciones fallibles
- ✅ Signature help

### Comandos Útiles
```bash
# Instalar dependencias
npm install

# Compilar
npm run compile

# Watch mode
npm run watch

# Empaquetar
npx vsce package

# Instalar extensión local
code --install-extension liva-vscode-0.12.0.vsix
```

---

## 🔗 Relación entre Proyectos

```
┌─────────────────────────────────────────────────────────────┐
│                      VS Code / Cursor                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │            vscode-extension (TypeScript)             │    │
│  │  • Syntax highlighting                               │    │
│  │  • Snippets                                          │    │
│  │  • LSP Client ◄──────────────────────┐              │    │
│  └──────────────────────────────────────┼──────────────┘    │
└─────────────────────────────────────────┼───────────────────┘
                                          │ JSON-RPC (stdio)
                                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    livac (Rust)                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              LSP Server (tower-lsp)                  │    │
│  │  • Completion, Diagnostics, Hover                    │    │
│  │  • Go to Definition, Find References                 │    │
│  └──────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Compiler Pipeline                       │    │
│  │  Lexer → Parser → Semantic → IR → Codegen → Rust    │    │
│  └──────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 Estado del Desarrollo

### Versiones Actuales
| Proyecto | Versión | Estado |
|----------|---------|--------|
| livac | v1.1.0-dev (tag: v1.0.2) | 🚀 Phase 11.1 & 11.2 complete (or fail + => one-liners) |
| vscode-extension | v0.12.0 | LSP integration complete |

### Releases Publicados
- **v1.0.2** - CI modernizado + multi-platform releases
- **v1.0.0** - First stable release! 54/54 bugs fixed

### Próximos Pasos (del ROADMAP)
1. ✅ ~~**Phase 10**: Code Formatter~~ - COMPLETADO
2. ✅ ~~**Phase 11.1**: `or fail` operator~~ - COMPLETADO
3. ✅ ~~**Phase 11.2**: `=>` one-liners para if/for/while~~ - COMPLETADO
4. **Phase 11.3**: Point-free / function references
5. **Opcional**: Debugging support (DAP), async generators, macros

---

## 🛠️ Flujo de Desarrollo Típico

### Cambios en el Compilador (livac)
1. Editar archivos en `livac/src/`
2. `cargo build` para compilar
3. `cargo test` para verificar
4. Probar con archivos `.liva` de ejemplo
5. Si afecta LSP, probar en VS Code

### Cambios en la Extensión (vscode-extension)
1. Editar archivos en `vscode-extension/src/`
2. `npm run compile` para compilar
3. F5 en VS Code para abrir Extension Development Host
4. Probar la funcionalidad
5. `npx vsce package` para empaquetar

### Cambios que Afectan Ambos
1. Actualizar el compilador primero
2. Recompilar con `cargo build --release`
3. Copiar binario: `cp target/release/livac ../vscode-extension/bin/`
4. Actualizar la extensión si es necesario
5. Probar integración completa

---

## 📝 Archivos Importantes para Contexto

### En livac/
- `ROADMAP.md` - Plan completo del proyecto
- `CHANGELOG.md` - Historial de cambios detallado
- `README.md` - Documentación del lenguaje
- `docs/QUICK_REFERENCE.md` - **Referencia rápida de sintaxis** ⭐
- `docs/` - Documentación completa (~27 archivos)
- `BUGS.md` - **Bugs encontrados/corregidos del dogfooding**

### En vscode-extension/
- `README.md` - Features de la extensión
- `CHANGELOG.md` - Historial de la extensión
- `package.json` - Configuración y capabilities

---

## ⚠️ Notas Importantes

1. **El archivo más grande** es `codegen.rs` (~400KB, ~11000 líneas) - contiene toda la generación de código Rust
2. **Nuevo**: `formatter.rs` para formateo de código (Phase 10)
3. **LSP usa stdio** para comunicación entre extensión y compilador
4. **Los tipos se infieren** pero pueden anotarse explícitamente
5. **Error binding** con `let value, err = expr` para manejo de errores
6. **`or fail`** (v1.1.0) simplifica propagación de errores
7. **One-liner `=>`** (v1.1.0) para if/for/while de una sola expresión
8. **Concurrencia híbrida**: `async/await` para I/O, `par` para CPU
9. **CI automatizado** con releases para Linux, macOS (x64+ARM), Windows

---

## 🔄 Última Actualización

- **Fecha**: 2026-02-11
- **Evento**: 🚀 Phase 11.1 & 11.2 Complete (v1.1.0 features)
- **Versión**: v1.1.0-dev (tag: v1.0.2)
- **Features completadas desde v1.0.0**:
  - ✅ **Phase 10**: Code Formatter (`--fmt`, `--fmt-check`, LSP formatting)
  - ✅ **Phase 11.1**: `or fail` operator para propagación de errores
  - ✅ **Phase 11.2**: `=>` one-liners para if/for/while
  - ✅ Top-level `const` declarations
  - ✅ `not`/`&&`/`||` operators (además de `and`/`or`/`!`)
  - ✅ CI modernizado con multi-platform releases (Linux, macOS x64/ARM, Windows)
- **Archivos nuevos**:
  - `src/formatter.rs` - Code formatter completo
  - `.github/workflows/release.yml` - Multi-platform release workflow
- **Tests**:
  - `tests/integration/test_v1_1_features.liva` - Tests de Phase 11
- **Próximo paso**: Phase 11.3 (Point-free references) o v1.1.0 release
