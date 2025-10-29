# Task Management System - Aplicación de Demostración

## 📍 Ubicación

La aplicación completa está en:
```
/home/fran/Projects/Liva/test_workspace/task-management-system/
```

## 🎯 Descripción

Sistema empresarial de gestión de tareas implementado **100% en Liva** demostrando:

- **Arquitectura Hexagonal** (Ports & Adapters)
- **Vertical Slicing** por features
- **Domain-Driven Design**
- **CQRS Pattern**
- **Event-Driven Architecture**

## 📂 Estructura

```
task-management-system/
├── README.md                     # Documentación principal
├── ARCHITECTURE.md               # Detalles arquitectónicos
├── BOOTSTRAP.md                  # Dependencias y compilación
├── PROJECT_SUMMARY.md            # Resumen ejecutivo
├── main.liva                     # Entry point
│
└── src/
    ├── shared/kernel/           # Domain primitives (4 archivos)
    ├── shared/utils/            # Utilities (1 archivo)
    └── features/tasks/          # Feature completa (6 archivos)
        ├── domain/
        ├── application/
        └── infrastructure/
```

## 🚀 Cómo Ejecutar

```bash
cd /home/fran/Projects/Liva/test_workspace/task-management-system
liva run main.liva
```

## 📊 Métricas

- **11 archivos .liva**
- **4 documentos markdown**
- **~2,000+ líneas de código**
- **25+ clases**
- **80+ funciones**
- **6 use cases**
- **8 value objects**

## ✨ Funcionalidades Demostradas

- ✅ Generics (Result<T, E>)
- ✅ Classes & Constructors
- ✅ Collections (list, map)
- ✅ String Templates
- ✅ Error Handling
- ✅ Value Objects
- ✅ Event System
- ✅ Tuples
- ✅ Repository Pattern
- ✅ HTTP Handlers

## 📖 Documentación

Ver archivos en el directorio para detalles completos:
- `README.md` - Guía de usuario y API
- `ARCHITECTURE.md` - Patrones y decisiones de diseño
- `PROJECT_SUMMARY.md` - Resumen completo del proyecto

## 🎓 Objetivo

Demostrar que **Liva es capaz de aplicaciones empresariales reales** con arquitecturas avanzadas y patrones profesionales.

---

**Rama**: feature/hexagonal-app-test  
**Fecha**: 2025-10-29  
**Nota**: Esta aplicación NO está en el repositorio git de livac, está en test_workspace
