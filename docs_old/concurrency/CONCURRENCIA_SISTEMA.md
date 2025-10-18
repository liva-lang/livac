# 🔄 Sistema de Concurrencia de Liva - Especificación Técnica Completa

**Versión:** 1.1  
**Fecha:** 18 de octubre de 2025  
**Estado:** Especificación de Referencia  
**Autores:** Equipo Liva

---

## � DOCUMENTACIÓN RELACIONADA

Este documento contiene la especificación técnica completa del sistema de concurrencia. Para otros aspectos:

- **[EXECUTION_MODES.md](EXECUTION_MODES.md)** - Las 7 formas de ejecutar funciones (LEER PRIMERO)
- **[ERROR_HANDLING.md](ERROR_HANDLING.md)** - Manejo de errores en contextos concurrentes
- **[PLAN_CONCURRENCIA.md](PLAN_CONCURRENCIA.md)** - Roadmap de implementación
- **[PHASE1_PROGRESS.md](PHASE1_PROGRESS.md)** - Estado actual de implementación
- **[README.md](README.md)** - Índice completo de documentación

---

## �📋 TABLA DE CONTENIDOS

1. [Visión y Filosofía](#visión-y-filosofía)
2. [Sintaxis Completa](#sintaxis-completa)
3. [Semántica de Ejecución](#semántica-de-ejecución)
4. [Sistema de Tipos](#sistema-de-tipos)
5. [Error Handling](#error-handling)
6. [Compilación a Rust](#compilación-a-rust)
7. [Edge Cases y Reglas](#edge-cases-y-reglas)
8. [Optimizaciones](#optimizaciones)
9. [Ejemplos Completos](#ejemplos-completos)
10. [Comparación con Otros Lenguajes](#comparación-con-otros-lenguajes)

---

## 🎯 VISIÓN Y FILOSOFÍA

### Principios de Diseño

**1. Separación de Concerns**
```liva
// La FUNCIÓN define QUÉ hace
getUser(id: number): User {
    let response = http.get($"/users/{id}")
    return response.json()
}

// La LLAMADA define CÓMO se ejecuta
let u1 = getUser(1)            // síncrono
let u2 = async getUser(2)      // asíncrono (IO-bound)
let u3 = par getUser(3)        // paralelo (CPU-bound)
let u4 = task async getUser(4) // handle manual
let u5 = task par getUser(5)   // handle paralelo manual
fire async getUser(6)          // fire-and-forget async
fire par getUser(7)            // fire-and-forget parallel
```

**Ver [EXECUTION_MODES.md](EXECUTION_MODES.md) para detalles de cada modo.**

**2. Inferencia Total**
```liva
// Sin anotaciones de tipo explícitas
let user = async getUser()
//  ^^^^ tipo inferido: Task<User>

// El compilador lo sabe todo
print(user.name)  // await implícito aquí
```

**3. Lazy Evaluation**
```liva
let user = async getUser()  // spawn task AHORA
print("Loading...")         // corre mientras fetch
print(user.name)            // await AQUÍ (primer uso)
```

**4. Error Handling como Valores**
```liva
let user, err = async getUser()
//       ^^^ error es un valor, no una excepción
if err {
    // manejar error
}
```

### Ventajas del Diseño

| Aspecto | Liva | Rust | JavaScript | Go | Python |
|---------|------|------|------------|----|---------| 
| **Sintaxis limpia** | ✅ | ⚠️ | ✅ | ✅ | ✅ |
| **Inferencia de tipos** | ✅ | ✅ | ⚠️ | ❌ | ⚠️ |
| **Await implícito** | ✅ | ❌ | ❌ | ✅ | ❌ |
| **Lazy evaluation** | ✅ | ⚠️ | ❌ | ✅ | ❌ |
| **Error handling** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Paralelismo real** | ✅ | ✅ | ❌ | ✅ | ⚠️ |

---

## 📝 SINTAXIS COMPLETA

### Declaración de Funciones

```liva
// Funciones normales (SIN async/par en declaración)
getUser(id: number): User {
    // implementación
}

compute(n: number): number {
    // CPU-intensive
}

// Funciones que retornan Result explícitamente
getUser(id: number): Result<User, Error> {
    if !valid(id) {
        return Err("Invalid ID")
    }
    return Ok(fetchUser(id))
}
```

**Regla:** Las funciones NUNCA se declaran como `async` o `parallel`. Son funciones normales.

### Ejecución Asíncrona

```liva
// Sintaxis base
let variable = async function_call()

// Con argumentos
let user = async getUser(1)
let data = async fetchData("https://api.com")

// Con métodos
let result = async object.method()
```

**Tipo inferido:** `Task<T>` donde `T` es el tipo de retorno de la función.

### Ejecución Paralela

```liva
// Sintaxis base
let variable = par function_call()

// CPU-bound
let result = par fibonacci(40)
let hash = par computeHash(data)

// También funciona con IO (menos común)
let user = par getUser(1)  // válido pero subóptimo
```

**Tipo inferido:** `Task<T>` (mismo que async, diferente runtime).

### Error Handling

```liva
// Sin error handling (puede panic)
let user = async getUser()

// Con error handling (safe)
let user, err = async getUser()

// Ignorar error explícitamente
let user, _ = async getUser()
```

### Fire and Forget

```liva
// Ejecutar sin esperar resultado
fire async logEvent("user_login")
fire par rebuildCache()

// Equivalente a:
let _ = async logEvent("user_login")
let _ = par rebuildCache()
```

### Task Explícita

```liva
// Obtener handle explícito (sin await automático)
let task = task async getUser()
//  ^^^^ tipo: Task<User>, no se await automáticamente

// Await manual cuando lo necesites
let user = task.await
```

---

## ⚙️ SEMÁNTICA DE EJECUCIÓN

### Ciclo de Vida de una Task

```liva
// 1. SPAWN: Task se crea y empieza a ejecutar
let user = async getUser()
//         ^^^^^ tokio::spawn() se ejecuta AHORA

// 2. CONCURRENT: Código síncrono corre mientras task ejecuta
print("Loading...")
doOtherStuff()
expensiveComputation()

// 3. AWAIT: Primera vez que se usa un campo/método
print(user.name)
//    ^^^^ await implícito AQUÍ

// 4. MATERIALIZED: Task ya está resuelta
print(user.email)  // No await, valor ya disponible
print(user.age)    // No await, valor ya disponible
```

### Reglas de Await Implícito

**Regla 1: Await en Primer Uso de Campo/Método**

```liva
let user = async getUser()

// Esto NO causa await (solo es la variable)
let u = user

// Esto SÍ causa await (acceso a campo)
let name = user.name
//         ^^^^ await aquí

// Ya no se await más
let email = user.email  // valor ya disponible
```

**Regla 2: Await en Retorno de Función**

```liva
myFunc(): User {
    let user = async getUser()
    return user  // await implícito aquí
}
```

**Regla 3: Await en Paso a Función**

```liva
processUser(user: User) {
    // ...
}

main() {
    let user = async getUser()
    processUser(user)  // await implícito aquí
}
```

**Regla 4: Await en Condicionales**

```liva
let user = async getUser()

if user.isAdmin {
//  ^^^^ await aquí
    print("Admin")
}
```

**Regla 5: Await en Operaciones**

```liva
let result = async compute()

let x = result + 10  // await implícito en result
let y = result * 2   // ya no se await, valor disponible
```

### Paralelismo Automático

```liva
// Múltiples tasks se ejecutan en paralelo automáticamente
let user1 = async getUser(1)  // spawn
let user2 = async getUser(2)  // spawn
let user3 = async getUser(3)  // spawn

// Todas se ejecutan concurrentemente
print("Loading 3 users...")

// Awaits en orden (pero ya ejecutándose)
print(user1.name)  // await user1
print(user2.name)  // await user2
print(user3.name)  // await user3
```

**Nota:** No necesitas `Promise.all()` o similar. El compilador lo hace automáticamente.

---

## 🔢 SISTEMA DE TIPOS

### Tipo Task<T>

```liva
// Inferido automáticamente
let user = async getUser()
//  ^^^^ tipo: Task<User>

// Puede anotarse (opcional)
let user: User = async getUser()
//       ^^^^ anotación del tipo FINAL, no Task
```

### Reglas de Inferencia

**1. async/par produce Task<T>**
```liva
getUser(): User
let u = async getUser()  // Task<User>
```

**2. Primer uso materializa a T**
```liva
let u = async getUser()  // Task<User>
print(u.name)            // u ahora es User
```

**3. Con error handling**
```liva
let user, err = async getUser()
//  ^^^^  ^^^ tipo: (User, Option<Error>)
```

### Type Checking

**El compilador verifica:**

```liva
getUser(): User

// OK
let user = async getUser()
print(user.name)  // User tiene campo name

// ERROR
print(user.nonExistent)  // User no tiene campo nonExistent
```

**Error en compilación:**
```
Error: Field 'nonExistent' not found on type 'User'
  → main.liva:5:12
  |
5 | print(user.nonExistent)
  |            ^^^^^^^^^^^
  |
  = note: User has fields: name, email, age
```

---

## 🛡️ ERROR HANDLING

### Sin Error Handling (Panic)

```liva
let user = async getUser()
print(user.name)  // puede panic si getUser falla
```

**Rust generado:**
```rust
let user_handle = tokio::spawn(get_user());
let user = user_handle.await.unwrap();  // panic aquí si falla
println!("{}", user.name);
```

### Con Error Handling (Safe)

```liva
let user, err = async getUser()
if err {
    print($"Error: {err}")
    return
}
print(user.name)  // seguro
```

**Rust generado:**
```rust
let user_handle = tokio::spawn(get_user());
let result = user_handle.await;

let (user, err) = match result {
    Ok(Ok(u)) => (u, None),
    Ok(Err(e)) => (User::default(), Some(e)),
    Err(e) => (User::default(), Some(e.into())),
};

if err.is_some() {
    println!("Error: {}", err.unwrap());
    return;
}
println!("{}", user.name);
```

### Ignorar Error Explícitamente

```liva
let user, _ = async getUser()
print(user.name)  // puede panic, pero intención clara
```

### Try/Catch (Alternativa)

```liva
try {
    let user = async getUser()
    print(user.name)
} catch (e: NetworkError) {
    print($"Network error: {e}")
} catch (e: ParseError) {
    print($"Parse error: {e}")
} catch (e) {
    print($"Unknown error: {e}")
}
```

---

## 🔨 COMPILACIÓN A RUST

### Función Simple

**Liva:**
```liva
getUser(id: number): User {
    let response = http.get($"/users/{id}")
    return response.json()
}
```

**Rust:**
```rust
fn get_user(id: i32) -> User {
    let response = http::get(&format!("/users/{}", id));
    response.json()
}
```

### Llamada Async

**Liva:**
```liva
main() {
    let user = async getUser(1)
    print("Loading...")
    print(user.name)
}
```

**Rust:**
```rust
#[tokio::main]
async fn main() {
    // Spawn task
    let user_task = tokio::spawn(async move {
        get_user(1)
    });
    
    println!("Loading...");
    
    // Await antes de usar
    let user = user_task.await.unwrap();
    println!("{}", user.name);
}
```

### Llamada Par

**Liva:**
```liva
main() {
    let result = par fibonacci(40)
    print("Computing...")
    print(result)
}
```

**Rust:**
```rust
fn main() {
    // Spawn thread
    let handle = std::thread::spawn(move || {
        fibonacci(40)
    });
    
    println!("Computing...");
    
    // Join antes de usar
    let result = handle.join().unwrap();
    println!("{}", result);
}
```

### Múltiples Tasks

**Liva:**
```liva
main() {
    let u1 = async getUser(1)
    let u2 = async getUser(2)
    let u3 = async getUser(3)
    
    print(u1.name)
    print(u2.name)
    print(u3.name)
}
```

**Rust:**
```rust
#[tokio::main]
async fn main() {
    // Spawn todas
    let task1 = tokio::spawn(async move { get_user(1) });
    let task2 = tokio::spawn(async move { get_user(2) });
    let task3 = tokio::spawn(async move { get_user(3) });
    
    // Await en orden
    let u1 = task1.await.unwrap();
    println!("{}", u1.name);
    
    let u2 = task2.await.unwrap();
    println!("{}", u2.name);
    
    let u3 = task3.await.unwrap();
    println!("{}", u3.name);
}
```

### Con Error Handling

**Liva:**
```liva
main() {
    let user, err = async getUser(1)
    if err {
        print($"Error: {err}")
        return
    }
    print(user.name)
}
```

**Rust:**
```rust
#[tokio::main]
async fn main() {
    let task = tokio::spawn(async move { get_user(1) });
    let result = task.await;
    
    let (user, err) = match result {
        Ok(Ok(u)) => (u, None),
        Ok(Err(e)) => (User::default(), Some(e)),
        Err(e) => (User::default(), Some(e.into())),
    };
    
    if let Some(e) = err {
        println!("Error: {}", e);
        return;
    }
    
    println!("{}", user.name);
}
```

---

## 📐 EDGE CASES Y REGLAS

### Caso 1: Task No Usada

**Código:**
```liva
let user = async getUser()
// nunca se usa user
```

**Comportamiento:**
- Task se ejecuta (ya fue spawned)
- No se hace await nunca
- Si retorna valor, se descarta
- Warning del compilador:

```
Warning: unused variable 'user'
  → main.liva:2:5
  |
2 | let user = async getUser()
  |     ^^^^ this task result is never used
  |
  = help: use `fire async getUser()` if you don't need the result
```

### Caso 2: Task en Condicional

**Código:**
```liva
let user = async getUser()
if someCondition {
    print(user.name)
}
```

**Comportamiento:**
- Task se spawn ANTES del if
- Task corre independientemente del condition
- Await solo si se entra al if
- Si no se entra, task completa pero no se usa

**Optimización posible:** El compilador podría detectar que `user` solo se usa en el if y mover el spawn dentro, pero por ahora no lo hace.

### Caso 3: Task Retornada

**Código:**
```liva
getAndReturn(): User {
    let user = async getUser()
    return user
}
```

**Comportamiento:**
- Task se spawn dentro de la función
- Await implícito en el return
- La función retorna `User`, no `Task<User>`

**Rust:**
```rust
async fn get_and_return() -> User {
    let task = tokio::spawn(async move { get_user() });
    task.await.unwrap()
}
```

### Caso 4: Task Pasada a Función

**Código:**
```liva
processUser(user: User) {
    print(user.name)
}

main() {
    let user = async getUser()
    processUser(user)
}
```

**Comportamiento:**
- Await implícito ANTES de llamar a `processUser`
- `processUser` recibe `User`, no `Task<User>`

**Rust:**
```rust
fn process_user(user: User) {
    println!("{}", user.name);
}

#[tokio::main]
async fn main() {
    let task = tokio::spawn(async move { get_user() });
    let user = task.await.unwrap();
    process_user(user);
}
```

### Caso 5: Task en Expresión

**Código:**
```liva
let result = async compute() + 10
```

**Comportamiento:**
- Await implícito antes de la suma
- `compute()` se resuelve primero
- Luego se suma 10

**Rust:**
```rust
let task = tokio::spawn(async move { compute() });
let value = task.await.unwrap();
let result = value + 10;
```

### Caso 6: Task en Loop

**Código:**
```liva
for i in 0..5 {
    let user = async getUser(i)
    print(user.name)
}
```

**Comportamiento:**
- Cada iteración spawn nueva task
- Cada iteración await su propia task
- Tasks NO corren en paralelo (await en el loop)

**Para paralelismo:**
```liva
// Mejor: spawn todas, luego await todas
let users = []
for i in 0..5 {
    users.push(async getUser(i))
}

for user in users {
    print(user.name)  // await aquí
}
```

### Caso 7: Task con Lifetime

**Código:**
```liva
main() {
    let data = "some data"
    let result = async process(data)
    print(result)
}
```

**Comportamiento:**
- `data` debe vivir hasta el await
- Compilador verifica lifetimes
- Error si `data` se libera antes

**Rust:**
```rust
async fn main() {
    let data = "some data";
    let task = tokio::spawn(async move {
        process(data)  // move captura data
    });
    let result = task.await.unwrap();
    println!("{}", result);
}
```

---

## ⚡ OPTIMIZACIONES

### Optimización 1: Await Combining

**Código:**
```liva
let u1 = async getUser(1)
let u2 = async getUser(2)
print(u1.name)
print(u2.name)
```

**Naive:**
```rust
let t1 = tokio::spawn(get_user(1));
let t2 = tokio::spawn(get_user(2));
let u1 = t1.await.unwrap();  // await 1
println!("{}", u1.name);
let u2 = t2.await.unwrap();  // await 2
println!("{}", u2.name);
```

**Optimizado:**
```rust
let t1 = tokio::spawn(get_user(1));
let t2 = tokio::spawn(get_user(2));
let (u1, u2) = tokio::join!(t1, t2);  // await ambos en paralelo
let u1 = u1.unwrap();
let u2 = u2.unwrap();
println!("{}", u1.name);
println!("{}", u2.name);
```

### Optimización 2: Dead Task Elimination

**Código:**
```liva
let user = async getUser()
// nunca se usa
```

**Naive:**
```rust
let task = tokio::spawn(get_user());
// task nunca se await
```

**Optimizado:**
```rust
// El compilador elimina la tarea completamente
// O emite warning y la deja
```

### Optimización 3: Inline Small Tasks

**Código:**
```liva
let x = async simple_function()
print(x)
```

**Si `simple_function()` es pequeña:**
```rust
// En vez de spawn
let x = simple_function();  // inline
println!("{}", x);
```

### Optimización 4: Task Reordering

**Código:**
```liva
let u1 = async getUser(1)
expensive_sync_computation()
print(u1.name)
```

**Optimizado:**
```rust
let t1 = tokio::spawn(get_user(1));
expensive_sync_computation();  // corre mientras fetch
let u1 = t1.await.unwrap();
println!("{}", u1.name);
```

**El compilador NO reordena tasks para preservar semántica observable.**

---

## 💡 EJEMPLOS COMPLETOS

### Ejemplo 1: API Client

```liva
use rust "reqwest" as http

User {
    id: number
    name: string
    email: string
}

getUser(id: number): Result<User, Error> {
    let url = $"https://api.example.com/users/{id}"
    let response = http.get(url)
    
    if response.status != 200 {
        return Err($"HTTP {response.status}")
    }
    
    return Ok(response.json())
}

main() {
    // Fetch 3 usuarios en paralelo
    let u1, err1 = async getUser(1)
    let u2, err2 = async getUser(2)
    let u3, err3 = async getUser(3)
    
    // Mostrar resultados
    if !err1 { print($"User 1: {u1.name}") }
    if !err2 { print($"User 2: {u2.name}") }
    if !err3 { print($"User 3: {u3.name}") }
}
```

### Ejemplo 2: CPU-Bound Processing

```liva
fibonacci(n: number): number {
    if n <= 1 return n
    return fibonacci(n - 1) + fibonacci(n - 2)
}

main() {
    print("Computing Fibonacci numbers...")
    
    // Paralelo en diferentes threads
    let f35 = par fibonacci(35)
    let f36 = par fibonacci(36)
    let f37 = par fibonacci(37)
    let f38 = par fibonacci(38)
    
    print("All tasks spawned, waiting for results...")
    
    // Joins implícitos
    print($"fib(35) = {f35}")
    print($"fib(36) = {f36}")
    print($"fib(37) = {f37}")
    print($"fib(38) = {f38}")
}
```

### Ejemplo 3: Mixed Workload

```liva
use rust "reqwest" as http

fetchUserData(id: number): User {
    // IO-bound
    let response = http.get($"/users/{id}")
    return response.json()
}

processUserData(user: User): ProcessedData {
    // CPU-bound
    let hash = computeExpensiveHash(user)
    let analysis = analyzeData(user)
    return ProcessedData(hash, analysis)
}

main() {
    // IO-bound: async
    let user1 = async fetchUserData(1)
    let user2 = async fetchUserData(2)
    
    print("Fetching users...")
    
    // CPU-bound: parallel
    let processed1 = par processUserData(user1)
    let processed2 = par processUserData(user2)
    
    print("Processing...")
    
    // Usar resultados
    print($"User 1: {processed1.hash}")
    print($"User 2: {processed2.hash}")
}
```

### Ejemplo 4: Pipeline Processing

```liva
fetchData(): RawData {
    // fetch from API
}

processData(data: RawData): ProcessedData {
    // CPU-intensive processing
}

saveData(data: ProcessedData): bool {
    // write to database
}

main() {
    // Pipeline asíncrono
    let raw = async fetchData()
    let processed = par processData(raw)  // await raw, luego process
    let saved = async saveData(processed) // await processed, luego save
    
    if saved {
        print("Pipeline completed successfully")
    } else {
        print("Pipeline failed")
    }
}
```

### Ejemplo 5: Error Handling Completo

```liva
fetchUserWithRetry(id: number): Result<User, Error> {
    let maxRetries = 3
    
    for attempt in 0..maxRetries {
        let user, err = async getUser(id)
        
        if !err {
            return Ok(user)
        }
        
        print($"Attempt {attempt + 1} failed: {err}")
        sleep(1000)  // wait 1 second
    }
    
    return Err("Max retries exceeded")
}

main() {
    let user, err = fetchUserWithRetry(1)
    
    if err {
        print($"Failed to fetch user: {err}")
        return
    }
    
    print($"Successfully fetched: {user.name}")
}
```

---

## 🔄 COMPARACIÓN CON OTROS LENGUAJES

### vs Rust

**Rust:**
```rust
async fn get_user(id: i32) -> User {
    // implementación
}

#[tokio::main]
async fn main() {
    let user = get_user(1).await;
    println!("{}", user.name);
}
```

**Liva:**
```liva
getUser(id: number): User {
    // implementación
}

main() {
    let user = async getUser(1)
    print(user.name)
}
```

**Diferencias:**
- Liva: `async` en llamada, await implícito
- Rust: `async` en declaración, await explícito

### vs JavaScript/TypeScript

**JavaScript:**
```javascript
async function getUser(id) {
    // implementación
}

async function main() {
    const user = await getUser(1);
    console.log(user.name);
}
```

**Liva:**
```liva
getUser(id: number): User {
    // implementación
}

main() {
    let user = async getUser(1)
    print(user.name)
}
```

**Diferencias:**
- Liva: await implícito, paralelismo real
- JS: await explícito, sin paralelismo real (excepto Workers)

### vs Go

**Go:**
```go
func getUser(id int) User {
    // implementación
}

func main() {
    ch := make(chan User)
    go func() {
        ch <- getUser(1)
    }()
    
    user := <-ch
    fmt.Println(user.Name)
}
```

**Liva:**
```liva
getUser(id: number): User {
    // implementación
}

main() {
    let user = async getUser(1)
    print(user.name)
}
```

**Diferencias:**
- Liva: sintaxis más limpia, inferencia de tipos
- Go: channels explícitos, menos inferencia

### vs Python

**Python:**
```python
async def get_user(id: int) -> User:
    # implementación

async def main():
    user = await get_user(1)
    print(user.name)

asyncio.run(main())
```

**Liva:**
```liva
getUser(id: number): User {
    // implementación
}

main() {
    let user = async getUser(1)
    print(user.name)
}
```

**Diferencias:**
- Liva: await implícito, compilado, más rápido
- Python: await explícito, interpretado, más lento

### Tabla Comparativa

| Feature | Liva | Rust | JavaScript | Go | Python |
|---------|------|------|------------|----|---------| 
| **Sintaxis** | `async call()` | `call().await` | `await call()` | `go call()` | `await call()` |
| **Declaración** | Normal | `async fn` | `async function` | Normal | `async def` |
| **Await** | Implícito | Explícito | Explícito | Canal | Explícito |
| **Inferencia** | Total | Parcial | Parcial | No | Parcial |
| **Paralelismo** | Real | Real | No | Real | Limitado |
| **Type safety** | Fuerte | Fuerte | Débil | Medio | Débil |
| **Error handling** | Valores | Result | Try/catch | Valores | Try/except |
| **Performance** | Alta | Alta | Media | Alta | Baja |

---

## 📊 ROADMAP DE IMPLEMENTACIÓN

### Fase 1: Core (ACTUAL)

- [x] Sintaxis básica `async`/`par`
- [x] Spawn tasks
- [x] Await implícito básico
- [ ] Error handling con dos variables
- [ ] Warnings para tasks no usadas
- [ ] Tests exhaustivos

### Fase 2: Optimizaciones

- [ ] Join combining (`tokio::join!`)
- [ ] Dead task elimination
- [ ] Task inlining
- [ ] Smart reordering

### Fase 3: Features Avanzadas

- [ ] `task` keyword para handles explícitos
- [ ] `fire` keyword para fire-and-forget
- [ ] Async iterators
- [ ] Async closures

### Fase 4: Tooling

- [ ] Debugger con task visualization
- [ ] Profiler con concurrency metrics
- [ ] Linter rules para concurrency
- [ ] IDE hints para await points

---

## 📚 REFERENCIAS

### Documentos Relacionados

- `AUDITORIA_COMPLETA_LIVA.md` - Auditoría general del lenguaje
- `docs/Liva_v0.6_spec.md` - Especificación completa del lenguaje
- `docs/Liva_v0.6_EBNF_AST.md` - Gramática formal
- `README.md` - Guía general del compilador

### Papers y Recursos

- **Rust Async Book:** https://rust-lang.github.io/async-book/
- **Tokio Documentation:** https://tokio.rs/
- **Go Concurrency Patterns:** https://go.dev/blog/pipelines
- **Effect Systems in Programming Languages** (Research)

### Ejemplos de Código

- `livac/main.liva` - Ejemplos completos
- `livac/tests/codegen/ok_async_*.liva` - Tests async
- `livac/tests/codegen/ok_parallel_*.liva` - Tests parallel

---

## ❓ FAQ

### ¿Por qué async/par en la llamada y no en la declaración?

**Separación de concerns.** La función define lógica, la llamada define estrategia de ejecución. Permite reusar la misma función sync, async, o parallel según necesidad.

### ¿Cómo sé cuándo usar async vs par?

- **async:** IO-bound (network, disk, database)
- **par:** CPU-bound (cálculos, procesamiento)

### ¿Qué pasa si uso par para IO?

Funciona, pero es subóptimo. Crea threads del OS que se bloquean esperando IO. Mejor usar async.

### ¿Qué pasa si uso async para CPU?

Funciona, pero no gana performance. Tokio usa threads pero con cooperative scheduling. CPU-bound bloquea el thread. Mejor usar par.

### ¿Cuándo se hace el await exactamente?

En el **primer uso** de un campo, método, operación, o paso a función que requiere el valor concreto.

### ¿Puedo hacer await explícito?

Sí, con `.await`:
```liva
let user = async getUser()
let u = user.await  // await explícito
```

### ¿Cómo manejo errores?

Con dos variables:
```liva
let value, err = async call()
if err {
    // manejar
}
```

### ¿Puedo ignorar errores?

Sí, con una variable (puede panic) o con `_`:
```liva
let value = async call()       // panic si falla
let value, _ = async call()    // panic si falla (explícito)
```

### ¿Las tasks corren en paralelo?

Sí, automáticamente:
```liva
let a = async f1()  // spawn
let b = async f2()  // spawn (corre en paralelo con f1)
let c = async f3()  // spawn (corre en paralelo con f1 y f2)
```

### ¿Cómo cancelo una task?

Actualmente no hay API de cancelación. Próxima feature.

### ¿Cómo espero múltiples tasks?

Automático al usarlas:
```liva
let a = async f1()
let b = async f2()
// ambas corren en paralelo

print(a)  // await a
print(b)  // await b
```

O con task explícita para control manual.

---

## 🎯 CONCLUSIÓN

El sistema de concurrencia de Liva es:

✅ **Innovador:** Único en la industria  
✅ **Simple:** Sintaxis mínima  
✅ **Poderoso:** Paralelismo real  
✅ **Seguro:** Type-safe con inferencia  
✅ **Ergonómico:** Await implícito  
✅ **Eficiente:** Compila a Rust optimizado  

Este diseño posiciona a Liva como un lenguaje de siguiente generación que combina lo mejor de múltiples paradigmas en un sistema coherente y elegante.

---

**Versión:** 1.0  
**Última actualización:** 18 de octubre de 2025  
**Mantenido por:** Equipo Liva

Para preguntas o discusiones sobre este documento, abrir un issue en el repositorio o contactar al equipo de desarrollo.
