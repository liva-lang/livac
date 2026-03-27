# Liva Language — Issues & Shortcomings

> **Source**: Self-hosting experiment (lexer + parser in Liva, ~900 lines)
> **Date**: 2026-03-24
> **Branch**: `feat/self-hosting-v2`
> **Status**: Documented during development; some are codegen bugs, others are language design gaps.

---

## Resumen

| Cat. | Total | Bloqueantes | Críticos | Menores |
|------|-------|-------------|----------|---------|
| Codegen (bugs del compilador) | 8 | ~~3~~ 0 ✅ | ~~3~~ 0 ✅ | ~~2~~ 0 ✅ |
| Diseño del lenguaje | 6 | 0 | ~~2~~ 0 ✅ | ~~4~~ 0 ✅ |
| Ergonomía / DX | 7 | 0 | ~~1~~ 0 ✅ | ~~6~~ 0 ✅ |
| **Total** | **21** | **0** | **0** | **0** |

> **🎉 All 21 issues resolved** — 10 FIXED, 7 CLOSED (deferred/not-an-issue), 4 already-implemented

---

## A. Bugs del Codegen (Compilador)

Errores en la generación de código Rust que impiden compilar programas válidos en Liva.

### A1. ~~🔴~~ ✅ String ownership — variables consumidas al pasarlas a funciones

**Severidad**: ~~Bloqueante~~ FIXED (B100)
**Error Rust**: `E0382: use of moved value`

**Descripción**: Cuando una variable `string` se pasa como argumento a una función libre y luego se vuelve a usar, el compilador genera código Rust que mueve el valor (ownership transfer) en la primera llamada.

El codegen ya tiene lógica de auto-clone (B17) para variables en `string_vars`, pero falla cuando la variable proviene de un method call (ej. `_peek()`) y no se registra en `string_vars`.

**Ejemplo Liva** (válido):
```liva
let ch = this._peek()
if isDigit(ch) {       // ← genera: is_digit(ch)  → mueve ch
    ...
}
if isAlpha(ch) {       // ← genera: is_alpha(ch)  → ERROR: ch ya movido
    ...
}
```

**Rust generado** (incorrecto):
```rust
let ch = self._peek();
if is_digit(ch) {      // ch moved here
    ...
}
if is_alpha(ch) {      // ERROR: value used after move
    ...
}
```

**Rust esperado**:
```rust
if is_digit(ch.clone()) {
    ...
}
if is_alpha(ch.clone()) {
    ...
}
```

**Causa raíz**: La variable `ch` (retorno de un method call que devuelve `String`) no se registra en `string_vars` del codegen. Solo se registran: literales string, parámetros tipados `string`, y campos de clase`string`. Falta cubrir: **variables asignadas desde method calls que retornan `string`**.

**Ubicación**: `codegen.rs` → `generate_normal_call()` ~L8472 y `collect_string_vars/track variables`

---

### A2. ~~🔴~~ ✅ Array index con `i32` — falta `as usize` en `for i in 0..arr.length`

**Severidad**: ~~Bloqueante~~ FIXED (B100)
**Error Rust**: `E0277: the type [T] cannot be indexed by i32`

**Descripción**: El patrón `for i in 0..tokens.length { let t = tokens[i] }` genera un loop con `i: i32` pero Rust necesita `usize` para indexar arrays.

El codegen ya tiene lógica de `as usize` para indexing (Bug #34) pero solo funciona si la variable está registrada en `array_vars`. Cuando `tokens` viene del retorno de una función (`tokenize()`), no se registra.

**Ejemplo Liva** (válido):
```liva
let tokens = tokenize(source)
for i in 0..tokens.length {
    let t = tokens[i]     // ← ERROR
}
```

**Rust generado** (incorrecto):
```rust
let tokens = tokenize(source);
for i in 0 .. (tokens.len() as i32) {
    let t = tokens[i];   // ERROR: i32 can't index [T]
}
```

**Rust esperado**:
```rust
for i in 0 .. (tokens.len() as i32) {
    let t = tokens[(i) as usize].clone();
}
```

**Causa raíz**: La variable `tokens` (retorno de función `tokenize()` que devuelve `[TokenWithSpan]`) no se registra en `array_vars`. El codegen necesita inferir del tipo de retorno que es un array.

**Ubicación**: `codegen.rs` → `Expr::Index` handler ~L7264 y tracking de `array_vars`

---

### A3. ~~🔴~~ ✅ Mutabilidad local no inferida — `let x = Foo(...)` debería ser `let mut x`

**Severidad**: ~~Bloqueante~~ FIXED (B100)
**Error Rust**: `E0596: cannot borrow as mutable`

**Descripción**: Cuando una variable local instancia una clase y luego llama un método que requiere `&mut self`, el codegen genera `let x` en lugar de `let mut x`.

El codegen tiene una heurística en `collect_mutated_vars_in_expr` que marca como mutada una variable cuando llama un método no-getter. Sin embargo, en funciones libres (no métodos de clase), la variable `lexer` debería detectarse como mutada por `lexer.tokenize()`.

**Ejemplo Liva** (válido):
```liva
tokenize(source: string): [TokenWithSpan] {
    let lexer = Lexer(source)
    return lexer.tokenize()   // tokenize() modifica self (push a array)
}
```

**Rust generado** (incorrecto):
```rust
pub fn tokenize(source: String) -> Vec<TokenWithSpan> {
    let lexer = Lexer::new(source.clone());
    return lexer.tokenize();   // ERROR: lexer not mut
}
```

**Causa raíz**: `collect_mutated_vars` probablemente no se ejecuta para funciones libres (top-level), o la heurística no considera `tokenize()` como mutante.

**Ubicación**: `codegen.rs` → `collect_mutated_vars_in_expr()` ~L1287 y generación de funciones libres

---

### A4. ~~🟡~~ ✅ Imports no usados generan warnings

**Severidad**: ~~Menor (warning, no error)~~ FIXED
**Implementación**: Codegen now emits `#[allow(unused_imports)]` before each generated `use` statement, both in module files and entry point. This suppresses Rust's `unused imports` warning for pass-through type imports.

**Descripción**: `import { Token, TokenWithSpan } from "./token.liva"` en `main.liva` genera `use crate::token::{Token, TokenWithSpan}` aunque esos símbolos solo se usan como tipos en el output de `tokenize()`, no directamente en el código.

**Ejemplo Liva**:
```liva
import { Token, TokenWithSpan } from "./token.liva"
import { tokenize } from "./lexer.liva"
// Token y TokenWithSpan se usan indirectamente vía tokenize() 
```

**Impacto**: No bloqueante, pero genera ruido en la compilación.

**Posible solución**: El compilador podría detectar imports no usados (ya lo hace el linter con W002) y suprimirlos del `use` generado, o añadir `#[allow(unused_imports)]` al módulo.

---

### A5. ~~🟡~~ ✅ Variable no usada en switch expression

**Severidad**: ~~Menor~~ FIXED
**Implementación**: Parser now accepts `Token::Underscore` as enum binding. Codegen generates `field: _` pattern. Semantic analysis skips `_` bindings. 3 tests added.

**Descripción**: El patrón `Token.RustBlock(v) => "rust{...}"` captura `v` pero no lo usa. Liva no tiene forma de escribir `Token.RustBlock(_)` con wildcard en destructuring de enum.

**Ejemplo Liva**:
```liva
Token.RustBlock(v) => "rust{...}"   // v no se usa
```

**Posible solución**: Soportar `Token.RustBlock(_)` para ignorar la variable capturada.

---

### A6. ~~🟡~~ 🔵 `charAt()` devuelve `string` — semántica inconsistente

**Severidad**: ~~Menor~~ CLOSED (deferred to B1)
**Resolución**: This is a natural consequence of Liva not having a `char` type (B1). The current behavior is correct — `charAt()` returns `string` which works for all comparisons and operations. Performance impact is negligible for typical use cases. If `char` type is added (B1), this can be revisited.

**Descripción**: `charAt()` operaba originalmente devolviendo `char` en Rust, pero se cambió a devolver `String` (Bug B95) para que `ch == "a"` funcione. Esto es correcto para Liva (no tiene tipo `char`), pero genera código verbose en Rust: `.chars().nth(pos).map(|c| c.to_string()).unwrap_or_default()`.

**Impacto**: Sin error, pero rendimiento inferior en loops intensivos (string allocation por cada carácter). Un tipo `char` nativo en Liva o una optimización de codegen que use `&str` slices mejoraría el rendimiento.

---

### A7. ~~🟡~~ 🔵 Enum con data en switch — destructuring genera variables no-`mut`

**Severidad**: ~~Menor~~ CLOSED (won't-fix)

**Descripción**: Cuando un switch expression destrucuta un enum con datos (`Token.IntLiteral(v)`), la variable `v` se genera como inmutable. Si el cuerpo necesitara mutar `v`, no sería posible.

**Resolución**: Cerrado como won't-fix. No hay caso real donde esto falle (510+ tests, 3 dogfooding rounds). Blanket `mut` generaría warnings de `unused_mut` en Rust. Si se necesita en el futuro, se implementará vía análisis semántico que detecte si el binding es mutado en el cuerpo del arm.

---

### A8. ~~🟡~~ 🔵 `isAlphaNumeric()` — auto-clone funciona pero por coincidencia

**Severidad**: ~~Menor~~ CLOSED (deferred to C6)
**Resolución**: The auto-clone mechanism works correctly for all known cases (510+ tests, 3 dogfooding rounds). The "fragility" is theoretical. If reference parameters (&string) are added (C6), this pattern would naturally improve.

**Descripción**: En `isAlphaNumeric(ch)`, el codegen genera `is_alpha(ch.clone()) || is_digit(ch.clone())` — esto funciona porque `ch` está registrado como parámetro `String`. Pero si `isAlphaNumeric` llamara a más funciones, el pattern sería frágil.

---

## B. Diseño del Lenguaje

Limitaciones o carencias del diseño del lenguaje Liva que dificultan escribir programas reales.

### B1. ~~🟡~~ 🔵 No hay tipo `char` — los caracteres son `string`

**Severidad**: ~~Crítico~~ CLOSED (design decision, deferred post-v2.0)
**Resolución**: Liva deliberately uses `string` for characters, matching the semantics of Python/TypeScript which also lack a separate char type. This simplifies the type system and avoids char/string conversion complexity. The current implementation works correctly for all use cases. Performance optimizations (like `&str` slices for single characters) can be added transparently in codegen without exposing a `char` type to users.

**Descripción**: Liva no tiene un tipo `char` nativo. Los métodos como `charAt()` devuelven `string`. Esto funciona semánticamente pero tiene consecuencias:
- Cada comparación de caracter (`ch == "a"`) compara `String` con `&str` en Rust
- El codegen debe generar `ch.as_str() >= "a"` para comparaciones de rango
- La iteración carácter a carácter aloca un `String` por carácter (heap allocation)

**Ejemplo afectado**:
```liva
isAlpha(ch: string): bool => ch >= "a" and ch <= "z" or ch >= "A" and ch <= "Z" or ch == "_"
```

**Sugerencia**: Introducir un tipo `char` que compile a Rust `char`, con conversión implícita `char → string` cuando sea necesario.

---

### B2. ~~🟡~~ 🔵 No hay `null` para tipos propios — `lookupKeyword` no puede devolver "no encontrado"

**Severidad**: ~~Crítico~~ CLOSED (design decision, deferred post-v2.0)
**Resolución**: Liva uses explicit error handling (`or value`, `or fail`, tuple destructuring) instead of nullable types. This is a deliberate design choice aligned with Rust's philosophy. The `lookupKeyword` workaround (using a default variant) is idiomatic Liva. Full nullable type support (`T?` → `Option<T>`) would be a major type system extension for a future version.

**Descripción**: En el lexer, `lookupKeyword(word)` debería devolver `Token?` (nullable) o un `Option<Token>`. Actualmente devuelve `Token` y usa el caso `_ => Token.Ident(word)` como fallback, que mezcla "no era keyword" con "es un identifier".

Liva no soporta tipos nullable (`T?`) ni `Option<T>` como tipo genérico. Esto obliga a usar convenciones (como el default case) en lugar de expresar la semántica correctamente.

**Ejemplo ideal** (no soportado aún):
```liva
lookupKeyword(word: string): Token? {
    return switch word {
        "let" => Token.Let,
        ...
        _ => null
    }
}
```

**Workaround actual**: El default arm devuelve `Token.Ident(word)`, que funciona para nuestro caso pero no es generalizable.

---

### B3. ~~🔵~~ ✅ No hay `enum` sin datos como constantes — verbosidad en token types

**Severidad**: ~~Menor~~ CLOSED (not an issue)
**Resolución**: Liva already supports enums with and without data (v2.0). The verbosity of 100+ variants in generated Rust code is inherent to the enum model and doesn't affect the user. No change needed.

**Descripción**: Liva ya soporta enums con y sin datos (v2.0). Sin embargo, la sintaxis es adecuada. No es un problema real, pero la definición de 100+ variantes genera mucho código Rust. Un compilador más maduro podría comprimir esto.

---

### B4. ~~🔵~~ ✅ No hay `match` exhaustivo con feedback

**Severidad**: ~~Menor~~ FIXED
**Implementación**: Semantic analyzer now stores enum variant lists (`enum_variants` map) and checks exhaustiveness in `check_enum_exhaustiveness()`. When all variants of an enum are covered in switch arms, `_` can be omitted. Missing variants produce error `E0904` listing which variants are uncovered. Supports Or-patterns. 2 tests added.

**Descripción**: El `switch` de Liva siempre requiere un `_ => ...` default case. No hay verificación de exhaustividad para enums (que si cubres todos los casos, puedes omitir `_`). Rust sí lo hace.

**Impacto**: Si se añade una nueva variante a un enum, el compilador no avisa que los `switch` existentes no la cubren.

---

### B5. ~~🔵~~ ✅ No hay `type alias`

**Severidad**: ~~Menor~~ FIXED (already supported)
**Implementación**: Type aliases were already implemented across all stages: lexer (`Token::Type`), parser (`type Name = TargetType`), semantic (`validate_type_alias` with circular reference detection), and codegen (inline expansion via `expand_type_alias`). Supports generics: `type Result<T> = (T, error)`.

**Descripción**: No se puede hacer `type TokenList = [TokenWithSpan]`. Esto obliga a repetir tipos largos.

---

### B6. ~~🔵~~ ✅ No hay pattern matching anidado

**Severidad**: ~~Menor~~ FIXED (already supported)
**Implementación**: Switch guards (`if condition`) were already implemented across all stages: AST (`guard: Option<Box<Expr>>`), parser (parses `if` after pattern), semantic (validates guard expression), and codegen (emits `if guard` in match arms). Syntax: `Pattern if condition => body`.

**Descripción**: No se puede hacer:
```liva
switch token {
    Token.IntLiteral(v) if v > 100 => "big"
    Token.IntLiteral(v) => "small"
}
```

Los guards (`if`) en los arms del switch no están soportados.

---

## C. Ergonomía / Developer Experience

Aspectos que hacen la experiencia de desarrollo menos fluida.

### C1. ~~🟡~~ ✅ `parseInt()` requiere destructuring — no hay conversión directa

**Severidad**: ~~Crítico~~ FIXED (already supported)
**Implementación**: The `or value` syntax already applies to `parseInt()` and `parseFloat()` via the B16 fix. `let x = parseInt(s) or 0` generates `match s.parse::<i32>() { Ok(v) => v, Err(_) => 0 }`, directly producing an `int` value without tuple destructuring.

**Descripción**: Para convertir un string a número, se debe usar `let val, err = parseInt(s)`, que devuelve una tupla `(number, error)`. No hay una versión simple que devuelva el número directamente (o falle).

**Ejemplo actual** (verbose):
```liva
let intVal, parseErr = parseInt(value)
if parseErr {
    intVal = 0
}
```

**Ejemplo ideal**:
```liva
let intVal = parseInt(value) or 0
// o
let intVal = value.toNumber()
```

---

### C2. ~~🔵~~ ✅ No hay `for i, item in array` — indexing manual

**Severidad**: ~~Menor~~ FIXED
**Implementación**: Parser already supported `for var1, var2 in expr`. Codegen now distinguishes Map iteration (`.iter()`) from Array enumerate (`.iter().enumerate()`) by checking `map_vars`. Index is cast to `i32` for Liva's `int` type. 3 tests added.

**Descripción**: Para iterar un array con índice, se necesita `for i in 0..arr.length { let item = arr[i] }`. Un `for i, item in arr` (enumerate) sería más ergonómico.

**Ejemplo actual**:
```liva
for i in 0..tokens.length {
    let t = tokens[i]
    print($"  {t.line}:{t.col}  {t.describe()}")
}
```

**Ejemplo ideal**:
```liva
for i, t in tokens {
    print($"  {t.line}:{t.col}  {t.describe()}")
}
```

---

### C3. ~~🔵~~ 🔵 Los parámetros `string` no se pasan por referencia

**Severidad**: ~~Menor~~ CLOSED (deferred to C6)
**Resolución**: The auto-clone mechanism handles ownership correctly. This is a codegen optimization that would be part of reference parameter support (C6). No user-facing issue.

**Descripción**: En Liva, todos los parámetros se pasan por valor. Para strings, esto implica que cada llamada a función que recibe un `string` potencialmente mueve o clona el valor. El codegen compensa con auto-clone, pero semánticamente sería más limpio que:
- Los parámetros readonly se pasen como `&str` en Rust
- Solo los parámetros mutados necesiten `String`

Esto no afecta al usuario de Liva, pero genera código Rust más pesado.

---

### C4. ~~🔵~~ ✅ No hay operador `+=` / `-=` / `*=`

**Severidad**: ~~Menor~~ FIXED
**Implementación**: Desugaring at parser level — `x += expr` → `Assign { target: x, value: Binary(x, +, expr), op: Some(Add) }`. Formatter uses `op` field to round-trip back to `+=` syntax. 7 tests added.

**Descripción**: El lexer tiene muchos patrones como:
```liva
this.pos = this.pos + 1
this.col = this.col + 1
content = content + ch
```

Con `+=` sería:
```liva
this.pos += 1
content += ch
```

---

### C5. ~~🔵~~ ✅ No hay `StringBuilder` o acumulador eficiente de strings

**Severidad**: ~~Menor~~ FIXED
**Implementación**: Codegen now detects the pattern `x = x + expr` (or `x += expr`) where `x` is a known string variable and generates `x.push_str(...)` instead of `x = format!("{}{}", x, expr)`. Handles string literals, string variables (`&var`), and other expressions (`&expr.to_string()`). 3 tests added.

**Descripción**: El lexer construye strings carácter a carácter:
```liva
let content = ""
content = content + ch   // repetido muchas veces
```

Cada `+` aloca un nuevo `String` en Rust. Un `StringBuilder` o detección automática del patrón append para usar `push_str()` mejoraría el rendimiento.

---

### C6. ~~🔵~~ 🔵 No hay forma de marcar parámetros como "no consume"

**Severidad**: ~~Menor~~ CLOSED (future enhancement, post-v2.0)
**Resolución**: Reference parameters (`&string`) would require deep changes across parser, semantic, codegen, and the type system. The auto-clone mechanism handles all ownership issues correctly for current use cases. This is a performance optimization for a future version, not a correctness issue.

**Descripción**: Las funciones helper como `isDigit(ch: string)` toman ownership del string. En Liva no hay forma de expresar "esta función solo lee el string, no lo consume". El codegen compensa con auto-clone, pero sería más explícito tener:
```liva
isDigit(ch: &string): bool    // reference parameter (futuro)
```

---

### C7. ~~🔵~~ ✅ `import` requiere extensión `.liva`

**Severidad**: ~~Menor~~ FIXED
**Implementación**: Module resolver and semantic validator now try appending `.liva` extension when the import path has no extension and does not exist. Both `module.rs` (resolve_import_path) and `semantic.rs` (validate_import) updated. LSP already had this fallback. 1 integration test added.

**Descripción**: Los imports deben incluir la extensión del archivo:
```liva
import { Token } from "./token.liva"
```

En muchos lenguajes, la extensión se omite:
```liva
import { Token } from "./token"
```

---

## D. Resumen de Acciones Propuestas

### Prioridad Alta (bloquean self-hosting)
1. **A1**: ✅ FIXED — Registrar en `string_vars` variables asignadas desde method calls que retornan `string`
2. **A2**: ✅ FIXED — Registrar en `array_vars` variables asignadas desde funciones que retornan `[T]`
3. **A3**: ✅ FIXED — `collect_mutated_vars` ahora analiza `Stmt::Return` + heurística `to*` corregida

### Prioridad Media (mejoran la experiencia)
4. **B1**: ✅ CLOSED — Design decision: string for chars, deferred post-v2.0
5. **B2**: ✅ CLOSED — Design decision: explicit error handling, deferred post-v2.0
6. **C1**: ✅ FIXED — `parseInt(s) or 0` ya funciona con `or value`
7. **C4**: ✅ FIXED — Operadores compuestos `+=`, `-=`, `*=`, `/=`, `%=`

### Prioridad Baja (calidad de vida)
8. **A4**: ✅ FIXED — `#[allow(unused_imports)]` en codegen
9. **A5**: ✅ FIXED — Soportar `_` en destructuring de enum
10a. **A6**: ✅ CLOSED — Deferred to B1
10b. **A7**: ✅ CLOSED — Won't-fix (not a real issue)
10c. **A8**: ✅ CLOSED — Deferred to C6
10d. **B3**: ✅ CLOSED — Not an issue (enums already supported)
10e. **B5**: ✅ FIXED — Type aliases already implemented
10f. **B6**: ✅ FIXED — Switch guards already implemented
10g. **C3**: ✅ CLOSED — Deferred to C6
10h. **C6**: ✅ CLOSED — Future enhancement, post-v2.0
10. **B4**: ✅ FIXED — Enum switch exhaustiveness checking (E0904)
11. **C2**: ✅ FIXED — `for i, item in array` (enumerate)
12. **C5**: ✅ FIXED — `push_str()` optimization for string append
13. **C7**: ✅ FIXED — Imports sin extensión `.liva`

---

> **Nota**: Los bugs del codegen (A1-A3) son los únicos bloqueantes para que el self-hosting lexer compile. Los demás son mejoras de diseño y ergonomía identificadas durante la experiencia de escribir un programa real de ~900 líneas en Liva.
