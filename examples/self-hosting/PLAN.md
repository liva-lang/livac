# Self-Hosting: Compilador de Liva escrito en Liva

> **Estado:** Fase 0 completada ✅ (FIX-1 ✅, FIX-2 ✅, FIX-3 ✅, FIX-4 ✅, FIX-5 ✅, FIX-6 ✅)
> **Última actualización:** 2026-03-31

---

## Objetivo

Reescribir el compilador `livac` en Liva. No es un port 1:1 del compilador Rust —
es un **rediseño** que corrige los errores arquitectónicos del compilador actual.

El compilador Rust actual (`src/`) se convierte en **bootstrap compiler**: solo existe
para compilar el compilador Liva la primera vez. Después, el compilador Liva se
compila a sí mismo.

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
| `token.liva` | ~315 | ✅ Build OK | TokenKind enum, Token class, lookupKeyword, tokenKindName |
| `ast.liva` | ~455 | ✅ Build OK | Expr/Stmt/TypeRef/Pattern enums, data classes, helper fns |
| `lexer.liva` | ~655 | ✅ Build OK | Hand-rolled scanner, todas las token types |
| `parser.liva` | ~2340 | ✅ Build OK | Recursive descent completo |

**Total:** ~3765 líneas de Liva → Rust sin errores.

**Después de Fase 0:** Reescribir los 4 módulos sin workarounds (~60 hacks eliminados)
siguiendo estrictamente `docs/guides/style-guide.md` y usando todas las capacidades
del lenguaje documentadas en `docs/QUICK_REFERENCE.md`.

### Auditoría de estilo (Fase 1 actual)

El código de la Fase 1 **no sigue la guía de estilos** ni aprovecha las capacidades
del lenguaje. Sirve como referencia funcional (compila, cubre la gramática completa),
pero la reescritura post-Fase 0 debe corregir todo esto:

| Violación | Instancias | Regla del style guide |
|-----------|------------|----------------------|
| `if X { single_stmt }` en vez de `if X => single_stmt` | **79** (64 parser + 15 lexer) | §1: One-liner `=>` vs Block `{}` |
| `x = x + y` en vez de `x += y` | **19** (11 parser + 8 lexer) | Compound assignment |
| Sin `pub` en métodos de API pública | **todas las clases** | §2: Naming — visibilidad explícita |
| Métodos de >30 líneas | **15** (12 parser + 3 lexer) | §4: ~20-30 líneas por función |
| Sin destructuring | **0 usos** | §7: Destructuring donde aplique |
| Sin `or fail` / `or default` | **0 usos** | §3: Error handling patterns |

**Capacidades del lenguaje NO aprovechadas:**

| Feature | Documentado en | Usado | Oportunidad |
|---------|---------------|-------|-------------|
| `if X =>` one-liner | style-guide §1 | ❌ | 79 if-blocks de una sola sentencia |
| `+=` compound assign | QUICK_REFERENCE | ❌ | 19 `x = x + y` |
| Point-free refs | style-guide §1 | ❌ | Sin callbacks en parser, N/A |
| `$"..."` templates | style-guide §10 | ✅ (2 usos) | Correcto en error messages |
| `not` keyword | SKILL.md | ✅ (~25 usos) | Bien usado |
| Section headers | style-guide §9 | ✅ | Buenos separadores `═══` y `───` |

### Fase 2: Análisis Semántico — EL CAMBIO GRANDE

> **Objetivo:** Anotar cada nodo del AST con su tipo resuelto.
> **Estimación:** 5-8 sesiones
> **Dependencia:** Fase 0 completada (necesitamos switch y params arreglados)

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
| 2.1 | **Scope tracker** | Variables declaradas, sus tipos, scope enter/leave |
| 2.2 | **Type resolver** | `TypeRef.Simple("string")` → tipo concreto con toda su info |
| 2.3 | **Expr typing** | Cada `Expr` recibe su tipo: `x.length` → `int` (sabemos que `x: string`) |
| 2.4 | **Function signatures** | Return types, param types, fallibility, async |
| 2.5 | **Class/Enum metadata** | Fields con tipos, variant fields, methods |
| 2.6 | **Import resolution** | Tipos de símbolos importados de otros módulos |
| 2.7 | **Liveness analysis** | Último uso → move, no-último → borrow/clone |

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
Fase 0: Fix Bootstrap
  [ ] FIX-1: let x: T? = value → Some(value)
  [ ] FIX-2: Enum reassignment sin Some() espurio
  [ ] FIX-3: switch genera match &expr cuando corresponde
  [ ] FIX-4: Params no-Copy por referencia &T
  [x] FIX-5: #[derive(Copy)] para enums unitarios
  [x] FIX-6: Borrar IrCodeGenerator dead code (~2.730 líneas codegen + 416 ir.rs + 994 lowering.rs + 185 tests)
  [ ] Tests: 515 tests verdes tras cada fix (era 520, -6 IR tests eliminados +1)
  [ ] Reescribir 4 módulos sin workarounds, siguiendo docs/guides/style-guide.md
       - if => one-liners (no bloques de 1 sentencia)
       - += compound assignment
       - pub en API pública
       - Funciones ≤30 líneas
       - $"..." templates siempre (nunca concatenación)
       - Nombres booleanos con is/has/can prefix

Fase 1: Frontend ✅
  [x] token.liva compila
  [x] ast.liva compila
  [x] lexer.liva compila
  [x] parser.liva compila

Fase 2: Semantic Analyzer
  [ ] 2.1: TypeContext struct + scope tracker
  [ ] 2.2: Type resolver (Simple/Array/Map/Optional → info concreta)
  [ ] 2.3: Expr typing (cada expresión anotada con su tipo)
  [ ] 2.4: Function signatures registry
  [ ] 2.5: Class/Enum metadata registry
  [ ] 2.6: Import resolution (tipos de otros módulos)
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

