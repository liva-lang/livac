# 🐛 Bugs y Carencias Detectadas

> **Fuente:** Liva Test Suite (`compiler/tests/liva/`)  
> **Última actualización:** 2026-03-31  
> **Prioridad:** ⚡ = bloqueante, 🔶 = importante, 🔷 = menor

---

## Codegen — Bugs en la generación de Rust

### B101 — `expect(err).toBeTruthy()` falla con `Option<Error>` ⚡
- **Ubicación:** `src/codegen.rs` — generación de `toBeTruthy` en test framework
- **Problema:** `expect(err).toBeTruthy()` genera `assert!(err)` en Rust, pero `err` es de tipo `Option<liva_rt::Error>` que no implementa `Not`. Debería generar `assert!(err.is_some())`.
- **Afecta:** `toBeTruthy()` y `toBeFalsy()` sobre variables de error binding (`let val, err = fn()`)
- **Rust error:** `E0600: cannot apply unary operator '!' to type 'Option<liva_rt::Error>'`
- **Workaround:** Usar `or <default>` en lugar de error binding + expect
- **Test afectado:** `error_handling.test.liva`

### B102 — `parseInt` error binding genera `String` en vez de `Option<Error>` 🔶
- **Ubicación:** `src/codegen.rs` — `generate_call_expr` para `parseInt`
- **Problema:** `let val, err = parseInt("abc")` genera `err` como `String` (de `.parse()` nativo Rust), pero `divide()` genera `err` como `Option<liva_rt::Error>`. Son tipos inconsistentes.
- **Consecuencia:** `if err` genera `.is_some()` sobre `String` → error `E0599`
- **Workaround:** Usar `parseInt("abc") or 0`
- **Test afectado:** `error_handling.test.liva`

### B103 — Generic classes: auto-Display impl carece de bound `Display` ⚡
- **Ubicación:** `src/codegen.rs` — generación de `impl Display for Box<T>`
- **Problema:** Cuando se genera el `impl Display` automático para una clase genérica `Box<T>`, el impl block solo tiene `impl<T: Clone>` pero necesita `impl<T: Clone + std::fmt::Display>`.
- **Rust error:** `E0277: T doesn't implement std::fmt::Display`
- **Workaround:** No usar clases genéricas en tests (funciones genéricas SÍ funcionan)
- **Test afectado:** `generics.test.liva`

### B104 — Generic class method `get(): T` mueve valor desde `&self` ⚡
- **Ubicación:** `src/codegen.rs` — generación de métodos que retornan `T`
- **Problema:** `get(): T => this.value` genera `self.value` que intenta mover `T` fuera de `&self`. Debería generar `self.value.clone()`.
- **Rust error:** `E0507: cannot move out of 'self.value' which is behind a shared reference`
- **Workaround:** No usar métodos que retornan `T` genérico
- **Test afectado:** `generics.test.liva`

### B105 — `toBe([])` con array vacío: inferencia de tipo falla 🔶
- **Ubicación:** `src/codegen.rs` — generación de `assert_eq!` con `vec![]`
- **Problema:** `expect([1,3,5].filter(x => x > 10)).toBe([])` genera `assert_eq!(..., vec![])` en Rust. El `vec![]` vacío no puede inferir tipo, y además hay ambigüedad de `PartialEq` con `serde_json::Value`.
- **Rust error:** `E0282: type annotations needed` + `E0283: ambiguous PartialEq`
- **Workaround:** Comparar `expect(arr.length()).toBe(0)` o usar `.filter(x => x > 20).toBe([30, 40])`
- **Test afectado:** `lambdas.test.liva`

### B106 — `reduce` con strings: tipo acumulador mismatch `&str` vs `String` 🔶
- **Ubicación:** `src/codegen.rs` — generación de `.fold()` para `reduce`
- **Problema:** `["a","b","c"].reduce("", (acc, s) => ...)` genera `.fold("", ...)` donde el acumulador inicial es `&str` pero el closure retorna `String` desde `format!()`.
- **Rust error:** `E0308: mismatched types — expected &str, found String`
- **Workaround:** Usar reduce solo con números
- **Test afectado:** `lambdas.test.liva`

### B107 — Point-free filter: `&&i32` vs `i32` deref mismatch 🔶
- **Ubicación:** `src/codegen.rs` — generación de point-free refs en `.filter()`
- **Problema:** `[1,2,3].filter(isEven)` genera `filter(|_x| is_even(_x))` pero `_x` es `&&i32` (desde `.iter()` + closure ref), y `isEven` espera `i32`.
- **Nota:** Point-free en `.map()` funciona correctamente. Solo falla en `.filter()`.
- **Rust error:** `E0308: mismatched types — expected i32, found &&{integer}`  
- **Workaround:** Usar lambda inline: `.filter(x => x % 2 == 0)`
- **Test afectado:** `lambdas.test.liva`

### B109 — Duplicate test names: sanitización colisiona ⚡
- **Ubicación:** `src/codegen.rs` — `generate_test_case`
- **Problema:** `test("accumulator with +=", ...)` y `test("accumulator with *=", ...)` ambos se sanitizan a `test_accumulator_with___`. El codegen no detecta ni resuelve colisiones.
- **Rust error:** `E0428: the name 'test_accumulator_with___' is defined multiple times`
- **Workaround:** Usar nombres de test que sean únicos después de sanitización
- **Test afectado:** `compound_assign.test.liva`

### B110 — Set `.has()` no funciona sobre resultado de `.union()` / `.intersection()` 🔶
- **Ubicación:** `src/codegen.rs` — generación de métodos sobre `HashSet` derivado
- **Problema:** Cuando `.union(b)` o `.intersection(b)` retornan un nuevo `HashSet`, las llamadas subsecuentes a `.has()` generan `.has()` en Rust (no existe), en vez de `.contains()`.
- **Nota:** `.has()` funciona correctamente sobre Sets creados directamente con `Set { ... }`.
- **Rust error:** `E0599: no method named 'has' found for struct 'HashSet'`
- **Workaround:** No encadenar `.has()` sobre resultados de union/intersection
- **Test afectado:** `collections.test.liva`

### B111 — Optional variable: `expect(maybe).toBe(42)` con `Option<i32>` 🔷
- **Ubicación:** `src/codegen.rs` — generación de `assert_eq!` con `Option<T>` vs `T`
- **Problema:** `let maybe: number? = null; maybe = 42; expect(maybe).toBe(42)` genera `assert_eq!(maybe, 42)` pero `maybe` es `Option<i32>` y `42` es `i32`.
- **Rust error:** `E0308: mismatched types — expected Option<i32>, found integer`
- **Workaround:** No usar optionals en test assertions
- **Test afectado:** `basics.test.liva`

---

## Parser — Bugs y limitaciones

### B108 — `defer <statement>` no soportado, solo `defer <expr>` 🔶
- **Ubicación:** `src/parser.rs` — parsing de `defer`
- **Problema:** `defer log += "text"` causa `E2000: Parse Error` porque el parser solo acepta `defer <expression>`, no `defer <statement>` (assignments, compound assignments).
- **Nota:** `defer arr.push(x)` funciona porque `.push()` es una expresión.
- **Workaround:** Solo usar `defer` con llamadas a funciones/métodos, no con assignments
- **Test afectado:** `defer.test.liva`

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

### B113 — `Process.exec` con `or "literal"` genera `&str` vs `String` mismatch 🔶
- **Repro:** `let output = Process.exec("cmd 2>&1 || true") or "EXEC_FAILED"`
- **Problema:** `Process.exec` devuelve `(Option<String>, String)`. El codegen de `or "value"` genera `if !err_str.is_empty() { "EXEC_FAILED" } else { opt.unwrap_or_default() }` donde la rama `or` es `&str` (literal) pero la rama `else` es `String`.
- **Rust error:** `E0308: if and else have incompatible types — expected &str, found String`
- **Workaround:** Usar `rust {}` block con `std::process::Command` directamente.
- **Fix:** El codegen debería generar `.to_string()` en el literal del `or`, o `.as_str()` / borrow en la rama `else`.

### B114 — `.as_str()` generado sobre `&str` en vez de `String` 🔶
- **Ubicación:** `src/codegen.rs` — llamadas a métodos que internamente usan `.as_str()`
- **Problema:** Ciertos métodos stdlib (Regex.replace, Date.add, Date.diff) generan `.as_str()` sobre una variable que ya es `&str`. En Rust nightly funciona, pero en stable `.as_str()` en `&str` no es estable (`feature(str_as_str)`).
- **Rust error:** `E0599: no method named 'as_str' found for reference '&str'`
- **Afecta:** `Regex.replace`, `Date.add`, `Date.diff`
- **Workaround:** Evitar esos métodos en tests. Los tests omiten esas funciones.
- **Fix:** El codegen debería detectar cuándo el receptor ya es `&str` y omitir `.as_str()`, o generar un borrow `&*` en su lugar.

---

## Carencias del lenguaje detectadas

### GAP-001 — No hay `toBeNull()` funcional en test framework 🔷
- Los matchers del test framework no manejan correctamente `Option<T>`.
- `expect(maybe).toBeNull()` debería generar `assert!(maybe.is_none())`.
- Relacionado con B111.

### GAP-002 — `or fail` en test functions no testeable 🔷
- `propagate(a, b): number { let r = divide(a, b) or fail "msg"; return r }` — la función es fallible, pero testing de errores requiere error binding que tiene bugs (B101, B102).
- Necesita un matcher `toThrow()` o `toFail()` funcional.

### GAP-003 — `Set.union()` / `Set.intersection()` devuelve HashSet crudo 🔶
- El resultado pierde los wrappers de Liva (`.has()`, `.size()`, etc.).
- Debería devolver un Set de Liva con todos los métodos disponibles.
