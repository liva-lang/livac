# Issues — Self-Hosting v4

> Bugs, carencias del lenguaje y feature requests encontrados durante el self-hosting.  
> Cada issue se documenta aquí para corregirlo después en el compilador Rust.

---

<!-- Template:
### ISSUE-NNN: Título descriptivo
- **Tipo:** BUG | LANGUAGE_GAP | FEATURE_REQUEST
- **Severidad:** BLOCKER | HIGH | LOW
- **Descripción:** Qué pasa
- **Código que falla:**
```liva
// snippet
```
- **Error:** mensaje de error o comportamiento incorrecto
- **Workaround:** si existe
- **Estado:** OPEN | FIXED | WONTFIX
-->

(Ningún issue registrado todavía — se irán añadiendo durante el desarrollo)

---

### ISSUE-001: `let x: T? = nonOptionalValue` no genera `Some(value)`
- **Tipo:** BUG
- **Severidad:** HIGH
- **Descripción:** Cuando se declara una variable opcional y se asigna un valor no-opcional directamente, el codegen Rust genera `let x: Option<T> = value;` en lugar de `let x: Option<T> = Some(value);`
- **Código que falla:**
```liva
let exprBodyOpt: Expr? = exprBody  // exprBody es Expr, no Expr?
```
- **Error:** `E0308: expected Option<Expr>, found Expr`
- **Workaround:** Usar patrón de dos pasos:
```liva
let exprBodyOpt: Expr? = null   // genera Option<Expr> = None;
exprBodyOpt = exprBody           // genera expr_body_opt = Some(expr_body);
```
- **Estado:** ✅ FIXED (generates `Some(value)` in VarDecl for T? types)

---

### ISSUE-002: Reassignment de variable enum genera `Some(Variant {...})` en lugar de `Variant {...}`
- **Tipo:** BUG
- **Severidad:** HIGH
- **Descripción:** Al reasignar una variable enum con un variant diferente, el codegen envuelve el nuevo variant en `Some()`, generando código Rust inválido.
- **Código que falla:**
```liva
let base: TypeRef = TypeRef.Named("", [])
if someCondition {
    base = TypeRef.Array(innerType)  // genera Some(TypeRef::Array {...})
}
```
- **Error:** `E0308: mismatched types — expected TypeRef, found Option<TypeRef>`
- **Workaround:** Usar `return` temprano en lugar de reasignación:
```liva
if someCondition { return TypeRef.Array(innerType) }
return TypeRef.Named("", [])
```
- **Estado:** ✅ FIXED (could not reproduce — codegen already handles correctly)

---

### ISSUE-003: `switch expr` consume la variable (Rust move semantics)
- **Tipo:** LANGUAGE_GAP
- **Severidad:** BLOCKER
- **Descripción:** Todo `switch variable { ... }` genera un `match variable { ... }` en Rust que toma ownership del valor. Después del switch, la variable está "moved" y no se puede usar. El arm `default:` no bind el valor original (genera `_ => {}`), por lo que el valor se pierde incluso en el arm por defecto.
- **Código que falla:**
```liva
let typeName = switch expr { Expr.Identifier(n) => n, _ => "" }
// expr ya no es usable aquí - fue consumida por el match
expr = Expr.MapLiteral(entries)  // Error: expr was moved
```
- **Error:** `E0382: use of moved value`
- **Workaround:** Múltiples estrategias:
  1. Extraer info sin switch (usar tokens/índices en vez de pattern matching sobre AST)
  2. Switch statement (no expression) donde TODOS los arms tienen return
  3. Pasar el valor a una función helper que lo consume en un único switch
  4. Trackear info adicional (bool flags, token indices) para evitar switches
- **Estado:** ✅ FIXED (generates `match &variable` for enum data switches)

---

### ISSUE-004: Parámetros de función toman ownership (no borrow)
- **Tipo:** LANGUAGE_GAP
- **Severidad:** HIGH
- **Descripción:** Todos los parámetros de función en Liva se pasan by-value (move) en el código Rust generado. No hay forma de pasar por referencia. Esto significa que llamar `myFunc(value)` mueve `value` y no se puede usar después.
- **Código que falla:**
```liva
let containsFail = this._exprContainsFail(exprBody)  // mueve exprBody
exprBodyOpt = exprBody  // Error: value already moved
```
- **Error:** `E0382: use of moved value`
- **Workaround:** Evitar funciones que inspeccionan valores. Usar token scanning u otras técnicas que no necesiten el valor AST:
```liva
let containsFail = this._rangeContainsFail(startPos, this.current)  // scan tokens instead
```
- **Estado:** ✅ FIXED (clone at call site for non-Copy types — enum/class/string/etc.)

---

### ISSUE-005: Field access de cross-module types genera `.get_field("name")`
- **Tipo:** BUG
- **Severidad:** HIGH
- **Descripción:** Acceder a campos de tipos definidos en otro módulo genera `.get_field("name")` en vez del acceso directo `.name`. Esto causa E0599 ya que `get_field` no existe.
- **Código que falla:**
```liva
let tok = this._peek()
let line = tok.line  // genera tok.get_field("line") en vez de tok.line
```
- **Error:** `E0599: no method named get_field found`
- **Workaround:** Acceder al campo vía la fuente directa:
```liva
let line = this.tokens[this.current].line  // acceso directo funciona
```
- **Estado:** OPEN

---

### ISSUE-006: Enum Optional field en constructores genera `Some(Some(..))`
- **Tipo:** BUG
- **Severidad:** MEDIUM
- **Descripción:** Cuando un enum variant tiene un campo `T?` y se le pasa una variable `T?`, el codegen genera `Some(variable)` que crea `Some(Option<T>)` en lugar de solo pasar `variable` directamente.
- **Código que falla:**
```liva
let rest: string? = null
rest = this._parseIdentifier()
return BindingPattern.ArrayPat(elements, rest)  // genera rest: Some(rest) → Some(Option<String>)
```
- **Error:** `E0308: expected String, found Option<String>`
- **Workaround:** Cambiar el campo de `T?` a `T` con sentinel value (ej: `""` para strings).
- **Estado:** OPEN

---

### ISSUE-007: `string` push a `[string?]` no genera `Some()` wrapping
- **Tipo:** BUG  
- **Severidad:** MEDIUM
- **Descripción:** Hacer `push(stringValue)` en un array declarado como `[string?]` no wrappea el valor en `Some()`.
- **Código que falla:**
```liva
let elements: [string?] = []
elements.push(this._parseIdentifier())  // expects Option<String>, got String
```
- **Error:** `E0308: expected Option<String>, found String`
- **Workaround:** Usar variable intermedia:
```liva
let elemName: string? = null
elemName = this._parseIdentifier()
elements.push(elemName)
```
- **Estado:** OPEN

---

### ISSUE-008: Switch expression con valor reutilizado requiere switch duplicado
- **Tipo:** LANGUAGE_GAP
- **Severidad:** MEDIUM
- **Descripción:** Cuando necesitas el resultado de un switch expression como dos tipos distintos (ej: `BinOp` para `Expr.Binary` y `BinOp?` para `AssignStmt.op`), no puedes reusar la variable porque el switch la consume (ISSUE-003) y no puedes hacer `let x: T? = y` (ISSUE-001). Se necesitan dos switches idénticos.
- **Código que falla:**
```liva
let op = switch this.tokens[compoundIdx].kind {
    TokenKind.PlusAssign => BinOp.Add, ...
}
let opForStmt: BinOp? = null
opForStmt = switch this.tokens[compoundIdx].kind {  // switch duplicado idéntico
    TokenKind.PlusAssign => BinOp.Add, ...
}
```
- **Causa raíz:** Combinación de ISSUE-001 + ISSUE-003.
- **Workaround:** Duplicar el switch.
- **Estado:** ✅ FIXED — resolved by ISSUE-001 + ISSUE-003 fixes.

---

### ISSUE-009: Constructor field order importa para move semantics
- **Tipo:** LANGUAGE_GAP
- **Severidad:** MEDIUM
- **Descripción:** En constructores, el orden de asignación de campos importa porque operaciones como `source.chars()` consumen `source` por move. Si `this.source = source` va después de `this.chars = source.chars()`, el Rust generado falla con E0382.
- **Código que falla:**
```liva
constructor(source: string) {
    this.source = source         // Si va después de chars → ERROR
    this.chars = source.chars()  // Consume source por move
}
```
- **Error:** `E0382: use of moved value: source`
- **Workaround:** Reordenar campos manualmente para que el uso que consume vaya último.
- **Estado:** OPEN — fix futuro: análisis de dependencias en constructor.

---

### ISSUE-010: `default` como nombre de campo colisiona con keyword Rust
- **Tipo:** BUG
- **Severidad:** LOW
- **Descripción:** El campo `default` de `Param` (para valores por defecto) es keyword de Rust. El sanitizer de nombres (`sanitize_name`) no lo cubre en todos los contextos (struct fields, pattern bindings).
- **Workaround:** Renombrar a `defaultVal` en ast.liva.
- **Estado:** OPEN

---

### ISSUE-011: String comparison no mueve en Rust pero el modelo mental dice que sí
- **Tipo:** LANGUAGE_GAP
- **Severidad:** LOW
- **Descripción:** `firstChar == "_"` en Rust usa `&self` (PartialEq), por lo que NO mueve. Pero el programador de Liva no tiene forma de saber si un operador mueve o no. Esto genera confusion y workarounds innecesarios.
- **Código confuso:**
```liva
_scanIdentifier(firstChar: string, ...) {
    let isPrivate = firstChar == "_"  // ¿mueve? NO, pero no es obvio
    let name = firstChar              // ¿mueve? SÍ
}
```
- **Workaround:** Ninguno necesario (funciona), pero revela que el modelo mental es confuso.
- **Estado:** INFO — se resuelve con liveness analysis (Fase 2.7).

---

### ISSUE-012: Switch expression con `_ => null` no genera Option correcto
- **Tipo:** BUG
- **Severidad:** MEDIUM
- **Descripción:** Cuando un switch expression tiene arms que devuelven `T` y un `_ => null`, el codegen no unifica los tipos. Genera `match { ... "x" => Color::Red, _ => None }` donde un arm es `Color` y otro `Option<Color>`. Debería generar `Some(Color::Red)` en los arms no-null y `None` en el default, o bien el `let` debería ser `Option<T>` y el return no wrappear.
- **Código que falla:**
```liva
lookupColor(name: string): Color? {
    let result = switch name {
        "red"   => Color.Red,
        _       => null
    }
    return result
}
```
- **Genera:**
```rust
let result = match name.as_str() {
    "red" => Color::Red,  // Color
    _ => None,            // Option<Color>  ← TYPE MISMATCH
};
return Some(result);      // double-wrap
```
- **Esperado:**
```rust
let result = match name.as_str() {
    "red" => Some(Color::Red),
    _ => None,
};
return result;
```
- **Workaround:** Usar sentinel value (`_ => TokenKind.EOF`) y comprobar después.
- **Estado:** OPEN

---

## Feature Requests

> Features del lenguaje que facilitarían el self-hosting significativamente.

### FR-001: Operador `is` para type checking
- **Prioridad:** Alta para self-hosting
- **Descripción:** Permite comprobar tipo de enum sin switch completo.
```liva
// Actual: switch con boilerplate
let isIdent = switch expr { Expr.Identifier(_) => true, _ => false }

// Propuesta:
if expr is Expr.Identifier { ... }
if expr is Expr.Identifier(name) { print(name) }  // con binding
```
- **Generaría:** `matches!(expr, Expr::Identifier(_))` o `if let Expr::Identifier(name) = &expr`

### FR-002: `if let` para pattern matching con binding
- **Prioridad:** Alta para self-hosting
```liva
if let Expr.Identifier(name) = expr {
    print(name)
}
```
- **Generaría:** `if let Expr::Identifier(name) = &expr { ... }`

### FR-003: Named constructor parameters
- **Prioridad:** Media
- **Descripción:** Con 5+ campos, los constructores posicionales son ilegibles.
```liva
// Actual:
FunctionDecl(name, typeParams, params, returnType, body, exprBody, isAsync, containsFail)

// Propuesta:
FunctionDecl(name: name, params: params, returnType: returnType, body: body, ...)
```

### FR-004: Enum methods / computed properties
- **Prioridad:** Media
```liva
enum TokenKind {
    Ident(name: string), IntLit(value: number), ...
    isLiteral(): bool => switch this { TokenKind.IntLit(_) => true, ... }
}
```

### FR-005: PartialEq automático para enums unitarios
- **Prioridad:** Alta para self-hosting
- **Descripción:** Permitir `op == BinOp.Add` en vez de switch.
```liva
if token.kind == TokenKind.LParen { ... }  // en vez de switch
```
- **Generaría:** `#[derive(PartialEq)]` en enums sin datos.

### FR-006: `clone()` explícito (escape hatch)
- **Prioridad:** Baja — solo si liveness analysis no cubre todos los casos
```liva
let copy = clone(originalValue)
```

---

## Codegen Bootstrap Fixes — Phase 2.1

> Fixes aplicados al compilador Rust durante el desarrollo de semantic.liva.

### SH-011: Switch expression mutation scanner
- **Tipo:** BUG (codegen bootstrap)
- **Archivo:** `src/codegen.rs` — `collect_mutated_vars_in_expr()`
- **Descripción:** `fields.push(...)` dentro de arms de `let _ = switch x { ... }` no
  marcaba `fields` como mutada, generando `let fields: Vec<T> = vec![]` sin `mut`.
- **Causa raíz:** `collect_mutated_vars_in_expr()` tenía `_ => {}` catch-all que
  ignoraba `Expr::Switch`. Los arms no se recorrían para buscar mutaciones.
- **Fix:** Añadido handler para `Expr::Switch` que recorre `SwitchBody::Block`/`Expr`.
- **Estado:** ✅ FIXED — 518 tests verdes

### SH-012: init_is_already_optional() no detecta Expr::Member
- **Tipo:** BUG (codegen bootstrap)
- **Archivo:** `src/codegen.rs` — `init_is_already_optional()`
- **Descripción:** `ParamSig(name, p.typeRef, ...)` generaba `Some(p.type_ref)` aunque
  `p.type_ref` ya es `Option<TypeRef>`. La función no manejaba `Expr::Member`.
- **Causa raíz:** `init_is_already_optional()` solo chequeaba `Identifier`, `Call`,
  `MethodCall`, `OptionalChain`. No comprobaba member access como `p.typeRef`.
- **Fix:** Añadido handler `Expr::Member` que busca el tipo del objeto en `var_types`
  y verifica si el campo es Optional en `class_optional_fields`.
- **Estado:** ✅ FIXED — 518 tests verdes

### SH-013: For-loop variables no registradas en var_types
- **Tipo:** BUG (codegen bootstrap)
- **Archivo:** `src/codegen.rs` — for-loop class instance tracking
- **Descripción:** `for p in decl.params` → `p` se registraba en `class_instance_vars`
  pero NO en `var_types`, impidiendo que SH-012 detectara el tipo de `p`.
- **Fix:** Añadido `var_types.insert(var_name, element_type)` junto al `class_instance_vars.insert()`.
- **Estado:** ✅ FIXED — 518 tests verdes
