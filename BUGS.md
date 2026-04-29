# 🐛 Bugs y Carencias Detectadas

> **Fuente:** Liva Test Suite (`compiler/tests/liva/`)  
> **Última actualización:** 2026-05-01  
> **Prioridad:** ⚡ = bloqueante, 🔶 = importante, 🔷 = menor

---

## Codegen — Bugs en la generación de Rust

### B101 — `expect(err).toBeTruthy()` falla con `Option<Error>` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de `toBeTruthy` en test framework
- **Problema:** `expect(err).toBeTruthy()` genera `assert!(err)` en Rust, pero `err` es de tipo `Option<liva_rt::Error>` que no implementa `Not`.
- **Fix aplicado:** Detecta si `actual_expr` es una variable de error binding (`error_binding_vars` / `option_value_vars`) y genera `.is_some()` / `.is_none()` en lugar de `assert!()`.
- **Tests añadidos:** error_handling.test.liva (2 tests: successful call no error, failed call has error)

### B102 — `parseInt` error binding genera `String` en vez de `Option<Error>` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — `generate_call_expr` para `parseInt`
- **Problema:** `let val, err = parseInt("abc")` generaba `err` como `String` (de `.parse()` nativo Rust), pero `divide()` generaba `err` como `Option<liva_rt::Error>`. Son tipos inconsistentes.
- **Fix aplicado:** parseInt/parseFloat ahora retornan `(T, Option<liva_rt::Error>)` con `Error::from()`. El error binding se trackea como `error_binding_vars` para `if err` → `.is_some()`.
- **Tests añadidos:** error_handling.test.liva (2 tests: valid parseInt no error, invalid parseInt has error)

### B103 — Generic classes: auto-Display impl carece de bound `Display` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de `impl Display for ClassName<T>`
- **Problema:** El impl block solo tenía `impl<T: Clone>` pero necesita `impl<T: Clone + std::fmt::Display>`.
- **Fix aplicado:** Se genera `impl_display_type_params` separado que añade `+ std::fmt::Display` a cada tipo genérico.
- **Tests añadidos:** generics.test.liva (2 tests: Container<T> con number y string)

### B104 — Generic class method `get(): T` mueve valor desde `&self` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de métodos que retornan `T`
- **Problema:** `get(): T => this.value` genera `self.value` que intenta mover `T` fuera de `&self`.
- **Fix aplicado:** Cuando el return type es un type parameter genérico de la clase y el método es `&self`, se añade `.clone()` al expr_body.
- **Tests añadidos:** generics.test.liva (Container<T>.get() works for number and string)

### B105 — `toBe([])` con array vacío: inferencia de tipo falla ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de `assert_eq!` con `vec![]`
- **Problema:** `expect(arr.filter(fn)).toBe([])` genera `assert_eq!(..., vec![])` con tipo ambiguo.
- **Fix aplicado:** Cuando expected es un array literal vacío, se genera `assert!(<actual>.is_empty())` en lugar de `assert_eq!`.
- **Tests añadidos:** lambdas.test.liva (2 tests: filter→empty, negated empty)

### B106 — `reduce` con strings: tipo acumulador mismatch `&str` vs `String` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de `.fold()` para `reduce`
- **Problema:** `["a","b"].reduce("", (acc, s) => ...)` genera `.fold("", ...)` donde el acumulador es `&str` pero el closure retorna `String`.
- **Fix aplicado:** (1) Si el valor inicial de reduce es un string literal, se genera `.to_string()`. (2) El param del elemento no usa `&` cuando `will_use_cloned=true` (non-Copy types).
- **Tests añadidos:** lambdas.test.liva (1 test: join with reduce)

### B107 — Point-free filter: `&&i32` vs `i32` deref mismatch ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de point-free refs en `.filter()`
- **Problema:** `[1,2,3].filter(isEven)` genera `filter(|_x| is_even(_x))` pero `_x` es `&i32` y `isEven` espera `i32`.
- **Fix aplicado:** Cuando `will_use_cloned=true` y el método es filter/find/some/every/count, se genera `func((*_x).clone())` para desreferenciar el argumento.
- **Tests añadidos:** lambdas.test.liva (1 test: [1..6].filter(isEven))

### B109 — Duplicate test names: sanitización colisiona ✅ FIXED
- **Ubicación:** `src/codegen.rs` — `generate_test_case`
- **Problema:** `test("accumulator with +=", ...)` y `test("accumulator with *=", ...)` ambos se sanitizan a `test_accumulator_with___`. El codegen no detectaba colisiones.
- **Fix aplicado:** Se añadió `used_test_names: HashMap<String, usize>` al `CodeGenerator`. Si un nombre ya existe, se añade sufijo `_N`.
- **Tests añadidos:** compound_assign.test.liva (2 tests con nombres que colisionan)

### B110 — Set `.has()` no funciona sobre resultado de `.union()` / `.intersection()` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — tracking de variables Set en VarDecl
- **Problema:** `set.union(b)` retorna un `HashSet`, pero la variable resultado no se trackea como `set_vars`, por lo que `.has()` no se traduce a `.contains()`.
- **Fix aplicado:** En VarDecl, cuando init es un MethodCall con método `union`/`intersection`/`difference`, se añade la variable a `set_vars`.
- **Tests añadidos:** set_methods.test.liva (3 tests: union, intersection, difference con .has())

### B111 — Optional variable: `expect(maybe).toBe(42)` con `Option<i32>` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — generación de `assert_eq!` con `Option<T>` vs `T`
- **Problema:** `let maybe: number? = null; maybe = 42; expect(maybe).toBe(42)` generaba `assert_eq!(maybe, 42)` pero `maybe` es `Option<i32>` y `42` es `i32`.
- **Fix aplicado:** En toBe/toEqual, detecta si actual_expr es `option_value_vars` o `error_binding_vars`. Si expected es null → `.is_none()`. Si no → wrappea expected en `Some()`.
- **Tests añadidos:** basics.test.liva (3 assertions: toBe(null), toBe(42), not.toBe(null))

---

## Parser — Bugs y limitaciones

### B108 — `defer <statement>` no soportado, solo `defer <expr>` ✅ FIXED
- **Ubicación:** `src/parser.rs` — parsing de `defer`
- **Problema:** `defer log += "text"` causaba `E2000: Parse Error` porque el parser solo aceptaba `defer <expression>`, no `defer <statement>` (assignments, compound assignments).
- **Fix aplicado:** `parse_defer_body()` ahora intenta parsear assignment/compound-assignment después de la expresión, igual que el statement parser principal.
- **Tests añadidos:** defer.test.liva (2 tests: compound assignment `+=`, simple assignment `=`)

### B112 — `defer` con mutation del mismo scope: borrowing conflict ⚡
- **Ubicación:** `src/codegen.rs` — generación de `_DeferGuard`
- **Problema:** `defer items.push(99)` genera una closure `_DeferGuard(Some(|| { items.push(99); }))` que captura `items` como mutable borrow. Después, cualquier uso de `items` (e.g. `items.push(4)` o `items.length()`) falla con `E0499`/`E0502` porque Rust no permite dos borrows mutables simultáneos.
- **Rust error:** `E0499: cannot borrow 'items' as mutable more than once` + `E0502: cannot borrow as immutable because also borrowed as mutable`
- **Nota:** Defer funciona en funciones top-level de programas reales donde no hay assertions posteriores. El problema aparece al intentar testear con `expect()` en el mismo scope.
- **Workaround:** Defer no testeable en unit tests que necesiten inspeccionar el estado post-defer
- **Test afectado:** `defer.test.liva`

---

## Codegen — Fixes aplicados en esta sesión ✅

Estos bugs fueron detectados Y corregidos directamente en el codegen:

### FIX-DEFAULT-PARAMS — Default parameters no se inyectaban en call sites ✅
- **Commit:** (pendiente)
- **Ubicación:** `src/codegen.rs` — `generate_function` + `generate_normal_call`
- **Problema:** `greet(name: string = "World")` generaba `fn greet(name: String)` en Rust, pero al llamar `greet()` sin args, no se inyectaba el default.
- **Fix:** Se añadió `function_defaults: HashMap<String, Vec<(usize, Expr)>>` al `CodeGenerator`. Al generar funciones, se registran los defaults. En `generate_normal_call`, se inyectan los args faltantes.

### FIX-STRING-SWITCH-OR — Switch expression con or-patterns y strings ✅
- **Commit:** (pendiente)
- **Ubicación:** `src/codegen.rs` — `generate_switch_expr`
- **Problema:** `switch day { "Saturday" | "Sunday" => ... }` generaba `match day { "Saturday" | "Sunday" => ... }` sin `.as_str()`, causando `E0308: expected String, found &str`.
- **Fix:** Se extendió `is_string_switch` para detectar `Pattern::Or(patterns)` que contengan `Literal::String`.

### FIX-ENUM-REF-CLONE — Enum data switch: bindings Copy no se clonaban ✅
- **Commit:** (pendiente)
- **Ubicación:** `src/codegen.rs` — `get_ref_clone_bindings`  
- **Problema:** `match &e { Expr::Num { value: v } => v }` — con `match &e`, `v` es `&i32` (referencia), pero el return espera `i32`. El codegen solo clonaba non-Copy types, omitiendo primitivos.
- **Fix:** Se simplificó `get_ref_clone_bindings` para clonar TODOS los bindings cuando se hace match por referencia (`.clone()` funciona tanto para Copy como non-Copy).

### B113 — `Process.exec` con `or "literal"` genera `&str` vs `String` mismatch ✅ FIXED
- **Repro:** `let output = Process.exec("cmd 2>&1 || true") or "EXEC_FAILED"`
- **Problema:** La rama `or` genera un `&str` literal pero la rama success es `String`.
- **Fix aplicado:** Se añade `.to_string()` al literal del `or` cuando es string, en la ruta `is_file` de VarDecl.
- **Tests añadidos:** process_functions.test.liva (1 test: exec with or default)

### B114 — `.as_str()` generado sobre `&str` en vez de `String` ✅ FIXED
- **Ubicación:** `src/codegen.rs` — llamadas a métodos que internamente usan `.as_str()`
- **Problema:** Ciertos métodos stdlib (Regex.replace, Date.add, Date.diff) generan `.as_str()` sobre una variable que ya es `&str`. En Rust nightly funciona, pero en stable `.as_str()` en `&str` no es estable (`feature(str_as_str)`).
- **Fix aplicado:** Se genera `let __repl = (EXPR).to_string()` / `let __liva_unit = (EXPR).to_string()` para garantizar que el valor es un `String` owned antes de llamar `.as_str()`.
- **Tests añadidos:** Regex.replace (2 tests), Date.add (1 test), Date.diff (1 test)

### B115 — `Dir.exists()` / `Dir.isDir()` con expresión inline genera borrow de temporal ✅ FIXED
- **Repro:** `Dir.exists(base + "/subdir")` — sin asignar a variable primero
- **Problema:** El codegen genera `Path::new(&format!(...))` pero `format!()` crea un temporal que se libera antes de que `Path::new` lo use.
- **Fix aplicado:** Se genera `let __arg = (EXPR).to_string(); Path::new(&__arg)` en todas las llamadas a File.exists, File.extension, Dir.isDir, Dir.exists, Dir.listRecursive/walk (5 ubicaciones).
- **Tests verificados:** dir_operations.test.liva usa `Dir.exists(base + "/sub")` inline correctamente.

---

## Bugs descubiertos por testing de apps complejas (2026-04-29) — ✅ TODOS FIXED

> Tres aplicaciones complejas (`compiler/tests/complex_apps/app[4-6]_*.liva`)
> ejercitan patrones avanzados (Map<K, Class>, mutación through index,
> `for k,v in map`, `self.field.concat`). Tras los fixes, las 3 apps
> producen stdout idéntico en bootstrap y gen-2.
>
> Regresiones cubiertas en `compiler/tests/regression/run.sh` (4 repros mínimas).

### B116 — Indexed assignment `self.field[i] = X` se pierde en gen-2 ⚡ ✅ FIXED
- **Repro:** `this.statuses[bi] = 1` dentro de `pub fn loan(&mut self, ...)`
- **Problema:** gen-2 emitía `self.statuses.clone()[(bi) as usize] = 1;` — mutación al clon que se descarta. **Mutación silenciosamente perdida**, sin error de compilación.
- **Fix:** `_emitAssign` (`compiler/src/codegen.liva`) ahora setea `_inAssignTarget = true` antes de emitir `idxObj` en el caso `Expr.Index`, suprimiendo el auto-clone de self-fields.

### B117 — `self.field = self.field.concat([x])` cannot move out of `&mut self` 🔶 ✅ FIXED
- **Repro:** `this.bookIds = this.bookIds.concat([id])` en método `&mut self`
- **Problema:** bootstrap emitía `{ let mut __v = self.book_ids; __v.extend(vec![id]); __v }` — movía `self.book_ids` fuera de `&mut self`. E0507.
- **Fix:** El emitter de `[T].concat(other)` (`src/codegen.rs`) ahora emite `let mut __v = obj.clone();` siempre. Gen-2 ya emitía bien.

### B118 — `let pts: Map<K, V> = {}` emitía `serde_json::json!({})` 🔶 ✅ FIXED
- **Repro:** `let pts: Map<string, Point> = {}` (Map vacío con tipo declarado)
- **Problema:** Bootstrap emitía `serde_json::json!({})` para todo `ObjectLiteral`, ignorando la anotación `Map<K,V>` → mismatched types `HashMap<...>` vs `Value`.
- **Fix:** En la rama `Stmt::VarDecl` (`src/codegen.rs`), si `init` es `ObjectLiteral` vacío y type_ref es `Map<_,_>`, emitir `std::collections::HashMap::new()`. Mismo trato para `Set<_>`.

### B119 — `for k, v in map` destructure falla en gen-2 🔶 ✅ FIXED
- **Repro:** `for sku, p in this.items { print(p.summary()) }`
- **Problema:** gen-2 emitía `let {sku} = {sku} as i32;` después del for, asumiendo enumerate. Pero en `map.iter()` la primera variable es `&K` (no `usize`), causando errores de tipo.
- **Fix:** En `_emitFor` (`compiler/src/codegen.liva`), si `isMapIteration`, sustituir el cast `as i32` por `.clone()` en la key (e item ya se clona).

### B120 — `arr.len()` (usize) sin cast a `i32` 🔶 ✅ FIXED
- **Repro:** `let n = arr.len(); while i < n` con `i: i32` → mismatched types.
- **Problema:** Bootstrap NO interceptaba `.len()` como método Liva: pasaba directamente a Rust devolviendo `usize`.
- **Fix:** Tratar `.len()` igual que `.length()` en `src/codegen.rs` → `(obj.len() as i32)`. Gen-2 ya emitía con cast.

### B121 — `let cur = ...; cur = ...` no declara mut 🔶 ✅ FALSE ALARM
- **Análisis post-fix:** Liva infiere `let mut` automáticamente cuando detecta reasignación. Tanto bootstrap como gen-2 emiten `let mut cur` correctamente. El error original observado era consecuencia de B118/B120 cascade.

### B122 — Mixed integer comparison emite `.as_str()` ⚡ ✅ RESOLVED via B120
- **Repro original:** `while i < n` con `i: i32` y `n` desde `let n = arr.len()`.
- **Análisis post-fix:** Con B120 arreglado, `.len()` retorna `i32`, así que `n: i32` y la comparación funciona en ambos compiladores. El reporte de `.as_str()` se debió a inferencia rota tras el mismatch — al alinear los tipos, desaparece.

### B123 — `dists.get(stringVar) or 0` 🔶 ✅ RESOLVED via B118
- **Repro:** `let d = dists.get(n) or 0` dentro de `for n in arrayOfStrings`.
- **Análisis post-fix:** El error E0308 original venía del init `let dists: Map<string, number> = {}` (B118), no del `.get()`. Tras B118 fixed, bootstrap emite correctamente `dists.get(&n).cloned().unwrap_or(0)`.

### B124 — `m.set(p.field, p)` partial-move ⚡ ✅ FIXED
- **Repro:** `class Inv { items: Map<string, Product>; add(p: Product) { this.items.set(p.sku, p) } }`.
- **Problema:** Bootstrap emite `self.items.insert(p.sku, p);` que en Rust mueve `p.sku` parcialmente y usa `p` después → E0382.
- **Fix:** En `generate_map_method_call` (set), si la key es `Expr::Member`, clonar (`p.sku.clone()`).
- **Adicional:** `let key = p.field; m.set(key, p)` también partial-move. Se extiende `expr_is_class_instance_field` para auto-clonar Member access cuando objeto es una instancia conocida (no this/self/enum/numeric tuple-index).

### B125 — Map de class fields: `obj.field.get(k) or Class()`, `for k,v in obj.field`, constructor `this.field = {}` ⚡ ✅ FIXED
- **Repro:** `Inventory.items: Map<string, Product>; constructor() { this.items = {} }; for sku, p in inv.items { ... }; let p = inv.items.get(k) or Product(...)`.
- **Problemas (3 sub-bugs):**
  1. Constructor: `this.items = {}` con Map type emitía `serde_json::json!({})` en lugar de `HashMap::new()`.
  2. `inv.items.get(k) or Product(...)` emitía doble unwrap (`.unwrap_or_default().unwrap_or(Product::new(...))`) porque `is_map_get_call` no reconocía receiver `Expr::Member`.
  3. `for k, v in obj.field` no se trataba como map iteration (caía a `.iter().enumerate()` arrojando tuplas en lugar de Product).
- **Fix bootstrap:**
  - Detectar empty Map literal (`{}`) en field assignments del constructor → `HashMap::new()`.
  - Extender `is_map_get_call` para handle `Expr::Member` receivers (heurística: prop name in `map_vars`).
  - Añadir `class_map_value_types` tracking; en for-loop dos-vars, reconocer `Expr::Member` cuyo prop esté en `map_vars` y registrar var2 como class instance del valor.
- **Fix gen-2 (codegen.liva):**
  - En `_emitClass`, registrar fields tipo `MapType` en `_mapVars` (nuevo helper `_isMapType`).
  - En `_emitFor`, switch sobre iterableExpr ahora cubre `Expr.MemberAccess`.
  - En el `or-default` switch, añadir caso `Expr.MemberAccess` para reconocer Map field receivers como `isOptionGet`.
  - En `_getExprTypeName`, extender el caso MemberAccess para resolver `var.field` cuando `var` es una instancia de clase conocida.
- **Test:** `compiler/tests/regression/b125_map_member_class.liva` y `complex_apps/app7_inventory.liva` (todas las patterns en uso simultáneo).

### B127 — `: T!` (Fallible return type) double-wraps con `Result<Result<T,Error>,Error>` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `findItem(c: string): Item! { let it = m.get(c) or fail "x"; return it }`.
- **Problema:** Bootstrap reconocía `T!` como `TypeRef::Fallible` y lo expandía a `Result<T,Error>`, pero al detectar `contains_fail=true` lo envolvía OTRA VEZ → `Result<Result<T,Error>,Error>`. Cargo rechazaba con E0308.
- **Fix bootstrap:** En el wrap auto-`Result` de funciones/métodos fallibles, si el return type ya es `TypeRef::Fallible(_)`, usar `to_rust_type()` directamente sin envolver.
- **Gen-2:** El parser de gen-2 ni siquiera acepta el sufijo `!`. Documentado como GAP-005 (gen-2 catch-up).

### B128 — `return fail "X"` dentro de función fallible genera Rust inválido ⚡ ✅ FIXED (bootstrap)
- **Repro:** `if x > limit { return fail "too big" }`.
- **Problema:** El handler `Stmt::Return` envolvía la expresión en `Ok(...)`, y `Expr::Fail` ya emite `return Err(...)`, dando `return Ok(            return Err(...));\n);` (paréntesis sueltos, return anidado, parse error en cargo).
- **Fix:** Caso especial: si `Stmt::Return` contiene `Expr::Fail`, emitir directamente `return Err(liva_rt::Error::from(msg));`.
- **Test:** `bootstrap_apps/app8_orders.liva` (línea `return fail $"over credit:{c.id}"`).

### B130 — `e.message` falla con `.get_field("message")` después de narrowing `if e != null` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let v, e = fallibleCall(); if e != null { print(e.message) }`.
- **Problema:** Tras `if let Some(e) = e {...}` (narrowing), `e` es `liva_rt::Error` (no Option), pero el codegen seguía emitiendo `e.as_ref().map(|x| x.message.as_str()).unwrap_or("None")` (válido solo para `Option<Error>`). Y al fallar, caía a la heurística JsonValue → `e.get_field("message").unwrap_or_default()` que tampoco compila.
- **Fix:** Nuevo set `narrowed_error_binding_vars`. Cuando `Stmt::If` narrowea un error binding, se inserta el nombre; el handler de `error.message` consulta el set y emite `e.message.clone()` en bloque narrowed, y el original `as_ref().map(...).unwrap_or("None")` fuera.
- **Test:** `bootstrap_apps/app8_orders.liva` (4 bloques `if eN != null { print(eN.message) }`).

### B131 — `let alice = m.get(k) or Class(...)` accede a `alice.field` con `.get_field("field")` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let alice = shop.customers.get("u1") or Customer("?","?",0); print(alice.credit)`.
- **Problema:** Tras `Map.get(k) or Class(...)`, la variable resultante no se registraba como instancia de clase. La heurística de `is_class_instance` fallaba y el property access caía a JsonValue path → `alice.get_field("credit")...` (Error E0599).
- **Fix:** Al final del handler `or_value`, si el `default_val` es un constructor de clase (Call cuyo callee es Identifier ∈ class_fields), insertar el var en `class_instance_vars` y registrar el tipo en `var_types`.
- **Test:** `bootstrap_apps/app8_orders.liva` (`let alice = shop.customers.get(...) or Customer(...)`).

### B132 — `Map.get(k) or fail "msg"` se trataba como expresión no-fallible (silently dropped fail) ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let c = shop.customers.get(id) or fail "no customer"`.
- **Problema:** El handler `or_fail_msg` no contemplaba el caso `is_map_get_call`, así que caía a "Non-fallible expression — just assign directly (or fail never triggers)" emitiendo `let c = m.get(&id).cloned().unwrap_or_default();`. Si la key no existe, devuelve `Default::default()` (Customer vacío) en lugar de propagar el error.
- **Fix:** Añadir branch `else if is_map_get_call(&var.init)` antes del fallback: emite `let c = match m.get(&k).cloned() { Some(v) => v, None => return Err(liva_rt::Error::new(msg, fn, loc)) };`.
- **Test:** `bootstrap_apps/app8_orders.liva` (`place("ghost", ...)` → `err4:no customer:ghost`).

### B133 — `let q = [start]` partial-moves `start` (E0382) cuando se usa después ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let queue = [start]; dist.set(start, 0)`.
- **Problema:** Bootstrap emitía `let queue = vec![start]; dist.insert(start.clone(), 0);` — el primer uso movía `start` (String), y `.clone()` en el segundo uso no podía compensar.
- **Fix:** En `Expr::ArrayLiteral`, si un elemento es `Expr::Identifier` con tipo no-Copy (string_vars / array_vars / map_vars / class_instance_vars), emitir `.clone()`.
- **Test:** `bootstrap_apps/app9_graph.liva` (`bfs(start, target)` con `let queue = [start]; dist.set(start, 0)`).

### B132b — `let mut` no se inferia para variables que vienen de `m.get(k) or []` y luego mutan ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let listA = adj.get(a) or []; listA.push(b)` → "cannot borrow as mutable" (E0596).
- **Problema:** Las ramas de `or_value` y `or_fail_msg` emitían `let X = ...` sin consultar `mutated_vars`; las mutaciones posteriores requerían `let mut X = ...`.
- **Fix:** Las 12 emisiones de `let {} = ` y `let {} = match ` en estas ramas ahora prefijan `mut` cuando `mutated_vars.contains(&var_name)`.
- **Test:** `bootstrap_apps/app9_graph.liva` (`addEdge` muta listas obtenidas con `or []`).

### B134 — `for k, v in mapField` con `Map<K, [T]>` o `Map<K, V>` local trataba `v` como string ⚡ ✅ FIXED (bootstrap)
- **Repro:** `groups: Map<string, [number]>; for k, vs in this.groups { for v in vs { ... } }` generaba `for v in vs.chars()`.
  También `let freq: Map<string, number> = {}; for k, v in freq { total = total + v }` generaba `format!("{}{}", total, v)` en vez de `total + v`.
- **Problema:** El handler de two-var for-loop registraba blindamente var2 en `string_vars`, sin consultar el tipo del valor del Map. Para campos de clase, `class_map_value_types` solo guardaba tipos `Simple`. Para Maps locales, no había tracking.
- **Fix:** (1) `class_map_value_types` ahora codifica también `[T]` para Array values y `{}` para Map. (2) Nuevo `local_map_value_types` poblado en let-binding con anotación `Map<K, V>`. (3) En el two-var for-loop el handler consulta ambos y registra var2 en `string_vars` / `array_vars` (con `typed_array_vars` para anidar) / `map_vars` según corresponda.
- **Test:** `bootstrap_apps/app10_stats.liva`, `app11_words.liva`.

### B135 — Switch arm Block con `if-else` final como tail-expr emitía `;` que descartaba el valor ⚡ ✅ FIXED (bootstrap)
- **Repro:** `return switch t { Tree.Node(l, r) => { let dl = depth(l); let dr = depth(r); if dl > dr { dl + 1 } else { dr + 1 } } }` → E0308 "expected i32, found ()".
- **Problema:** El handler de `SwitchBody::Block` solo trataba `Stmt::Return` como tail-expression. Cualquier otro statement final (incluido `Stmt::If` o `Stmt::Expr`) se emitía con `;` final que dropea el valor.
- **Fix:** Nuevo helper `generate_stmt_as_tail_expr` que emite (a) `Stmt::Expr(e)` como `e` sin `;`, (b) `Stmt::If` con else como `if c { ... } else { ... }` recursivamente con tail-expr en cada rama, (c) `Stmt::Return` como antes. Aplicado en ambas ramas SwitchBody::Block (con y sin pattern bindings).
- **Test:** `bootstrap_apps/app12_tree.liva`, `app13_calc.liva`.

### B136 — `Set.size()` no estaba implementado ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let s: Set<number> = {}; s.size()` → "no method named `size` found for struct `HashSet`".
- **Problema:** `size` no estaba en la lista de métodos reconocidos para Sets, y no tenía codegen.
- **Fix:** Añadido a `is_set_method` matcher y handler que emite `(set.len() as i32)`.
- **Test:** `bootstrap_apps/app14_setops.liva`.

### B137 — `instance.count("literal")` no `.to_string()`-ea el argumento ⚡ ✅ FIXED (bootstrap)
- **Repro:** método de usuario `count(category: string): number` invocado como `lib.count("fiction")` emitía `lib.count("fiction")` (sin `.to_string()`), `expected String, found &str`.
- **Problema:** El parche B10 (que evita la rama de iteradores cuando el objeto es una instancia de clase) no propagaba la conversión `&str → String` para argumentos string-literal/var.
- **Fix:** En el handler de B10 ahora se aplica `.to_string()` para `Expr::Literal::String` y `.clone()` para `string_vars` / `class_instance_vars`.
- **Test:** `bootstrap_apps/app15_library.liva`.

### B138 — `fail "msg"` en posición de expresión emitía `;\n` rompiendo switch arms ⚡ ✅ FIXED (bootstrap)
- **Repro:** `return switch s { State.Idle => switch a { Action.Start => State.Running; _ => fail "..." } }` → "expected `,`, `.`, `?`, `}`, or an operator, found `;`".
- **Problema:** `Expr::Fail` siempre emitía `\twrite_indent return Err(...);\n`. En contexto stmt eso se duplicaba con el `;` de `Stmt::Expr`; en contexto switch-arm rompía la sintaxis.
- **Fix:** `Expr::Fail` ahora emite sólo `return Err(liva_rt::Error::from(...))` (sin indent ni `;`). El stmt context añade `;` por separado.
- **Test:** `bootstrap_apps/app16_fsm.liva`.

### B139 — switch arms en función `T!` no auto-envuelven valores no-fail en `Ok(...)` 🔶 OPEN
- **Repro:** `transition(s, a): State! { return switch s { State.Idle => switch a { Action.Start => State.Running, _ => fail "..." } ... } }` → arm `State.Running` queda como `State::Running` pero la función espera `Result<State, Error>`. Sólo el ternary-with-fail (línea 7920) wrappea automáticamente.
- **Workaround:** reemplazar el switch externo por if-else encadenado con `return State.Running` explícito (ver `bootstrap_apps/app16_fsm.liva`).
- **Pendiente:** detectar en `generate_switch_expr` cuando alguna arm contiene `fail` y wrappear las otras arms en `Ok(...)`. Misma idea que el ternary B127.

### B140 — `or <default>` propagaba fallibilidad al caller incorrectamente ⚡ ✅ FIXED (bootstrap)
- **Repro:** `safe(x): number { let n = fail_fn(x) or 0; return n + 1 }`; el caller recibía `E0701: 'safe' can fail` aunque el `or 0` ya consume el error localmente.
- **Problema:** `semantic.rs::stmt_contains_fail` para `Stmt::VarDecl` devolvía `true` siempre que el init contenía una llamada fallible, sin importar si era `or fail` (que sí propaga) o `or <default>` / error binding (que no propagan).
- **Fix:** en `Stmt::VarDecl`: si `or_fail_msg.is_some()` → propaga; si `is_fallible` (error binding o `or <default>`) → NO propaga; resto → recursar en init.
- **Test:** `bootstrap_apps/app17_pipeline.liva` (función `sum_parsed` y `count_bad`).

### B141 — point-free fn ref en `.reduce(init, fn)` rompe firma de `fold` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `arr.reduce(0, max_of)` con `max_of(a, b): number` → emite `arr.iter().fold(0, max_of)` pero `fold` espera `FnMut(B, &T) -> B`.
- **Fix:** en `generate_method_call` de `reduce`, cuando arg[1] es `Expr::Identifier`, envolver en closure `|acc, x| fn(acc, x.clone())`.
- **Test:** `bootstrap_apps/app17_pipeline.liva`.

### B142 — for nested sobre `[[T]]` no detectaba el array interno ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let groups: [[number]] = [...]; for g in groups { for x in g { ... } }` → inner for trataba `g` como string y emitía `g.chars()` con `flat_total = format!(...)` (concatenación de strings) en lugar de suma.
- **Problema:** `typed_array_vars` solo registraba `TypeRef::Simple` como element type. Para `Array(Array(T))` perdía la dimensión interna y caía en el default-string-iterable.
- **Fix:** en VarDecl con `TypeRef::Array(TypeRef::Array(T))`, registrar element type como `"[T]"`. En el for-loop sobre arrays, si el element type es `"[X]"` → `g` se registra en `array_vars` y `typed_array_vars` con elem `X`, habilitando el inner for.
- **Test:** `bootstrap_apps/app17_pipeline.liva`.

### B143 — `s.toInt() or fail "msg"` no era fallible ⚡ ✅ FIXED (bootstrap)
- **Repro:** `parse_int(s): number { let n = s.toInt() or fail $"bad: {s}"; return n }` — `or fail` no propagaba el error de parse: `toInt()` emite `parse::<i32>().unwrap_or(0)`.
- **Fix:** en `or fail` para `MethodCall::toInt|toFloat`, emitir `match s.parse::<T>() { Ok(v) => v, Err(e) => return Err(liva_rt::Error::chain(...)) }`.
- **Test:** `bootstrap_apps/app17_pipeline.liva`.

### B144 — Parámetros `Map<K,V>` y `Set<T>` no se registraban en map_vars/set_vars ⚡ ✅ FIXED (bootstrap)
- **Repro:** `render(tpl: string, vars: Map<string, string>) { let v = vars.get(key) or "" }` → emite `vars.get(key.clone())` (sin `&`, sin `.cloned()`), E0308.
- **Problema:** Los parámetros con `TypeRef::Array` se trackeaban en `array_vars`/`typed_array_vars`, pero los de `TypeRef::Map` y `TypeRef::Set` se ignoraban, así que el dispatch a `generate_map_method_call` / `generate_set_method_call` no se activaba.
- **Fix:** En la rama de tracking de params, añadir `TypeRef::Map(_, V)` → `map_vars` + `local_map_value_types` con encoding (Simple / `[T]` / `{}`); `TypeRef::Set(_)` → `set_vars`.
- **Test:** `bootstrap_apps/app18_template.liva`.

### B145 — `string.indexOf(needle, fromIndex)` ignoraba el segundo argumento ⚡ ✅ FIXED (bootstrap)
- **Repro:** `tpl.indexOf("{{", i)` en un loop while emite `tpl.find("{{")` ignorando `i` → loop infinito o panic en `substring`.
- **Problema:** El handler de `indexOf` solo emitía el caso de un argumento.
- **Fix:** Cuando `args.len() >= 2`, emitir un block que clona el receiver, computa `__from = i as usize`, y hace `__s[__from..].find(&needle).map(|i| (i + __from) as i32).unwrap_or(-1)` (con guard si `__from >= __s.len()`).
- **Test:** `bootstrap_apps/app18_template.liva`.

### B146 — `pq.pop()` en clase de usuario recibía `.expect("pop from empty array")` ⚡ ✅ FIXED (bootstrap)
- **Repro:** Una clase `PriorityQueue` con método `pop(): number` usado como `pq.pop()` emitía `pq.pop().expect("pop from empty array")` → E0599 (`i32` no tiene `expect`).
- **Problema:** El post-transform `(ArrayAdapter::Seq, "pop") => ".expect(...)"` se aplicaba sin comprobar si el receiver era una instancia de clase de usuario.
- **Fix:** Antes de añadir `.expect("pop from empty array")`, comprobar que el receiver no esté en `class_instance_vars`.
- **Test:** `bootstrap_apps/app19_pq.liva`.

### B147 — `arr.reverse()` sobre `[number]` emitía `.chars().rev().collect::<String>()` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `let asc = desc.reverse()` con `desc: [number]` (variable de loop sobre array de arrays) emitía `desc.chars().rev().collect::<String>()` → E0599 (`Vec<i32>` no tiene `chars`).
- **Problema:** `is_string_method` matcheaba `reverse` siempre que el adapter fuera `Seq`, sin discriminar por tipo del receiver.
- **Fix:** Detectar `reverse` sobre receiver registrado en `array_vars` (y no en `string_vars`) y emitir `{ let mut __v = receiver.clone(); __v.reverse(); __v }`.
- **Test:** `bootstrap_apps/app19_pq.liva`.

### B148 — `this.X` no se rebinds en cuerpo no-asignación del constructor 🔶 OPEN
- **Repro:**
  ```liva
  HashMap {
      cap: number
      keys: [string]
      constructor() {
          this.cap = 8
          for i in 0..this.cap {     // ← `this` queda raw en Rust
              this.keys.push("")
          }
      }
  }
  ```
- **Problema:** El constructor de Liva tiene un esquema en dos fases: (1) emitir stmts no-`this.field=`, (2) emitir `Self { ... }`. Las stmts de fase (1) que leen `this` salen al output como literal `this`, pero `Self` aún no existe.
- **Fix propuesto:** Detectar stmts que tocan `this`, deferir su emisión a después del `Self {...}` con un alias mutable (`let mut __obj = Self {...}; <deferred stmts: this→__obj>; __obj`). Requiere helper `expr_uses_this` y un campo `this_alias: Option<String>` para sustituir el identificador.
- **Workaround:** Escribir el constructor con locales (`let cap = 8; let ks: [string] = []; ...`) y al final `this.cap = cap; this.keys = ks`.
- **Test:** `bootstrap_apps/app21_hashmap.liva` (workaround aplicado).

### B149 — Vars locales del constructor mutadas no se emiten con `mut` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `constructor() { let ks: [string] = []; for i in 0..n { ks.push("") } ... }` → E0596: `cannot borrow ks as mutable`.
- **Problema:** El handler de constructor entraba al loop de stmts sin ejecutar el pre-pass `collect_mutated_vars_in_block` que sí corre en métodos normales (`mutated_vars` quedaba vacío).
- **Fix:** Pre-poblar `self.mutated_vars` con las mutaciones del cuerpo antes de emitir las stmts del constructor.
- **Test:** `bootstrap_apps/app21_hashmap.liva`.

### B150 — Método de usuario `obj.method("literal")` no convertía string lit a `String` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `m.get("apple")` con `HashMap.get(key: string)` emitía `m.get("apple")` (`&str`) → E0308 (esperaba `String`).
- **Problema:** B137 solo cubría `count`. Para el resto de métodos de usuario, los args literales-string no se promocionaban.
- **Fix:** En el loop genérico de args de `generate_method_call_expr`, cuando el receiver está en `class_instance_vars` y el arg es `Literal::String`, append `.to_string()`.
- **Test:** `bootstrap_apps/app21_hashmap.liva`.

### B151 — String interpolation con `\"` escapado dentro de `${...}` no se parsea ⚡ OPEN
- **Repro:** `print($"a:{m.get(\"apple\")}")` emite literal `a:{m.get(\"apple\")}` (sin sustituir la expresión).
- **Problema:** El lexer del string template trata `\"` como cierre o no lo unescapa correctamente dentro del `${...}` placeholder.
- **Workaround:** Computar el valor en una variable local antes y usarla simple en el placeholder: `let v = m.get("apple"); print($"a:{v}")`.
- **Test:** `bootstrap_apps/app22_glob.liva` (workaround aplicado).

### B152 — `impl Display for Class<T>` con campo `[T]` faltaba bound `Debug` ⚡ ✅ FIXED (bootstrap)
- **Repro:** `Stack<T> { items: [T] }` generaba `write!(f, "Stack {{ items: {:?} }}", self.items)` con `impl<T: Clone + Display>` — falta `Debug` para `{:?}` → E0277.
- **Fix:** Pre-scan los campos antes de emitir el `impl Display`. Si alguno usa `{:?}` (Array / Map / Set / Optional / enum), añadir `std::fmt::Debug` a las bounds de cada type param.
- **Test:** `bootstrap_apps/app23_stack.liva`.

### B153 — Free generic functions sin bounds `Clone` rompían en patrones comunes ⚡ ✅ FIXED (bootstrap)
- **Repro:** `firstOf<T>(items: [T], fallback: T): T { return items[0] }` emitía `items[0].clone()` pero el type param `T` no tenía bound `Clone` → E0599.
- **Fix:** Auto-añadir `Clone + std::fmt::Display` a cada type param de funciones libres genéricas (mismo trato que ya recibían las clases vía B103). Si el usuario define constraints explícitas, se anexan.
- **Tests:** `bootstrap_apps/app23_stack.liva`. Snapshot `feature_generic_function` actualizada para reflejar las nuevas bounds.


## Carencias del lenguaje detectadas

### GAP-001 — No hay `toBeNull()` funcional en test framework ✅ RESOLVED
- `expect(maybe).toBeNull()` ya genera `assert!(maybe.is_none())` correctamente.
- Con B111 fixed, `expect(maybe).toBe(null)` también funciona.

### GAP-002 — `or fail` en test functions no testeable 🔷
- `propagate(a, b): number { let r = divide(a, b) or fail "msg"; return r }` — la función es fallible, pero testing de errores requiere error binding.
- Con B101 y B102 fixed, el error binding ahora funciona. `toThrow()` existe para panic-based testing.
- Pendiente: verificar que `or fail` + error binding funciona end-to-end.

### GAP-003 — `Set.union()` / `Set.intersection()` devuelve HashSet crudo 🔶
- El resultado pierde los wrappers de Liva (`.has()`, `.size()`, etc.).
- Debería devolver un Set de Liva con todos los métodos disponibles.

### GAP-004 — `print(a, b, ...)` con varios argumentos diverge entre bootstrap y gen-2 🔶
- **Repro:** `print("count:", 3)` emite `count:3` en bootstrap pero `count: 3` en gen-2.
- **Análisis:** Bootstrap concatena argumentos sin separador, gen-2 inserta espacio (estilo Python).
- **Workaround actual:** usar string interpolation `print($"count:{x}")` en código portable.
- **Decisión pendiente:** unificar al estilo Python (separador espacio) para consistencia. Afecta `complex_apps` snapshots si se cambia.

### GAP-005 — Gen-2 lag en patrones avanzados de error handling y Map<K,V> (V no-trivial) 🔶
- **Síntomas observados al ejecutar `bootstrap_apps/app8_orders.liva` y `app9_graph.liva` con gen-2:**
  - Parser de gen-2 rechaza el sufijo `T!` ("Expected identifier").
  - `let q = [start]` no clona Identifiers no-Copy en array literals (E0382).
  - `let listA = m.get(k) or []; listA.push(...)` no infiere `mut`.
  - `Map.get(k) or fail "msg"` cae a `unwrap_or_default()` (igual que bootstrap antes de B132).
  - Error binding `e.message` post-narrowing emite `.get_field("message")` (igual que bootstrap antes de B130).
- **Plan:** mirror las correcciones B127–B133 en `compiler/src/codegen.liva` cuando se decida priorizar paridad. Mientras tanto los apps que disparan estos casos viven en `compiler/tests/bootstrap_apps/` (no se ejecutan contra gen-2).

### GAP-006 — String interpolation no admite escapes `\"` dentro de `{...}` 🔷
- **Repro:** `print($"k:{m.get(\"key\")}")` produce literalmente `k:{m.get(\"key\")}` (no expande la interpolación).
- **Workaround:** asignar el resultado a una variable antes: `let v = m.get("key"); print($"k:{v}")`.
- **Pendiente:** lexer/parser de string interpolation debería entender escapes dentro del bloque `{...}` o permitir comillas dobles sin escape.

### GAP-007 — Sin sintaxis para tipos función ni inferencia de closures-as-return 🔷
- **Repro 1 (anotación):** `makeAdder(x: number): (number) => number { ... }` ⇒ E2000 "Expected expression" en el parser. La gramática de `parse_type` no contempla `(T) => U` ni `fn(T) -> U`.
- **Repro 2 (inferencia):** `makeCounter() { return () => { ... } }` (sin tipo de retorno explícito) compila, pero el codegen infiere `-> f64` para el cuerpo y luego falla al llamar `c1()` con E0618 "expected function, found f64".
- **Impacto:** imposible escribir
  - funciones que devuelven closures (currying, factories de counters/loggers).
  - parámetros tipados como callbacks (`f: (number) => number`).
  - colecciones tipadas de lambdas (`[(T) => U]`).
- **Workarounds parciales:**
  - Pasar lambdas como argumentos (sí funciona vía el slot inferido en `forEach`/`map`/etc.).
  - Para "fábricas" de funciones, sustituir por una clase con un método (la clase actúa como closure).
- **Pendiente:** añadir `TypeRef::Fn(args, ret)` al AST + parser para `(T1, T2) => U`, y propagar `impl Fn` / `Box<dyn Fn>` en codegen. Necesita decidir captura por valor vs ref y `Fn` vs `FnMut`.


