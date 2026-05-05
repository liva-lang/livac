# Bootstrap → Gen-2 Parity Inventory

> **Source:** `livac/src/*.rs` (CONGELADO post-`ba7f263`).
> **Target:** `livac/compiler/src/*.liva` (gen-2, escrito en Liva).
> **Goal:** v2.1 — gen-2 cubre 100 % del bootstrap → eliminar `livac/src/*.rs` salvo `liva_rt`.
>
 > **Status (2026-05-05, PLAN.md A.1 audit):** ✅ **21/21 bootstrap_apps pasan con gen-2** (re-verificado).
> Los items marcados ⏳ Tier 1 abajo están **cerrados por outcome**: sus tests asociados
> pasan en `compiler/tests/bootstrap_apps/run_gen2.sh`. Una auditoría línea-a-línea queda
> como deuda menor; no bloquea v2.0.
>
> ⚠️ **Hallazgo nuevo durante la auditoría F.4:** `arr[i].mutMethod()` sobre clases
> de usuario emite `.clone().method()` y pierde la mutación. Ver `BUGS.md § B157`.
> Este bug **es bloqueante** para benchmarks fiables y para casos reales de uso.
> ✅ **FIXED (2026-05-05, commit `3463ce5`):** `_suppressIndexElemClone` flag en ambos
> compiladores; particle sim checksum coincide con Rust.
>
> **Métrica viva:** `bash compiler/tests/run_all.sh` (incluye este gate).
>
> **Pendiente real para v2.1** (no cubierto aún por bootstrap_apps):
> - HTTP routing en self-host (axum + async closures) — ver `BACKLOG § 9.4`.
> - Stdlib emission con tuplas nativas (DB.open re-wrap) — ídem.
> - Multi-file imports cobertura ≥50 % en `module.rs`.

---

## Leyenda

- ✅ portado y testado
- 🚧 en progreso
- ⏳ pendiente
- ⚠️ requiere refactor previo (típicamente unificación de tipos de error)
- 🔷 minor / nice-to-have
- 🔶 importante
- ⚡ bloqueante

---

## Tier 1 — Self-contained (sin refactor previo)

Ordenados por simpleza creciente. Empezar por aquí.

| ID | Estado | Prio | Descripción | Bootstrap fix | Test |
|----|--------|------|-------------|---------------|------|
| B151 | ✅ | 🔷 | Escapes `\"` dentro de `${...}` en string interpolation | parser+lexer | `13b93c0` |
| B152 | ⏳ | 🔶 | `impl Display for Class<T>` con campo `[T]` requiere bound `Debug` | `codegen.rs` Display impl pre-scan | `bootstrap_apps/app23_stack.liva` |
| B153 | ⏳ | 🔶 | Free generic functions auto `Clone + Display` | `codegen.rs::generate_function` type-param bounds | `bootstrap_apps/app23_stack.liva` |
| GAP-007 | ⏳ | ⚡ | Function types `(T) => U` → `Box<dyn Fn(T) -> U>` | AST `TypeRef::Fn` + parser + codegen wrap | `bootstrap_apps/app28_closures.liva` |
| B147 | ✅ | ⚡ | `arr.reverse()` en expr-ctx → block-expression | codegen | `a3bba46` |
| B146 | ✅ | ⚡ | `pq.pop()` / `this.method()` en user class — no array dispatch | codegen | `cfa30c3` + `aa56f23` |
| BIN-PAREN | ✅ | ⚡ | Binary precedence parens `(idx-1)/2` | codegen | `a3bba46` |
| EMPTY-SET | ✅ | ⚡ | `let s: Set<T> = {}` → `HashSet::new()` | codegen | `0477c3b` |
| SET-SIZE | ✅ | 🔶 | `set.size()`, `set.count()` → `.len() as i32` | codegen | `0477c3b` |
| FMT-DEBUG | ✅ | ⚡ | `$"{vec}"` usa `{:?}` (no `{}`) para Vec/Map/Set | codegen | `0477c3b` |
| OR-RET | ✅ | ⚡ | `return Map.get(k) or default` (no var-decl) | codegen `_emitBinary` | `c2f63f9` |
| B145 | ⏳ | ⚡ | `string.indexOf(needle, fromIndex)` con 2 args | `codegen.rs` indexOf handler | `bootstrap_apps/app18_template.liva` |
| B144 | ⏳ | ⚡ | Parámetros `Map<K,V>` y `Set<T>` se deben registrar en map_vars/set_vars | `codegen.rs` param tracking | `bootstrap_apps/app18_template.liva` |
| B142 | ⏳ | ⚡ | `for g in groups` sobre `[[T]]` debe registrar element type `[T]` | `codegen.rs` typed_array_vars + VarDecl | `bootstrap_apps/app17_pipeline.liva` |
| B141 | ⏳ | ⚡ | `arr.reduce(0, fn_ref)` debe envolver fn-ref en closure | `codegen.rs::generate_method_call` reduce | `bootstrap_apps/app17_pipeline.liva` |
| B137 | ⏳ | 🔶 | User method `obj.method("literal")` debe `.to_string()` literal | `codegen.rs::generate_method_call_expr` | `bootstrap_apps/app21_hashmap.liva` (B150) |
| B150 | ⏳ | 🔶 | (extiende B137) genérico para cualquier user method | igual | igual |
| B149 | ⏳ | 🔶 | Vars locales mutadas en constructor deben emitirse `let mut` | `codegen.rs` constructor pre-pass mutated_vars | `bootstrap_apps/app21_hashmap.liva` |
| B148 | ⏳ | ⚡ | `this.X` lectura tras asignación dentro de constructor | `codegen.rs` ctor refactor + Member rewrite | `bootstrap_apps/app27_b148.liva` |
| B135 | ⏳ | 🔶 | Switch-arm con `if`-tail | `codegen.rs::generate_switch_expr` | `bootstrap_apps/app16_fsm.liva` (parcial) |
| B136 | ⏳ | 🔶 | `Set.size` (vs `.size()`) | `codegen.rs::generate_member_expr` | (test pendiente) |
| B134 | ⏳ | 🔶 | `for k, v in map` typing en gen-2 | `codegen.rs::generate_for_stmt` Map branch | `bootstrap_apps/app17_pipeline.liva` |

---

## Tier 2 — Requiere refactor de error handling (⚠️)

Estos fixes asumen `Result<T, liva_rt::Error>`. Gen-2 hoy usa `Result<T, String>`.
**Prerrequisito:** unificar tipo de error en gen-2 → `liva_rt::Error::chain(msg, fn, loc, cause)`.

| ID | Estado | Prio | Descripción |
|----|--------|------|-------------|
| ERR-UNIFY | ⏳ | ⚡ | Migrar gen-2 a `liva_rt::Error` (prerrequisito de Tier 2) |
| B127 | ⏳ | ⚡ | `: T!` (Fallible return) double-wrap |
| B128 | ⏳ | ⚡ | `return fail "X"` en función fallible |
| B129 | ⏳ | ⚡ | Error binding chain |
| B130 | ⏳ | ⚡ | `e.message` post-narrowing |
| B131 | ⏳ | ⚡ | `Map.get(k) or fail "msg"` |
| B132 | ⏳ | ⚡ | `m.get(k).expect(...)` chain |
| B133 | ⏳ | ⚡ | Array literal con fallible elements |
| B138 | ⏳ | 🔶 | `fail` en posición de expresión |
| B140 | ⏳ | ⚡ | `or <default>` no debe propagar fallibilidad |
| B143 | ⏳ | ⚡ | `s.toInt() or fail "msg"` propaga error con chain |
| B139 | ⏳ | 🔶 | switch arms en función `T!` auto-wrap `Ok(...)` |

---

## Tier 3 — Map<K, Class> y self-mutation patterns

| ID | Estado | Prio | Descripción |
|----|--------|------|-------------|
| B116 | ⏳ | ⚡ | Indexed assignment `self.field[i] = X` perdido en gen-2 |
| B117 | ⏳ | 🔶 | `self.field = self.field.concat([x])` no debe mover de `&mut self` |
| B118 | ⏳ | 🔶 | `let pts: Map<K,V> = {}` debe emitir `HashMap::new()` no `serde_json` |
| B119 | ⏳ | 🔶 | `for k, v in map` destructure en gen-2 |
| B120 | ⏳ | 🔶 | `arr.len()` cast `usize → i32` |
| B124 | ⏳ | ⚡ | `m.set(p.field, p)` partial-move |
| B125 | ⏳ | ⚡ | Map de class fields completo |

---

## Tier 4 — Resolved en bootstrap, NO portados

Items sólo presentes en bootstrap pero útiles:

| ID | Notas |
|----|-------|
| FIX-DEFAULT-PARAMS | default values en parámetros de funciones |
| FIX-STRING-SWITCH-OR | `switch s { "a"\|"b" => ... }` |
| FIX-ENUM-REF-CLONE | match `&e` con bindings Copy |

---

## Tier 5 — Open en bootstrap (no portar todavía, decidir primero)

| ID | Notas |
|----|-------|
| B112 | `defer items.push(x)` + uso posterior — borrowing conflict (necesita rediseño DeferGuard) |
| GAP-002 | `or fail` en test functions — verificar si ya funciona |
| GAP-003 | `Set.union/intersection` debe envolver en Set wrapper |
| GAP-004 | `print(a, b)` separador inconsistente entre bootstrap y gen-2 — decisión de spec |

---

## Refactor previos (Fase C)

Antes de portar masivo, gen-2 necesita esto para ser **escalable**:

1. **Modularizar `codegen.liva` (7463 líneas → ~7 módulos):**
   - `codegen/expr.liva` — `_emitExpr` y derivadas
   - `codegen/stmt.liva` — `_emitStmt` y derivadas
   - `codegen/types.liva` — `TypeRef → Rust string`
   - `codegen/class.liva` — `_emitClassDecl`, Display/Debug impls
   - `codegen/method.liva` — dispatch (Array/Map/Set/String/User)
   - `codegen/runtime.liva` — string templates, collection literals
   - `codegen/error.liva` — fail, Result, Error::chain, `?`
2. **Centralizar estado tipado en `EmitContext`:**
   - Hoy `RustEmitter` tiene ~20 campos `Map<string, bool>` dispersos
   - Pasar a `_typeCtx: TypeContext` (ya existe parcial) + `_emitState: EmitState`
3. **Tabla de dispatch de métodos** por receiver type, no por nombre.
4. **Tests unitarios** por módulo en `compiler/tests/codegen_modules/`.

---

## Métricas

- Bootstrap LOC (Rust): ~17.6k
- Gen-2 LOC actual (Liva): ~16.2k (post Phase 11 modularización: target ≤9k en `codegen/*.liva`)
- Bootstrap_apps: **21/21 verde con bootstrap y 21/21 verde con gen-2** (post Phase 10).
- Idempotencia: gen-2 ≡ gen-3 (src + binary).

## Métrica de avance

`bootstrap_apps` ejecutados con gen-2:
- **2026-04-30: 21/21 pasan** ✅ — meta v2.1 alcanzada al cierre de Phase 10.
- Script: `compiler/tests/bootstrap_apps/run_gen2.sh` (también incluido en `compiler/tests/run_all.sh`).

> Lo que sigue debajo es el **historial** del camino recorrido. Algunos
> items aún marcados ⏳ pueden estar implícitamente resueltos porque su
> test asociado entra en los 21/21 — pendiente auditoría 1-a-1 si se
> quiere cerrar formalmente cada ID.

### Errores observados (15 fallos)

| App | Error principal | Mapea a |
|-----|----------------|---------|
| app10_stats | `no method iter found for &mut Stats` | tracking de `self.field` en métodos `&mut self` |
| app12_tree | E0308 mismatched types | recursive enum / Box |
| app14_setops | `Vec<integer>` no implementa Display | print de array |
| app15_library | `Book doesn't implement Display` | auto-Display impl missing (B103/B152 family) |
| app16_fsm | parser: `Expected { but got !` | parser de `T!` (return fallible) |
| app17_pipeline | `?` operator on non-Try | error handling (Tier 2 — B127..B143) |
| app18_template | E0308 | B144/B145 (Map params, indexOf 2-arg) |
| app19_pq | `expect found for type i32` | B146 (user pop) |
| app21_hashmap | `HashMap defined multiple times` | colisión user-class vs std (B150 family) |
| app23_stack | `expect found for type i32/String` | B146 + B153 (auto-Display) |
| app25_parser | parser: `Expected ) but got <EOF>` | B151 (escape `\"` en `${...}`) |
| app27_b148 | `no method iter found for Bag` | B148 (`this.X` reads en ctor) |
| app28_closures | parser: "Unexpected type in parentheses" | GAP-007 (function types) |
| app8_orders | `?` operator on non-Try | error handling (Tier 2) |
| app9_graph | E0308 | error handling + indexed assign |

### Quick wins identificados (Fase D, orden recomendado revisado)

1. **Parser-only fixes** (más fáciles, no tocan codegen):
   - app16_fsm — parser de sufijo `T!`
   - app25_parser — B151 escapes en interpolación
   - app28_closures — GAP-007 function types
2. **Codegen autocontenidos** (bug singular):
   - app19_pq — B146 (no `.expect()` sobre user `pop()`)
   - app27_b148 — B148 (`this.X` reads en ctor)
   - app21_hashmap — B150 (literal-string args en user methods)
3. **Display/Debug auto-impl:**
   - app15_library, app23_stack, app14_setops — B152, B153
4. **Method dispatch fixes:**
   - app18_template — B144 (Map/Set params), B145 (indexOf)
   - app10_stats, app27_b148 — `.iter()` sobre `self.field`
5. **Error handling completo (Tier 2):**
   - app8_orders, app9_graph, app12_tree, app17_pipeline — requiere `liva_rt::Error` migration

