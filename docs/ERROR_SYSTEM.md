# Sistema de Errores Mejorado - Liva Compiler v0.6

## 🎯 Resumen

El compilador Liva ahora cuenta con un sistema de errores completamente mejorado que proporciona:

- ✅ **Códigos de error únicos** (E1xxx, E2xxx, E0xxx, E3xxx)
- ✅ **Ubicación precisa** (archivo, línea y columna)
- ✅ **Snippet de código** con indicador visual
- ✅ **Mensajes descriptivos** con contexto
- ✅ **Sugerencias útiles** para resolver el error
- ✅ **Formato colorizado** en terminal
- ✅ **Salida JSON** para integración con IDEs
- ✅ **Integración con VS Code** con subrayado rojo en tiempo real

## 📋 Características

### 1. Formato Terminal Mejorado

```
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → test_errors.liva:6:7

     6 │
       │ let x = 20
       │     ^^^
       │

  ⓘ Variable 'x' already defined in this scope

  💡 Consider using a different name or removing the previous declaration of 'x'
────────────────────────────────────────────────────────────
```

**Componentes:**
- 🔴 Código de error y título en rojo
- 📍 Ubicación exacta (archivo:línea:columna)
- 📝 Snippet del código problemático
- 👉 Indicador visual (^^^) apuntando al error
- ℹ️ Mensaje descriptivo
- 💡 Sugerencia de solución

### 2. Formato JSON para IDEs

```bash
livac file.liva --check --json
```

Salida:
```json
{
  "location": {
    "file": "file.liva",
    "line": 6,
    "column": 7,
    "source_line": "  let x = 20"
  },
  "code": "E0001",
  "title": "Variable 'x' already defined in this scope",
  "message": "Variable 'x' already defined in this scope",
  "help": "Consider using a different name or removing the previous declaration of 'x'"
}
```

### 3. Integración con VS Code

La extensión de Liva para VS Code aprovecha el sistema de errores para:

- ✅ **Validación en tiempo real** mientras escribes código
- ✅ **Subrayado rojo** en la ubicación exacta del error
- ✅ **Tooltip informativo** al pasar el mouse sobre el error
- ✅ **Panel de problemas** con lista de todos los errores
- ✅ **Navegación rápida** a los errores con Ctrl+Click

## 🏗️ Arquitectura

### Estructura de Error

```rust
pub struct SemanticErrorInfo {
    pub location: Option<ErrorLocation>,
    pub code: String,
    pub title: String,
    pub message: String,
    pub help: Option<String>,
}

pub struct ErrorLocation {
    pub file: String,
    pub line: usize,
    pub column: Option<usize>,
    pub source_line: Option<String>,
}
```

### Tipos de Error

```rust
pub enum CompilerError {
    LexerError(SemanticErrorInfo),      // E1xxx
    ParseError(SemanticErrorInfo),      // E2xxx
    SemanticError(SemanticErrorInfo),   // E0xxx
    TypeError(SemanticErrorInfo),       // E4xxx (reservado)
    CodegenError(SemanticErrorInfo),    // E3xxx
    IoError(String),                    // No estructurado
    RuntimeError(String),               // No estructurado
}
```

## 📚 Códigos de Error

### Categorías

- **E1xxx** - Errores de Lexer (tokens inválidos)
- **E2xxx** - Errores de Parser (sintaxis incorrecta)
- **E0xxx** - Errores Semánticos (lógica del programa)
- **E3xxx** - Errores de Codegen (generación de código)
- **E4xxx** - Errores de Tipos (reservado para sistema de tipos futuro)

Ver [ERROR_CODES.md](ERROR_CODES.md) para la lista completa de códigos.

## 🧪 Ejemplos de Uso

### Compilación Normal

```bash
livac program.liva
```

Salida con error:
```
🧩 Liva Compiler v0.6
→ Compiling program.liva
Error: 
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → program.liva:6:7
  ...
```

### Modo Check (sin compilar)

```bash
livac program.liva --check
```

### Modo JSON (para IDEs)

```bash
livac program.liva --check --json
```

Salida: JSON estructurado para parsing automático

### Modo Verbose

```bash
livac program.liva --verbose
```

Muestra el código Rust generado además de errores.

## 🔧 Implementación

### En el Compilador (Rust)

```rust
// Crear error con ubicación
let error = self.error_with_span(
    "E0001",
    "Variable already defined",
    &format!("Variable '{}' already defined in this scope", name),
    Some(span)
);

// Con sugerencia
let error = error.with_help(
    "Consider using a different name or removing the previous declaration"
);

// Retornar error
return Err(CompilerError::SemanticError(error));
```

### En la Extensión (TypeScript)

```typescript
// La extensión automáticamente:
// 1. Ejecuta: livac file.liva --check --json
// 2. Parsea el JSON de error
// 3. Crea diagnóstico con ubicación precisa
// 4. Muestra subrayado rojo en el editor
// 5. Actualiza panel de problemas
```

## 📈 Mejoras Futuras

### Corto Plazo
- [ ] Agregar más códigos de error específicos
- [ ] Mejorar mensajes de error existentes
- [ ] Agregar más sugerencias de corrección
- [ ] Soporte para múltiples errores en una compilación

### Largo Plazo
- [ ] Sistema de fix automático (Quick Fixes)
- [ ] Errores con contexto multi-línea
- [ ] Sugerencias basadas en similitud (¿quisiste decir...?)
- [ ] Errores con código de ejemplo correcto
- [ ] Documentación inline en tooltips

## 🤝 Contribuir

Para agregar un nuevo error:

1. **Asignar código:** Elige el código apropiado según la categoría
2. **Crear error estructurado:** Usa `SemanticErrorInfo::new()`
3. **Agregar ubicación:** Usa `.with_location()` y `.with_column()`
4. **Agregar snippet:** Usa `.with_source_line()`
5. **Agregar ayuda:** Usa `.with_help()`
6. **Documentar:** Actualiza ERROR_CODES.md
7. **Testear:** Crea un test case

### Ejemplo Completo

```rust
let span = var.span.unwrap();
let (line, column) = span.start_position(&self.source_map);
let source_line = self.get_source_line(line);

let error = SemanticErrorInfo::new(
    "E0001",
    "Variable already defined",
    &format!("Variable '{}' already defined in this scope", name)
)
.with_location(&self.source_file, line)
.with_column(column)
.with_source_line(source_line.unwrap_or_default())
.with_help("Consider using a different name or removing the previous declaration");

return Err(CompilerError::SemanticError(error));
```

## 🎨 Personalización

### Desactivar Colores

```bash
NO_COLOR=1 livac program.liva
```

### Formato Compacto (solo en JSON)

```bash
livac program.liva --json
```

### Desactivar Validación en VS Code

En settings.json:
```json
{
  "liva.liveValidation": false
}
```

## 📖 Referencias

- [ERROR_CODES.md](ERROR_CODES.md) - Lista completa de códigos de error
- [error_messages_improvements.md](error_messages_improvements.md) - Notas de implementación
- [VSCODE_INTEGRATION.md](../vscode-extension/VSCODE_INTEGRATION.md) - Integración con VS Code

---

**Versión:** 0.6  
**Última actualización:** Octubre 2025  
**Estado:** ✅ Implementado y funcional
