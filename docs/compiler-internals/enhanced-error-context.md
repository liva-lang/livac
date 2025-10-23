# Enhanced Error Context - Phase 5.2

## Mejoras Implementadas

### 1. Contexto Extendido
Los errores ahora muestran 2 líneas antes y 2 líneas después del error para mejor comprensión del contexto.

### 2. Subrayado Preciso
El subrayado bajo el token del error ahora tiene la longitud exacta del token en lugar de ser siempre 3 caracteres.

### 3. Numeración de Líneas
Las líneas de contexto muestran sus números correspondientes para facilitar la navegación en el código fuente.

## Ejemplo de Error con Contexto Mejorado

### Archivo: test_parse_context.liva
```liva
// Test para mostrar el contexto de error de parsing

add(a, b) => a + b

main() {
  let x = 5
  let y = 10
  let z = 15
  
  // Este error debería mostrar contexto mejorado
  let resultado x + y  // Falta el '=' 
  
  let a = 20
  let b = 30
  
  print(resultado)
}
```

### Salida del Error:
```
🧩 Liva Compiler v0.8
→ Compiling examples/manual-tests/test_parse_context.liva
Error: 
● E2000: Parse Error
────────────────────────────────────────────────────────────
  → <input>:11:17

     9 │   
    10 │   // Este error debería mostrar contexto mejorado
    11 │
       │ let resultado x + y  // Falta el '=' 
       │               ^
    12 │   
    13 │   let a = 20
       │

  ⓘ Expected Assign
────────────────────────────────────────────────────────────
```

## Mejoras Técnicas

### Archivo: `livac/src/error.rs`

1. **Actualización de `ErrorLocation`**:
   - Agregado campo `length: Option<usize>` para longitud precisa del token
   - Agregado campo `context_before: Option<Vec<String>>` para líneas anteriores
   - Agregado campo `context_after: Option<Vec<String>>` para líneas posteriores

2. **Nuevos métodos constructores**:
   - `with_length(usize)`: Establece la longitud del token a subrayar
   - `with_context(Vec<String>, Vec<String>)`: Establece las líneas de contexto

3. **Formato mejorado**:
   - Muestra hasta 2 líneas antes del error
   - Muestra hasta 2 líneas después del error
   - Subrayado del token con longitud precisa usando `^` repetidos
   - Numeración de líneas para todas las líneas de contexto

### Archivo: `livac/src/parser.rs`

1. **Actualización de `error_with_help()`**:
   - Extrae líneas de contexto del código fuente
   - Calcula la longitud del token desde el span
   - Pasa toda la información de contexto al error

### Archivo: `livac/src/semantic.rs`

1. **Nueva función `get_context_lines()`**:
   - Extrae líneas antes y después de una posición dada
   - Parámetro configurable de número de líneas de contexto
   - Maneja casos límite (inicio/fin de archivo)

2. **Actualización de `make_error_with_span()`**:
   - Llama a `get_context_lines()` con contexto de 2 líneas
   - Calcula longitud del token desde el span
   - Pasa contexto completo al error

## Estado de Implementación

✅ **Completado**:
- Estructura de datos extendida en `ErrorLocation`
- Métodos constructores para contexto y longitud
- Formato mejorado con contexto extendido
- Implementación en parser para errores de parsing
- Implementación en semantic analyzer (infraestructura)
- Subrayado preciso basado en longitud del token
- Tests manuales creados

⚠️ **Limitaciones Conocidas**:
- Algunos errores semánticos no tienen acceso a span porque el AST no incluye información de posición
- Para usar el contexto mejorado, los errores deben usar `make_error_with_span()` con un span válido
- Errores legacy que usan conversión directa de string no tendrán contexto

## Tests

Los siguientes archivos de test demuestran las mejoras:

1. **test_parse_context.liva**: Error de parsing con contexto mejorado
2. **test_did_you_mean.liva**: Sugerencia "Did you mean?" (Phase 5.1)
3. **test_import_typo.liva**: Sugerencia para imports incorrectos (Phase 5.1)

## Próximos Pasos (Phase 5.3)

- Implementar categorías de error más específicas
- Añadir códigos de error consistentes para todos los tipos
- Mejorar mensajes de error con más detalles
