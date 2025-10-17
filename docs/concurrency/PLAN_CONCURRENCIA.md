# 🚀 Plan de Implementación - Sistema de Concurrencia

**Rama:** `feature/concurrency-improvements`  
**Fecha Inicio:** 18 de octubre de 2025  
**Objetivo:** Implementar completamente el sistema de concurrencia según la especificación

---

## 📋 ESTADO ACTUAL

### ✅ Lo que Funciona

1. **Sintaxis básica**
   - `async function_call()` ✅
   - `par function_call()` ✅
   - Parsing correcto

2. **Compilación básica**
   - Genera código Rust ✅
   - Usa tokio para async ✅
   - Usa threads para parallel ✅

3. **Documentación**
   - Especificación completa ✅
   - Ejemplos documentados ✅
   - Edge cases identificados ✅

### ❌ Lo que Falta

1. **Error handling con dos variables**
   ```liva
   let user, err = async getUser()  // ❌ No implementado
   ```

2. **Await implícito inteligente**
   - Actualmente: await en asignación
   - Debería: await en primer uso de campo/método

3. **Warnings**
   - Task no usada
   - Error no chequeado
   - Task en condición sin uso

4. **Optimizaciones**
   - Join combining
   - Dead task elimination

5. **Tests exhaustivos**
   - Edge cases
   - Error handling
   - Múltiples tasks

---

## 🎯 FASES DE IMPLEMENTACIÓN

### FASE 1: Error Handling (1-2 semanas) 🔴 CRÍTICO

**Objetivo:** Implementar `let value, err = async call()`

#### Tareas:

1. **Parser (parser.rs)**
   - [ ] Detectar binding con dos variables
   - [ ] Distinguir `let a, b = ...` de `let a = ..., b = ...`
   - [ ] Validar que segunda variable se llame `err` o `_`
   
2. **AST (ast.rs)**
   - [ ] Extender `VarDecl` para múltiples bindings
   - [ ] Agregar flag `has_error_binding: bool`
   
3. **Semantic (semantic.rs)**
   - [ ] Validar que función retorna Result<T, E>
   - [ ] Type checking de ambas variables
   - [ ] Warning si `err` no se usa
   
4. **Codegen (codegen.rs)**
   - [ ] Generar pattern matching para Result
   - [ ] Manejar Ok/Err correctamente
   - [ ] Default value para T si hay error

#### Ejemplo Target:

**Liva:**
```liva
let user, err = async getUser()
if err {
    print(err)
    return
}
print(user.name)
```

**Rust generado:**
```rust
let task = tokio::spawn(async move { get_user() });
let result = task.await;

let (user, err) = match result {
    Ok(Ok(u)) => (u, None),
    Ok(Err(e)) => (User::default(), Some(e)),
    Err(e) => (User::default(), Some(e.into())),
};

if err.is_some() {
    println!("{}", err.unwrap());
    return;
}
println!("{}", user.name);
```

#### Tests:
- [ ] `test_async_error_binding.liva`
- [ ] `test_par_error_binding.liva`
- [ ] `test_error_unused_warning.liva`
- [ ] `test_error_wrong_name.liva` (error)

---

### FASE 2: Await Implícito Inteligente (2-3 semanas) 🟡 IMPORTANTE

**Objetivo:** Await solo en primer uso, no en asignación

#### Tareas:

1. **Análisis de Uso (semantic.rs)**
   - [ ] Tracking de variables Task<T>
   - [ ] Detectar primer uso (campo/método/operación)
   - [ ] Marcar punto de await en AST
   
2. **Codegen (codegen.rs)**
   - [ ] NO generar await en asignación
   - [ ] Generar await en primer uso
   - [ ] Cache de valor awaited
   
3. **Type System (semantic.rs)**
   - [ ] Tipo Task<T> antes de await
   - [ ] Tipo T después de await
   - [ ] Inferencia correcta

#### Ejemplo Target:

**Liva:**
```liva
let user = async getUser()  // Task<User>, NO await aquí
print("loading...")         // corre mientras fetch
print(user.name)            // await AQUÍ, ahora es User
print(user.email)           // ya es User, no await
```

**Rust generado:**
```rust
let user_task = tokio::spawn(async move { get_user() });
println!("loading...");

// Await en primer uso
let user = user_task.await.unwrap();
println!("{}", user.name);
println!("{}", user.email);
```

#### Tests:
- [ ] `test_lazy_await_basic.liva`
- [ ] `test_lazy_await_field_access.liva`
- [ ] `test_lazy_await_method_call.liva`
- [ ] `test_lazy_await_operation.liva`
- [ ] `test_lazy_await_multiple_tasks.liva`

---

### FASE 3: Warnings y Lints (1 semana) 🟢 NICE TO HAVE

**Objetivo:** Avisar al usuario de problemas comunes

#### Warnings:

1. **Task no usada**
   ```liva
   let user = async getUser()
   // nunca se usa user
   
   Warning: unused variable 'user'
   Help: use `fire async getUser()` if you don't need the result
   ```

2. **Error no chequeado**
   ```liva
   let user, err = async getUser()
   print(user.name)  // no chequeó err
   
   Warning: 'err' not checked before using 'user'
   Help: check `if err` before accessing 'user'
   ```

3. **Task en condición sin uso en else**
   ```liva
   let user = async getUser()
   if condition {
       print(user.name)
   }
   // Si condition es false, task nunca se usa
   
   Warning: 'user' may not be used in all branches
   ```

#### Tests:
- [ ] `test_warning_unused_task.liva`
- [ ] `test_warning_unchecked_error.liva`
- [ ] `test_warning_conditional_unused.liva`

---

### FASE 4: Optimizaciones (2-3 semanas) 🟢 PERFORMANCE

**Objetivo:** Generar código Rust más eficiente

#### Optimizaciones:

1. **Join Combining**
   ```rust
   // En vez de:
   let u1 = t1.await;
   let u2 = t2.await;
   
   // Generar:
   let (u1, u2) = tokio::join!(t1, t2);
   ```

2. **Dead Task Elimination**
   ```liva
   let user = async getUser()
   // nunca se usa
   
   // No generar el spawn
   ```

3. **Task Inlining**
   ```liva
   let x = async simple()  // función muy pequeña
   print(x)
   
   // Generar directamente:
   let x = simple();
   ```

#### Tests:
- [ ] Benchmarks de performance
- [ ] Tests de optimización
- [ ] Comparación con código manual

---

### FASE 5: Features Avanzadas (4+ semanas) 🔵 FUTURO

1. **Task handles explícitos**
   ```liva
   let task = task async getUser()
   let user = task.await
   ```

2. **Fire and forget**
   ```liva
   fire async logEvent()
   ```

3. **Async iterators**
   ```liva
   for async item in fetchItems() {
       print(item)
   }
   ```

4. **Cancelación**
   ```liva
   let task = task async longOperation()
   task.cancel()
   ```

---

## 🧪 ESTRATEGIA DE TESTING

### Tests por Fase:

```
tests/concurrency/
├── phase1_error_handling/
│   ├── ok_error_binding.liva
│   ├── ok_error_check.liva
│   ├── ok_error_ignore.liva
│   └── err_wrong_binding.liva
├── phase2_lazy_await/
│   ├── ok_await_on_field.liva
│   ├── ok_await_on_method.liva
│   ├── ok_await_on_operation.liva
│   └── ok_multiple_tasks.liva
├── phase3_warnings/
│   ├── warn_unused_task.liva
│   ├── warn_unchecked_error.liva
│   └── warn_conditional_task.liva
└── phase4_optimizations/
    ├── bench_join_combining.liva
    └── bench_task_inlining.liva
```

### Comandos de Testing:

```bash
# Tests de fase específica
cargo test --test concurrency_tests -- phase1

# Tests con output
cargo test --test concurrency_tests -- --nocapture

# Benchmarks
cargo bench concurrency

# Tests de integración
./run_concurrency_tests.sh
```

---

## 📊 MÉTRICAS DE ÉXITO

### Fase 1:
- [ ] 100% tests passing
- [ ] Error handling funcional
- [ ] Código Rust generado correcto

### Fase 2:
- [ ] Await solo en primer uso
- [ ] Performance igual o mejor
- [ ] Tipo correcto antes/después await

### Fase 3:
- [ ] Warnings útiles
- [ ] No false positives
- [ ] Mensajes claros

### Fase 4:
- [ ] 20% mejora en performance (promedio)
- [ ] Código generado más limpio
- [ ] Sin regresiones

---

## 🔄 WORKFLOW

### Ciclo de Desarrollo:

1. **Implementar feature**
   - Escribir código
   - Agregar tests
   - Documentar cambios

2. **Validar**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

3. **Commit**
   ```bash
   git add .
   git commit -m "feat(concurrency): implement error binding"
   ```

4. **Push periódico**
   ```bash
   git push origin feature/concurrency-improvements
   ```

5. **Review incremental**
   - Cada fase es reviewable
   - Merge progresivo si se desea

---

## 📝 CONVENCIONES DE COMMITS

### Prefijos:

- `feat(concurrency):` - Nueva feature
- `fix(concurrency):` - Bug fix
- `test(concurrency):` - Tests
- `docs(concurrency):` - Documentación
- `refactor(concurrency):` - Refactoring
- `perf(concurrency):` - Performance

### Ejemplos:

```bash
git commit -m "feat(concurrency): add error binding in parser"
git commit -m "test(concurrency): add tests for lazy await"
git commit -m "fix(concurrency): correct await insertion point"
git commit -m "docs(concurrency): update spec with implementation notes"
```

---

## 🎯 PRIORIDADES

### Semana 1-2: FASE 1 (Error Handling) 🔴
**Crítico:** Necesario para producción

### Semana 3-5: FASE 2 (Lazy Await) 🟡
**Importante:** Característica distintiva

### Semana 6: FASE 3 (Warnings) 🟢
**Nice to have:** Mejora experiencia

### Semana 7+: FASE 4 (Optimizaciones) 🟢
**Performance:** Cuando lo básico funcione

---

## 📚 REFERENCIAS

- `docs/CONCURRENCIA_SISTEMA.md` - Especificación técnica completa
- `docs/AUDITORIA_COMPLETA_LIVA.md` - Auditoría y análisis
- `docs/Liva_v0.6_spec.md` - Spec general del lenguaje

---

## 🤝 COLABORACIÓN

### Daily Standup:
- ¿Qué hiciste ayer?
- ¿Qué harás hoy?
- ¿Hay bloqueadores?

### Weekly Review:
- Demo de features completadas
- Revisión de tests
- Planificación próxima semana

### Branch Protection:
- Tests deben pasar
- Code review requerido
- No commits directos a main

---

## ✅ CHECKLIST DE INICIO

- [x] Rama creada: `feature/concurrency-improvements`
- [x] Documentación en `docs/`
- [x] Plan de trabajo creado
- [ ] Tests base creados
- [ ] Equipo notificado
- [ ] Ambiente de desarrollo listo

---

## 🚀 ESTADO ACTUAL

**Rama:** `feature/concurrency-improvements`  
**Fase actual:** Preparación  
**Próximo:** Implementar Fase 1 (Error Handling)

**Comando para empezar:**
```bash
cd /home/fran/Projects/Liva/livac
git checkout feature/concurrency-improvements
cargo test  # Verificar que todo funciona
```

---

**Última actualización:** 18 de octubre de 2025  
**Mantenido por:** Equipo Liva

¡Vamos a hacer que Liva tenga el mejor sistema de concurrencia! 🎉
