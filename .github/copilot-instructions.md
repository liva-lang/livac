# 🔧 Liva Compiler Context

> **Proyecto:** livac - El compilador de Liva  
> **Versión:** v1.1.0-dev (tag: v1.0.2)  
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
├── codegen.rs        # IR → Código Rust (~400KB, ~11000 líneas)
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
- **Phase 11.3** (Point-free): 📋 Pendiente

---

## ⚠️ Notas para Desarrollo

1. **codegen.rs** es el archivo más grande (~11000 líneas) - contiene toda la generación de Rust
2. **formatter.rs** maneja el formateo de código
3. Los tests están en `tests/` y se ejecutan con `cargo test`
4. El LSP se comunica por stdio con la extensión VS Code
