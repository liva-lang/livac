# Mejoras en Mensajes de Error - Liva Compiler

## Resumen

Esta rama (`improve-error-messages`) implementa mejoras significativas en el sistema de mensajes de error del compilador Liva, proporcionando información más precisa y útil sobre los errores de compilación.

## Cambios Implementados

### 1. Tracking de Ubicación en el AST

Se añadió información de `Span` (ubicación en el código fuente) a las estructuras del AST:

- **`VarBinding`**: Ahora incluye `span: Option<crate::span::Span>` para rastrear la ubicación de cada variable declarada
- **`ConstDecl`**: Ahora incluye `span: Option<crate::span::Span>` para rastrear la ubicación de cada constante

### 2. Captura de Spans en el Parser

El parser (`src/parser.rs`) fue actualizado para:

- Añadir métodos auxiliares `current_span()` y `previous_span()` para obtener la ubicación del token actual/anterior
- Capturar y asignar spans al crear `VarBinding` y `ConstDecl`
- Preservar la información de ubicación durante el parsing

### 3. Mejoras en el Análisis Semántico

El `SemanticAnalyzer` (`src/semantic.rs`) ahora:

- Utiliza el método `error_with_span()` para crear errores con información de ubicación precisa
- Genera códigos de error específicos:
  - `E0001`: Variable ya definida en el ámbito actual
  - `E0002`: Constante ya definida en el ámbito actual
- Proporciona sugerencias útiles para resolver los errores

### 4. Formato de Error Mejorado

El formato de error (`src/error.rs`) ahora muestra:

1. **Código de error** (ej: E0001)
2. **Ubicación precisa** (archivo:línea:columna)
3. **Snippet del código** con la línea problemática
4. **Indicador visual** (`^^^`) apuntando exactamente al token problemático
5. **Mensaje descriptivo** del error
6. **Sugerencia útil** (💡) para resolver el error

## Ejemplo de Mejora

### Antes:
```
Error: 
● : Variable 'err' already defined in this scope
────────────────────────────────────────────────────────────

  ⓘ Variable 'err' already defined in this scope
────────────────────────────────────────────────────────────
```

### Después:
```
● E0001: Variable 'err' already defined in this scope
────────────────────────────────────────────────────────────
  → main.liva:319:19

   319 │
       │ let userResult3,err = validateUser("", "pass123")
       │                 ^^^
       │

  ⓘ Variable 'err' already defined in this scope

  💡 Consider using a different name or removing the previous declaration of 'err'
────────────────────────────────────────────────────────────
```

## Archivos Modificados

- `src/ast.rs`: Añadido campo `span` a `VarBinding` y `ConstDecl`
- `src/parser.rs`: Captura de spans durante el parsing
- `src/semantic.rs`: Uso de spans para generar errores mejorados
- `src/error.rs`: Formato de error mejorado con columna e indicador visual
- `test_error_display.liva`: Archivo de test para verificar las mejoras

## Beneficios

1. **Mejor experiencia de desarrollo**: Los desarrolladores pueden identificar rápidamente dónde está el problema
2. **Menos tiempo de debugging**: La ubicación precisa y el indicador visual reducen el tiempo necesario para encontrar errores
3. **Mensajes más informativos**: Las sugerencias ayudan a resolver los errores más rápidamente
4. **Consistencia**: Formato estandarizado con códigos de error para toda la base de código

## Próximos Pasos

Posibles mejoras futuras:

1. Añadir spans a más estructuras del AST (expresiones, statements)
2. Implementar más códigos de error para diferentes tipos de errores semánticos
3. Añadir información sobre la ubicación de la primera declaración en errores de duplicados
4. Soporte para múltiples errores por compilación
5. Integración con IDEs para mostrar errores inline

## Testing

Para probar las mejoras:

```bash
# Compilar el proyecto
cargo build --release

# Probar con main.liva (contiene un error de variable duplicada)
livac main.liva

# Probar con el archivo de test
livac test_error_display.liva
```
