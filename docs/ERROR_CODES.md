# Liva Compiler Error Codes

Esta documentación lista todos los códigos de error del compilador Liva, organizados por categoría.

## 🔴 Errores de Lexer (E1xxx)

### E1000: Invalid Token
**Descripción:** Se encontró un token inválido durante el análisis léxico.

**Causa común:** Caracteres no reconocidos o secuencias de caracteres inválidas.

**Ejemplo:**
```liva
let x = @invalid
```

**Solución:** Verifica que no haya caracteres especiales no permitidos en tu código.

---

## 🟠 Errores de Parser (E2xxx)

### E2000: Parse Error
**Descripción:** Error general de parsing, estructura de código incorrecta.

**Causa común:** Sintaxis incorrecta, tokens inesperados, estructura de código malformada.

**Ejemplo:**
```liva
let x =  // falta el valor
```

**Solución:** Revisa la sintaxis y asegúrate de que todas las expresiones estén completas.

### E2001: Unclosed Interpolation
**Descripción:** Interpolación de string template no cerrada.

**Causa común:** Falta un `}` de cierre en una interpolación de string template.

**Ejemplo:**
```liva
let msg = $"Hello {name"  // falta el }
```

**Solución:** Asegúrate de que cada `{` tenga su correspondiente `}` en interpolaciones.

### E2002: Empty Interpolation
**Descripción:** Interpolación de string template vacía.

**Causa común:** Se usó `{}` sin contenido en un string template.

**Ejemplo:**
```liva
let msg = $"Value: {}"  // interpolación vacía
```

**Solución:** Agrega una expresión dentro de las llaves o remueve la interpolación vacía.

### E2003: Unmatched Closing Brace
**Descripción:** Carácter `}` sin pareja en string template.

**Causa común:** Uso incorrecto de `}` en un string template.

**Ejemplo:**
```liva
let msg = $"Value: }"  // } sin {
```

**Solución:** Usa `}}` para escapar un `}` literal en string templates.

---

## 🟡 Errores Semánticos (E0xxx)

### E0001: Variable Already Defined
**Descripción:** Variable declarada múltiples veces en el mismo ámbito.

**Causa común:** Intentar declarar una variable con el mismo nombre dos veces.

**Ejemplo:**
```liva
let x = 10
let x = 20  // error: x ya está definida
```

**Solución:** Usa un nombre diferente o elimina la declaración duplicada.

### E0002: Constant Already Defined
**Descripción:** Constante declarada múltiples veces en el mismo ámbito.

**Causa común:** Intentar declarar una constante con el mismo nombre dos veces.

**Ejemplo:**
```liva
const MAX = 100
const MAX = 200  // error: MAX ya está definida
```

**Solución:** Usa un nombre diferente para la segunda constante.

### E0003 - E0999: Otros Errores Semánticos
Estos códigos están reservados para errores semánticos adicionales como:
- Tipo de dato incorrecto
- Función no definida
- Parámetros incorrectos
- Violaciones de visibilidad
- Errores de async/await
- Etc.

---

## 🔵 Errores de Codegen (E3xxx)

### E3000: Invalid Binding Pattern
**Descripción:** Patrón de binding inválido en let statement.

**Causa común:** Usar múltiples bindings sin patrón fallible.

**Ejemplo:**
```liva
// Error interno - normalmente no visible para el usuario
```

**Solución:** Este es un error interno del compilador. Reportar como bug.

### E3001: No Rust Code Generated
**Descripción:** El compilador no pudo generar código Rust.

**Causa común:** Error interno durante la generación de código.

**Solución:** Reportar como bug con el código fuente que causó el error.

### E3002: No Cargo.toml Generated
**Descripción:** El compilador no pudo generar el archivo Cargo.toml.

**Causa común:** Error interno durante la generación de configuración.

**Solución:** Reportar como bug con el código fuente que causó el error.

---

## 📊 Formato de Errores

Todos los errores estructurados en Liva siguen este formato:

```
● E0001: Variable 'x' already defined in this scope
────────────────────────────────────────────────────────────
  → file.liva:6:7

     6 │
       │ let x = 20
       │     ^^^
       │

  ⓘ Variable 'x' already defined in this scope

  💡 Consider using a different name or removing the previous declaration of 'x'
────────────────────────────────────────────────────────────
```

### Componentes:

1. **Encabezado:** Código de error y título
2. **Ubicación:** Archivo, línea y columna
3. **Snippet de código:** La línea problemática con indicador visual
4. **Mensaje:** Descripción detallada del error
5. **Ayuda:** Sugerencia para resolver el error (opcional)

---

## 🔧 Integración con IDEs

El compilador soporta salida en formato JSON para integración con IDEs:

```bash
livac file.liva --check --json
```

Salida JSON:
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

Este formato permite a editores como VS Code mostrar:
- Subrayado rojo en la ubicación exacta del error
- Tooltip con información detallada
- Sugerencias de corrección

---

## 🎯 Mejores Prácticas

1. **Lee el código de error:** Cada código identifica el tipo específico de problema
2. **Revisa la ubicación:** La línea y columna indican exactamente dónde está el error
3. **Lee las sugerencias:** El mensaje de ayuda (💡) proporciona cómo resolver el error
4. **Consulta esta documentación:** Para entender mejor la causa y solución

---

## 🚀 Contribuir

Si encuentras un error que no está bien documentado o tiene un mensaje confuso:

1. Abre un issue en GitHub
2. Incluye el código de error
3. Proporciona el código fuente que causó el error
4. Sugiere mejoras al mensaje o documentación

¡Gracias por ayudar a mejorar Liva!
