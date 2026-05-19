# Test Fixtures

Archivos `.liva` utilizados por los tests del compilador.

## 📁 Estructura

```
fixtures/
├── features/           # Tests por característica del lenguaje
│   ├── basic/          # Variables, loops, funciones básicas
│   ├── generics/       # Genéricos, traits, constraints
│   ├── http-json/      # HTTP client, JSON parsing
│   ├── manual/         # Tests manuales variados
│   ├── modules/        # Sistema de módulos, imports
│   ├── pattern-matching/  # Switch, exhaustiveness, tuples
│   └── stdlib/         # Métodos de array/string
├── integration/        # Proyectos multi-archivo
├── lsp/                # Tests del Language Server
└── regression/         # Tests de bugs corregidos
```

## 🔗 Convención de Nombres

- `ok_*.liva` - Debe compilar sin errores
- `err_*.liva` - Debe fallar con error específico  
- `test_*.liva` - Test general de característica
- `bug*.liva` - Regresión de bug corregido

## ▶️ Ejecutar Tests

```bash
# Todos los tests
cargo test

# Tests específicos
cargo test lexer
cargo test parser
cargo test semantics
cargo test codegen
cargo test integration
```

## 📝 Agregar Nuevos Fixtures

1. Crear archivo en la carpeta apropiada
2. Si es un bug fix, agregarlo a `regression/` con número de issue
3. Actualizar el test de Rust correspondiente si es necesario
