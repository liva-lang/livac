# 🎯 Modos de Ejecución en Liva

## Introducción

Liva ofrece **7 formas distintas** de ejecutar una función, cada una diseñada para un caso de uso específico. Este documento explica cuándo y cómo usar cada una.

## 📊 Tabla Comparativa Completa

| Sintaxis | Retorna | Bloquea | Error Binding | Lazy Eval | Caso de Uso |
|----------|---------|---------|---------------|-----------|-------------|
| `f()` | Valor | Sí (inmediato) | ✅ | No | Ejecución síncrona normal |
| `async f()` | Valor | En primer uso | ✅ | Sí | I/O async con lazy await |
| `par f()` | Valor | En primer uso | ✅ | Sí | CPU paralelo con lazy join |
| `task async f()` | JoinHandle | No | ❌ | No | Control manual async |
| `task par f()` | JoinHandle | No | ❌ | No | Control manual parallel |
| `fire async f()` | Void | No | ❌ | N/A | Fire-and-forget async |
| `fire par f()` | Void | No | ❌ | N/A | Fire-and-forget parallel |

## 1. 🔵 Ejecución Normal (Síncrona)

### Sintaxis
```liva
let result = functionName(args)
```

### Comportamiento
- Ejecuta la función **inmediatamente**
- **Bloquea** hasta que la función termine
- Retorna el valor directamente
- Error binding disponible

### Ejemplo Básico
```liva
divide(a: number, b: number): number {
    if b == 0 fail "Division by zero"
    return a / b
}

main() {
    let result = divide(10, 2)
    print($"Result: {result}")  // Output: 5
}
```

### Con Error Binding
```liva
main() {
    let result, err = divide(10, 0)
    if err != "" {
        print($"Error: {err}")  // Output: Division by zero
    } else {
        print($"Result: {result}")
    }
}
```

### Código Rust Generado
```rust
// Sin error binding
let result = divide(10, 2);

// Con error binding
let (result, err) = match divide(10, 0) { 
    Ok(v) => (v, "".to_string()), 
    Err(e) => (Default::default(), e.message) 
};
```

### ✅ Cuándo Usar
- Operaciones rápidas y síncronas
- Computación local sin I/O
- Cuando el orden de ejecución es importante
- Cuando necesitas el resultado inmediatamente

### ❌ Cuándo NO Usar
- Operaciones de I/O lentas (usa `async`)
- Computación intensiva de CPU (usa `par`)
- Cuando quieres hacer otras cosas mientras esperas

---

## 2. 🟢 Async (Lazy Await)

### Sintaxis
```liva
let value = async functionName(args)
```

### Comportamiento
- **NO bloquea** inmediatamente
- Retorna un valor "lazy"
- Se ejecuta en background
- **Bloquea automáticamente** en primer uso del valor (Phase 2)
- Error binding disponible

### Ejemplo Básico
```liva
fetchUser(id: number): User {
    // Simula llamada HTTP
    return User("Alice", id)
}

main() {
    // NO bloquea aquí - lanza la operación
    let user = async fetchUser(1)
    
    // Hacer otras cosas mientras fetch está en progreso
    doSomething()
    
    // Bloquea AQUÍ cuando accedes al valor (Phase 2)
    print($"User: {user.name}")  // Await implícito
}
```

### Con Error Binding (Phase 1 ✅)
```liva
getUser(id: number): User {
    if id == 0 fail "Invalid ID"
    return User("Test", id)
}

main() {
    let user, err = async getUser(0)
    
    if err != "" {
        print($"Error: {err}")
    } else {
        print($"User: {user.name}")
    }
}
```

### Código Rust Generado
```rust
// Básico (actualmente hace await inmediato, Phase 2 lo hará lazy)
let mut user = liva_rt::spawn_async(async move { 
    fetch_user(1) 
}).await.unwrap();

// Con error binding
let (user, err) = match liva_rt::spawn_async(async move { 
    get_user(0) 
}).await.unwrap() { 
    Ok(v) => (v, "".to_string()), 
    Err(e) => (Default::default(), e.message) 
};
```

### ✅ Cuándo Usar
- Operaciones de I/O (HTTP, database, files)
- Llamadas a APIs externas
- Operaciones de red
- Cuando quieres hacer otras cosas mientras esperas

### ❌ Cuándo NO Usar
- Computación intensiva de CPU (usa `par`)
- Cuando necesitas control manual del timing (usa `task async`)
- Para operaciones que no te importa el resultado (usa `fire async`)

---

## 3. 🟡 Parallel (Lazy Join)

### Sintaxis
```liva
let result = par functionName(args)
```

### Comportamiento
- **NO bloquea** inmediatamente
- Ejecuta en thread/worker separado
- Retorna un valor "lazy"
- **Bloquea automáticamente** en primer uso del valor (Phase 2)
- Error binding disponible

### Ejemplo Básico
```liva
heavyComputation(n: number): number {
    // Computación intensiva
    let result = n * n * n
    return result
}

main() {
    // NO bloquea aquí - lanza el cómputo
    let result = par heavyComputation(1000)
    
    // Hacer otras cosas mientras computa
    doOtherWork()
    
    // Bloquea AQUÍ cuando accedes al valor (Phase 2)
    print($"Result: {result}")  // Join implícito
}
```

### Con Error Binding (Phase 1 ✅)
```liva
compute(n: number): number {
    if n < 0 fail "Negative number"
    return n * n
}

main() {
    let result, err = par compute(-5)
    
    if err != "" {
        print($"Error: {err}")
    } else {
        print($"Result: {result}")
    }
}
```

### Código Rust Generado
```rust
// Básico (actualmente hace await/join inmediato)
let mut result = liva_rt::spawn_parallel(move || 
    heavy_computation(1000)
).await.unwrap();

// Con error binding
let (result, err) = match liva_rt::spawn_parallel(move || 
    compute(-5)
).await.unwrap() { 
    Ok(v) => (v, "".to_string()), 
    Err(e) => (Default::default(), e.message) 
};
```

### ✅ Cuándo Usar
- Computación intensiva de CPU
- Procesamiento de imágenes
- Cálculos matemáticos complejos
- Cuando tienes múltiples cores disponibles

### ❌ Cuándo NO Usar
- Operaciones de I/O (usa `async`)
- Cuando necesitas control manual (usa `task par`)
- Para operaciones que no te importa el resultado (usa `fire par`)

---

## 4. 🔷 Task Async (Control Manual)

### Sintaxis
```liva
let handle = task async functionName(args)
let result = await handle
```

### Comportamiento
- Retorna un **JoinHandle** (no el valor)
- **NO bloquea** automáticamente
- Te da control manual de cuándo hacer await
- NO tiene error binding automático
- Útil para composición compleja

### Ejemplo Básico
```liva
main() {
    // Lanzar la operación, obtener handle
    let handle = task async fetchUser(1)
    
    // Hacer MUCHAS otras cosas
    processData()
    doCalculations()
    prepareResponse()
    
    // Esperar manualmente cuando lo necesites
    let user = await handle
    print($"User: {user}")
}
```

### Lanzar Múltiples Tasks
```liva
main() {
    // Lanzar 3 tasks en paralelo
    let t1 = task async fetchUser(1)
    let t2 = task async fetchUser(2)
    let t3 = task async fetchUser(3)
    
    // Hacer otras cosas mientras TODAS se ejecutan
    prepareUI()
    
    // Esperar todas
    let u1 = await t1
    let u2 = await t2
    let u3 = await t3
    
    print($"Users: {u1.name}, {u2.name}, {u3.name}")
}
```

### Código Rust Generado
```rust
let mut handle = liva_rt::spawn_async(async move { 
    fetch_user(1) 
});

// ... hacer otras cosas ...

let mut user = handle.await;
```

### ✅ Cuándo Usar
- Necesitas control preciso del timing
- Quieres lanzar múltiples operaciones y esperar después
- Composición compleja de async operations
- Patrones avanzados de concurrencia

### ❌ Cuándo NO Usar
- Cuando la sintaxis simple `async` es suficiente
- Cuando quieres error binding automático
- Para fire-and-forget (usa `fire async`)

---

## 5. 🟠 Task Par (Control Manual Paralelo)

### Sintaxis
```liva
let handle = task par functionName(args)
let result = await handle
```

### Comportamiento
- Retorna un **JoinHandle** de thread
- **NO bloquea** automáticamente
- Control manual de sincronización
- NO tiene error binding automático
- Útil para fork-join patterns

### Ejemplo Básico
```liva
main() {
    // Lanzar computación en thread separado
    let handle = task par heavyComputation(1000)
    
    // Procesar otros datos mientras computa
    let localData = processLocalData()
    
    // Esperar resultado cuando lo necesites
    let result = await handle
    
    // Combinar resultados
    let final = combineResults(localData, result)
}
```

### Fork-Join Pattern
```liva
main() {
    // Fork: Lanzar múltiples computaciones
    let t1 = task par compute(100)
    let t2 = task par compute(200)
    let t3 = task par compute(300)
    
    // Hacer trabajo local
    let local = processLocal()
    
    // Join: Esperar y combinar
    let r1 = await t1
    let r2 = await t2
    let r3 = await t3
    
    let total = r1 + r2 + r3 + local
}
```

### Código Rust Generado
```rust
let mut handle = liva_rt::spawn_parallel(move || { 
    heavy_computation(1000) 
});

// ... hacer otras cosas ...

let mut result = handle.await;
```

### ✅ Cuándo Usar
- Fork-join patterns
- Necesitas control preciso de threads
- Múltiples computaciones paralelas con sincronización manual
- Patrones avanzados de paralelismo

### ❌ Cuándo NO Usar
- Cuando la sintaxis simple `par` es suficiente
- Operaciones de I/O (usa `task async`)
- Para fire-and-forget (usa `fire par`)

---

## 6. 🔴 Fire Async (Fire-and-Forget Async)

### Sintaxis
```liva
fire async functionName(args)
```

### Comportamiento
- **NO retorna nada** (void)
- **NO bloquea** nunca
- Ejecuta en background
- **NO tiene error binding** (errores se pierden)
- Para operaciones no críticas

### Ejemplo Básico
```liva
main() {
    // Ejecuta en background, continúa inmediatamente
    fire async logEvent("User logged in")
    fire async sendEmail(user, "Welcome!")
    
    // Estas líneas se ejecutan INMEDIATAMENTE
    // No espera a que log o email terminen
    print("Continuing...")
}
```

### Casos de Uso Típicos
```liva
// Logging no crítico
fire async logEvent($"Action performed at {timestamp}")

// Analytics
fire async trackEvent("button_clicked", metadata)

// Notificaciones que pueden fallar
fire async sendPushNotification(user, "New message")

// Sincronización en background
fire async syncToCloud(localData)

// Webhooks
fire async callWebhook(url, payload)
```

### Código Rust Generado
```rust
liva_rt::fire_async(async move { 
    log_event("User logged in".to_string()); 
});

liva_rt::fire_async(async move { 
    send_email(user, "Welcome!".to_string()); 
});
```

### ✅ Cuándo Usar
- Logging no crítico
- Analytics/telemetry
- Notificaciones que pueden fallar
- Operaciones donde NO te importa si fallan
- Background sync no crítico

### ❌ Cuándo NO Usar
- Operaciones críticas que DEBEN tener éxito
- Cuando necesitas saber si hubo error
- Cuando necesitas el resultado
- Operaciones que requieren sincronización

---

## 7. 🟣 Fire Par (Fire-and-Forget Parallel)

### Sintaxis
```liva
fire par functionName(args)
```

### Comportamiento
- **NO retorna nada** (void)
- **NO bloquea** nunca
- Ejecuta en thread separado
- **NO tiene error binding** (errores se pierden)
- Para procesamiento en background no crítico

### Ejemplo Básico
```liva
main() {
    // Lanza en thread separado, continúa inmediatamente
    fire par cleanupTempFiles()
    fire par updateStatistics()
    
    // Estas líneas se ejecutan INMEDIATAMENTE
    print("Cleanup started in background")
}
```

### Casos de Uso Típicos
```liva
// Cleanup no crítico
fire par cleanupOldLogs()
fire par deleteTempFiles()

// Procesamiento en background
fire par generateThumbnails(images)
fire par rebuildSearchIndex()

// Tareas de mantenimiento
fire par compactDatabase()
fire par optimizeCache()

// Procesamiento no crítico
fire par processLowPriorityQueue()
```

### Código Rust Generado
```rust
liva_rt::fire_parallel(move || { 
    cleanup_temp_files(); 
});

liva_rt::fire_parallel(move || { 
    update_statistics(); 
});
```

### ✅ Cuándo Usar
- Cleanup/mantenimiento no crítico
- Procesamiento en background de baja prioridad
- Tareas que pueden fallar sin consecuencias
- Operaciones que NO requieren sincronización

### ❌ Cuándo NO Usar
- Operaciones críticas
- Cuando necesitas saber el resultado
- Computación donde los errores son importantes
- Operaciones que requieren coordinación

---

## 🎓 Guía de Decisión

### Diagrama de Flujo

```
¿Te importa el resultado?
│
├─ NO ─→ ¿Async o CPU?
│        ├─ Async (I/O) ─→ fire async f()
│        └─ CPU          ─→ fire par f()
│
└─ SÍ ─→ ¿Necesitas control manual?
         │
         ├─ SÍ ─→ ¿Async o CPU?
         │        ├─ Async (I/O) ─→ task async f()
         │        └─ CPU          ─→ task par f()
         │
         └─ NO ─→ ¿Quieres lazy evaluation?
                  │
                  ├─ SÍ ─→ ¿Async o CPU?
                  │        ├─ Async (I/O) ─→ async f()
                  │        └─ CPU          ─→ par f()
                  │
                  └─ NO ─→ f() (normal síncrono)
```

### Tabla de Decisión Rápida

| Necesito... | Usa esto |
|-------------|----------|
| Resultado inmediato | `f()` |
| I/O con lazy await | `async f()` |
| CPU con lazy join | `par f()` |
| Control manual async | `task async f()` |
| Control manual parallel | `task par f()` |
| Fire-and-forget I/O | `fire async f()` |
| Fire-and-forget CPU | `fire par f()` |
| Error handling | Cualquiera excepto `fire` |

## 🔄 Conversión entre Modos

### De Normal a Async/Par
```liva
// Normal - bloquea
let user = fetchUser(1)

// Async - lazy await
let user = async fetchUser(1)

// Par - lazy join
let result = par compute(100)
```

### De Lazy a Task
```liva
// Lazy - await automático
let user = async fetchUser(1)
print(user.name)  // Await aquí

// Task - await manual
let handle = task async fetchUser(1)
// ... hacer cosas ...
let user = await handle
```

### De Task a Fire
```liva
// Task - obtienes handle
let handle = task async logEvent("test")

// Fire - no obtienes nada
fire async logEvent("test")
```

## 💡 Patrones Comunes

### Patrón 1: Múltiples Tasks + Lazy
```liva
// Lanzar tasks con control manual
let t1 = task async fetchUser(1)
let t2 = task async fetchUser(2)

// Lazy evaluation para computación local
let localResult = par processLocal()

// Fire para logging
fire async logEvent("Processing started")

// Await manual de tasks
let u1 = await t1
let u2 = await t2

// Lazy join automático
print(localResult)
```

### Patrón 2: Error Handling Mixto
```liva
// Critical: con error binding
let user, err = async fetchUser(id)
if err != "" {
    logError(err)
    return
}

// Non-critical: fire and forget
fire async sendWelcomeEmail(user)
```

### Patrón 3: Fork-Join con Lazy
```liva
// Fork: Lanzar múltiples computaciones
let t1 = task par compute(100)
let t2 = task par compute(200)

// Lazy evaluation mientras tasks corren
let extra = par quickCompute(10)

// Join tasks
let r1 = await t1
let r2 = await t2

// Lazy join automático
let total = r1 + r2 + extra
```

## 📚 Referencias

- [CONCURRENCIA_SISTEMA.md](CONCURRENCIA_SISTEMA.md) - Especificación técnica completa
- [ERROR_HANDLING.md](ERROR_HANDLING.md) - Error handling detallado
- [PLAN_CONCURRENCIA.md](PLAN_CONCURRENCIA.md) - Roadmap de implementación

---

**Última actualización:** 18 de octubre de 2025
