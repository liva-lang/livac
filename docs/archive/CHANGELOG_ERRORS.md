# Changelog - Sistema de Errores Liva v0.6

## [0.6.1] - 2025-10-17

### 🎉 Nueva Funcionalidad: Sistema de Errores Completo

#### ✨ Mejoras Principales

##### 1. **Errores Estructurados con Códigos Únicos**
- Implementado sistema de códigos de error categorizados:
  - `E1xxx` - Errores de Lexer
  - `E2xxx` - Errores de Parser
  - `E0xxx` - Errores Semánticos
  - `E3xxx` - Errores de Codegen
- Cada error tiene un código único e identificable

##### 2. **Ubicación Precisa del Error**
- Los errores ahora muestran:
  - Archivo
  - Línea
  - Columna exacta
  - Snippet del código problemático
- Indicador visual (`^^^`) apuntando al error exacto

##### 3. **Mensajes Mejorados**
- Título descriptivo del error
- Mensaje detallado con contexto
- Sugerencias útiles (💡) para resolver el problema
- Formato colorizado en terminal

##### 4. **Salida JSON para IDEs**
- Flag `--json` para obtener errores en formato estructurado
- Facilita integración con editores e IDEs
- Formato:
  ```json
  {
    "location": {"file": "...", "line": 6, "column": 7, "source_line": "..."},
    "code": "E0001",
    "title": "...",
    "message": "...",
    "help": "..."
  }
  ```

##### 5. **Integración con VS Code**
- Validación en tiempo real mientras escribes
- Subrayado rojo en la ubicación exacta del error
- Tooltips con información completa del error
- Panel de problemas actualizado automáticamente
- Debounce de 500ms para evitar validación excesiva

#### 📝 Códigos de Error Implementados

##### Lexer (E1xxx)
- **E1000** - Invalid Token: Token no reconocido en el código

##### Parser (E2xxx)
- **E2000** - Parse Error: Error general de parsing
- **E2001** - Unclosed Interpolation: Interpolación de string sin cerrar
- **E2002** - Empty Interpolation: Interpolación vacía en string template
- **E2003** - Unmatched Closing Brace: `}` sin pareja en string template

##### Semantic (E0xxx)
- **E0001** - Variable Already Defined: Variable declarada múltiples veces
- **E0002** - Constant Already Defined: Constante declarada múltiples veces

##### Codegen (E3xxx)
- **E3000** - Invalid Binding Pattern: Patrón de binding inválido
- **E3001** - No Rust Code Generated: Error interno de generación
- **E3002** - No Cargo.toml Generated: Error interno de configuración

#### 🔧 Cambios Técnicos

##### Compiler (Rust)
- Modificado `CompilerError` para usar `SemanticErrorInfo` en todos los errores estructurados
- Actualizado lexer para generar errores con ubicación precisa
- Mejorado parser con método `error_with_help()` para errores con sugerencias
- Semantic analyzer ya usaba el sistema mejorado, se mantiene igual
- Exportados tipos necesarios en `lib.rs`: `SemanticErrorInfo`, `ErrorLocation`

##### Extension (TypeScript)
- Mejorada función `createDiagnosticFromJson()` para usar información de columna
- Ajustado rango de diagnóstico para resaltar ~3 caracteres desde la posición del error
- Mantenido debounce de 500ms en validación en tiempo real

##### CLI
- Flag `--json` ya existía, se mejoró para suprimir mensajes de progreso
- Mejorado manejo de errores para usar `error.to_json()` en todos los casos

#### 📚 Documentación Nueva

1. **ERROR_CODES.md** - Lista completa de códigos de error con:
   - Descripción de cada error
   - Causas comunes
   - Ejemplos de código
   - Soluciones sugeridas

2. **ERROR_SYSTEM.md** - Documentación completa del sistema:
   - Arquitectura del sistema de errores
   - Formato de salida
   - Ejemplos de uso
   - Guía para contribuir
   - Integración con IDEs

3. **README.md** actualizado con:
   - Sección sobre el sistema de errores
   - Enlaces a documentación
   - Ejemplos visuales

4. **vscode-extension/README.md** actualizado con:
   - Características de error reporting
   - Ejemplos de errores en el editor
   - Configuración de validación

#### 🎨 Formato de Error Ejemplo

```
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → test.liva:6:7

     6 │
       │ let x = 20
       │     ^^^
       │

  ⓘ Variable 'x' already defined in this scope

  💡 Consider using a different name or removing the previous declaration of 'x'
────────────────────────────────────────────────────────────
```

#### 🧪 Testing

- Creado `test_errors.liva` - Archivo de prueba con error de variable duplicada
- Creado `test_semantic_errors.liva` - Pruebas de errores semánticos
- Verificado funcionamiento en:
  - ✅ Terminal con colores
  - ✅ Salida JSON con `--json`
  - ✅ VS Code con subrayado rojo y tooltips

#### 🚀 Mejoras Futuras Planeadas

- [ ] Agregar más códigos de error específicos para todos los casos
- [ ] Soporte para múltiples errores en una compilación
- [ ] Quick fixes automáticos en VS Code
- [ ] Sugerencias basadas en similitud ("¿quisiste decir...?")
- [ ] Documentación inline en tooltips

#### 🐛 Bugs Corregidos

- Errores de lexer ahora tienen ubicación precisa
- Errores de parser muestran columna correcta
- Salida JSON correctamente formateada sin mensajes extra
- Extensión de VS Code muestra ubicación exacta del error

#### ⚡ Mejoras de Rendimiento

- Debounce de 500ms en validación en tiempo real evita sobrecarga
- Archivos temporales se limpian correctamente después de validación
- Validación solo se ejecuta si `liveValidation` está habilitado

#### 📦 Archivos Modificados

**Compiler:**
- `src/error.rs` - Tipo de error unificado
- `src/lexer.rs` - Errores con ubicación
- `src/parser.rs` - Errores mejorados con ayuda
- `src/codegen.rs` - Error estructurado
- `src/main.rs` - Soporte JSON mejorado
- `src/lib.rs` - Exports públicos

**Extension:**
- `src/extension.ts` - Parsing JSON mejorado
- `README.md` - Documentación actualizada

**Documentation:**
- `docs/ERROR_CODES.md` - Nueva
- `docs/ERROR_SYSTEM.md` - Nueva
- `README.md` - Actualizado

**Tests:**
- `test_errors.liva` - Nuevo
- `test_semantic_errors.liva` - Nuevo
- `test_all_errors.liva` - Nuevo

---

### Notas de Migración

Si tienes código que maneja errores del compilador:

**Antes:**
```rust
CompilerError::LexerError(String)
CompilerError::ParseError { line, col, msg }
```

**Ahora:**
```rust
CompilerError::LexerError(SemanticErrorInfo)
CompilerError::ParseError(SemanticErrorInfo)
```

Los errores ahora son más ricos en información y siguen una estructura consistente.

---

### Agradecimientos

Este sistema de errores fue diseñado inspirándose en compiladores modernos como:
- Rust compiler (rustc)
- TypeScript compiler (tsc)
- Elm compiler

Gracias a la comunidad por el feedback y sugerencias. 🙏

---

**Autor:** Fran Nadal  
**Fecha:** 17 de octubre de 2025  
**Versión:** 0.6.1
