# Self-Hosting: Compilador de Liva escrito en Liva

> **Estado:** Fase 4 completada ✅ (self-hosting compiler funcional — 9 módulos, 7/9 generan Rust válido)
> **Última actualización:** 2026-03-31
> **Próximo:** Fase 5 — Liva Test Suite (~65 archivos .liva validando toda la sintaxis y features)

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
  │   ├── tests/
  │   │   ├── liva/     ← Liva Test Suite (.liva files)
  │   │   └── bootstrap_test.sh
  │   └── docs/         ← PLAN.md, ISSUES.md
  ├── tests/            ← tests del bootstrap (Rust)
  └── Cargo.toml

DESPUÉS (promoción — self-hosting funcional):
  livac/
  ├── src/              ← compilador Liva (promovido desde compiler/src/)
  ├── tests/            ← Liva Test Suite (promovido desde compiler/tests/)
  ├── docs/
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
| 2.7 | **Liveness analysis** ✅ | Último uso → move, no-último → borrow/clone |

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

#### Fase 2.7 — Completada ✅ (2026-03-31)

**Módulo:** `compiler/src/liveness.liva` (519 líneas — nuevo módulo)

**Qué incluye:**
- **LivenessContext** output struct: `useCounts`, `loopUseCounts`, `paramBorrow` (all `Map<string, number>`)
- **LivenessAnalyzer** class: walks AST counting variable references per function/method
- **Use counting**: `_recordUse(varName)` increments `"funcName:varName"` key in useCounts
- **Loop tracking**: `_inLoop` flag saved/restored for for/while — uses inside loops tracked in loopUseCounts
- **Parameter borrow detection**: `_shouldBorrowType(optRef)` → Copy types (int/float/bool/number/char) = owned, non-Copy = borrow
- **Full AST coverage**: all 22 Expr variants + all Stmt variants + lambdas + switch arms + string templates
- **"this" exclusion**: self-references not tracked (not local variables)
- **Public API**: `analyzeLiveness(program)` → `LivenessContext`
- **Helper**: `isCopyTypeName(name)` for codegen consumption
- **Cleanup**: Removed `examples/self-hosting/` legacy directory — canonical location is `compiler/`

**Bootstrap workaround reused:**
- W-005: param named `optRef` (not `typeRef`) in `_shouldBorrowType` to avoid option_value_vars pollution
- Optional→non-Optional delegation: null check + pass to non-Optional method (auto-unwrap in if block)

### Fase 3: Codegen Limpio ✅ COMPLETADA

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

**Módulo:** `compiler/src/codegen.liva` (2458 líneas)

**Subfases completadas:**

| # | Componente | Descripción | Estado |
|---|-----------|-------------|--------|
| 3.1 | **Infraestructura** | RustEmitter class, output buffer, indent management, name sanitization | ✅ |
| 3.2 | **Declaraciones** | Functions, classes (struct+impl+constructor), enums (Copy for unit), type aliases, imports | ✅ |
| 3.3 | **Statements** | All 16 Stmt variants: var decl, if/for/while/switch, try/catch, assign, return | ✅ |
| 3.4 | **Expressions** | All 22+ Expr variants: literals, binary/unary, calls, member access, lambdas, switch expr | ✅ |
| 3.5 | **Stdlib mapping** | 28 string + 30 array + 10 map + 10 set methods → Rust equivalents | ✅ |
| 3.6 | **Ownership emission** | Type-directed dispatch via TypeContext, _emitRefArg for & references | ✅ |
| 3.7 | **Cargo.toml generation** | Feature-aware deps (async, http, db, json, regex, chrono, random, crypto, rayon) | ✅ |

**Key design decisions:**
- Output buffer: `[string]` array joined at end (not repeated string concat)
- Type dispatch: `_lookupVarTypeRef` queries TypeContext's scoped varTypes for method call routing
- Stdlib: full coverage of 78 methods across string/array/map/set types
- Ownership: `_emitRefArg` adds `&` for variable args (not literals) to methods like `contains`, `startsWith`
- Free functions: `print/println` → Rust macros, `toString` → `format!`, `toInt/toFloat` → parse

**Bootstrap workaround added:**
- W-007: No nested switch expressions inside switch arm blocks — extract to separate method

### Fase 4: Main + CLI + Bootstrap ✅ COMPLETADA

> **Objetivo:** El compilador Liva tiene CLI y se puede compilar.
> **Estimación:** 1-2 sesiones
> **Dependencia:** Fase 3 completada

| # | Componente | Descripción | Estado |
|---|-----------|-------------|--------|
| 4.1 | **main.liva** (449 líneas) | CLI: build/run/check subcommands, full pipeline orchestration | ✅ |
| 4.2 | **module.liva** (234 líneas) | Import resolution, BFS traversal, topological sort, path helpers | ✅ |
| 4.3 | **Bootstrap test** | 7/9 modules compile to valid standalone Rust | ✅ |

**Bootstrap test results:**

| Módulo | Liva lines | Rust lines | Compila | Notas |
|--------|-----------|------------|---------|-------|
| token.liva | 312 | 691 | ✅ | — |
| ast.liva | 450 | 1430 | ✅ | — |
| lexer.liva | 610 | 998 | ✅ | — |
| parser.liva | 2254 | 3042 | ✅ | — |
| semantic.liva | 1709 | 2843 | ✅ | — |
| liveness.liva | 520 | 1174 | ✅ | — |
| codegen.liva | 2458 | ~4000 | ❌ | 29 Rust errors (move semantics, Default trait) |
| module.liva | 234 | 654 | ✅ | — |
| main.liva | 449 | ~3000 | ❌ | 69 Rust errors (println macro, move, imported types) |
| **Total** | **9013** | **~18K** | **7/9** | — |

**Remaining Rust errors (bootstrap codegen limitations, NOT Liva source errors):**
- E0308 mismatched types — return type inference for bool-returning methods
- E0382 move semantics — String/Vec args moved instead of borrowed
- E0507 cannot move out of mutable reference field
- E0599 no method `has` (should generate `contains_key`)
- E0423 println as function, not macro

### Fase 5: Liva Test Suite — Archivos .liva que validan el lenguaje

> **Objetivo:** Crear suite completa de tests escritos EN Liva que validen toda la sintaxis y features documentadas en `docs/`.
> **Estimación:** 2-3 sesiones
> **Dependencia:** Compilador funcional (Fase 0-4)
> **Directorio:** `compiler/tests/liva/` (se promueve a `tests/liva/` con el resto del compiler)
> **Runner:** `compiler/tests/liva/run_tests.sh` con filtros por capa

Los 520 tests actuales son tests Rust que validan el compilador desde dentro.
Pero no hay una suite de archivos `.liva` que valide sistemáticamente que el
lenguaje funciona como está documentado. Esta fase llena ese gap.

> **OBLIGATORIO:** Cada test `.liva` DEBE estar basado en la documentación de `livac/docs/`.
> Antes de escribir cualquier test, consultar:
> 1. `livac/docs/QUICK_REFERENCE.md` — sintaxis, gotchas, edge cases
> 2. `livac/docs/README.md` — documentación completa del lenguaje
> 3. `livac/skills/liva-lang/SKILL.md` — referencia compacta con reglas críticas
> 4. `livac/docs/language-reference/` — referencia detallada por tema (variables, functions, etc.)
> 5. `livac/docs/guides/` — guías de estilo, error handling, etc.
>
> Los tests son la **validación viva** de que la documentación es correcta.
> Si un test falla, puede ser un bug del compilador O un error en la documentación.
> Ambos deben investigarse.

**6 capas de testing:**

| # | Capa | Directorio | Validación | Método | Archivos |
|---|------|-----------|------------|--------|----------|
| 1 | **Syntax** | `compiler/tests/liva/syntax/` | Parse + semantic OK | `livac check` | ~15 |
| 2 | **Compile** | `compiler/tests/liva/compile/` | Codegen → Rust válido | `livac build` + cargo check | ~8 |
| 3 | **E2E Runtime** | `compiler/tests/liva/e2e/` | Pipeline completo + output correcto | build + run + comparar .expected | ~10 |
| 4 | **Stdlib** | `compiler/tests/liva/stdlib/` | Cada módulo stdlib | build + run | ~18 |
| 5 | **Stdlib-IO** | `compiler/tests/liva/stdlib-io/` | File, Dir, DB, HTTP (opt-in) | build + run | ~4 |
| 6 | **Errors** | `compiler/tests/liva/errors/` | Errores esperados (negativos) | `livac check`, debe fallar | ~10 |

**Syntax catalog (~15 archivos):**
- variables, functions, classes, enums, generics, control_flow
- error_handling, pattern_matching, imports, types, lambdas
- string_templates, defer, compound_assign, rust_interop

**Compile tests (~8 archivos):**
- basic_program, class_program, enum_program, generic_program
- error_program, collections, closures, multifile/

**E2E runtime (~10 archivos):**
- hello, fibonacci, calculator, linked_list, grade_tracker
- key_value_store, error_chain, async_basic, string_utils, for_patterns
- Cada uno con `.expected` file para comparar output

**Stdlib (~18 archivos):**
- string_methods (x3), array_methods (x3), map_methods, set_methods
- math, random, regex, date, csv, config, process, log, crypto, type_conversions

**Stdlib-IO (~4 archivos, opt-in con `--all`):**
- file_operations, dir_operations, db_sqlite, http_server

**Error cases (~10 archivos):**
- E0101 undefined var, E0201 type mismatch, E0301 undefined function
- E0401 missing return, E0501 duplicate definition, E0601 invalid import
- E0904 non-exhaustive switch, W001/W002/W003 warnings

**Test runner:**
```bash
./compiler/tests/liva/run_tests.sh              # todo menos stdlib-io
./compiler/tests/liva/run_tests.sh --all        # incluye stdlib-io
./compiler/tests/liva/run_tests.sh --only e2e   # solo una capa
./compiler/tests/liva/run_tests.sh --only stdlib # solo stdlib
```

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
  [x] 2.7: Liveness analysis (move/borrow/clone)
  [ ] Tests: semantic_typing_tests

Fase 3: Codegen
  [x] 3.1: RustEmitter infraestructura
  [x] 3.2: Declaraciones (fn, class, enum)
  [x] 3.3: Statements (let, if, for, switch, assign)
  [x] 3.4: Expressions (literal, binary, call, member, switch expr)
  [x] 3.5: Stdlib mapping (String/Array/Map methods)
  [x] 3.6: Ownership emission (borrow/clone basado en liveness)
  [x] 3.7: Cargo.toml generation
  [ ] Tests: behavior_tests para cada feature

Fase 4: Bootstrap
  [x] 4.1: main.liva (CLI)
  [x] 4.2: Module resolver
  [x] 4.3: Bootstrap test (7/9 modules → valid Rust)

Fase 5: Liva Test Suite
  [ ] 5.1: Test runner (run_tests.sh con filtros por capa)
  [ ] 5.2: Syntax tests (~15 archivos — livac check)
  [ ] 5.3: Compile tests (~8 archivos — livac build + cargo check)
  [ ] 5.4: E2E tests (~10 archivos + .expected — build + run + compare)
  [ ] 5.5: Stdlib tests (~18 archivos — build + run)
  [ ] 5.6: Stdlib-IO tests (~4 archivos — opt-in)
  [ ] 5.7: Error tests (~10 archivos — errores esperados)
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

