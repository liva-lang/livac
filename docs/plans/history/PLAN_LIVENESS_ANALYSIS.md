# Plan: Liveness Analysis para Ownership Invisible

> **⚠️ Estado: HISTÓRICO (2026-04-28).** Este plan ya está mayoritariamente
> implementado. La implementación real vive en:
> - `src/liveness.rs` (bootstrap Rust)
> - `compiler/src/liveness.liva` (self-hosted)
> - Optimizaciones aplicadas en **Phase 8.1–8.10** y **Phase 9.1–9.6/9.9/9.10**
>   (ver `compiler/docs/PLAN.md`).
>
> Los checkboxes sin marcar de este documento corresponden al diseño original
> y NO reflejan trabajo pendiente. Para optimizaciones futuras, ver `compiler/docs/PLAN.md`.

> **Objetivo (original):** Hacer que Liva maneje ownership/borrowing/cloning automáticamente,
> invisible para el programador, con coste cero (o mínimo) en rendimiento.
>
> **Técnica:** Last-use analysis (inspirada en Swift ARC, Mojo, Val)
>
> **Regla básica:**
> - Último uso de una variable → **move** (coste cero)
> - Uso no-último que solo lee → **borrow** `&` (coste cero)
> - Uso no-último que necesita owned → **clone** (único caso con coste)
>
> **Resuelve:** ISSUE-003 (switch move), ISSUE-004 (param move), y la mayoría
> de workarounds del self-hosting.

---

## 1. Dónde encaja en el pipeline

```
Pipeline actual:
  Lexer → Parser → Semantic → Desugaring → Lowering → CodeGen

Pipeline con liveness:
  Lexer → Parser → Semantic → ★ Liveness → Desugaring → Lowering → CodeGen
                                   │
                                   ▼
                          Anota cada uso de variable con:
                          UseKind::Move | UseKind::Borrow | UseKind::Clone
```

### Nuevo módulo: `src/liveness.rs`

Se inserta **después del semantic analysis** (porque necesita tipos resueltos)
y **antes de desugaring/codegen** (porque éstos consumen las anotaciones).

En [src/lib.rs](../../src/lib.rs):
```rust
// Después de semantic analysis:
let analyzed_ast = semantic::analyze_with_source(...)?;

// NUEVO: Liveness analysis
let annotated_ast = liveness::analyze(&analyzed_ast)?;

// Desugaring usa el AST anotado
let desugar_ctx = desugaring::desugar(annotated_ast)?;
```

---

## 2. Estructuras de datos

### 2.1 UseKind — cómo se usa cada variable en cada punto

```rust
/// src/liveness.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UseKind {
    /// Último uso en este scope — se puede mover (coste cero)
    Move,
    /// No es el último uso, solo lectura — borrow con & (coste cero)
    Borrow,
    /// No es el último uso pero necesita owned (ej: pasarlo a un campo struct) — .clone()
    Clone,
}
```

### 2.2 UseInfo — metadata de cada punto de uso

```rust
#[derive(Debug, Clone)]
pub struct UseInfo {
    /// Nombre de la variable
    pub var_name: String,
    /// Tipo de uso determinado por el análisis
    pub kind: UseKind,
    /// Si es argumento de función, ¿el parámetro necesita owned?
    pub needs_owned: bool,
    /// Posición en el AST (para matching durante codegen)
    pub node_id: u64,
}
```

### 2.3 LivenessContext — resultado del análisis completo

```rust
pub struct LivenessContext {
    /// Map de node_id → UseKind para cada referencia a variable
    pub use_kinds: HashMap<u64, UseKind>,
    /// Variables que necesitan `mut` (porque son reasignadas)
    pub needs_mut: HashSet<String>,
    /// Funciones cuyos parámetros deben ser por referencia
    pub borrow_params: HashMap<String, Vec<bool>>,  // fn_name → [is_borrow per param]
}
```

### 2.4 Extensión del AST — añadir `node_id` a `Expr`

Actualmente `Expr::Identifier(String)` no tiene ID único. Necesitamos uno para
mapear cada uso a su `UseKind`.

**Opción A: Añadir campo `node_id` a Expr** (invasivo, cambia mucho código)

**Opción B: Usar un counter global y HashMap<(scope, index) → UseKind>** (menos invasivo)

**Opción C (recomendada): Side-table con posición span**
Dado que el parser ya genera `Span` para muchos nodos, podemos usar la
posición (línea, columna) como key única para cada `Expr::Identifier`.
```rust
pub use_kinds: HashMap<(usize, usize), UseKind>,  // (line, col) → UseKind
```
Esto evita modificar el AST.

---

## 3. Algoritmo

### 3.1 Visión general

```
Para cada función/método:
  1. Forward pass: recopilar TODOS los usos de cada variable
     (cada Expr::Identifier + cada Expr::MemberAccess sobre variable)
  2. Backward pass: desde el último uso hacia el primero,
     marcar último → Move, resto → Borrow o Clone
  3. Ajustar: si el uso está en un loop, nunca es "último" (siempre Borrow/Clone)
  4. Ajustar: si pasa a struct field/array push, necesita Clone (no basta &)
```

### 3.2 Forward pass — recopilar usos

```rust
fn collect_uses(body: &BlockStmt) -> HashMap<String, Vec<VarUse>> {
    // Recorrer todo el AST recursivamente
    // For each Expr::Identifier(name):
    //   Registrar VarUse { name, location, context }
    // For each Expr::MemberAccess con base Identifier:
    //   Registrar como uso de la variable base
    // Contadores de scope (if, for, while, switch) para tracking
}

struct VarUse {
    var_name: String,
    location: (usize, usize),  // span position
    context: UseContext,
    in_loop: bool,             // ¿está dentro de for/while?
    in_branch: bool,           // ¿está dentro de if/switch?
}

enum UseContext {
    /// Variable usada como valor (let x = y, return y)
    Value,
    /// Variable pasada como argumento a función
    Argument { fn_name: String, param_idx: usize },
    /// Variable usada en switch discriminant
    SwitchDiscriminant,
    /// Variable accedida con .field
    FieldAccess,
    /// Variable usada en method call (obj.method())
    MethodCall { method: String },
    /// Variable asignada a campo de struct/enum
    StructField,
    /// Variable pushed a un array
    ArrayPush,
}
```

### 3.3 Backward pass — determinar UseKind

```rust
fn determine_use_kinds(uses: &HashMap<String, Vec<VarUse>>) -> HashMap<(usize, usize), UseKind> {
    let mut result = HashMap::new();
    
    for (var_name, var_uses) in uses {
        // Ordenar por posición (último primero)
        let sorted: Vec<_> = var_uses.iter().rev().collect();
        
        for (i, use_info) in sorted.iter().enumerate() {
            let kind = if i == 0 && !use_info.in_loop {
                // ÚLTIMO uso y no está en loop → MOVE
                UseKind::Move
            } else if needs_owned_value(&use_info.context) {
                // No es último pero necesita owned (struct field, array push) → CLONE
                UseKind::Clone
            } else {
                // No es último, solo lectura → BORROW
                UseKind::Borrow
            };
            result.insert(use_info.location, kind);
        }
    }
    result
}

fn needs_owned_value(ctx: &UseContext) -> bool {
    matches!(ctx, 
        UseContext::StructField | 
        UseContext::ArrayPush |
        UseContext::Argument { .. }  // TODO: refinar cuando tengamos info de parámetros
    )
}
```

### 3.4 Casos especiales

#### Loops (for/while)
Dentro de un loop, **ningún uso es "último"**, porque el loop puede
ejecutarse de nuevo. Todo es Borrow o Clone.

```liva
for item in items {
    print(item)        // Borrow (lee, no es último porque loop repite)
    list.push(item)    // Clone (necesita owned para push)
}
```

#### Branches (if/else, switch)
Si una rama hace move, la otra también debe hacerlo (o la variable
deja de existir en una rama pero no en la otra). Solución:
- Si TODAS las ramas son last-use → Move en todas
- Si alguna rama NO es last-use → Borrow/Clone en todas

```liva
if condition {
    return x    // Move OK (return = último uso)
} else {
    print(x)   // ¿Es último uso? Depende del código después del if
}
```

#### Switch expression con enum
El caso más importante (ISSUE-003): `switch expr { ... }` genera `match expr { ... }`.
Con liveness analysis:
- Si `expr` no se usa después del switch → `match expr { ... }` (move, zero-cost)
- Si `expr` se usa después → `match &expr { ... }` (borrow, zero-cost)

```liva
// Caso 1: último uso → match expr (move)
let name = switch shape {
    Shape.Circle(r) => "circle"
    _ => "other"
}
// shape no se usa más → match shape { ... }

// Caso 2: sigue usándose → match &expr (borrow)
let name = switch shape {
    Shape.Circle(r) => "circle"
    _ => "other"  
}
print(shape)  // shape se usa después → match &shape { ... }
```

#### Parámetros de función
Con liveness, los parámetros se pueden analizar desde el **caller side**:
- Si el caller necesita la variable después de la llamada → pasar `&value`
- Si es el último uso del caller → pasar `value` (move)

Pero esto requiere que la **firma de la función** sea coherente. Dos opciones:

**Opción 1 (simple): Todos los params no-Copy son `&T` por defecto**
```rust
// Liva: fn greet(name: string)
// Rust:  fn greet(name: &str)  // o fn greet(name: &String)
```
El caller siempre pasa `&var`. Si la función necesita owned, hace `.to_string()` internamente.

**Opción 2 (optimal): Análisis bidireccional**
La función analiza si necesita owned (ej: almacena el param en struct field).
Si no necesita owned → genera `&T`. Si necesita → genera `T` y el caller decide
si pasar move o clone según liveness.

**Recomendación: Opción 1 para v1, Opción 2 para v2.**

---

## 4. Cambios en CodeGen

### 4.1 Variable references

Actualmente en [src/codegen.rs](../../src/codegen.rs#L7212):
```rust
Expr::Identifier(name) => {
    write!(self.output, "{}", self.sanitize_name(name)).unwrap();
}
```

Con liveness:
```rust
Expr::Identifier(name) => {
    let sanitized = self.sanitize_name(name);
    match self.get_use_kind(name, current_span) {
        UseKind::Move => write!(self.output, "{}", sanitized),
        UseKind::Borrow => write!(self.output, "&{}", sanitized),
        UseKind::Clone => write!(self.output, "{}.clone()", sanitized),
    }
}
```

### 4.2 Switch statement

Actualmente (line 6728):
```rust
self.output.push_str("match ");
self.generate_expr(&switch_stmt.discriminant)?;
```

Con liveness:
```rust
self.output.push_str("match ");
if self.should_borrow_switch_discriminant(&switch_stmt.discriminant) {
    self.output.push_str("&");
}
self.generate_expr(&switch_stmt.discriminant)?;
```

Y en los pattern arms, cuando el discriminant es `&`:
```rust
// Pattern with references
Shape::Circle(ref r) => { ... }
```

### 4.3 Function parameters

Actualmente (line 4977, `generate_params`): siempre genera `name: Type`.

Con liveness (Opción 1):
```rust
// Para tipos no-Copy (String, Vec, struct, enum con data):
write!(result, "{}: &{}", param_name, type_str)
// Para tipos Copy (i32, f64, bool, char):
write!(result, "{}: {}", param_name, type_str)
```

### 4.4 Function calls (caller side)

Actualmente genera `my_func(arg)`.

Con liveness:
```rust
// Si param es &T y arg es la última vez que se usa:
my_func(&arg)  // borrow (podríamos optimizar a move si firma lo permite)

// Si param es T (owned) y NO es última vez:
my_func(arg.clone())  // clone necesario
```

---

## 5. Qué tipos son Copy (no necesitan liveness)

| Tipo Liva | Tipo Rust | Copy? | Necesita liveness |
|-----------|-----------|-------|-------------------|
| `int` / `number` | `i32` | ✅ | No |
| `float` | `f64` | ✅ | No |
| `bool` | `bool` | ✅ | No |
| `char` | `char` | ✅ | No |
| `string` | `String` | ❌ | **Sí** |
| `[T]` (array) | `Vec<T>` | ❌ | **Sí** |
| `Map<K,V>` | `HashMap<K,V>` | ❌ | **Sí** |
| `Set<T>` | `HashSet<T>` | ❌ | **Sí** |
| `class` | `struct` | ❌ | **Sí** |
| `enum` (con data) | `enum` | ❌ | **Sí** |
| `enum` (sin data) | `enum` | ❌** | **Sí*** |

** Los enums sin data podrían derivar `Copy` automáticamente (quick win independiente).

---

## 6. Plan de implementación por fases

### Fase 0: Quick Wins (sin liveness) — ~2 sesiones

Arreglos que no requieren liveness y resuelven parte de los issues:

- [ ] **Copy para enums sin data:** En codegen, si `EnumDecl` tiene SOLO unit variants,
  generar `#[derive(Clone, Copy)]`. Resuelve parcialmente ISSUE-003 para enums simples.
  
- [ ] **Fix ISSUE-001:** `let x: T? = value` → generar `Some(value)` automáticamente.
  Cambio en codegen: detectar cuando TypeRef es Optional y el init no lo es.

- [ ] **Fix ISSUE-006:** No double-wrap Optional en enum constructors.

- [ ] **`match &expr` para switch:** Heurística simple (sin liveness completo):
  Si la variable del switch se usa después en el mismo scope → generar `match &var`.
  Requiere un scan forward simple, no liveness completo.

### Fase 1: Infraestructura — ~3 sesiones

- [ ] Crear `src/liveness.rs` con estructuras `UseKind`, `UseInfo`, `LivenessContext`
- [ ] Implementar `collect_uses()` — forward pass que recorre el AST
- [ ] Implementar `determine_use_kinds()` — backward pass
- [ ] Tests unitarios con ASTs sintéticos
- [ ] Integrar en pipeline (`lib.rs`): entre semantic y desugaring

### Fase 2: Codegen Integration — ~3 sesiones

- [ ] Pasar `LivenessContext` al `CodeGenerator`
- [ ] Modificar `generate_expr` para Identifier: emitir `&x`, `x`, `x.clone()`
- [ ] Modificar switch codegen: `match &var` cuando corresponde
- [ ] Parámetros de función: `&String` para strings, `&T` para structs/enums
- [ ] Ajustar function calls en caller para pasar `&arg`
- [ ] Tests: recompilar self-hosting sin workarounds

### Fase 3: Refinamiento — ~2 sesiones

- [ ] Loop analysis: marcar usos dentro de for/while
- [ ] Branch analysis: unificar UseKind entre ramas if/else
- [ ] Method receivers: `&self` vs `&mut self` automático
- [ ] Optimizar: cuando el tipo es pequeño (< 64 bytes), preferir Clone sobre Borrow
- [ ] Tests de regresión: los 500+ tests existentes deben seguir pasando

### Fase 4: Parámetros bidireccionales — ~2 sesiones (futuro)

- [ ] Analizar cuerpo de cada función para determinar si params necesitan owned
- [ ] Generar firma `fn foo(x: &T)` o `fn foo(x: T)` según el análisis
- [ ] Caller adapta: `&x` o `x.clone()` o `x (move)` según liveness + firma

---

## 7. Ejemplo completo: antes y después

### Código Liva

```liva
fn processUser(user: User) {
    let name = user.name
    print("Processing: " + name)
    
    switch user.role {
        Role.Admin(level) => print("Admin level " + level.toString())
        Role.User => print("Regular user")
    }
    
    saveToDb(user)
}
```

### Rust generado HOY (sin liveness)

```rust
fn process_user(user: User) {           // moved in
    let name = user.name;               // moves user.name out → user partially moved!
    println!("Processing: {}", name);
    match user.role {                    // ERROR: user partially moved
        Role::Admin(level) => println!("Admin level {}", level),
        Role::User => println!("Regular user"),
    }
    save_to_db(user);                   // ERROR: user was moved
}
```

### Rust generado CON liveness

```rust
fn process_user(user: User) {                    // param: owned (last use is save_to_db)
    let name = user.name.clone();                 // Clone: user se usa después
    println!("Processing: {}", name);             // Move: último uso de name
    match &user.role {                            // Borrow: user se usa después
        Role::Admin(level) => println!("Admin level {}", level),
        Role::User => println!("Regular user"),
    }
    save_to_db(user);                             // Move: último uso de user
}
```

### Conteo de costes
- `user.name.clone()` — 1 clone (String, ~40 bytes heap) — **inevitable**
- `match &user.role` — borrow, **coste cero**
- `save_to_db(user)` — move, **coste cero**
- Total: **1 clone** donde Python haría reference counting en todo momento

---

## 8. Métricas de éxito

1. **Self-hosting sin workarounds:** Los 4 archivos (token, ast, lexer, parser) compilan
   sin los patrones de dos pasos, sin `_isIdentKindAt`, sin `_rangeContainsFail`, etc.
2. **Tests verdes:** Los 500+ tests existentes pasan sin cambios.
3. **Rendimiento:** No measurable regression en compilación (liveness es O(n) en tamaño del AST).
4. **Issues cerrados:** ISSUE-001, ISSUE-003, ISSUE-004, ISSUE-006 resueltos.

---

## 9. Riesgos y mitigaciones

| Riesgo | Probabilidad | Mitigación |
|--------|-------------|------------|
| Cambiar AST rompe 500+ tests | Alta | Usar side-table (no modificar AST) |
| Liveness incorrecto en closures/lambdas | Media | Fase 1: tratar lambdas como "último uso desconocido" → siempre Clone |
| Firmas de función incompatibles entre módulos | Media | Fase 1: parámetros siempre `&T` (coherente) |
| `ref` patterns en match son verbosos | Baja | Codegen ya maneja patterns, añadir `ref` es directo |
| Struct parcialmente movido (user.name) | Media | Detectar field access como uso de la variable padre |

---

## 10. Trabajo relacionado

- **Swift:** Ownership por ARC + optimización de último uso (Copy-on-Write). Liva no tiene RC.
- **Mojo:** Ownership sin GC, `owned` vs `borrowed` vs `inout` explicito. Liva lo infiere.
- **Val:** Last-use por defecto, `inout` para mutación. Inspiración directa.
- **Rust (actual):** El programador gestiona manualmente. Liva automatiza esto.

---

## Apéndice A: Pseudocódigo del análisis completo

```rust
pub fn analyze(program: &Program) -> LivenessContext {
    let mut ctx = LivenessContext::new();
    
    for item in &program.items {
        match item {
            TopLevel::Function(func) => analyze_function(func, &mut ctx),
            TopLevel::Class(class) => {
                for member in &class.members {
                    if let Member::Method(method) = member {
                        analyze_method(method, &mut ctx);
                    }
                }
            }
            _ => {}
        }
    }
    
    ctx
}

fn analyze_function(func: &FunctionDecl, ctx: &mut LivenessContext) {
    let mut uses: HashMap<String, Vec<VarUse>> = HashMap::new();
    
    // 1. Collect all variable uses
    if let Some(body) = &func.body {
        collect_uses_block(body, &mut uses, false, false);
    }
    
    // 2. Determine UseKind for each use
    for (var_name, var_uses) in &uses {
        let n = var_uses.len();
        for (i, vu) in var_uses.iter().enumerate() {
            let is_last = i == n - 1;
            let kind = if is_last && !vu.in_loop {
                UseKind::Move
            } else if vu.needs_owned() {
                UseKind::Clone
            } else {
                UseKind::Borrow
            };
            ctx.use_kinds.insert(vu.location, kind);
        }
    }
    
    // 3. Determine parameter borrowing
    let borrow_flags: Vec<bool> = func.params.iter().map(|p| {
        let name = p.name().unwrap_or("_");
        // If param is used only once (and that's the last use), it can be owned
        // Otherwise, it should be borrowed
        let param_uses = uses.get(name);
        match param_uses {
            Some(uses) if uses.len() <= 1 => false,  // owned OK
            _ => !is_copy_type(&p.type_ref),          // borrow non-Copy
        }
    }).collect();
    
    ctx.borrow_params.insert(func.name.clone(), borrow_flags);
}
```
