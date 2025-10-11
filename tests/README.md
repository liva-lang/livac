# Estrategia de Tests para Liva

Este directorio contiene una estrategia completa de testing por capas para el compilador de Liva, cubriendo desde el lexer hasta la integración con Cargo.

## Estructura de Tests

```
tests/
├── lexer/           # Tests del lexer (tokenización)
├── parser/          # Tests del parser (AST)
├── semantics/       # Tests de análisis semántico
├── desugar/         # Tests de desugaring
├── integration/     # Tests end-to-end
├── snapshots/       # Snapshots de insta (auto-generados)
└── *.rs            # Harnesses de testing
```

## Capas de Testing

### 1. Lexer Tests (`lexer_tests.rs`)

**Casos correctos:**
- `ok_identifiers.liva` - Identificadores básicos, protegidos, privados
- `ok_literals.liva` - Literales (int, float, string, char, bool)
- `ok_operators.liva` - Operadores aritméticos y lógicos
- `ok_keywords.liva` - Palabras clave y control de flujo
- `ok_comments.liva` - Comentarios de línea y bloque

**Casos de error:**
- `err_unknown_token.liva` - Token desconocido
- `err_unclosed_string.liva` - String no cerrada
- `err_unclosed_char.liva` - Char no cerrado
- `err_unclosed_comment.liva` - Comentario no cerrado

### 2. Parser Tests (`parser_tests.rs`)

**Casos correctos:**
- `ok_functions_oneliner.liva` - Funciones de una línea
- `ok_functions_block.liva` - Funciones con bloque
- `ok_classes.liva` - Clases con visibilidad
- `ok_control_flow.liva` - if/else, while, for, switch
- `ok_expressions.liva` - Expresiones complejas
- `ok_imports.liva` - Imports y uso de crates

**Casos de error:**
- `err_unclosed_paren.liva` - Paréntesis no cerrado
- `err_unclosed_brace.liva` - Llave no cerrada
- `err_case_without_switch.liva` - Case sin switch
- `err_return_outside_function.liva` - Return fuera de función
- `err_duplicate_default.liva` - Default duplicado

### 3. Semantics Tests (`semantics_tests.rs`)

**Casos correctos:**
- `ok_type_inference.liva` - Inferencia de tipos
- `ok_async_inference.liva` - Inferencia de async
- `ok_visibility.liva` - Reglas de visibilidad
- `ok_rust_types.liva` - Tipos Rust nativos

**Casos de error:**
- `err_number_plus_float.liva` - Mezcla de tipos
- `err_private_access.liva` - Acceso a privado
- `err_protected_access.liva` - Acceso a protegido
- `err_undefined_type.liva` - Tipo no definido
- `err_async_without_await.liva` - Async sin await

### 4. Desugar Tests (`desugar_tests.rs`)

**Casos correctos:**
- `ok_functions_oneliner.liva` - Funciones → fn
- `ok_classes.liva` - Clases → struct + impl
- `ok_async_parallel_fire.liva` - Concurrencia → tokio
- `ok_string_templates.liva` - Templates → format!
- `ok_rust_crates.liva` - use rust → Cargo.toml

### 5. Integration Tests (`integration_tests.rs`)

**Proyectos completos:**
- `proj_hello/` - Hello World básico
- `proj_async/` - Proyecto con async
- `proj_classes/` - Proyecto con clases

Cada proyecto se compila completamente y verifica que:
- Se generan `main.rs` y `Cargo.toml`
- El código Rust es válido
- La estructura del proyecto es correcta

### 6. Property Tests (`property_tests.rs`)

**Tests de idempotencia:**
- `parse → pretty → parse` debe ser idempotente
- Robustez ante entradas aleatorias
- No panics con código válido

## Herramientas Utilizadas

### Insta (Snapshots)
```bash
# Actualizar snapshots después de cambios
cargo insta review
cargo insta accept
```

### Proptest (Property Testing)
```bash
# Ejecutar property tests con más casos
PROPTEST_CASES=10000 cargo test property_tests
```

### Cargo Tarpaulin (Coverage)
```bash
# Instalar
cargo install cargo-tarpaulin

# Ejecutar con coverage
cargo tarpaulin --out Html
```

## Ejecutar Tests

### Todos los tests
```bash
./run_tests.sh
```

### Tests específicos
```bash
# Solo lexer
cargo test --test lexer_tests

# Solo integración
cargo test --test integration_tests

# Con output detallado
cargo test -- --nocapture
```

### Actualizar snapshots
```bash
cargo insta review
cargo insta accept
```

## Convenciones

### Naming
- `ok_*.liva` - Casos correctos
- `err_*.liva` - Casos de error
- `*.snap` - Snapshots de insta
- `*.diag` - Diagnósticos esperados

### Estructura de archivos
- Un archivo `.liva` por caso de prueba
- Snapshots auto-generados por insta
- Diagnósticos en texto plano con rangos

### Harnesses
- Un archivo `*_tests.rs` por capa
- Helpers comunes para casos ok/err
- Assertions específicas por tipo de test

## CI/CD

Los tests están diseñados para ejecutarse en CI:

```yaml
# GitHub Actions example
- name: Run tests
  run: |
    cargo test --all-features
    cargo insta review --accept
```

## Métricas de Calidad

- **Cobertura de código**: >90% en módulos críticos
- **Casos de borde**: Cubiertos en cada capa
- **Regresiones**: Detectadas por snapshots
- **Robustez**: Verificada por property tests

## Extensión

Para añadir nuevos tests:

1. Crear archivo `.liva` en la carpeta correspondiente
2. Añadir test case en el harness
3. Ejecutar y aceptar snapshots si es necesario
4. Verificar que pasa en CI

Esta estrategia asegura que el compilador de Liva sea robusto, mantenible y libre de regresiones.
