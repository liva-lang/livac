# Liva Examples

Ejemplos demostrativos del lenguaje Liva. Cada carpeta es un proyecto independiente.

## 🚀 Ejecutar Ejemplos

```bash
# Desde la raíz del proyecto livac
livac examples/hello-world/main.liva --run
livac examples/calculator/calculator.liva --run
livac examples/http-api/main.liva --run
livac examples/json-processing/main.liva --run
livac examples/concurrency/main.liva --run
```

## 📁 Proyectos

### Básicos

| Carpeta | Descripción | Características |
|---------|-------------|-----------------|
| [hello-world/](hello-world/) | Tu primer programa | Variables, loops, templates |
| [calculator/](calculator/) | Calculadora interactiva | Clases, métodos, switch |
| [json-processing/](json-processing/) | Procesar datos JSON | filter, map, reduce |

### HTTP & APIs

| Carpeta | Descripción | Características |
|---------|-------------|-----------------|
| [http-api/](http-api/) | Cliente HTTP REST | async/await, HTTP, JSON |
| [http-crud/](http-crud/) | CRUD completo (JSONPlaceholder) | GET, POST, PUT, DELETE |
| [crypto-tracker/](crypto-tracker/) | Precios crypto (CoinGecko) | HTTP + JSON + CLI args |

### Concurrencia

| Carpeta | Descripción | Características |
|---------|-------------|-----------------|
| [concurrency/](concurrency/) | Programación concurrente | async/par, task, await |
| [parallel-search/](parallel-search/) | Grep paralelo recursivo | `.par().map()`, File I/O |

### Multi-archivo

| Carpeta | Descripción | Características |
|---------|-------------|-----------------|
| [github-dashboard/](github-dashboard/) | Dashboard GitHub (mock) | 8 archivos, imports, módulos |
| [github-dashboard-real/](github-dashboard-real/) | Dashboard GitHub (API real) | HTTP real, config, 5 archivos |

### Dogfooding (programas reales para encontrar bugs)

| Carpeta | Descripción | Características |
|---------|-------------|-----------------|
| [dogfooding-v1/](dogfooding-v1/) | Student Grade Tracker | 262 líneas, bugs #63-#74 |
| [dogfooding-v2/](dogfooding-v2/) | Inventory Manager | 577 líneas, 21 tests, Map/Set/Enum |

### Test Framework

| Carpeta | Descripción | Características |
|---------|-------------|-----------------|
| [tests/](tests/) | Ejemplos del test framework | describe/test/expect, lifecycle hooks |

## 📚 Más Recursos

- [Documentación](../docs/README.md)
- [Referencia Rápida](../docs/QUICK_REFERENCE.md)
- [Guía de Errores](../docs/ERROR_HANDLING_GUIDE.md)

## 🧪 Tests

Los archivos de prueba del compilador están en `tests/fixtures/`.
Estos ejemplos son solo para demostración y aprendizaje.
