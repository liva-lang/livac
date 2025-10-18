# 📚 Documentación de Concurrencia de Liva

## 📑 Índice de Documentos

### 🎯 Guías Principales

1. **[EXECUTION_MODES.md](EXECUTION_MODES.md)** - Las 7 formas de ejecutar funciones
   - Comparativa completa: normal, async, par, task async, task par, fire async, fire par
   - Cuándo usar cada una
   - Ejemplos prácticos y código Rust generado
   - **LEER PRIMERO** - Conceptos fundamentales

2. **[CONCURRENCIA_SISTEMA.md](CONCURRENCIA_SISTEMA.md)** - Especificación técnica completa
   - Filosofía de diseño (call-site control)
   - Lazy await/join en detalle
   - Reglas de await implícito
   - Edge cases y comportamiento técnico

3. **[ERROR_HANDLING.md](ERROR_HANDLING.md)** - Manejo de errores en contextos concurrentes
   - Error binding con async/par: `let value, err = async f()`
   - Diferencias entre fire (ignora errores) y task (manual)
   - Patrones comunes de error handling
   - Propagación de errores

### 📋 Planificación y Progreso

4. **[PROGRESS.md](PROGRESS.md)** 🎯 **CONTEXT FILE** - Estado actual y próximos pasos
   - **Archivo único de contexto para continuar el proyecto**
   - Qué está completado (Phase 1 ✅) y qué falta (Phase 2 ⏳)
   - Índice de archivos necesarios por tipo de tarea
   - Roadmap visual y próximos pasos
   - **Usa este archivo cuando necesites contexto completo**

5. **[PLAN_CONCURRENCIA.md](PLAN_CONCURRENCIA.md)** - Plan de implementación en 5 fases
   - Phase 1: Error binding ✅ COMPLETADO
   - Phase 2: Lazy await/join (próxima)
   - Phase 3: Mejoras ergonómicas
   - Phase 4: Optimizaciones
   - Phase 5: Features avanzados

6. **[PHASE1_PROGRESS.md](PHASE1_PROGRESS.md)** - Reporte de Phase 1 completada [DEPRECADO → Ver PROGRESS.md]
   - Cambios implementados
   - Tests y validación
   - Código Rust generado
   - Limitaciones conocidas

7. **[REORGANIZATION_SUMMARY.md](REORGANIZATION_SUMMARY.md)** - Resumen de reorganización de docs
   - Overview de todo el sistema
   - Quick reference
   - Enlaces a documentación detallada

### 🚀 Getting Started

8. **[INICIO_RAMA.md](INICIO_RAMA.md)** - Quick start para desarrollo
   - Setup del branch feature/concurrency-improvements
   - Cómo ejecutar tests
   - Estructura del proyecto
   - Workflow de desarrollo

## 🎓 Flujo de Aprendizaje Recomendado

### Para Usuarios de Liva (Programadores)
1. ✅ **EXECUTION_MODES.md** - Aprende las 7 formas de ejecutar código
2. ✅ **ERROR_HANDLING.md** - Manejo de errores en concurrencia
3. ✅ **CONCURRENCIA_SISTEMA.md** (secciones 1-3) - Conceptos avanzados

### Para Desarrolladores del Compilador (Implementación)
1. ✅ **PROGRESS.md** 🎯 - **EMPIEZA AQUÍ** - Estado y contexto completo
2. ✅ **EXECUTION_MODES.md** - Entender los 7 modos
3. ✅ **PLAN_CONCURRENCIA.md** - Ver tareas pendientes

### Para Continuar el Proyecto
**Solo necesitas: `PROGRESS.md`** - Todo el contexto en un archivo
2. ✅ **PLAN_CONCURRENCIA.md** - Roadmap de implementación
3. ✅ **PHASE1_PROGRESS.md** - Estado actual
4. ✅ **CONCURRENCIA_SISTEMA.md** - Especificación técnica completa

### Para Contribuidores
1. ✅ **RESUMEN_DOCUMENTACION.md** - Overview rápido
2. ✅ **EXECUTION_MODES.md** - Entender el diseño
3. ✅ **PLAN_CONCURRENCIA.md** - Ver qué falta implementar

## 📊 Estado Actual (18 oct 2025)

### ✅ Implementado y Funcionando
- ✅ **Error binding**: `let value, err = async f()`
- ✅ **Async calls**: `async fetchUser()`
- ✅ **Parallel calls**: `par compute()`
- ✅ **Task handles**: `task async f()`, `task par f()`
- ✅ **Fire-and-forget**: `fire async log()`, `fire par cleanup()`
- ✅ **String conversion** en closures async/par
- ✅ **Auto-async detection** para main() con async/par
- ✅ **Default derive** para clases

### 🚧 En Desarrollo (Phase 2)
- ⏳ **Lazy await/join** - Await implícito en primer acceso
- ⏳ **Type inference** para Task<T>
- ⏳ **Optimización** de código generado

### 📅 Planificado (Phases 3-5)
- 📋 Soporte para `_` en error binding
- 📋 Better error types (Option<String>)
- 📋 Validación semántica estricta
- 📋 Cancelación de tasks
- 📋 Timeouts
- 📋 Structured concurrency

## 🔗 Enlaces Rápidos

### Ejemplos de Código
- [main.liva](../../main.liva) - Demo completa de todas las features
- [tests/concurrency/](../../tests/concurrency/) - Test suite completo

### Código Fuente Relevante
- [src/parser.rs](../../src/parser.rs) - Parsing de sintaxis concurrente
- [src/semantic.rs](../../src/semantic.rs) - Análisis semántico (async inference)
- [src/codegen.rs](../../src/codegen.rs) - Generación de código Rust
- [src/ast.rs](../../src/ast.rs) - AST definitions (ExecPolicy)

## 📖 Sintaxis Quick Reference

```liva
// Normal (síncrono)
let result = divide(10, 2)

// Async (lazy await)
let user = async fetchUser(1)
print(user.name)  // Await implícito (Phase 2)

// Parallel (lazy join)
let result = par compute(100)
print(result)  // Join implícito (Phase 2)

// Task async (control manual)
let handle = task async fetchUser(1)
let user = await handle

// Task par (control manual)
let handle = task par compute(100)
let result = await handle

// Fire async (fire-and-forget)
fire async logEvent("action")

// Fire par (fire-and-forget)
fire par cleanup()

// Error binding (Phase 1 ✅)
let value, err = async divide(10, 0)
if err != "" {
    print($"Error: {err}")
}
```

## 🤝 Contribuir

Si quieres contribuir al sistema de concurrencia:

1. Lee **EXECUTION_MODES.md** para entender el diseño
2. Revisa **PLAN_CONCURRENCIA.md** para ver tareas pendientes
3. Sigue **INICIO_RAMA.md** para setup
4. Consulta **CONCURRENCIA_SISTEMA.md** para detalles técnicos

## 📝 Notas

- Todos los documentos están en español para consistencia
- Los ejemplos de código usan sintaxis Liva actual (v0.6)
- El código Rust generado es del compilador actual
- Features marcadas con ✅ están implementadas y testeadas
- Features marcadas con ⏳ están en desarrollo
- Features marcadas con 📋 están planificadas

---

**Última actualización:** 18 de octubre de 2025
**Branch:** feature/concurrency-improvements
**Version:** Liva v0.6 + Phase 1 improvements
