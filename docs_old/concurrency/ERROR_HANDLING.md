# 🛡️ Error Handling en Contextos Concurrentes

## Introducción

El manejo de errores en Liva es consistente entre contextos síncronos y concurrentes, pero cada modo de ejecución tiene diferentes capacidades de error handling.

## 📊 Error Handling por Modo de Ejecución

| Modo | Error Binding | Manejo Manual | Errores Silenciosos |
|------|---------------|---------------|---------------------|
| Normal `f()` | ✅ `let v, e = f()` | ✅ | ❌ |
| Async `async f()` | ✅ `let v, e = async f()` | ✅ | ❌ |
| Par `par f()` | ✅ `let v, e = par f()` | ✅ | ❌ |
| Task Async | ❌ | ✅ await handle | ❌ |
| Task Par | ❌ | ✅ await handle | ❌ |
| Fire Async | ❌ | ❌ | ✅ Se pierden |
| Fire Par | ❌ | ❌ | ✅ Se pierden |

## 1. Error Binding (Phase 1 ✅)

### Sintaxis
```liva
let value, error = expression
```

### Con Funciones Normales
```liva
divide(a: number, b: number): number {
    if b == 0 fail "Division by zero"
    return a / b
}

main() {
    // Error binding con llamada normal
    let result, err = divide(10, 0)
    
    if err != "" {
        print($"Error: {err}")  // Output: Division by zero
    } else {
        print($"Result: {result}")
    }
}
```

### Con Async Calls
```liva
fetchUser(id: number): User {
    if id == 0 fail "Invalid user ID"
    // ... fetch user from API ...
    return user
}

main() {
    // Error binding con async
    let user, err = async fetchUser(0)
    
    if err != "" {
        print($"Error fetching user: {err}")
    } else {
        print($"User: {user.name}")
    }
}
```

### Con Par Calls
```liva
compute(n: number): number {
    if n < 0 fail "Negative number not allowed"
    return n * n
}

main() {
    // Error binding con parallel
    let result, err = par compute(-5)
    
    if err != "" {
        print($"Computation error: {err}")
    } else {
        print($"Result: {result}")
    }
}
```

### Código Rust Generado

```rust
// Normal
let (result, err) = match divide(10, 0) { 
    Ok(v) => (v, "".to_string()), 
    Err(e) => (Default::default(), e.message) 
};

// Async
let (user, err) = match liva_rt::spawn_async(async move { 
    fetch_user(0) 
}).await.unwrap() { 
    Ok(v) => (v, "".to_string()), 
    Err(e) => (Default::default(), e.message) 
};

// Par
let (result, err) = match liva_rt::spawn_parallel(move || 
    compute(-5)
).await.unwrap() { 
    Ok(v) => (v, "".to_string()), 
    Err(e) => (Default::default(), e.message) 
};
```

## 2. Comparación de Errores

### Estado Actual
```liva
if err != "" {
    // Hay error
}
```

### ¿Por qué String y no null?

Liva actualmente no tiene un tipo `null` nativo. Los errores se representan como strings:
- String vacío `""` = sin error
- String no vacío = mensaje de error

### Futura Mejora (Phase 3)
```liva
// Propuesta para Phase 3
if err.isSome() {
    print($"Error: {err.unwrap()}")
}

// O más simple
if err? {
    print($"Error: {err}")
}
```

## 3. Ignorar Errores

### Con Variable Dummy
```liva
// Actualmente: usar variable dummy
let value, ignored = async fetchUser(1)
print($"Value: {value}")
```

### Futura Mejora (Phase 3)
```liva
// Propuesta: underscore para ignorar
let value, _ = async fetchUser(1)
print($"Value: {value}")
```

## 4. Task Handles: Error Handling Manual

### Task Async
```liva
main() {
    let handle = task async fetchUser(1)
    
    // ... hacer otras cosas ...
    
    // Await retorna el valor directamente
    // Si hay error, se propaga al await
    let user = await handle  // Puede fallar aquí
    
    print($"User: {user.name}")
}
```

**Problema:** No hay forma de capturar el error con task handles actualmente.

**Workaround:** Wrapper function con error handling
```liva
fetchUserSafe(id: number): (User, string) {
    let user, err = fetchUser(id)
    return (user, err)
}

main() {
    let handle = task async fetchUserSafe(1)
    let (user, err) = await handle
    
    if err != "" {
        print($"Error: {err}")
    }
}
```

### Task Par
```liva
// Mismo comportamiento que task async
let handle = task par compute(100)
let result = await handle  // Puede fallar

// Workaround con wrapper
computeSafe(n: number): (number, string) {
    let result, err = compute(n)
    return (result, err)
}

let handle = task par computeSafe(-5)
let (result, err) = await handle
```

## 5. Fire: Errores Silenciosos

### Fire Async
```liva
sendEmail(to: string, subject: string) {
    if to == "" fail "Invalid email"
    // ... send email ...
}

main() {
    // Si falla, el error se PIERDE
    fire async sendEmail("", "Test")
    
    // Continúa ejecutando, sin saber si funcionó
    print("Email sent (maybe)")
}
```

### Fire Par
```liva
processImage(path: string) {
    if path == "" fail "Invalid path"
    // ... process image ...
}

main() {
    // Si falla, el error se PIERDE
    fire par processImage("")
    
    // Continúa ejecutando
    print("Processing started (hopefully)")
}
```

### ⚠️ Importante sobre Fire

**Fire = "No me importa si falla"**

Si te importa que la operación tenga éxito:
- ❌ NO uses `fire`
- ✅ USA `async` o `par` con error binding
- ✅ USA `task async` o `task par` con wrapper

## 6. Propagación de Errores

### En Funciones Síncronas
```liva
innerFunction(): number {
    let result, err = divide(10, 0)
    if err != "" {
        // Re-propagar el error
        fail err
    }
    return result
}

outerFunction() {
    let value, err = innerFunction()
    if err != "" {
        print($"Error in outer: {err}")
    }
}
```

### En Funciones Async
```liva
async fetchAndProcess(id: number): Result {
    let user, err = async fetchUser(id)
    if err != "" {
        fail $"Failed to fetch user: {err}"
    }
    
    let processed, err2 = async processUser(user)
    if err2 != "" {
        fail $"Failed to process: {err2}"
    }
    
    return processed
}

main() {
    let result, err = async fetchAndProcess(1)
    if err != "" {
        print($"Pipeline error: {err}")
    }
}
```

## 7. Patrones Comunes

### Patrón 1: Error Handling en Cadena
```liva
main() {
    let user, err1 = async fetchUser(1)
    if err1 != "" {
        logError($"Fetch failed: {err1}")
        return
    }
    
    let profile, err2 = async fetchProfile(user.id)
    if err2 != "" {
        logError($"Profile failed: {err2}")
        return
    }
    
    let settings, err3 = async fetchSettings(user.id)
    if err3 != "" {
        logError($"Settings failed: {err3}")
        return
    }
    
    // Todo exitoso
    displayUser(user, profile, settings)
}
```

### Patrón 2: Error Handling con Defaults
```liva
main() {
    let config, err = async fetchConfig()
    
    // Si falla, usar default
    if err != "" {
        print($"Warning: using default config: {err}")
        config = getDefaultConfig()
    }
    
    startApp(config)
}
```

### Patrón 3: Error Accumulation
```liva
main() {
    let errors = []
    
    let u1, e1 = async fetchUser(1)
    if e1 != "" errors.push(e1)
    
    let u2, e2 = async fetchUser(2)
    if e2 != "" errors.push(e2)
    
    let u3, e3 = async fetchUser(3)
    if e3 != "" errors.push(e3)
    
    if errors.length > 0 {
        print($"Errors occurred: {errors}")
    }
}
```

### Patrón 4: Critical vs Non-Critical
```liva
main() {
    // CRITICAL: Debe tener éxito
    let user, err = async fetchUser(userId)
    if err != "" {
        logError($"CRITICAL: {err}")
        return  // Abort
    }
    
    // NON-CRITICAL: Fire and forget
    fire async logAnalytics("user_fetched", user.id)
    fire async sendNotification(user, "Welcome back")
    
    // Continuar con user válido
    displayDashboard(user)
}
```

### Patrón 5: Retry con Error Handling
```liva
fetchWithRetry(id: number, maxRetries: number): (User, string) {
    let retries = 0
    
    while retries < maxRetries {
        let user, err = async fetchUser(id)
        
        if err == "" {
            return (user, "")  // Éxito
        }
        
        print($"Retry {retries + 1}/{maxRetries}: {err}")
        retries = retries + 1
        sleep(1000)  // Esperar 1 segundo
    }
    
    // Falló todos los reintentos
    return (User("", 0), "Max retries exceeded")
}

main() {
    let user, err = fetchWithRetry(1, 3)
    if err != "" {
        print($"Failed after retries: {err}")
    }
}
```

## 8. Best Practices

### ✅ DO: Error Binding para Operaciones Críticas
```liva
// BIEN: Maneja el error
let user, err = async fetchUser(id)
if err != "" {
    handleError(err)
    return
}
```

### ❌ DON'T: Ignorar Errores en Operaciones Críticas
```liva
// MAL: Ignora el error
let user, _ = async fetchUser(id)
// ¿Qué pasa si fetchUser falló?
```

### ✅ DO: Fire para Operaciones No Críticas
```liva
// BIEN: Logging puede fallar sin problema
fire async logEvent("action")

// BIEN: Analytics no es crítico
fire async trackMetrics(data)
```

### ❌ DON'T: Fire para Operaciones Críticas
```liva
// MAL: Payment DEBE tener éxito
fire async processPayment(order)  // ❌ NO!

// BIEN: Con error handling
let success, err = async processPayment(order)
if err != "" {
    notifyFailure(order, err)
}
```

### ✅ DO: Logging de Errores
```liva
let result, err = async operation()
if err != "" {
    logError($"Operation failed: {err}")
    // Continuar o abortar según criticidad
}
```

### ✅ DO: Proporcionar Contexto
```liva
let user, err = async fetchUser(id)
if err != "" {
    fail $"Failed to fetch user {id}: {err}"
}
```

## 9. Debugging Errors

### Logging Automático (Propuesta Phase 3)
```liva
// Configuración global
config {
    logAllErrors: true
    logFireErrors: true  // Log errores en fire calls
}

// Todos los errores se loggean automáticamente
let value, err = async fetchUser(id)
// Si err != "", se loggea automáticamente

fire async sendEmail(user)
// Si falla, se loggea si logFireErrors = true
```

### Stack Traces (Propuesta Phase 4)
```liva
let user, err = async fetchUser(id)
if err != "" {
    print(err.stackTrace)  // Mostrar stack trace completo
}
```

## 10. Limitaciones Actuales

### ❌ No hay underscore para ignorar
```liva
// NO funciona actualmente
let value, _ = async f()

// Workaround
let value, ignored = async f()
```

### ❌ Task handles sin error binding
```liva
// NO hay forma de capturar error
let handle = task async fetchUser(1)
let user = await handle  // Falla si hay error

// Workaround: wrapper function
```

### ❌ Fire errors se pierden
```liva
// No hay forma de saber si falló
fire async sendEmail(user)

// Workaround: logging interno
sendEmailSafe(user: User) {
    let success, err = sendEmail(user)
    if err != "" {
        logError($"Email failed: {err}")
    }
}

fire async sendEmailSafe(user)
```

## 11. Roadmap

### Phase 2: Lazy Await/Join
- Error handling no cambia
- Await es implícito en primer uso
- Errores siguen usando binding

### Phase 3: Ergonomía
- ✅ Soporte para `_` en binding
- ✅ Better error types (Option<String>)
- ✅ Logging automático de fire errors
- ✅ Error context mejorado

### Phase 4: Avanzado
- ✅ Stack traces
- ✅ Error recovery automático
- ✅ Retry policies integrados
- ✅ Error categorization

## 📚 Referencias

- [EXECUTION_MODES.md](EXECUTION_MODES.md) - Modos de ejecución completos
- [CONCURRENCIA_SISTEMA.md](CONCURRENCIA_SISTEMA.md) - Especificación técnica
- [PHASE1_PROGRESS.md](PHASE1_PROGRESS.md) - Implementación actual

---

**Última actualización:** 18 de octubre de 2025
