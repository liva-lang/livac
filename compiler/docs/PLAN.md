# Self-Hosting: Compilador de Liva escrito en Liva

> **Estado:** Fase 2.6 completada ✅ (semantic.liva — Import resolution)
> **Última actualización:** 2026-03-31
> **Próximo:** Fase 2.7 — Liveness analysis

---

## Objetivo

Reescribir el compilador `livac` en Liva. No es un port 1:1 del compilador Rust —
es un **rediseño** que corrige los errores arquitectónicos del compilador actual.

El compilador Rust actual (`src/`) se convierte en **bootstrap compiler**: solo existe
para compilar el compilador Liva la primera vez. Después, el compilador Liva se
compila a sí mismo.

## Estructura del repo

```
AHORA (durante desarrollo):
  livac/
  ├── src/              ← bootstrap compiler (Rust) — necesario para compilar
  ├── compiler/
  │   ├── src/          ← compilador Liva (en desarrollo)
  │   └── docs/         ← PLAN.md, ISSUES.md
  ├── tests/            ← tests del bootstrap
  └── Cargo.toml

DESPUÉS (Fase 4 completada — self-hosting funcional):
  livac/
  ├── src/              ← compilador Liva (promovido desde compiler/src/)
  ├── docs/
  ├── tests/            ← tests del compilador Liva
  └── bootstrap/        ← compilador Rust (congelado, solo para primer build)
      ├── src/
      ├── tests/
      └── Cargo.toml
```

Cuando el compilador Liva llegue a Fase 4 (se compila a sí mismo), `compiler/src/`
se promueve a `src/` y el código Rust actual se archiva en `bootstrap/`.

## Por qué reescribir (no reparar)

El compilador Rust tiene un defecto fundamental: **no tiene sistema de tipos**.
El codegen (19.172 líneas) usa ~20 HashSets para "recordar" tipos de variables:

```rust
// codegen.rs actual — el codegen ADIVINA tipos:
string_vars: HashSet<String>,         // "¿es string?"
array_vars: HashSet<String>,          // "¿es array?"
json_value_vars: HashSet<String>,     // "¿es JSON?"
class_instance_vars: HashSet<String>, // "¿es clase?"
// ...y 15 más

// Esto lleva a hacks como:
if !self.array_vars.contains(var_name)
    && !var_name.contains("person")   // ← literalmente mira el nombre
    && !var_name.contains("user")     // ← de la variable para decidir
{
    // "Probably JSON" — generate .get_field()
}
```

Esto no se arregla con parches. Se arregla con un **semantic analyzer que anote
tipos en el AST antes de que codegen lo vea**. Eso es lo que hace el compilador nuevo.

## Arquitectura del compilador nuevo vs actual

```
COMPILADOR RUST (actual — bootstrap):

  Lexer → Parser → Semantic(valida) → Codegen(19K líneas, adivina tipos)
                         ↓
                    NO anota tipos

COMPILADOR LIVA (nuevo):

  Lexer → Parser → Semantic(valida + TIPA) → Codegen(~4K líneas, mecánico)
                         ↓
                    TypeContext: cada Expr tiene su tipo resuelto
```

## Referencia

- Bugs y feature requests: **`ISSUES.md`** (11 bugs + 6 feature requests)
- Plan de liveness analysis: **`docs/plans/PLAN_LIVENESS_ANALYSIS.md`**
- Guía de estilos: **`docs/guides/style-guide.md`**
- Referencia rápida del lenguaje: **`docs/QUICK_REFERENCE.md`**
- Skill reference: **`skills/liva-lang/SKILL.md`**

---

## Roadmap completo

### Fase 0: Fix Bootstrap — Arreglar el compilador Rust

> **Objetivo:** Que escribir Liva sea cómodo. No arreglar todo — solo lo blockeante.
> **Estimación:** 2-3 sesiones

El compilador Rust solo necesita ser "lo bastante bueno" para compilar el compilador Liva.
Estos son los bugs que bloquean la escritura cómoda de Liva:

| # | Fix | Impacto | Estado |
|---|-----|---------|--------|
| FIX-1 | `let x: T? = value` → generar `Some(value)` (ISSUE-001) | Elimina 15+ workarounds | ✅ |
| FIX-2 | Reassignment de enum sin `Some()` espurio (ISSUE-002) | Elimina 8 workarounds | ✅ No reproduce |
| FIX-3 | `switch` genera `match &expr` si variable se usa después (ISSUE-003) | **Blocker** — sin esto no hay pattern matching útil | ✅ |
| FIX-4 | Non-Copy params: clone at call site (ISSUE-004) | **Blocker** — sin esto no puedes pasar structs a funciones | ✅ |
| FIX-5 | `#[derive(Copy)]` para enums unitarios (QF-1) | BinOp, Visibility, etc. dejan de dar move errors | ✅ |
| FIX-6 | Borrar `IrCodeGenerator` dead code (~3.700 líneas) | Limpieza, menos confusión | ✅ |

**Criterio de salida:** Los 520 tests actuales siguen verdes + los 4 módulos del
self-hosting (token/ast/lexer/parser) se pueden reescribir sin workarounds.

### Fase 1: Frontend en Liva ✅ COMPLETADA

| Módulo | Líneas | Estado | Notas |
|--------|--------|--------|-------|
| `token.liva` | 312 | ✅ Idiomatic | TokenKind enum, Token class, lookupKeyword, tokenKindName |
| `ast.liva` | 450 | ✅ Idiomatic | Expr/Stmt/TypeRef/Pattern enums, data classes, helper fns |
| `lexer.liva` | 610 | ✅ Idiomatic | Hand-rolled scanner, todas las token types |
| `parser.liva` | 2254 | ✅ Idiomatic | Recursive descent completo |

**Total:** 3626 líneas de Liva → Rust sin errores (era 3765, −139 tras rewrite idiomático).

**Rewrite idiomático completado:** Los 4 módulos siguen `docs/guides/style-guide.md`:
`+=` compound assignments, `if X =>` one-liners, `=> expr` one-liner functions,
comentarios WHY-not-WHAT. Todos compilan a Rust sin errores.

### Auditoría de estilo (Fase 1 — ✅ Completada)

Los 4 módulos han sido reescritos idiomáticamente:

| Violación original | Corregidas | Estado |
|-----------|------------|--------|
| `if X { single_stmt }` → `if X => single_stmt` | **75** (57 parser + 13 lexer + 5 token/ast) | ✅ Hecho |
| `x = x + y` → `x += y` | **102** (53 parser + 41 lexer + 8 token/ast) | ✅ Hecho |
| `{ return expr }` → `=> expr` one-liner fns | **7** (1 parser + 5 lexer + 1 token) | ✅ Hecho |
| Comentarios WHAT-not-WHY eliminados | ~20 | ✅ Hecho |

**Líneas eliminadas:** 3765 → 3626 (−139 líneas)

**Pendiente para futuro (no bloquea Fase 2):**
- `pub` en métodos de API pública (requiere feature no implementada aún)
- Métodos >30 líneas (12 parser + 3 lexer — justificados por complejidad inherente)
- `or fail` / `or default` (no aplica en parser — no hay errores recuperables)
- Destructuring (no hay oportunidades naturales en parser/lexer)

### Fase 2: Análisis Semántico — EL CAMBIO GRANDE

> **Objetivo:** Anotar cada nodo del AST con su tipo resuelto.
> **Estimación:** 5-8 sesiones
> **Dependencia:** Fase 0 completada (necesitamos switch y params arreglados)
> **Estado:** 2.1 completada ✅

Esta es la pieza que el compilador Rust NO tiene y por la que el codegen es un
monstruo de 19K líneas. El semantic analyzer nuevo produce un `TypeContext`:

```liva
TypeContext {
    // Tipo de cada variable en cada scope
    varTypes: Map<string, TypeRef>
    
    // Tipo resuelto de cada expresión (indexado por posición)
    exprTypes: Map<(number, number), TypeRef>
    
    // Para cada función: si cada parámetro necesita owned o borrow
    paramModes: Map<string, [ParamMode]>
}

enum ParamMode { Owned, Borrow }
```

**Subfases:**

| # | Componente | Descripción |
|---|-----------|-------------|
| 2.1 | **Scope tracker** ✅ | Variables declaradas, sus tipos, scope enter/leave |
| 2.2 | **Type resolver** ✅ | `TypeRef.Simple("string")` → tipo concreto con toda su info |
| 2.3 | **Expr typing** ✅ | Cada `Expr` recibe su tipo: `x.length` → `int` (sabemos que `x: string`) |
| 2.4 | **Function signatures** ✅ | Return types, param types, fallibility, async |
| 2.5 | **Class/Enum metadata** ✅ | Fields con tipos, variant fields, methods |
| 2.6 | **Import resolution** ✅ | Tipos de símbolos importados de otros módulos |
| 2.7 | **Liveness analysis** | Último uso → move, no-último → borrow/clone |

#### Fase 2.1 — Completada ✅ (2026-03-31)

**Módulo:** `compiler/src/semantic.liva` (647 líneas)

**Qué incluye:**
- Tipos: `Symbol`, `SymbolKind` (enum), `FunctionSig`, `ParamSig`, `ClassInfo`, `FieldInfo`,
  `EnumInfo`, `VariantInfo`, `EnumFieldInfo`, `TypeAliasInfo`, `Diagnostic`, `DiagnosticLevel` (enum)
- Output: `TypeContext` (scopes + functions + classes + enums + typeAliases + diagnostics)
- `SemanticAnalyzer` class:
  - Flat symbol table (`"scopeId:name"` → Symbol) — evita acceso anidado a Maps
  - Scope management (`_enterScope`/`_leaveScope` con `_scopeParents: [number]`)
  - Registration pass (recolecta declaraciones top-level: funciones, clases, enums, type aliases, constantes)
  - Analysis pass (recorre AST declarando variables en sus scopes)
  - Helpers: `_analyzeBlockOpt(BlockStmt?)`, `_analyzeIfBodyOpt(IfBody?)`, `_declareVarOpt(string?)`
- Factory functions (`makeParamSig`, `makeFunctionSig`, `makeFieldInfo`)
  — evitan double Some() wrapping en constructores

**Bootstrap codegen fixes (3 parches, 518 tests verdes):**
- **SH-011**: `collect_mutated_vars_in_expr` ahora recorre `Expr::Switch` arms
- **SH-012**: `init_is_already_optional()` handler para `Expr::Member` (detecta campos Optional)
- **SH-013**: For-loop `var_types` tracking (loop vars registradas en `var_types`)

**Workarounds documentados (limitaciones del bootstrap):**
- Factory functions para pasar Optional values a constructores (evita double `Some()`)
- `if x != null` genera `if let Some(x) = x` (auto-unwrap, no necesita `or` keyword)
- `let thisType: TypeRef? = TypeRef.Simple(...)` para pasar a params `TypeRef?`
- `_ => { 0 }` en todos los arms de switch expression (consistencia de tipos)
- Single-char loop vars para structs importados (heurística `len==1`)

#### Fase 2.2 — Completada ✅ (2026-04-01)

**Módulo:** `compiler/src/semantic.liva` (1212 líneas, +564 desde Phase 2.1)

**Qué incluye:**
- **Type pool**: `_typePool: [TypeRef]` + `_varTypeIdx: Map<string, number>` — almacena tipos resueltos de variables
- **Type resolver**: `resolveTypeRef(t: TypeRef): TypeRef` — sigue aliases, resuelve recursivamente todos los 9 variantes de TypeRef
- **Expression type inference**: `inferExprType(expr: Expr): TypeRef` — infiere tipos de todas las variantes de Expr:
  - Literales (Int→number, Float→float, Str→string, Char→char, Bool→bool, Null→null)
  - Identificadores (lookup en scope chain via `_varTypeIdx`)
  - Colecciones: ArrayLiteral, MapLiteral, SetLiteral, TupleLiteral
  - Operadores: Binary (Add→infer, comparisons→bool, arithmetic→number), Unary (Neg→number, Not→bool)
  - Calls (constructor→ClassName, function→return type), MethodCall, MemberAccess
  - Lambda→fn, Ternary→then branch, StructLiteral→name, Unwrap→inner, OptionalChain→Optional
- **String/Array method type tables**: 15 string method types + 15 array method types
- **For-loop type inference**: `_inferIterableElemType` — Array→inner, Set→inner, Map→Tuple(k,v), string→char, range→number
- **Type utilities**: `_typeToString(TypeRef): string`, `typesEqual(a, b): bool`, `isUnknownType(t): bool`
- **Variable type storage**: `_setVarType(name, TypeRef)` → pool index; `lookupVarType(name): TypeRef` → walks scope chain

**New workarounds (limitaciones del bootstrap):**
- NO `return` dentro de switch arm blocks: el parser Liva no lo soporta. Usar patrón de variable mutable:
  `let result = ...; let _ = switch t { Arm => { result = ...; 0 } }; return result`
- Variables con nombres únicos por switch arm: Liva no crea scopes separados por arm
  (e.g., `tupleResolved`, `genResolved`, `unionResolved` en vez de reutilizar `resolved`)
- String template trick para clonar strings de struct fields: `$"{stmt.variable}"`
  genera `format!()` que borrowea en vez de mover
- No pre-resolver aliases ni return types en registration (causa double-move).
  Dejar lookups lazy para Phase 2.3+

#### Fase 2.3 — Completada ✅ (2026-04-01)

**Módulo:** `compiler/src/semantic.liva` (1328 líneas, +116 desde Phase 2.2)

**Qué incluye:**
- **Type index maps**: `_funcRetTypeIdx`, `_fieldTypeIdx`, `_methodRetTypeIdx` (Map<string, number>)
  — indexes de pool por función, campo y método para lookup O(1)
- **Second indexing pass**: `_indexTypeInfo(program)` — recorre items después de registration
  para poblar los index maps antes del analysis pass
- **Index helpers**: `_indexFuncRetType`, `_indexClassTypes`, `_indexFieldType`, `_indexMethodRetType`
- **Lookup methods filled**: `lookupFuncReturnType`, `_lookupMethodReturnType`, `_lookupFieldType`
  — ahora usan los index maps (antes eran stubs devolviendo "unknown")
- **Expression analysis**: `_analyzeExpr(expr)` — recorre expresiones via `inferExprType`
  durante el analysis pass para ejercitar el type resolver
- **Statement analysis enhancements**: `_analyzeStmt` ahora maneja Assign, Switch, ExprStmt, Return, Throw, Fail
- **Control flow analysis**: `_analyzeIf` analiza condición, `_analyzeWhile` analiza condición
- **Helper methods**: `_analyzeReturnOpt`, `_analyzeAssign`, `_analyzeSwitch`
- **Factory function**: `_addTypeOpt(optRef: TypeRef?)` — rutas Optional values por param auto-unwrap
- **TypeContext enriched**: `funcRetTypes`, `fieldTypes`, `methodRetTypes` fields

**New workaround (limitación del bootstrap):**
- W-005: `option_value_vars` leaks across class methods in codegen. Si un param se llama `t: TypeRef?`,
  todos los `for t in ...` en otros métodos del mismo class generan `.as_ref().unwrap()` incorrecto.
  **Fix:** Usar nombres únicos para params Optional (e.g., `optRef` en vez de `t`).

### Fase 3: Codegen Limpio

> **Objetivo:** Generar Rust mecánicamente desde AST + TypeContext.
> **Estimación:** 4-6 sesiones
> **Dependencia:** Fase 2 completada

Con los tipos resueltos, codegen no necesita adivinar nada:

```liva
// Codegen nuevo: mecánico
fn generateMemberAccess(obj: Expr, prop: string, ctx: TypeContext) {
    let objType = ctx.getType(obj)
    
    switch objType {
        TypeRef.Simple("string") => {
            if prop == "length" { emit(".len()") }
            // ...
        }
        TypeRef.Array(_) => {
            if prop == "length" { emit(".len()") }
            // ...
        }
        _ => emit($".{prop}")  // campo directo
    }
}
```

**Sin HashSets. Sin "person" hacks. Sin adivinanzas.**

**Subfases:**

| # | Componente | Descripción |
|---|-----------|-------------|
| 3.1 | **Infraestructura** | RustEmitter class, indent management, module output |
| 3.2 | **Declaraciones** | Funciones, clases, enums, type aliases, imports |
| 3.3 | **Statements** | VarDecl (con tipos!), if/for/while/switch, assign |
| 3.4 | **Expressions** | Literals, binary ops, calls, member access, switch expr |
| 3.5 | **Stdlib mapping** | String/Array/Map methods → Rust equivalents |
| 3.6 | **Ownership emission** | Usar liveness info para `&x`, `x`, `x.clone()` |
| 3.7 | **Cargo.toml generation** | Dependencies según features usadas |

### Fase 4: Main + CLI + Bootstrap

> **Objetivo:** El compilador Liva se compila a sí mismo.
> **Estimación:** 1-2 sesiones

| # | Componente | Descripción |
|---|-----------|-------------|
| 4.1 | **main.liva** | CLI args parsing, subcommands |
| 4.2 | **Module resolver** | Imports, file discovery, compilation order |
| 4.3 | **Bootstrap test** | `livac build compiler.liva` → `compiler` → `compiler build compiler.liva` → mismo output |

---

## Estrategia de Testing

### Estado actual de los tests (auditoría)

| Fichero | Tests | Qué valida | Calidad |
|---------|-------|------------|---------|
| `codegen_tests.rs` | 568 | Liva source → snapshot del Rust generado | ⚠️ **Frágil:** cada snapshot incluye ~200 líneas de `liva_rt` runtime. Un cambio en el runtime rompe TODOS los snapshots. Testan output textual, no comportamiento. |
| `parser_tests.rs` | 58 | Source → AST snapshot | ✅ Buenos. Validan estructura del AST. |
| `semantics_tests.rs` | 40 | Source → errores semánticos esperados | ✅ Buenos. Validan detección de errores. |
| `lexer_tests.rs` | 20 | Source → token snapshot | ✅ Buenos. |
| `linter_tests.rs` | 24 | Source → warnings esperados | ✅ Buenos. |
| `integration_tests.rs` | 21 | Compilar proyectos `.liva` end-to-end | ⚠️ **No compilan el Rust generado** por defecto (requiere `LIVA_RUN_CARGO_CHECK=1`) |
| `codegen_ir_tests.rs` | 6 | IR-based codegen (¡dead code path!) | ❌ **Irrelevante** — testan el `IrCodeGenerator` que no se usa |
| `property_tests.rs` | 12 | Proptest: fuzzing del pipeline | ✅ Buenos pero pocos. |
| `destructuring_parser_tests.rs` | 12 | Parser destructuring | ✅ Buenos. |
| `generics_parser_tests.rs` | 23 | Parser generics | ✅ Buenos. |
| `desugar_tests.rs` | 13 | Desugaring snapshots | ✅ Buenos. |
| `http_tests.rs` | 10 | HTTP codegen | ⚠️ Solo validan que compila, no que funciona. |
| **Total** | **520** | | |

### Problemas de los tests actuales

1. **Los codegen tests (568) son snapshot tests del texto Rust generado.** Si cambias
   una coma en el runtime `liva_rt`, se rompen todos. No validan que el Rust **compile**
   ni que **funcione** — solo que sea textualmente idéntico al snapshot.

2. **Los integration tests no compilan el Rust por defecto.** Generan archivos y
   verifican que existen, pero no ejecutan `cargo check`. La validación real está
   desactivada (`LIVA_RUN_CARGO_CHECK=1`).

3. **No hay tests de comportamiento.** No hay ningún test que diga "este programa
   Liva, al ejecutarse, imprime X". Todo es "genera este texto Rust".

4. **Los `codegen_ir_tests` testan dead code.** El `IrCodeGenerator` no se usa.

### Estrategia de testing para el compilador nuevo

#### Nivel 1: Tests de pipeline por fase

```
lexer_tests:     source → [Token]              (ya existen, buenos)
parser_tests:    [Token] → AST                  (ya existen, buenos)
semantic_tests:  AST → TypeContext + errores     (NUEVO — hay que crear)
codegen_tests:   AST + TypeContext → Rust code   (NUEVO — reemplaza los actuales)
```

#### Nivel 2: Tests de compilación (el Rust generado compila)

```
Para cada programa .liva de test:
  1. livac compila → genera Rust
  2. cargo check → el Rust generado es válido
  3. ✅ si compila, ❌ si no

Esto reemplaza los snapshot tests frágiles con validación real.
```

#### Nivel 3: Tests de comportamiento (el programa hace lo que debe)

```
Para cada programa .liva de test que tenga main():
  1. livac compila → genera Rust
  2. cargo build → binario
  3. Ejecuta binario → captura stdout
  4. Compara stdout con .expected
  5. ✅ si match, ❌ si no
```

**Ejemplo:**
```
tests/
  behavior/
    hello.liva          → "Hello World!\n"
    hello.expected
    fibonacci.liva      → "0 1 1 2 3 5 8 13 21 34\n"
    fibonacci.expected
    enum_match.liva     → "Circle: r=5\n"
    enum_match.expected
```

#### Nivel 4: Test de bootstrap (self-hosting)

```
1. livac-rust compila compiler.liva → compiler-v1 (binario Rust)
2. compiler-v1 compila compiler.liva → compiler-v2
3. diff compiler-v1-output compiler-v2-output → debe ser idéntico
4. ✅ si idéntico = el compilador se reproduce a sí mismo
```

### Qué tests se mantienen, cuáles se borran, cuáles se crean

| Acción | Tests | Motivo |
|--------|-------|--------|
| **Mantener** | lexer_tests (20) | Buenos, validan tokenización |
| **Mantener** | parser_tests (58) + destructuring (12) + generics (23) | Buenos, validan AST |
| **Mantener** | semantics_tests (40) | Buenos, validan errores |
| **Mantener** | linter_tests (24) | Buenos, validan warnings |
| **Mantener** | property_tests (12) | Buenos, fuzzing |
| **Mantener** | desugar_tests (13) | Buenos |
| **Refactor** | integration_tests (21) | Activar `cargo check` por defecto |
| **Gradual** | codegen_tests (568) | Migrar de snapshot → compilación real |
| **Borrado** | ~~codegen_ir_tests (6)~~ | ✅ Eliminado en FIX-6 junto con ir.rs + lowering.rs |
| **Crear** | semantic_typing_tests | TypeContext validation |
| **Crear** | behavior_tests | Ejecución end-to-end |
| **Crear** | bootstrap_test | Self-hosting validation |

---

## Checklist de hitos

```
Fase 0: Fix Bootstrap ✅
  [x] FIX-1: let x: T? = value → Some(value)
  [x] FIX-2: Enum reassignment sin Some() espurio (could not reproduce)
  [x] FIX-3: switch genera match &expr cuando corresponde
  [x] FIX-4: Params no-Copy por referencia (clone at call site)
  [x] FIX-5: #[derive(Copy)] para enums unitarios
  [x] FIX-6: Borrar IrCodeGenerator dead code (~4.400 líneas)
  [x] Fix: Boxed bindings in match-by-reference (*b.clone())
  [x] Tests: 518 tests verdes
  [x] Reescribir 4 módulos idiomáticamente (style-guide)
       - if => one-liners ✅ (75 convertidos)
       - += compound assignment ✅ (102 convertidos)
       - => one-liner functions ✅ (7 convertidos)
       - Comentarios WHY-not-WHAT ✅

Fase 1: Frontend ✅ (idiomatic rewrite done)
  [x] token.liva — 312 líneas, idiomatic
  [x] ast.liva — 450 líneas, idiomatic
  [x] lexer.liva — 610 líneas, idiomatic
  [x] parser.liva — 2254 líneas, idiomatic

Fase 2: Semantic Analyzer
  [x] 2.1: TypeContext struct + scope tracker (semantic.liva — 647 líneas)
  [x] 2.2: Type resolver (Simple/Array/Map/Optional → info concreta)
  [x] 2.3: Expr typing (cada expresión anotada con su tipo)
  [x] 2.4: Function signatures registry
  [x] 2.5: Class/Enum metadata registry
  [x] 2.6: Import resolution (tipos de otros módulos)
  [ ] 2.7: Liveness analysis (move/borrow/clone)
  [ ] Tests: semantic_typing_tests

Fase 3: Codegen
  [ ] 3.1: RustEmitter infraestructura
  [ ] 3.2: Declaraciones (fn, class, enum)
  [ ] 3.3: Statements (let, if, for, switch, assign)
  [ ] 3.4: Expressions (literal, binary, call, member, switch expr)
  [ ] 3.5: Stdlib mapping (String/Array/Map methods)
  [ ] 3.6: Ownership emission (borrow/clone basado en liveness)
  [ ] 3.7: Cargo.toml generation
  [ ] Tests: behavior_tests para cada feature

Fase 4: Bootstrap
  [ ] 4.1: main.liva (CLI)
  [ ] 4.2: Module resolver
  [ ] 4.3: Bootstrap test (compiler compila compiler → mismo output)
```

---

## Regla: Todo código Liva sigue la documentación

> **OBLIGATORIO:** Todo código del self-hosting DEBE seguir `docs/guides/style-guide.md`.
> Antes de escribir cualquier módulo nuevo, leer:
> 1. `docs/guides/style-guide.md` — convenciones idiomáticas
> 2. `docs/QUICK_REFERENCE.md` — features del lenguaje con gotchas
> 3. `skills/liva-lang/SKILL.md` — reglas críticas y anti-patterns

El código debe ser **ejemplo de Liva idiomático**. Si el compilador no soporta
una feature idiomática, documentarlo en ISSUES.md y usar workaround mínimo.

---

## Documentos relacionados

| Documento | Qué contiene |
|-----------|-------------|
| `ISSUES.md` | 11 bugs + 6 feature requests del self-hosting |
| `docs/plans/PLAN_LIVENESS_ANALYSIS.md` | Diseño técnico del liveness analysis |
| `docs/guides/style-guide.md` | Guía de estilos idiomáticos de Liva |
| `docs/QUICK_REFERENCE.md` | Referencia rápida con gotchas y features |

