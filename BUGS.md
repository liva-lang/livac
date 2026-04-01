# 🐛 Bugs y Carencias Detectadas

> **Fuente:** Liva Test Suite (`compiler/tests/liva/`)  
> **Última actualización:** 2026-03-31  
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
