# Issues — Self-Hosting Compiler

> ⚠️ **DOCUMENTO HISTÓRICO.**  
> Inventario de bugs/limitaciones encontrados durante el self-hosting
> Phase 7–9. Muchos items están resueltos en práctica y cubiertos por
> los gates actuales (`compiler/tests/run_all.sh`: selfhost_apps 21/21,
> multifile_apps 3/3, regression 5/5, complex_apps 4/4, e2e_selfhost 5/5).  
> **Fuente de verdad actual:** `compiler/PARITY.md` (paridad gen-2 vs
> bootstrap por feature) + `BACKLOG.md` Fase 11/12.

> Bugs, limitaciones y feature requests encontrados durante el self-hosting.  
> **Última actualización:** 2026-04-14

---

## Resumen

| Estado | Cantidad |
|--------|----------|
| ✅ FIXED | 13 |
| 🔴 OPEN (bugs codegen) | 0 |
| 🟡 OPEN (language gaps) | 4 (ISSUE-005, 006, 007, 009) |
| ⚠️ ARCH | 0 (all resolved in Phase 6.3) |

---

## Issues del bootstrap (compilador Rust → compilando Liva)

### ISSUE-001: `let x: T? = nonOptionalValue` no genera `Some(value)` — ✅ FIXED
- **Tipo:** BUG — **Severidad:** HIGH
- **Fix:** Generates `Some(value)` in VarDecl for T? types

### ISSUE-002: Reassignment de variable enum genera `Some(Variant{})` — ✅ FIXED
- **Tipo:** BUG — **Severidad:** HIGH
- **Fix:** Could not reproduce — codegen already handles correctly

### ISSUE-003: `switch expr` consume la variable (Rust move semantics) — ✅ FIXED
- **Tipo:** LANGUAGE_GAP — **Severidad:** BLOCKER
- **Fix:** Generates `match &variable` for enum data switches

### ISSUE-004: Parámetros de función toman ownership (no borrow) — ✅ FIXED
- **Tipo:** LANGUAGE_GAP — **Severidad:** HIGH
- **Fix:** Clone at call site for non-Copy types (enum/class/string/etc.)

### ISSUE-005: Field access de cross-module types genera `.get_field("name")` — 🟡 OPEN
- **Tipo:** BUG — **Severidad:** HIGH
- **Descripción:** Acceder a campos de tipos definidos en otro módulo genera `.get_field("name")` en vez de `.name`
- **Workaround:** Acceder al campo vía la fuente directa: `this.tokens[this.current].line`

### ISSUE-006: Enum Optional field en constructores genera `Some(Some(..))` — 🟡 OPEN
- **Tipo:** BUG — **Severidad:** MEDIUM
- **Descripción:** Cuando un enum variant tiene campo `T?` y se pasa `T?`, genera `Some(variable)` → `Some(Option<T>)`
- **Workaround:** Cambiar campo de `T?` a `T` con sentinel value

### ISSUE-007: `string` push a `[string?]` no genera `Some()` wrapping — 🟡 OPEN
- **Tipo:** BUG — **Severidad:** MEDIUM
- **Descripción:** `push(stringValue)` en `[string?]` no wrappea en `Some()`
- **Workaround:** Usar variable intermedia con tipo explícito

### ISSUE-008: Switch expression con valor reutilizado — ✅ FIXED
- **Tipo:** LANGUAGE_GAP — **Severidad:** MEDIUM
- **Fix:** Resolved by ISSUE-001 + ISSUE-003 fixes

### ISSUE-009: Constructor field order importa para move semantics — 🟡 OPEN
- **Tipo:** LANGUAGE_GAP — **Severidad:** MEDIUM
- **Descripción:** Operaciones como `source.chars()` consumen `source` por move. El orden de asignación de campos importa.
- **Workaround:** Reordenar campos manualmente

### ISSUE-010: `default` como nombre de campo colisiona con keyword Rust — ✅ FIXED
- **Tipo:** BUG — **Severidad:** LOW
- **Workaround aplicado:** Renombrado a `defaultVal` en ast.liva

---

## Root Cause bugs del codegen self-hosted (codegen.liva)

> Estos son bugs en el codegen del compilador self-hosted, no del bootstrap.
> Se detectaron al escribir la Test Suite (Fase 5).

### RC2: `toBeTruthy`/`toBeFalsy` en `Option<T>` — ✅ FIXED
- **Severidad:** MEDIA
- **Descripción:** Las assertions `expect(x).toBeTruthy()` generaban `assert!(x)` para `Option<T>`.
- **Fix:** `_tryEmitExpectChain` detecta tipo Option via `_lookupVarTypeRef` y emite `.is_some()`/`.is_none()`

### RC3: `self.field.clone().push(x)` muta el clon — ✅ FIXED
- **Severidad:** ALTA
- **Descripción:** Todo acceso a `self.field` generaba `.clone()`, perdiendo mutaciones.
- **Fix:** `_emitMethodCall` detecta métodos mutadores (push/pop/insert/remove/clear/reverse/sort/set) en `self.field` y setea `_inAssignTarget=true` para suprimir `.clone()`

### RC6: `.par()` no implementado — ✅ FIXED
- **Severidad:** BAJA
- **Descripción:** No había dispatch para `.par()`.
- **Fix:** Nuevo helper `_emitIterPrefix(obj, withCloned)` detecta `.par()` chains y emite `.par_iter()` en vez de `.iter()`. Usado en map/filter/forEach/find/some/every/reduce/flatMap.

### RC7: `async fn` nunca se emite — ✅ FIXED
- **Severidad:** ALTA
- **Descripción:** Las funciones nunca se emitían como `async fn`.
- **Fix:** `_emitFunction` y `_emitMethod` ahora chequean `decl.isAsyncInferred` y emiten `pub async fn`. Para `main`, también emiten `#[tokio::main]`.

### RC9: `!(expr)` pierde paréntesis — ✅ FIXED
- **Severidad:** MEDIA
- **Descripción:** `_emitUnary` para `UnOp.Not` generaba `!expr` sin paréntesis para operandos compuestos.
- **Fix:** `_emitUnary` detecta Binary/Ternary/Unary operands y envuelve en `!(...)`

### RCs corregidos

| RC | Descripción | Estado |
|----|-------------|--------|
| RC1 | Map.get `or <value>` generaba `\|\|` en vez de `unwrap_or` | ✅ FIXED en codegen.liva L1494-1505 |
| RC2 | `toBeTruthy`/`toBeFalsy` en `Option<T>` generaba `assert!(x)` | ✅ FIXED — detecta Option → `.is_some()`/`.is_none()` |
| RC3 | `self.field.clone().push(x)` mutaba el clon | ✅ FIXED — detecta métodos mutadores, suprime `.clone()` |
| RC5 | `rust {}` multi-statement blocks | ✅ FIXED — lexer captura contenido completo |
| RC6 | `.par()` no implementado | ✅ FIXED — `_emitIterPrefix` → `.par_iter()` |
| RC7 | `async fn` nunca se emitía | ✅ FIXED — detecta `isAsyncInferred` → `pub async fn` + `#[tokio::main]` |
| RC8 | `const` con string generaba `to_string()` no-const | ✅ FIXED en codegen.liva L1275-1284 |
| RC9 | `!(expr)` pierde paréntesis | ✅ FIXED — detecta operandos compuestos → `!(...)` |

---

## Debilidades arquitectónicas

### ARCH-001: Stdlib dispatch es if-else chain
- **Impacto:** `_emitStringMethod()`, `_emitArrayMethod()`, `_emitGenericMethodCall()` son ~200 líneas cada uno de if-else. Añadir un método nuevo requiere encontrar el lugar correcto en la cadena.
- **Fix propuesto:** Reemplazar con `Map<string, fn>` dispatch tables. Cada método es una entrada en un map.
- **Esfuerzo:** Medio (refactor, no cambia comportamiento)

### ARCH-002: `_emitGenericMethodCall()` duplica lógica
- **Impacto:** Fallback para tipos desconocidos que duplica los métodos tipados. Si se arregla un bug en `_emitStringMethod()`, hay que arreglarlo también en `_emitGenericMethodCall()`.
- **Fix propuesto:** Unificar — si tipo desconocido, intentar resolver primero; si imposible, emitir Rust directo (`.method(args)`) sin try-all-types.
- **Esfuerzo:** Medio

### ARCH-003: Sin error propagation en codegen
- **Impacto:** Codegen escribe `/* unknown */` o `todo!()` para casos no manejados. Los errores los detecta el compilador Rust downstream, no Liva.
- **Fix propuesto:** Acumular errores en `[Diagnostic]` y reportarlos con ubicación. El usuario ve qué feature de Liva no se pudo compilar.
- **Esfuerzo:** Medio

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

## Bootstrap Workarounds — Phase 2.2

> Limitaciones descubiertas del bootstrap durante Type Resolver (semantic.liva).
> No requieren fixes al codegen — se resuelven con patrones en el código Liva.

### W-001: No `return` inside switch arm blocks
- **Tipo:** PARSER LIMITATION
- **Descripción:** `{ return TypeRef.Simple("number"); 0 }` falla con "Expected expression"
  en la posición del `;` después del return. El parser Liva no reconoce `return` como
  statement válido dentro de bloques de switch arm expression.
- **Workaround:** Patrón de variable mutable:
  ```liva
  let result = TypeRef.Simple("unknown")
  let _ = switch t { Arm => { result = value; 0 } }
  return result
  ```
- **Estado:** ⚠️ DOCUMENTED — workaround funcional

### W-002: Variable names collide across switch arms
- **Tipo:** SEMANTIC LIMITATION
- **Descripción:** `let resolved: [TypeRef] = []` en arm de Tuple colisiona con mismo
  nombre en arm de Union. Liva no crea scopes separados por switch arm block.
- **Workaround:** Usar nombres únicos: `tupleResolved`, `genResolved`, `unionResolved`.
- **Estado:** ⚠️ DOCUMENTED — workaround funcional

### W-003: Struct field strings not auto-cloned for multi-use
- **Tipo:** CODEGEN LIMITATION
- **Descripción:** `stmt.variable` usado en `declareVar()` y `_setVarType()` genera
  use-after-move en Rust. El codegen no añade `.clone()` para strings de struct fields.
- **Workaround:** String template trick: `let copy = $"{stmt.variable}"` genera
  `format!("{}", stmt.variable)` que borrowea sin mover.
- **Estado:** ⚠️ DOCUMENTED — workaround funcional

### W-004: Struct field Optional auto-unwrap broken
- **Tipo:** CODEGEN LIMITATION
- **Descripción:** `if decl.returnType != null { ... decl.returnType ... }` no auto-unwraps
  struct fields. Solo funciona con variables locales simples.
- **Workaround:** Extraer a variable local: `let rt: TypeRef? = decl.returnType; if rt != null { ... }`
  O evitar el patrón y no pre-resolver tipos en registration pass.
- **Estado:** ⚠️ DOCUMENTED — workaround funcional

## Bootstrap Workarounds — Phase 2.3

> Limitaciones descubiertas del bootstrap durante Expr typing (semantic.liva).

### W-005: `option_value_vars` leaks across class methods
- **Tipo:** CODEGEN BUG
- **Descripción:** Si un método tiene un parámetro `t: TypeRef?`, el nombre `t` queda
  registrado en `option_value_vars` del codegen. Luego, en otros métodos del mismo class,
  `for t in array` genera `t.as_ref().unwrap().name` en vez de `t.name`, porque el codegen
  cree que `t` sigue siendo Optional.
- **Workaround:** Usar nombres únicos para params Optional: `optRef` en vez de `t`.
- **Estado:** ⚠️ DOCUMENTED — workaround funcional

## Bootstrap Workarounds — Phase 2.4

> Limitaciones descubiertas del bootstrap durante Function signatures (semantic.liva).

### W-006: bare `return` after `=>` not supported
- **Tipo:** PARSER LIMITATION
- **Descripción:** `if cond => return` genera parse error "Expected expression".
  El parser trata `=> return` como one-liner expression, but bare `return` is a statement.
- **Workaround:** Use block form: `if cond { return }`
- **Estado:** ⚠️ DOCUMENTED — workaround funcional
