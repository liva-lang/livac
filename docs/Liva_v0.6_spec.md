# 📘 Liva — Especificación del Lenguaje (v0.6)

> *Tan simple como TypeScript. Tan expresivo como Python. Tan seguro y rápido como Rust.*

---

## 🧠 1. Filosofía general

**Liva** es un lenguaje de alto nivel con tipado fuerte e inferido, diseñado para compilar directamente a **Rust**, heredando su seguridad de memoria y rendimiento, pero con una sintaxis legible, natural e intuitiva.

### Principios:
1. **Sintaxis clara y mínima** — sin palabras innecesarias.  
2. **Tipado fuerte** con inferencia automática.  
3. **Concurrencia híbrida** (asincronía + paralelismo real).  
4. **Encapsulación real** (`_` y `__`), pero sin ruido visual.  
5. **Compatibilidad total con Rust y sus crates.**

> **Estado del compilador (abril 2025):** la canalización real incluye un paso de **IR interno** y un módulo auxiliar `liva_rt` generado automáticamente cuando se usan primitivas `async`, `parallel`, `task` o `fire`. La validación semántica profunda está en curso y se endurecerá conforme avance el plan descrito en `docs/refactor_plan.md`.

---

## 🔤 2. Sintaxis básica

### Comentarios
```js
// Comentario simple
/* Comentario multilínea */
```

### Variables y constantes
```js
let nombre = "Fran"
let edad: number = 41
const PI = 3.1416
```

- `let` → mutable  
- `const` → inmutable  
- Tipos opcionales (`number`, `float`, `string`, `bool`, etc.)

---

## 🧩 3. Tipos de datos

### Tipos básicos
```js
number   // alias de i32
float    // alias de f64
bool
char
string
bytes
```

### Alias y compatibilidad Rust
Liva permite **usar directamente los tipos primitivos de Rust**:

```
i8, i16, i32, i64, i128, isize
u8, u16, u32, u64, u128, usize
f32, f64
bool, char, string, bytes
```

```js
let contador: u64 = 0
let temperatura: f32 = 21.5
let id: i128 = 999999999999
```

---

## 🧱 4. Estructuras y clases

No se usa `class` ni `fun`.  
Basta con el nombre del tipo y su bloque:

```js
Persona {
  nombre: string        // público
  _edad: number = 0     // protegido
  __dni: string         // privado

  saludar() {
    print($"Hola, soy {this.nombre}")
  }

  _info() {
    print($"Edad: {this._edad}")
  }

  __recalcularDni() {
    print("Solo dentro de la clase")
  }
}
```

### Traducción a Rust
```rust
pub struct Persona {
    pub nombre: String,
    pub(super) edad: i32,  // protected
    dni: String,           // private
}

impl Persona {
    pub fn saludar(&self) {
        println!("Hola, soy {}", self.nombre);
    }

    pub(super) fn info(&self) {
        println!("Edad: {}", self.edad);
    }

    fn recalcular_dni(&self) {
        println!("Solo dentro de la clase");
    }
}
```

---

## 🔐 5. Encapsulación

Inspirado en Python, pero con **protección real** en compilación.

| Prefijo | Nivel | Acceso permitido | Traducción Rust |
|----------|--------|------------------|------------------|
| *(ninguno)* | **public** | Desde cualquier módulo | `pub` |
| `_` | **protected** | Clase y subclases | `pub(super)` |
| `__` | **private** | Solo dentro de la clase | *(sin pub)* |
| `__nombre__` | Reservado | Motor del lenguaje | — |

```js
Empleado : Persona {
  mostrar() {
    print(this.nombre)   // ✅ público
    print(this._edad)    // ✅ protegido
    print(this.__dni)    // ❌ privado
  }
}
```

---

## ⚙️ 6. Operadores lógicos y aritméticos

Liva permite usar tanto **palabras naturales** (`and`, `or`, `not`) como los símbolos tradicionales (`&&`, `||`, `!`).

```js
if x > 18 and x < 65 {
  print("Working age")
}

if not isEmpty(name) && len(name) > 2 {
  print("Valid")
}
```

**Equivalencias en Rust:**

| Liva | Rust |
|--------------|------|
| `and` | `&&` |
| `or` | `||` |
| `not` | `!` |

**Precedencia:** `not` > `and` > `or`.

---

## 🧮 7. Funciones

### Declaración normal
```js
sum(a, b): number {
  return a + b
}
```

### Retorno implícito (una línea)
```js
sum(a, b): number => a + b
doble(x) => x * 2
isAdult(age) => age >= 18
```

### Con genéricos
```js
max<T>(a: T, b: T): T => (a > b) ? a : b
```

### Asincrónicas (auto-detectadas)
```js
fetchUser() {
  let res = async http.get("url")
  return res.json()
}
```

➡️ Traducción:
```rust
async fn fetch_user() -> User {
    let res = http::get("url").await.unwrap();
    res.json().await.unwrap()
}
```

---

## 🧭 8. Control de flujo

```js
if edad > 18 {
  print("Adulto")
} else {
  print("Menor")
}

for i in 0..10 {
  print(i)
}

while cond {
  break
}

switch color {
  case "red": print("Rojo")
  case "blue": print("Azul")
  default: print("Otro")
}
```

---

## 🧰 9. Módulos e imports

```js
import math
import net.http as http
```

➡️ Traducción:
```rust
use crate::math::*;
use crate::net::http as http;
```

---

## ⚡ 10. Concurrencia híbrida

Liva combina:
- **Asincronía cooperativa** (`async`) → sin hilos nuevos.
- **Paralelismo real** (`parallel`) → usa hilos del sistema operativo.

El modo se elige **en la llamada**, no en la definición.

---

### 🔹 Asincronía cooperativa (`async`)

```js
let user = async fetchUser()
print(user.name)
```

➡️ Traducción:
```rust
let user = tokio::spawn(fetch_user());
let user = user.await.unwrap();
```

---

### 🔹 Paralelismo real (`parallel`)

```js
let a = parallel heavyCalc(1)
let b = parallel heavyCalc(2)
print(a + b)
```

➡️ Traducción:
```rust
let a = std::thread::spawn(|| heavy_calc(1));
let b = std::thread::spawn(|| heavy_calc(2));
let a = a.join().unwrap();
let b = b.join().unwrap();
```

---

### 🔹 Fire and Forget

```js
fire async sendMetrics()
fire parallel rebuildCache()
```

➡️ Traducción:
```rust
tokio::spawn(send_metrics());
std::thread::spawn(|| rebuild_cache());
```

---

## 💬 11. Strings y plantillas

```js
let saludo = $"Hola {nombre}, tienes {edad} años"
```

➡️ Traducción:
```rust
let saludo = format!("Hola {}, tienes {} años", nombre, edad);
```

---

## 🔗 12. Interoperabilidad con Rust

### Importar crates
```js
use rust "serde_json"
use rust "reqwest" as http
use rust "blake3"
```

➡️ Añade a `Cargo.toml`:
```toml
[dependencies]
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
blake3 = "1.5"
```

### Usar funciones Rust
```js
let s = serde_json.to_string({ "name": "Fran" })
```

➡️ Traducción:
```rust
let s = serde_json::to_string(&json!({"name": "Fran"})).unwrap();
```

---

## 🧩 13. Tipos equivalentes

| Liva | Rust | Descripción |
|--------------|------|-------------|
| `number` | `i32` | entero |
| `float` | `f64` | decimal |
| `bool` | `bool` | lógico |
| `char` | `char` | carácter |
| `string` | `String` | texto |
| `bytes` | `Vec<u8>` | binario |
| `array<T>` | `Vec<T>` | lista |
| `{}` | `serde_json::Value` | mapa genérico |
| `Option<T>` | `Option<T>` | opcional |
| `Result<T,E>` | `Result<T,E>` | resultado seguro |

---

## 🧠 14. Detección automática de `async`

Si una función contiene llamadas `async`, el compilador la marca automáticamente como `async fn`.

```js
fetchData() {
  let res = async http.get("url")
  return res.text()
}
```

➡️ Traducción:
```rust
async fn fetch_data() -> String {
    let res = http::get("url").await.unwrap();
    res.text().await.unwrap()
}
```

---

## 🧩 15. Ejemplo completo

```js
use rust "reqwest" as http
use rust "blake3"

Persona {
  nombre: string
  _edad: number
  __dni: string

  saludar() {
    print($"Hola, soy {this.nombre}")
  }

  _hash() => blake3.hash(this.__dni)
}

main() {
  let p = Persona("Fran", 41, "XYZ123")
  async saveUser(p)
  fire async logEvent("created")
}
```

➡️ Traducción a Rust:
```rust
use reqwest as http;
use blake3;

pub struct Persona {
    pub nombre: String,
    pub(super) edad: i32,
    dni: String,
}

impl Persona {
    pub fn saludar(&self) {
        println!("Hola, soy {}", self.nombre);
    }

    pub(super) fn hash(&self) -> String {
        format!("{:x}", blake3::hash(self.dni.as_bytes()))
    }
}

#[tokio::main]
async fn main() {
    let p = Persona {
        nombre: "Fran".into(),
        edad: 41,
        dni: "XYZ123".into(),
    };

    tokio::spawn(save_user(p));
    tokio::spawn(log_event("created"));
}
```

---

## ⚙️ 16. Pipeline del compilador (`livac`)

1. **Lexer / Parser** → genera el AST.  
2. **Inferencia de tipos** → deduce tipos estáticos.  
3. **Análisis de concurrencia** → detecta `async`, `parallel`, `fire`.  
4. **Detección automática de `async`**.  
5. **Encapsulación real (`_` / `__`)**.  
6. **Desugaring** → genera código Rust (`tokio`, `thread`, etc.).  
7. **Interop resolver** → maneja `use rust`.  
8. **Codegen** → genera `Cargo.toml` y `main.rs`.  
9. **Cargo Runner** → compila y ejecuta.

---

## 🧠 17. Seguridad

- Sin `unsafe`.  
- Sin `null`.  
- Sin data races (gracias a Rust).  
- Propiedad y préstamos inferidos.  
- `Result` y `Option` por defecto.

---

## ⚡ 18. Resumen general

| Categoría | Liva | Rust |
|------------|--------------|-------|
| Variable | `let x = 1` | `let mut x = 1;` |
| Función | `f(a,b):T => expr` | `fn f(a: T, b: T) -> T { expr }` |
| Async interno | `f(){ let x = async g() }` | `async fn f()` |
| Concurrente | `async f()` | `tokio::spawn(f())` |
| Paralelo | `parallel f()` | `thread::spawn(|| f())` |
| Fire & Forget | `fire async f()` | `tokio::spawn(f());` |
| Privado | `__campo` | *(sin pub)* |
| Protegido | `_campo` | `pub(super)` |
| Público | `campo` | `pub` |
| `and/or/not` | `&&/||/!` | equivalentes |
| `number` | `i32` | tipo base entero |

---

## 🧩 19. Lema final

> **Liva**: el lenguaje que combina la legibilidad de Python, la simplicidad de TypeScript y la potencia y seguridad de Rust.
