# 📘 Liva v0.6 → Rust — Desugaring Rules & Canonical Examples

> **Purpose:** deterministic, machine-readable mapping for an AI code generator.  
> **Scope:** every construct shows **(A) Input** Liva → **(B) Canonical Rust Output**.  
> **Style:** minimal prose, stable rules, no ambiguity.

---

## 0. Global Conventions

- Rust toolchain: **Tokio runtime** (multi-thread by default) for async.
- Error handling in examples: `unwrap()` for brevity unless noted; in production prefer `?` and `Result`.
- Identifiers: Liva `CamelCase` types ⇒ Rust `snake_case` functions, `CamelCase` structs.
- Visibility:
  - public: no prefix → `pub`.
  - protected: `_name` → `pub(super)` and subtypes live in `mod <type>_mod::subtypes`.
  - private: `__name` → private item (no `pub`).
- Numeric aliases: `number → i32`, `float → f64`. All native Rust primitives accepted.
- Logical operators: `and/or/not` ≡ `&&/||/!` (same precedence, short-circuit).
- Lazy-await/join: first usage of a spawned value injects `.await`/`.join()`.
- Fire & forget: `fire async|parallel f()` spawns w/o handle and no warning.
- Concurrency helpers: IR-based codegen injects a `liva_rt` module that wraps `tokio::spawn` / `std::thread::spawn`; future work will move this into a dedicated crate.

---

## 1. Variables and Types

### 1.1 Primitives & Aliases
**(A) Liva**
```ls
let a: number = 10
let b: float = 3.5
let c: u64 = 0
let ok: bool = true
let s: string = "hi"
let bs: bytes = [1,2,3]
```
**(B) Rust**
```rust
let mut a: i32 = 10;
let mut b: f64 = 3.5;
let mut c: u64 = 0;
let mut ok: bool = true;
let mut s: String = "hi".into();
let mut bs: Vec<u8> = vec![1u8, 2u8, 3u8];
```

### 1.2 Constants
**(A)**
```ls
const PI = 3.1416
```
**(B)**
```rust
const PI: f64 = 3.1416;
```

---

## 2. Functions

### 2.1 Block Function
**(A)**
```ls
sum(a: number, b: number): number {
  return a + b
}
```
**(B)**
```rust
fn sum(a: i32, b: i32) -> i32 { a + b }
```

### 2.2 One-liner (implicit return)
**(A)**
```ls
inc(n: number): number => n + 1
isAdult(age) => age >= 18
```
**(B)**
```rust
fn inc(n: i32) -> i32 { n + 1 }
fn is_adult(age: i32) -> bool { age >= 18 }
```

### 2.3 Generics
**(A)**
```ls
max<T>(a: T, b: T): T => (a > b) ? a : b
```
**(B)**
```rust
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }
```

### 2.4 Auto-async definition
**(A)**
```ls
fetchUser() {
  let res = async http.get("url")
  return res.json()
}
```
**(B)**
```rust
async fn fetch_user() -> serde_json::Value {
    let res = http::get("url").await.unwrap();
    res.json::<serde_json::Value>().await.unwrap()
}
```

---

## 3. Classes / Structs (No `class`, No `fun`)

### 3.1 Public / Protected `_` / Private `__`
**(A)**
```ls
Persona {
  nombre: string
  _edad: number
  __dni: string

  saludar() {
    print($"Hola, soy {this.nombre}")
  }

  _info() { print(this._edad) }
  __recalc() { /* ... */ }
}
```
**(B)**
```rust
pub struct Persona {
    pub nombre: String,
    pub(super) edad: i32,
    dni: String,
}

impl Persona {
    pub fn saludar(&self) {
        println!("Hola, soy {}", self.nombre);
    }
    pub(super) fn info(&self) { println!("{}", self.edad); }
    fn recalc(&self) { /* ... */ }
}
```

### 3.2 Subclass access to protected
**(A)**
```ls
Empleado : Persona {
  mostrar() { print(this._edad) }
}
```
**(B)**
```rust
mod persona_mod {
    pub struct Persona { pub nombre: String, pub(super) edad: i32, dni: String }
    impl Persona { /* ... */ }
    pub mod subtypes {
        use super::*;
        pub struct Empleado { pub base: Persona }
        impl Empleado {
            pub fn mostrar(&self) { println!("{}", self.base.edad); }
        }
    }
}
pub use persona_mod::Persona;
pub use persona_mod::subtypes::Empleado;
```

---

## 4. Control Flow & Operators

### 4.1 If / Else with `and/or/not` and `&&/||/!`
**(A)**
```ls
if not isEmpty(name) and len(name) > 2 {
  print("Valid")
} else if x > 18 || x < 65 {
  print("Working age")
}
```
**(B)**
```rust
if !is_empty(&name) && len(&name) > 2 {
    println!("Valid");
} else if x > 18 || x < 65 {
    println!("Working age");
}
```

### 4.2 Loops
**(A)**
```ls
for i in 0..10 { print(i) }
while cond { break }
```
**(B)**
```rust
for i in 0..10 { println!("{}", i); }
while cond { break; }
```

### 4.3 Switch
**(A)**
```ls
switch color {
  case "red": print("Rojo")
  case "blue": print("Azul")
  default: print("Otro")
}
```
**(B)**
```rust
match color.as_str() {
    "red" => println!("Rojo"),
    "blue" => println!("Azul"),
    _ => println!("Otro"),
}
```

---

## 5. Strings & Templates

**(A)**
```ls
let saludo = $"Hola {nombre}, tienes {edad} años"
```
**(B)**
```rust
let saludo = format!("Hola {}, tienes {} años", nombre, edad);
```

---

## 6. Concurrency (Call Site Semantics)

### 6.1 async — cooperative (Tokio), lazy await
**(A)**
```ls
let u = async fetchUser()
otherWork()
print(u.name)
```
**(B)**
```rust
let u = tokio::spawn(fetch_user());
other_work();
let u = u.await.unwrap();
println!("{}", u["name"]);
```

### 6.2 parallel — OS thread, lazy join
**(A)**
```ls
let a = parallel heavyCalc(1)
let b = parallel heavyCalc(2)
print(a + b)
```
**(B)**
```rust
let a = std::thread::spawn(|| heavy_calc(1));
let b = std::thread::spawn(|| heavy_calc(2));
let a = a.join().unwrap();
let b = b.join().unwrap();
println!("{}", a + b);
```

### 6.3 task async/parallel — explicit await
**(A)**
```ls
let t1 = task async fetchUser()
let t2 = task parallel processData()
let u = await t1
let d = await t2
```
**(B)**
```rust
let t1 = tokio::spawn(fetch_user());
let t2 = std::thread::spawn(|| process_data());
let u = t1.await.unwrap();
let d = t2.join().unwrap();
```

### 6.4 fire async/parallel — no handle, no warning
**(A)**
```ls
fire async sendMetrics()
fire parallel rebuildCache()
```
**(B)**
```rust
tokio::spawn(send_metrics());
std::thread::spawn(|| rebuild_cache());
```

### 6.5 Mixed example (auto-async inside def + async call)
**(A)**
```ls
fetchOne() {
  let res = async http.get("url")
  return res.json()
}

main() {
  let a = fetchOne()          // implicit await
  let b = async fetchOne()    // spawn + lazy await at use
  print(a.id)
  print(b.id)
}
```
**(B)**
```rust
async fn fetch_one() -> serde_json::Value {
    let res = http::get("url").await.unwrap();
    res.json::<serde_json::Value>().await.unwrap()
}

#[tokio::main]
async fn main() {
    let a = fetch_one().await.unwrap();
    let b = tokio::spawn(fetch_one());
    let b = b.await.unwrap();
    println!("{}", a["id"]);
    println!("{}", b["id"]);
}
```

---

## 7. Errors (Result) & Exceptions (throw/try-catch)

**(A)**
```ls
divide(a, b) {
  if b == 0 throw "División por cero"
  return a / b
}

try {
  let r = divide(4, 0)
} catch (e) {
  print("Error: " + e)
}
```
**(B)**
```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 { return Err("División por cero".into()); }
    Ok(a / b)
}

fn main_logic() {
    match divide(4, 0) {
        Ok(r) => println!("{}", r),
        Err(e) => println!("Error: {}", e),
    }
}
```

---

## 8. Interop with Rust

### 8.1 use rust "crate"
**(A)**
```ls
use rust "serde_json"
use rust "reqwest" as http
```
**(B) Cargo.toml**
```toml
[dependencies]
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
```
**(B) Rust**
```rust
use serde_json;
use reqwest as http;
```

### 8.2 External function
**(A)**
```ls
extern rust {
  fun blake3_hash(data: bytes): string
}
let h = blake3_hash("hola")
```
**(B)**
```rust
extern "Rust" {
    fn blake3_hash(data: &[u8]) -> String;
}
let h = unsafe { blake3_hash(b"hola") };
```

---

## 9. Deterministic Rewrite Rules (Summary)

1. **Types**: `number→i32`, `float→f64`. Others map 1:1 to Rust primitives.
2. **Strings**: `$"a {x}"` → `format!("a {}", x)`.
3. **Visibility**: `__x` private; `_x` protected→`pub(super)`; else `pub`.
4. **Classes**: `Type { fields+methods }` → `pub struct Type {..} + impl Type {..}`.
5. **One-liner def**: `f(p):T => e` → `fn f(p) -> T { e }`.
6. **Auto-async def**: body contains async → `async fn`.
7. **Calls**:
   - `async f()` → `let h = tokio::spawn(f());` + lazy `.await` at first use.
   - `parallel f()` → `let h = std::thread::spawn(|| f());` + lazy `.join()`.
   - `task async|parallel f()` → returns handle; await/join only when requested.
   - `fire async|parallel f()` → spawn, drop handle (no warnings).
8. **Logic ops**: normalize to `&&/||/!` with short-circuit.
9. **Warnings**: unused result of async/parallel (non-void) unless `fire` or assigned to `_`.

---

## 10. Minimal Smoke Suite (Liva → Rust)

- ✅ Variables/types (1.1)  
- ✅ Functions (2.1–2.4)  
- ✅ Classes/visibility (3.1–3.2)  
- ✅ Control/ops (4.1–4.3)  
- ✅ Templates (5)  
- ✅ Concurrency (6.1–6.5)  
- ✅ Errors (7)  
- ✅ Interop (8.1–8.2)
