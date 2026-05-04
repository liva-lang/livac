# Changelog

All notable changes to the Liva compiler will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Source of truth for:** released versions, per-tag change summaries.  
> **Companion docs:** `BACKLOG.md` (open tasks, work-in-progress),
> `ROADMAP.md` (high-level vision and phases).

## [Unreleased] — 2026-04-30 — Self-host codegen polish

### Fixed (self-host codegen — discovered via dogfooding examples)
- **Top-level `const` double-registration**: `_analyzeConstDecl` no longer calls
  `declareConst` because the `_registerConst` pre-pass already did. Resolves
  "Variable already declared in scope" for module-level `const` bindings (e.g.
  `examples/dogfooding-v1/main.liva`).
- **Interface emission**: classes with no fields and only bodyless methods are
  now treated as compile-time-only validation contracts and skipped during
  Rust emission, matching bootstrap behavior. Fixes E0308 from empty `fn` stubs
  expecting `String` returns.
- **`substring(start)` single-arg form**: `_emitStringMethod` and the
  `_emitMethodCall` fallback now emit open-ended slices `s[start..]` when only
  one argument is supplied (was emitting `..() as usize`, invalid Rust).
- **Underscore tuple bindings**: `let _, err = fallible()` no longer emits
  `let (mut _, mut err)` (Rust rejects `mut _`). Names equal to `_` skip the
  `mut` qualifier; real names keep it.
- **`serde_json::json!` macro emission**: `Response.json({...})` now lowers
  object/map/array literals to JSON-literal syntax `{"key": expr, ...}` via
  the new `_emitJsonArg` helper. The macro grammar rejected the previous
  HashMap-builder block expansion.

### Added (testing)
- **`tests/cli_subcommand_tests.rs`** — 10 end-to-end tests invoking the
  `livac` binary directly: `check` / `check --json`, `fmt` / `fmt --check`,
  `lint` / `lint --json`, `--help`, no-args, unknown-subcommand. Raises
  coverage on `main.rs` argument-parsing/dispatch surface. Total: 528/528.

### Validation
All gates green after each commit:
- rebuild_selfhost: 4/4 (gen-2 src ≡ gen-3 src + binary, idempotent)
- bootstrap_apps: 21/21 boot · 21/21 gen-2 parity
- regression: 5/5 · complex_apps: 4/4 · e2e_selfhost: 5/5
- cargo: 528/528

## [2.0.0-dev] - 2026-05-01 — v2.0 stress-test refinements

### Fixed (codegen — discovered by extended `bootstrap_apps/` stress tests)
- **B134 — `for k, v in map` con `Map<K, [T]>` o `Map<K, V>` local**: el handler
  registraba `v` como string por defecto. Ahora consulta `class_map_value_types`
  (extendido para `[T]` y `{}`) y un nuevo `local_map_value_types` (poblado en
  `let x: Map<K,V>`). Registra `v` en `array_vars`/`map_vars`/`string_vars`
  según corresponda y propaga `typed_array_vars` para nested loops.
- **B135 — switch arm Block con `if-else` final como tail-expr**: las
  expresiones-statement y if-else en posición final se emitían con `;` final,
  perdiendo el valor (E0308). Nuevo helper `generate_stmt_as_tail_expr` emite
  recursivamente `Stmt::Expr`, `Stmt::Return` y `Stmt::If` como expresiones.
- **B136 — `Set.size()` no implementado**: añadido al matcher de set methods y
  emitido como `(set.len() as i32)`.
- **B137 — `instance.count("literal")` sin `.to_string()`**: el parche B10
  (skip array-iterator pipeline cuando objeto es instancia de clase) no
  convertía argumentos string-literal a `String`. Ahora aplica `.to_string()`
  para literales y `.clone()` para `string_vars` / `class_instance_vars`.
- **B138 — `fail "msg"` rompía switch arms**: `Expr::Fail` emitía
  `\twrite_indent return Err(...);\n`, duplicando `;` en contexto stmt y
  rompiendo arms (`expected ',' or '}'`). Ahora emite sólo
  `return Err(liva_rt::Error::from(...))`; el `Stmt::Expr` añade el `;`.

### Added
- `bootstrap_apps/app10_stats.liva` — stats con `Map<string, [number]>`
- `bootstrap_apps/app11_words.liva` — word frequency con map mutable y top-N
- `bootstrap_apps/app12_tree.liva` — recursive enum `Tree` con switch+if tail-expr
- `bootstrap_apps/app13_calc.liva` — recursive enum `Expr` evaluator
- `bootstrap_apps/app14_setops.liva` — Set ops + filter/map/distinct
- `bootstrap_apps/app15_library.liva` — Map<string, [Class]> + user method `count`
- `bootstrap_apps/app16_fsm.liva` — enums + fallible transitions con `fail`
- `bootstrap_apps/app17_pipeline.liva` — chained iterator pipelines, nested
  arrays, point-free reduce, fallible `toInt() or fail`, error binding count.

### Fixed (round 3)
- **B140 — `or <default>` propagaba fallibilidad incorrectamente**: en
  `semantic.rs::stmt_contains_fail` la rama `Stmt::VarDecl` devolvía true
  cuando el init contenía un fallible call, sin distinguir `or fail`
  (sí propaga) de `or <default>` / error binding (no propagan). Ahora respeta
  `is_fallible` y `or_fail_msg` por separado.
- **B141 — point-free fn ref en `.reduce`**: `arr.reduce(0, my_fn)` emitía
  `iter().fold(0, my_fn)`, pero `fold` requiere `FnMut(B, &T) -> B`. Ahora se
  envuelve el ident en `|acc, x| my_fn(acc, x.clone())`.
- **B142 — for nested sobre `[[T]]`**: el inner for caía en string-iterable
  por defecto (emitía `g.chars()`). VarDecl con `Array(Array(T))` ahora
  registra el element type como `"[T]"`; el for-loop dispatch desempaqueta
  esa codificación y registra `g` como array.
- **B143 — `s.toInt() or fail`**: `toInt()` emitía `parse::<i32>().unwrap_or(0)`
  ignorando el `or fail`. Caso especial añadido en el `or fail` handler para
  `MethodCall::toInt|toFloat`: emite match con `return Err(Error::chain(...))`.
- **B144 — params `Map<K,V>` y `Set<T>` no trackeados**: parámetros con esos
  type refs no se registraban en `map_vars` / `set_vars`, así que `vars.get(k)`
  no usaba el handler especializado y emitía argumentos sin `&` y sin
  `.cloned()` (E0308). Ahora se trackean igual que los de `TypeRef::Array`.
- **B145 — `string.indexOf(needle, fromIndex)`**: el segundo argumento se
  ignoraba — el handler solo emitía `find(needle)` desde el inicio. Ahora,
  cuando hay 2 args, emite `__s[__from..].find(&needle).map(|i| (i + __from)
  as i32).unwrap_or(-1)` con guard si `__from >= __s.len()`.
- **B146 — `pop()` en instancia de clase de usuario**: el post-transform de
  array `(Seq, "pop")` añadía `.expect("pop from empty array")` sin verificar
  el tipo del receiver, rompiendo `pq.pop(): number` con E0599. Ahora se
  comprueba `class_instance_vars` antes de aplicarlo.
- **B147 — `arr.reverse()` sobre `[T]` emitía string-reverse**:
  `is_string_method` matcheaba `reverse` siempre que el adapter fuera `Seq`,
  emitiendo `.chars().rev().collect::<String>()` aun para `Vec<i32>`. Ahora
  cuando el receiver está en `array_vars` se emite
  `{ let mut __v = recv.clone(); __v.reverse(); __v }`.

### Added (round 5)
- `bootstrap_apps/app19_pq.liva` — PriorityQueue (min-heap) con `_siftUp` /
  `_siftDown`, `pop()` / `peek()` / `size()` y `[number].reverse()` para
  ordenar descendiente.
- `bootstrap_apps/app20_shapes.liva` — enum `Shape` con `Circle/Rect/Triangle`
  y dispatch vía `switch` con patrones `Shape.Variant(_)`.
- `bootstrap_apps/app21_hashmap.liva` — hash table from-scratch (parallel
  arrays + linear probing + rehash on load factor) que ejercita B149/B150.

### Fixed (round 6 — bootstrap)
- **B149 — locales mutadas del constructor sin `mut`**: el handler de
  constructor entraba al loop de stmts sin ejecutar el pre-pass
  `collect_mutated_vars_in_block`. Resultado: `let ks = []; ks.push(...)`
  emitía `let ks` y rompía con E0596. Ahora se pre-popula `self.mutated_vars`
  con las mutaciones del cuerpo.
- **B150 — string literal a método de usuario sin `.to_string()`**: B137
  solo cubría `count`. Para el resto de métodos de usuario (`get`, `has`, etc.)
  los args literales-string emitían `&str` mientras el método esperaba
  `String` (E0308). Ahora, en el loop genérico de args, cuando el receiver
  está en `class_instance_vars` y el arg es `Literal::String`, se hace append
  `.to_string()`.

### Known (round 6)
- **B148 — `this.X` no se rebinds en cuerpo no-asignación del constructor**:
  cuando un constructor lee/usa `this` fuera de una asignación directa
  `this.field = expr`, el codegen emite `this` literal pero `Self` aún no
  existe → E0425. Workaround: usar locales y al final asignar a `this.field`.
  Fix propuesto requiere alias mutable post-`Self {...}` con sustitución
  `this`→`__obj`. Aplazado por riesgo en 518 tests.

### Added (round 7)
- `bootstrap_apps/app22_glob.liva` — recursive glob matcher (`?` y `*`),
  `string.charAt`/`length`, `[string].filter` por matcher.
- `bootstrap_apps/app23_stack.liva` — `Stack<T>` genérico (push/pop/peek)
  con instancias `T = number` y `T = string`, más `firstOf<T>` (free
  generic function) que ejercitan B152 y B153.

### Fixed (round 7 — bootstrap)
- **B152 — `impl Display for Class<T>` faltaba bound `Debug`**: cuando un
  campo de tipo container (`[T]`, `Map<K,T>`, `Set<T>`, `T?`) se formatea
  con `{:?}`, las bounds del type param solo incluían `Display` → E0277.
  Ahora se pre-escanea los campos: si alguno usa `{:?}`, se añade
  `std::fmt::Debug` a cada type param.
- **B153 — Free generic functions sin `Clone`**: el codegen emite
  `items[0].clone()` para devolver elementos de `[T]`, pero `T` solo tenía
  las bounds explícitas (vacías por defecto) → E0599. Auto-añadidos
  `Clone + std::fmt::Display` a cada type param de funciones libres
  genéricas (mismo trato que ya recibían las clases vía B103). Snapshot
  `feature_generic_function` actualizada.

### Known (round 7)
- **B151 — string interpolation con `\"` escapado**: `$"a:{m.get(\"k\")}"`
  emite el placeholder literal en vez de evaluar la expresión. Workaround:
  guardar el valor en una variable local antes (`let v = m.get("k"); $"a:{v}"`).

### Added (round 8)
- `bootstrap_apps/app24_lexer.liva` — arithmetic tokenizer (char-by-char
  scan + multi-digit number accumulation, enum `Token` with 7 variants).
- `bootstrap_apps/app25_parser.liva` — recursive-descent parser sobre los
  tokens de app24 que construye un AST `Expr` recursivo y lo evalúa
  (precedencia, paréntesis, parser stateful en `class Parser`).
- `bootstrap_apps/app26_window.liva` — sliding-window stats (min/max/sum)
  sobre array de números.
- Validation: 19/19 bootstrap_apps verde.

### Known (round 8)
- **GAP-007 — sin tipos función ni inferencia de closures-as-return**:
  `(T) => U` no se acepta como tipo (E2000), y un `return () => ...` sin
  tipo de retorno explícito infiere `-> f64`, rompiendo la llamada después.
  Bloquea fábricas de closures (counters, currying) y dispatch tables
  tipadas. Workaround: encapsular la closure en una clase con un método.

### Added (round 9)
- `bootstrap_apps/app27_b148.liva` — ejercita B148 directamente: constructor
  que lee `this.cap` en la condición del while y llena `this.items` en el
  cuerpo. Sin workaround.

### Fixed (round 9 — bootstrap)
- **B148 — `this.X` reads en cuerpo del constructor** (ANTES OPEN): refactor
  del codegen del constructor. Phase 1 ahora emite `let mut __field_X = ...;`
  inline en orden de fuente; los demás stmts del constructor son emitidos
  normalmente y `generate_expr` reescribe `this.X` (lectura) a `__field_X`
  cuando `X` ya fue asignado. Phase 2 queda reducido a `Self { x: __field_x }`.
  Mantiene last-write-wins y elimina los problemas de borrow del esquema
  antiguo.
- **B151 — escapes `\"` dentro de `${...}`** (ANTES OPEN): `parse_string_template_parts`
  ahora maneja escapes (`\"`→`"`, `\\`→`\`, `\n`/`\r`/`\t`) al recolectar
  el contenido del placeholder, antes de pasarlo al sub-parser. Ya se puede
  escribir `print($"k:{m.get(\"key\")}")` directamente.

### Added (round 10)
- `bootstrap_apps/app28_closures.liva` — ejercita `apply` y `compose` con
  parámetros tipados como `(number) => number`.

### Fixed (round 10 — bootstrap)
- **GAP-007 — tipos función como anotación** (ANTES OPEN): nueva variante
  `TypeRef::Fn(args, ret)` en el AST que se emite como `Box<dyn Fn(args) -> ret>`.
  El parser acepta `() => U`, `(T) => U`, `(T1, T2) => U` (peek de `=>` después
  del `)` en `parse_base_type`). El codegen registra los tipos de los parámetros
  en `function_param_types` y, al pasar un `Lambda` literal a un slot que espera
  `Box<dyn Fn>`, lo envuelve automáticamente en `Box::new(...)`. Cobertura de
  cascade: `expand_type_alias`, `format_type_ref`, `validate_type_ref`,
  `validate_json_parse_type_hint` (rechaza con E0904), etc.
  Permite callbacks tipados (`f: (number) => number`) y composición
  (`compose(f, g, x) = f(g(x))`). Limitación abierta: la inferencia de
  closures-as-return sin anotación todavía falla; el workaround actual es
  anotar el retorno con el tipo función explícito.

### Added (round 4)
- `bootstrap_apps/app18_template.liva` — motor de templating `{{var}}` con
  `Map<string, string>` y `string.indexOf(needle, fromIndex)`.


## [2.0.0-dev] - 2026-04-29 — v2.0 al 100% (Release Ready)

### Fixed (codegen — discovered by complex apps testing)
- **B116 — gen-2 silent data loss on `self.field[i] = X`**: `_emitAssign` ahora
  setea `_inAssignTarget = true` antes de emitir el objeto indexado, suprimiendo
  el auto-clone que hacía la mutación caer en un `.clone()` descartado.
- **B117 — bootstrap E0507 en `self.field.concat([x])`**: el emitter de
  `[T].concat(other)` clona obj siempre (`{ let mut __v = obj.clone(); ... }`),
  evitando mover desde `&mut self`.
- **B118 — bootstrap `let m: Map<K,V> = {}` emitía serde_json**: la rama VarDecl
  ahora detecta `ObjectLiteral` vacío + type_ref `Map<_,_>` y emite
  `std::collections::HashMap::new()`. Mismo fix para `Set<_>`.
- **B119 — gen-2 `for k, v in map` destructure roto**: el shadow `let k = k as i32`
  era válido sólo para `enumerate()`. Ahora si `isMapIteration` se emite
  `let k = k.clone()` (key dueña) en lugar del cast.
- **B120 — bootstrap `.len()` no casteaba a `i32`**: tratamiento explícito en
  `generate_method_call` igual que `.length()` → `(obj.len() as i32)`.
- B121, B122, B123 — confirmados como falsos positivos / cascada de los anteriores
  (ver `BUGS.md` para análisis post-fix).

### Validation post-fix
- 518/518 cargo tests verdes
- 9/9 modules en bootstrap_test
- 5/5 programas en e2e_selfhost (basics, enums_match, errors, stdlib, calculator)
- Self-host idempotente: gen-2 src/bin == gen-3 src/bin
- 3/3 complex apps (`compiler/tests/complex_apps/app[4-6]_*`) stdout-match boot↔gen-2
- 4/4 regression repros (`compiler/tests/regression/b11[8]_*`, `b12[1-3]_*`)
- Bench oficial: line 0.98x · csv 0.93x · word 0.93x · map 1.10x (todos < 1.15x)

### Added
- **v2.0 closing blocks (5/5 done)**:
  - **Bloque 1 — Cross-module `&str`**: pre-pass global de signaturas en `main.liva`
    permite que parámetros `&str` se propaguen a través de fronteras de módulos.
    Word counting bench: 1.23x → 0.98x.
  - **Bloque 3 — Coverage tooling**: `cargo-llvm-cov` instalado, baseline registrado
    en `docs/PROJECT_STRUCTURE.md` (62.81% regions / 62.36% lines).
    Targets `make coverage` / `make coverage-html`.
  - **Bloque 4 — E2E self-hosted bench**: `compiler/tests/e2e_selfhost.sh` —
    compila programas con bootstrap y con gen-2, ejecuta ambos y diffs stdout.
    5/5 programas (basics, enums_match, errors, stdlib, calculator) pasan idénticos.
    `compiler/tests/rebuild_selfhost.sh` automatiza la cadena bootstrap → gen-1 → gen-2 → gen-3.
  - **Bloque 5 — BACKLOG / ROADMAP / CHANGELOG** sincronizados a v2.0 final.

### Analysis (not shipped)
- **Bloque 2 — Box<str> Map values**: análisis técnico cerrado.
  No se implementa porque (a) el bench Map usa `Map<string, number>`, no `Map<K, String>`;
  (b) el idiom `m.get(k) or default` siempre clona, así que Box<str> no ahorra CPU bajo el
  API actual; (c) el ahorro de memoria 24B→16B no cambia el bench (todo cabe en L2).
  Reabrible en v2.x si surge un hotpath con `Map<K, String>`.

### Codegen fixes (during Bloque 4)
- `or default` lowering ahora distingue 3 paths según el tipo de la expresión:
  - User free-fn returning `Result<T,Error>` → `match X { Ok(v) => v, Err(_) => default }`
  - Map/Set/Array `.get/first/last/find` → `obj.get(&k).cloned().unwrap_or(default)`
    (nuevo helper `_emitOptionGetWithDefault`)
  - Stdlib inline tuple → patrón `{ let (opt, err_str) = ...; ... }` preservado
- Eliminado `.to_string()` redundante en split→for fusion cuando el parámetro
  ya es `&str` (verificación vía `_strRefParams`).

### Final benchmark (all metrics under 1.15x gate)
- Line counting: 1.07x · CSV building: 1.00x · Word counting: 0.98x · Map build+lookup: 1.09x
- Sort + Filter+Map: <6ms diferencia · Class invariants: paridad o mejor

### Validation locked
- 518 cargo tests · bootstrap_test 9/9 · e2e_selfhost 5/5
- gen-2 source ≡ gen-3 source (`diff -r = 0`)
- gen-2 release binary ≡ gen-3 release binary (`cmp = 0`)

## [2.0.0-dev] - 2026-04-28

### Added
- **Self-hosting Phase 9: Borrow optimizations + official benchmark**
  - Phase 9.1–9.6, 9.8, 9.9, 9.10 done; 9.7 / 9.11 absorbidos por Fase 10.
  - 9.4 fix: `for &x in arr` now emits `&arr` so the deref pattern matches when
    the iterable is a single-use Identifier (`Vec<i32>` and friends).
  - 9.8 (Entry-API peephole) hardened: hoist `_sanitizeName` call to avoid
    `E0502` self-borrow in generated Rust at release optimization.
  - `benchmarks/run_official.sh` — multi-run, median-aware bench runner with
    per-metric Liva/Rust ratio table; honors `LIVAC` env var.
  - `benchmarks/RESULTS.md` — gen-2 self-host vs hand-written Rust:
    - Strings: 1.06x – 2.11x
    - Collections: 1.35x – 2.50x
    - Classes: parity or better
  - Idempotence: gen-2 source ≡ gen-3 source byte-for-byte; gen-2 release
    binary ≡ gen-3 release binary byte-for-byte (`cmp = 0`).
- **Phase 10 plan published** (prerequisite of v2.0):
  - Tier 1 — last-use numbering (10.1), param escape analysis (10.2),
    iterator chain fusion (10.3). v2.0 release blocked until peor-bench
    < 1.15x vs hand-written Rust.
  - Detailed in `compiler/docs/PLAN.md` § Fase 10 and `BACKLOG.md`.

## [2.0.0-dev] - 2026-04-20

### Added
- **Self-hosting Phase 8: Calidad del Rust Generado COMPLETE**
  - **Phase 8.5:** `&str` params for private methods — 77 params converted, 56 `.into()` at call sites
  - **Phase 8.6:** `for item in &vec` borrow iteration — 138→80 clone-iterations
  - **Phase 8.7:** Eliminate redundant `format!("{}", x)` — 77→1 format! calls
  - **Phase 8.8:** `self.field` clone suppression in comparisons — 89→78 clones
  - **Phase 8.9:** Liveness-based let-binding clone elision — 1100→996 clones
  - **Phase 8.10:** Benchmark suite — 3 programs (strings, collections, classes) × Liva + Rust
  - **Result:** 6/10 benchmarks within <10% of hand-written Rust
  - Numeric, class, and enum code at parity with hand-written Rust

### Fixed
- Duplicate `let methodName` declaration in `_emitMethod` (detected by gen-1 semantic analysis)

## [2.0.0-dev] - 2026-04-15

### Added
- **Self-hosting Phase 7: Self-Compilation COMPLETE**
  - **Phase 7.1:** Gen-1 (compiled by bootstrap) produces valid Rust for all 9 modules — 253→0 cargo errors
  - **Phase 7.2:** Gen-2 (compiled by gen-1) produces identical output to gen-1 — idempotence verified
  - 12,226 lines of generated Rust from 11,854 lines of Liva
  - Full pipeline: bootstrap → gen-1 → gen-2 with byte-identical output (sorted)

### Fixed
- **Clone reduction:** gen-1 output 2830 → 1633 clones (42% reduction)
  - Eliminated `EnumVariant.clone()` in function args — enum constructors are always owned
  - Eliminated `None.clone()` in let-bindings — always Copy
  - Skip clone for Binary expression results in let-bindings — owned values
  - Skip clone for enum variant assignments (contains `::`)
  - Expanded `self.field.clone()` suppression to ALL method calls (not just mutating)
  - Smart `Map.set()` cloning via `_emitClonedArg` (skip for literals/single-use)
  - Copy-type field detection via TypeContext (int/bool/float skip clone)
- **String comparison optimization:** suppress `.to_string()` for `==` and `!=` with string literals
  - `String == "literal"` now emits `name == "literal"` instead of `name == "literal".to_string()`
  - Reduces allocations in hot paths like `_sanitizeName`, `_emitIdentifier`
- **2000x performance improvement** in gen-2 binary (42s → 0.021s per file)
  - Suppressed `self.field.clone()` for array indexing — was cloning entire Vec per access
  - Suppressed `self.field.clone()` for `.length`/`.size` property access
  - Changed for-loop iteration from `.iter().cloned()` to `.clone()` (clone Vec once, not each element)
  - Smart let-binding cloning: skip clone for call results, blocks, literals, owned values

### Known Issues
- Gen-2 binary codegen is slow (~minutes) due to deeply nested match expressions in generated Rust
  - Gen-2 `check` works fast (2s for full compiler)
  - Root cause: Liva switch-in-switch generates deeply nested Rust matches (27 vs 3 in bootstrap)

## [2.0.0-dev] - 2026-04-01

### Added
- **Liva Test Suite Phase 5 — 4 of 6 layers complete**
  - **Syntax tests:** 15 files — all pass `livac check`
  - **Compile tests:** 8 files — `livac build` + cargo check
  - **E2E tests:** 18 files — 117 assertions via `livac test --verbose`
  - **Stdlib tests:** 6 files — 97 assertions (string, array, map, set, math, types)
  - **Error tests:** 10 files — validates 10 error codes (E0001, E0310, E0701, E0901-E0904, E1000, E2000, E4004)
  - **Test runner:** `run_tests.sh` with 6-layer support + `run_error_tests.sh`
  - Total: **57 test files, 214+ assertions**

### Fixed
- **FIX-DEFAULT-PARAMS**: Function default parameters now inject at call sites when args missing
- **FIX-STRING-SWITCH-OR**: String switch or-patterns (`"a" | "b"`) now correctly detect `.as_str()` need
- **FIX-ENUM-REF-CLONE**: Match-by-reference now clones ALL bindings (not just non-Copy)
- **Runner `set -e` compatibility**: Replaced `((PASS++))` with `PASS=$((PASS + 1))`

### Documented
- **BUGS.md**: 12 codegen bugs (B101-B112) + 3 language gaps (GAP-001 to GAP-003)

## [2.0.0-dev] - 2026-04-01

### Added
- **Self-hosting Phase 3: Codegen Limpio** — `compiler/src/codegen.liva` (2458 lines, new module)
  - RustEmitter class: output buffer, indent management, name sanitization (camelCase→snake_case)
  - Type emission: all 9 TypeRef variants → Rust types (Vec, HashMap, HashSet, Option, Result, tuples)
  - Declarations: functions, classes (struct+impl+constructor), enums (Copy for unit), type aliases, imports
  - Statements: all 16 Stmt variants (var decl, if/for/while/switch, try/catch, assign, return)
  - Expressions: all 22+ Expr variants (literals, binary/unary, calls, member access, lambdas, switch expr)
  - Stdlib mapping: 78 methods (28 string + 30 array + 10 map + 10 set) → Rust equivalents
  - Free function mapping: print/println → macros, toString → format!, toInt/toFloat → parse
  - Ownership helpers: type-directed _emitRefArg for & references
  - Cargo.toml generation with feature-aware dependencies (async, http, db, json, regex, etc.)
  - Type-directed method dispatch via TypeContext lookup (no more HashSet guessing)
  - Public API: generateRust(program, typeCtx, liveCtx)

- **Self-hosting Phase 4: Main + CLI + Bootstrap**
  - `compiler/src/main.liva` (449 lines) — CLI entry point: build/run/check subcommands
  - `compiler/src/module.liva` (234 lines) — Module resolver: BFS import resolution, topological sort
  - `compiler/tests/bootstrap_test.sh` — Bootstrap validation script
  - Full pipeline: read → lex → parse → semantic → liveness → codegen → write → cargo build
  - Single-file and multi-file compilation modes
  - Bootstrap test: 7/9 modules compile to standalone valid Rust (10,859 lines from 9,013 Liva)

### Self-hosting Summary
- **9 modules, 9,013 lines of Liva** — complete compiler from lexer to CLI
- **7/9 modules generate valid standalone Rust** via bootstrap compiler
- **Phases 0-4 complete** — all planned functionality implemented
- Remaining: codegen.liva and main.liva Rust errors (bootstrap limitations, not Liva source errors)

## [2.0.0-dev] - 2026-03-31

### Added
- **Self-hosting Phase 2.7: Liveness Analysis** — `compiler/src/liveness.liva` (519 lines, new module)
  - LivenessContext output: useCounts, loopUseCounts, paramBorrow maps
  - LivenessAnalyzer class: walks AST counting variable uses per function/method
  - Loop tracking: `_inLoop` flag saved/restored for for/while nesting
  - Parameter borrow detection: Copy types → owned, non-Copy → borrow
  - Full AST coverage: all 22 Expr variants, all Stmt variants, lambdas, switch arms
  - Query helper: isCopyTypeName for codegen consumption
  - Public API: analyzeLiveness(program) entry point
  - Phase 2 COMPLETE — semantic analyzer fully functional
- Removed `examples/self-hosting/` legacy directory (canonical location: `compiler/`)

## [2.0.0-dev] - 2026-03-31

### Added
- **Self-hosting Phase 2.6: Import Resolution** — `compiler/src/semantic.liva` (1708 lines, +62)
  - Import registration: `_registerImport` processes TopLevel.Import items
  - Shallow type stubs: uppercase imported names get stub ClassInfo for type resolution
  - TypeContext enriched: importedSymbols + importSources for codegen
  - Query methods: isImportedSymbol, getImportSource

## [2.0.0-dev] - 2026-03-31

### Added
- **Self-hosting Phase 2.5: Class/Enum Metadata** — `compiler/src/semantic.liva` (1646 lines, +140)
  - Constructor validation: `_validateStructLiteral` checks field count vs ClassInfo
  - Map method type table: 10 methods (has/get/set/delete/keys/values/entries/size/isEmpty/clear)
  - Set method type table: 7 methods (has/add/delete/size/isEmpty/toArray/clear)
  - Enum variant inference in `_inferCallByName` and `_inferMemberOnSimple`
  - Method dispatch on MapType and SetType in `_inferMethodCallType`
  - Metadata query API: getClassFieldNames, getClassMethodNames, getEnumVariantNames, isEnumUnit, getVariantFieldCount

## [2.0.0-dev] - 2026-03-31

### Added
- **Self-hosting Phase 2.4: Function Signatures** — `compiler/src/semantic.liva` (1506 lines, +178)
  - Current function tracking: `_currentFuncName` + `_currentFuncFallible` with save/restore
  - Param type storage: `_storeParamType` stores parameter types in pool during analysis
  - Return type validation: `_validateReturn` compares inferred vs declared return types
  - Call argument count validation: `_validateCallArgs` checks against function signatures
  - Fallibility tracking: `_trackCallFallibility` propagates fallibility from callees
  - 1 new bootstrap workaround (W-006: bare return after => not supported)

## [2.0.0-dev] - 2026-04-01

### Added
- **Self-hosting Phase 2.3: Expr Typing** — `compiler/src/semantic.liva` (1328 lines, +116)
  - Type index maps: `_funcRetTypeIdx`, `_fieldTypeIdx`, `_methodRetTypeIdx` for O(1) lookup
  - Second indexing pass: `_indexTypeInfo(program)` populates maps after registration
  - Lookup methods filled: `lookupFuncReturnType`, `_lookupMethodReturnType`, `_lookupFieldType`
  - Expression analysis: `_analyzeExpr` exercises type resolver during analysis pass
  - Statement analysis: Assign, Switch, ExprStmt, Return, Throw, Fail handling
  - Control flow: `_analyzeIf`/`_analyzeWhile` analyze conditions
  - Factory function: `_addTypeOpt(optRef: TypeRef?)` for safe Optional → pool index
  - TypeContext enriched with funcRetTypes, fieldTypes, methodRetTypes
  - 1 new bootstrap workaround documented (W-005: option_value_vars leak)
- **Self-hosting Phase 2.2: Type Resolver** — `compiler/src/semantic.liva` (1212 lines, +564)
  - Type pool: `_typePool: [TypeRef]` + `_varTypeIdx: Map<string, number>` for storing resolved types
  - `resolveTypeRef(t: TypeRef): TypeRef` — recursive resolution of all 9 TypeRef variants
  - `inferExprType(expr: Expr): TypeRef` — infers types for all Expr variants:
    Literals, Identifiers, StringTemplate, Array/Map/Set/Tuple literals,
    Binary/Unary ops, Call, MethodCall, MemberAccess, Lambda, Ternary,
    StructLiteral, Unwrap, OptionalChain, Fail, RustBlock
  - String method type tables (15 methods: length, trim, contains, split, etc.)
  - Array method type tables (15 methods: push, pop, filter, find, sort, etc.)
  - For-loop element type inference via `_inferIterableElemType`
  - Type utilities: `_typeToString`, `typesEqual`, `isUnknownType`, `_unwrapOptionalType`
  - Variable type storage during analysis: `_analyzeVarDecl`, `_analyzeConstDecl`, `_analyzeFor`
  - 4 new bootstrap workarounds documented (W-001 through W-004)

## [2.0.0-dev] - 2026-03-31

### Added
- **Self-hosting Phase 2.1: Semantic Analyzer** — `compiler/src/semantic.liva` (647 lines)
  - TypeContext, Scope, Symbol, FunctionSig, ClassInfo, EnumInfo, Diagnostic types
  - SemanticAnalyzer: scope management, flat symbol table (`"scopeId:name"` keys)
  - Registration pass (collects all top-level declarations)
  - Analysis pass (walks AST, declares variables in proper scopes)
  - Factory functions (makeParamSig, makeFunctionSig, makeFieldInfo) for Optional field routing
  - Compiles to Rust without errors via bootstrap compiler
- **Bootstrap codegen fix SH-011** — Switch expression mutation scanner
  - `collect_mutated_vars_in_expr()` now descends into `Expr::Switch` arms
  - Fixes `let mut` detection for variables mutated inside switch expression arms
- **Bootstrap codegen fix SH-012** — `init_is_already_optional()` for `Expr::Member`
  - Detects Optional struct fields via `var_types` + `class_optional_fields`
  - Prevents double `Some()` wrapping when passing already-Optional member access to constructors
- **Bootstrap codegen fix SH-013** — For-loop `var_types` tracking
  - Loop variables from `for p in arr` now registered in `var_types` for type detection
- **Compound assignment operators** — `+=`, `-=`, `*=`, `/=`, `%=`
  - Desugared at parser level: `x += expr` → `Assign { target: x, value: x + expr, op: Add }`
  - Formatter round-trips correctly using `op` field
  - 5 new lexer tokens (PlusAssign, MinusAssign, StarAssign, SlashAssign, PercentAssign)
  - Works with variables, member access (`c.count += 1`), array index (`arr[0] += 1`), in loops
  - 7 new tests (codegen snapshots + formatter)
- **Wildcard `_` in enum switch destructuring** — `EnumName.Variant(_)` ignores captured value
  - Parser accepts `Token::Underscore` in enum variant bindings
  - Codegen generates `field_name: _` in Rust match pattern
  - Semantic analysis skips `_` bindings (not counted as variables)
  - Works with mixed bindings: `Expr.Add(l, _)` captures `l`, ignores right
  - 3 new tests (codegen snapshots + formatter)
- **`for i, item in array` (enumerate)** — iterate arrays with index
  - Codegen detects Map vs Array by checking `map_vars`
  - Array: generates `.iter().enumerate()` with `i as i32` cast for Liva `int` type
  - Map: unchanged (`.iter()` with key-value clone)
  - 3 new tests (codegen snapshots + formatter)
- **Suppress unused import warnings** — `#[allow(unused_imports)]` on generated `use` statements
  - Applies to both module files and entry point
  - Eliminates Rust warnings for pass-through type imports
- **Extensionless imports** — `import { X } from "./module"` (without `.liva`)
  - Module resolver and semantic validator try appending `.liva` when path has no extension
  - LSP already had this fallback; now compiler matches
  - 1 integration test added
- **String append `push_str` optimization** — `content += ch` → `content.push_str(&ch)`
  - Detects `x = x + expr` / `x += expr` pattern for known string variables
  - Generates `push_str()` instead of `format!("{}{}", x, expr)` — O(1) vs O(n) per append
  - Handles string literals, string variables, and non-string expressions
  - 3 codegen tests added
- **Enum exhaustive switch checking** — omit `_` when all variants are covered
  - Semantic analyzer stores enum variant lists (`enum_variants`)
  - `check_enum_exhaustiveness()` validates coverage, supports Or-patterns
  - Error `E0904` lists missing variants when switch is non-exhaustive
  - 2 tests (codegen + semantic error snapshot)
- **`arr.sortBy(fn)` method** — Sort arrays by a key extraction function
  - Returns new sorted array (non-mutating)
  - Key function receives owned element clone; works with Copy and non-Copy types
  - Generates Rust `sort_by` with `partial_cmp` for universal comparability
  - Works with class instances (`users.sortBy(u => u.age)`), primitives, and string length
- **`arr.groupBy(fn)` method** — Group array elements by key function → `Map<K, [V]>`
  - Returns `HashMap<K, Vec<V>>` grouped by key extraction
  - Uses `entry().or_insert_with(Vec::new).push()` pattern
  - Result tracked as `map_vars` for proper codegen handling
  - Works with class fields, numeric expressions, boolean predicates
- **Array method type propagation** — `sort`, `sortBy`, `reversed`, `distinct`, `flat`, `flatten`, `take`, `drop`, `slice`, `chunks`, `flatMap` results now properly tracked as typed arrays
  - Fixes print format for array results (`{:?}` instead of broken `{}`)
  - Enables correct for-loop iteration over method results (class field access)
  - Fixed camelCase variable name lookup in for-loop type propagation
- 4 new snapshot tests, 503 tests total

### Fixed
- **Bug #90:** `.length` on class instances with a `length` field now emits `.length` (not `.len()`)
  - Checks `var_types` + `class_fields` before applying the `.len()` rule
- **Bug #91:** `array[i].field` now always `.clone()` through class-typed array index
  - Replaces hardcoded field name list (Bug #51) with universal clone for all fields
- **Bug #92:** Struct array indexing verified to generate `.clone()` via `typed_array_vars`
- **Bug #94:** Variables from typed array indexing (`let x = arr[i]`) now tracked as `string_vars` / `class_instance_vars`
  - Enables auto-clone when the variable is passed to multiple function calls
- Self-hosting experiment: lexer (660 lines) + parser (948 lines) written in Liva — identified and fixed 4 codegen bugs

### Changed
- **Recursive enums (auto-boxing)** — Enum variants can reference their own enum type
  - Auto-detection: fields whose type matches the enum name are automatically boxed
  - Codegen: recursive fields emit `Box<T>` in enum definition
  - Construction: `Box::new()` auto-wrapping at variant construction sites
  - Pattern matching: auto-dereference (`let binding = *binding;`) in match arms
  - Array fields (`[T]`) don't need boxing — `Vec<T>` already provides indirection
  - Pre-scan phase populates `boxed_enum_fields` metadata for all program items
  - Works in both `generate_method_call_expr` and `generate_normal_call` paths
  - 5 new tests (4 snapshot + 1 assertion), 499 tests total

### LANGUAGE_ISSUES — All 21 Resolved
- **10 FIXED**: A1-A5 (codegen bugs), C1-C2, C4-C5, C7 (ergonomics), B4 (exhaustive enums)
- **4 already-implemented**: B5 (type alias), B6 (switch guards), C1 (`parseInt or 0`)
- **7 CLOSED**: A6/A8/C3 (deferred to C6), A7 (won't-fix), B1/B2 (design decisions, post-v2.0), B3 (not an issue), C6 (future enhancement)

## [1.9.0-dev] - 2026-03-23

### Added
- **`defer` statement** — Register cleanup actions that execute when the scope exits (LIFO order)
  - Single expression: `defer DB.close(db)` 
  - Block form: `defer { print("cleanup"); File.close(f) }`
  - LIFO execution: last `defer` registered runs first (like Go/Swift)
  - Works with `return`, `fail`, and normal scope exit
  - Full compiler pipeline: Lexer → Parser → Semantic → IR → Lowering → Codegen → Formatter → Linter
  - CodeGen: Rust scope-guard pattern using `Drop` trait
  - 6 new tests (5 snapshot + 1 formatter)

### Dogfooding v3 — TODO API REST 🐕

**Complete REST API built in Liva (~195 lines) validating HTTP Server + SQLite + JSON.stringify end-to-end.**

```liva
// Full CRUD API for task management
main() {
    let db, err = DB.open("todos.db")
    let app = Server.create()
    
    app.get("/tasks", (req) => {
        let rows, qerr = DB.query(db, "SELECT * FROM tasks")
        Response.json(JSON.stringify(rows))
    })
    
    app.post("/tasks", (req) => {
        let title = extractJsonField(req.body, "title")
        DB.exec(db, "INSERT INTO tasks (title) VALUES (?)", [title])
        // ...
    })
    
    app.listen(3000)
}
```

#### Endpoints Tested
- `GET /health` — Health check
- `POST /tasks` — Create task (with title + description)
- `GET /tasks` — List all tasks (JSON array via JSON.stringify)
- `GET /tasks/:id` — Get single task
- `PUT /tasks/:id` — Update task (partial update with defaults)
- `DELETE /tasks/:id` — Delete task

#### Bugs Fixed (7 total)
- **B83**: `Map.get()` returned `Option<String>` instead of `String` — added `.unwrap_or_default()`
- **B84**: DB Connection not thread-safe for async handlers — wrapped in `Arc<Mutex<>>`
- **B85**: `rows[0]` moved HashMap out of Vec — added `.clone()` for map_array_vars
- **B86**: DB params consumed String variables — new `generate_db_params_vec()` helper
- **B87**: `req.body` assigned vars not tracked as strings — added string_vars tracking
- **B88**: axum 0.8 uses `{param}` not `:param` — convert in route path generation
- **B89**: `indexOf` two-arg not supported — Liva source workaround

#### Compiler Changes
- **Codegen**: `suppress_map_get_unwrap` flag for `or default`/`or "value"` paths
- **Codegen**: DB.open wrapped in `Arc<Mutex<>>`, DB.exec/query use `.lock().unwrap()`
- **Codegen**: `generate_db_params_vec()` helper — per-element `.to_string()`
- **Codegen**: Route paths convert `:param` → `{param}` for axum 0.8
- **Codegen**: `map_array_vars` tracked for Vec index cloning
- **Codegen**: `req.body` assignments tracked in `string_vars`
- **Codegen**: `rows[0]` from `map_array_vars` tracked as `map_vars`
- **Tests**: 482 total (3 snapshots updated)
- **Example**: `examples/dogfooding-v3/main.liva`

## [1.8.0-dev] - 2026-03-23

### Added - DB Module (SQLite) 🗄️

**4 database functions with crate `rusqlite` (bundled) auto-injected — embedded SQLite with zero external dependencies.**

```liva
// Open database
let db, err = DB.open("myapp.db")

// Create tables and insert data
let _, err2 = DB.exec(db, "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
let _, err3 = DB.exec(db, "INSERT INTO users (name) VALUES (?)", ["Alice"])

// Query rows → [Map<string, string>]
let rows, err4 = DB.query(db, "SELECT * FROM users")
for row in rows {
    print("Name: " + row.get("name"))
}

// Close connection
DB.close(db)
```

#### Functions
- `DB.open(path)` → `connection, error` — open/create SQLite database (fallible)
- `DB.exec(db, sql[, params])` → `_, error` — execute SQL statements (fallible)
- `DB.query(db, sql[, params])` → `rows, error` — query returning `[Map<string, string>]` (fallible)
- `DB.close(db)` — close connection

#### Compiler Changes
- **Desugaring**: `has_db` flag detects `DB.*` usage
- **Codegen**: `generate_db_function_call()` — 4 methods (open, exec, query, close)
- **Codegen**: `rusqlite = { version = "0.32", features = ["bundled"] }` auto-injected into Cargo.toml
- **Codegen**: `db_vars` tracking for `rusqlite::Connection` variables
- **Codegen**: `map_array_vars` tracking for query results (`Vec<HashMap<String,String>>`)
- **Codegen**: Auto-unwrap `row.get()` in string concatenation context
- **Codegen**: Special `DB.open` fallback (Connection has no Default → uses `open_in_memory`)
- **Tests**: 2 new snapshot tests (458 total)
- **Docs**: `db.md`, stdlib README updated
- **Example**: `examples/db-demo/main.liva`

### Added - Linter (Static Analysis) 🔍

**4 warning rules detecting code smells without blocking compilation.**

```bash
# CLI usage
livac lint main.liva        # Human-readable output
livac lint main.liva --json  # JSON for IDE integration
```

```
warning [W001]: Unused variable
  --> main.liva:5
      5 |     let y = 10
   = Variable 'y' is declared but never used
   help: Prefix with underscore to suppress: _y
```

#### Warning Codes
- **W001**: Variable declared but never used (`_` prefix suppresses)
- **W002**: Import declared but never used
- **W003**: Unreachable code after `return`/`fail`/`break`/`continue`
- **W004**: Comparison always true/false (self-comparison, literal comparisons)

#### Compiler Changes
- **New module**: `linter.rs` — AST-based static analysis
- **New CLI subcommand**: `livac lint <file> [--json]`
- **Tests**: 24 new linter tests (482 total)
- **Docs**: `docs/language-reference/linter.md`

## [1.6.0-dev] - 2026-03-23

### Added - File & Dir Extended Operations 📁

**11 File functions + 7 Dir functions — complete filesystem toolkit for scripts and data processing.**

#### File — New Functions (6)

```liva
// Copy, move, rename files
let ok, err = File.copy("src.txt", "backup.txt")
let ok, err = File.move("draft.txt", "final.txt")

// File metadata
let bytes, err = File.size("data.bin")        // Size in bytes
let ext = File.extension("photo.jpg")         // "jpg" (no error binding)

// Line-oriented I/O
let lines, err = File.readLines("data.csv")   // Returns [string]
let ok, err = File.writeLines("out.txt", ["line1", "line2"])
```

#### Dir — New Functions (5)

```liva
// Check, create, delete directories
let exists = Dir.exists("./output")                  // true only for dirs
let ok, err = Dir.create("./output/reports/2026")    // mkdir -p
let ok, err = Dir.delete("./temp")                   // rm -rf (recursive)

// Recursive listing
let files, err = Dir.listRecursive("./src")          // All files, relative paths
let files, err = Dir.walk("./docs")                  // Alias for listRecursive
```

#### Compiler Changes

- **Codegen**: 6 new `File.*` methods in `generate_file_function_call()` (copy, move, size, extension, readLines, writeLines)
- **Codegen**: 5 new `Dir.*` methods in `generate_dir_function_call()` (exists, create, delete, listRecursive, walk)
- **Codegen**: Updated `is_file_call()` to recognize all new fallible methods
- **Codegen**: Track `Dir.listRecursive`/`Dir.walk`/`File.readLines` results as `[string]` arrays
- **Parser**: Allow `move` keyword as method name in `parse_method_name()` for `File.move()`
- **Tests**: 4 new snapshot tests (total: 232 codegen tests)
- **Docs**: Updated `file-io.md`, `QUICK_REFERENCE.md`, stdlib README

### Added - Regex Module 🔍

**5 regex functions with crate `regex` auto-injected — no `use rust` needed.**

```liva
let isEmail = Regex.test("^[\\w.+-]+@[\\w.-]+\\.[a-zA-Z]{2,}$", email)
let found, err = Regex.match("\\d+", "Order #42")
let numbers = Regex.findAll("\\d+", "a1b22c333")
let clean = Regex.replace("\\s+", text, " ")
let parts = Regex.split("[,;]", "a,b;c")
```

#### Compiler Changes

- **Desugaring**: `has_regex` flag in `DesugarContext` for crate auto-injection
- **Codegen**: `generate_regex_function_call()` with 5 methods (test, match, findAll, replace, split)
- **Codegen**: `is_file_call()` extended for `Regex.match` (returns tuple)
- **Codegen**: `regex = "1"` auto-added to Cargo.toml when `Regex.*` is used
- **Parser**: Allow `test` keyword as method name in `parse_method_name()`
- **Tests**: 2 new snapshot tests (total: 232 codegen, 6 desugar)

### Added - Date Module 📅

**First-class Date type with 4 constructors, 6 properties, 4 methods — uses `chrono` crate (auto-injected).**

```liva
let now = Date.now()                               // Current date/time
let birthday = Date.new(1990, 6, 15)               // Specific date (midnight)
let parsed, err = Date.parse("2026-03-11", "YYYY-MM-DD")  // Fallible parse
let ts = Date.timestamp()                          // Unix epoch ms

// Properties
print(now.year)   // 2026
print(now.month)  // 3

// Methods
let formatted = now.format("DD/MM/YYYY")           // "23/03/2026"
let nextWeek = now.add(7, "days")                  // Date arithmetic
let age = now.diff(birthday, "years")              // 35
let iso = now.toString()                           // "2026-03-23T14:30:00"

// Comparisons + Interpolation
if nextWeek > now { print($"Future: {nextWeek}") }
```

#### Compiler Changes

- **AST**: `TypeRef::Simple("Date")` → `chrono::NaiveDateTime`
- **Desugaring**: `has_date` flag in `DesugarContext` for crate auto-injection
- **Codegen**: `generate_date_function_call()` with 4 constructors (now, new, parse, timestamp)
- **Codegen**: `generate_date_method_call()` with 4 methods (format, add, diff, toString)
- **Codegen**: Date property access via `chrono::Datelike`/`chrono::Timelike` traits (year, month, day, hour, minute, second)
- **Codegen**: `is_file_call()` extended for `Date.parse` (returns tuple), special epoch default
- **Codegen**: `chrono = "0.4"` auto-added when `Date.*` or `Log.*` is used
- **Codegen**: Date interpolation in `$"..."` auto-formats as ISO 8601
- **Codegen**: `date_vars` tracking for property/method dispatch on Date variables
- **Tests**: 3 new snapshot tests (total: 236 codegen, 6 desugar)
- **Docs**: `docs/language-reference/stdlib/date.md` — complete documentation

### Added - CSV Module 📊

**8 CSV functions with Table support — pure Rust `std`, no external crates.**

```liva
// Read/write raw CSV
let data, err = CSV.read("data.csv")
let ok, err = CSV.write("output.csv", data)

// Read/write with headers (Table = [Map<string, string>])
let table, err = CSV.readTable("ventas.csv")
let headers = CSV.headers(table)           // ["producto", "region", "ventas"]
let names = CSV.column(table, "name")     // Extract column values
CSV.writeTable("result.csv", table)

// Pure parsing (no I/O)
let rows = CSV.parse("a,b\n1,2")
let csv = CSV.stringify(rows)

// Custom separator (TSV)
let tsv, err = CSV.read("data.tsv", "\t")
```

#### Compiler Changes

- **Codegen**: `generate_csv_function_call()` with 8 methods (read, write, readTable, writeTable, parse, stringify, headers, column)
- **Codegen**: Inline CSV parser handles quoted fields, escaped quotes, custom separators
- **Codegen**: `is_file_call()` extended for `CSV.read`, `CSV.write`, `CSV.readTable`, `CSV.writeTable`
- **Codegen**: Track `CSV.read` and `CSV.readTable` results as array variables
- **Tests**: 2 new snapshot tests (total: 237 codegen)
- **Docs**: `docs/language-reference/stdlib/csv.md` — complete documentation

### Added - Random, Crypto, Process Modules 🔧

**3 new stdlib modules — 13 functions total for randomness, hashing, and process control.**

#### Random — 5 functions (crates `rand` + `uuid` auto-injected)

```liva
let n = Random.nextInt(1, 100)         // Random integer in [min, max]
let f = Random.nextFloat(0.0, 1.0)     // Random float in [min, max] (args optional)
let pick = Random.choice(["a", "b", "c"])  // Random element
let mixed = Random.shuffle([1, 2, 3])  // Shuffled copy
let id = Random.uuid()                 // UUID v4 string
```

#### Crypto — 4 functions (crates `sha2`, `md-5`, `base64` auto-injected)

```liva
let hash = Crypto.sha256("hello")       // Hex-encoded SHA-256
let md = Crypto.md5("hello")            // Hex-encoded MD5
let encoded = Crypto.base64Encode("hello")  // Base64 string
let decoded, err = Crypto.base64Decode(encoded)  // Fallible
```

#### Process — 4 functions (no external crates, uses `std::process`)

```liva
let output, err = Process.exec("ls -la")   // Run command, capture stdout
let pid, err = Process.spawn("sleep 10")   // Spawn background process
let myPid = Process.pid()                  // Current process PID
Process.exit(0)                            // Exit with code
```

#### Compiler Changes

- **Codegen**: `generate_random_function_call()` with 5 methods (nextInt, nextFloat, choice, shuffle, uuid)
- **Codegen**: `generate_crypto_function_call()` with 4 methods (sha256, md5, base64Encode, base64Decode)
- **Codegen**: `generate_process_function_call()` with 4 methods (exec, spawn, pid, exit)
- **Desugaring**: `has_random` flag for `rand`/`uuid` crate auto-injection
- **Desugaring**: `has_crypto` flag for `sha2`/`md-5`/`base64` crate auto-injection
- **Codegen**: `is_file_call()` extended for `Process.exec`, `Process.spawn`, `Crypto.base64Decode`
- **Tests**: 3 new snapshot tests (1 per module)

### HTTP Server (v1.7)
- `Server.create()` — creates an axum Router instance
- `app.get/post/put/delete(path, handler)` — route registration with lambda handlers
- `app.listen(port)` — starts HTTP server on specified port
- `req.params.get("key")` — path parameter extraction (`:param` syntax)
- `req.body` — request body access for POST/PUT handlers
- `Response.text(content)` — plain text response (200 OK)
- `Response.json(data)` — JSON response (200 OK)
- `Response.status(code)` — status-only response
- Auto-injected `axum = "0.8"` dependency
- Async main inference when `Server` is used
- 3 snapshot tests, server.md + response.md docs, http-server example

## [1.5.0] - 2026-03-20

### Added - `rust { }` Interop 🦀

**Inline Rust code blocks and enhanced crate management unlock the entire Rust ecosystem from Liva.**

#### `rust { }` Expression Blocks

```liva
// Inline Rust code as an expression
let result = rust {
    let x: i32 = 42;
    x * 2
}

// Use statements are hoisted to file top
let hash = rust {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert("key", "value");
    map.len()
}
```

#### Enhanced `use rust` with Version & Features

```liva
use rust "chrono" version "0.4"
use rust "uuid" version "1.0" features ["v4", "serde"]
use rust "my_lib" as mylib

// Add features to internal crates (merge, not override)
use rust "tokio" features ["net", "io-util"]
```

#### Internal Crate Protection (E9002)

```liva
// ERROR: E9002 — Cannot override internal crate version
use rust "tokio" version "2.0"
// Internal crates: tokio(v1), serde(v1.0), serde_json(v1.0),
//                  reqwest(v0.11), rayon(v1.11), rand(v0.8)
```

#### Compiler Changes

- **Lexer**: Pre-scan `rust { }` blocks with balanced brace counting (handles nested braces, Rust strings, comments)
- **Parser**: `rust { }` as expression + `use rust` with optional `version`/`features`
- **Desugaring**: `RustCrateDep` struct with name/alias/version/features
- **Semantic**: E9002 validation for internal crate version override attempts
- **IR + Lowering**: `RustBlock` variant pass-through
- **Codegen**: Inline block emission, `use` hoisting, Cargo.toml version/features/merge
- **Formatter**: Round-trip support for `rust { }` and `use rust` with all options
- **Error codes**: E9002 + new `Interop` error category (9000-9999)
- **Tests**: 348 total (7 new: 5 codegen, 1 desugar, 1 semantic)

### Added - Logging Module `Log` 📋

**Structured logging with smart table rendering for maps, arrays, and JSON.**

#### Basic Logging

```liva
Log.info("Server started")           // 2026-03-12T10:30:00 [INFO ] Server started
Log.warn("Disk space low")
Log.error("Connection failed")
Log.debug("Payload received")         // Only with --verbose
Log.setLevel("debug")                 // Change minimum level at runtime
```

#### Variadic Arguments

```liva
Log.info("User", username, "logged in from", ip)
// 2026-03-12T10:30:00 [INFO ] User alice logged in from 192.168.1.1
```

#### Smart Table Rendering

```liva
// Map with 4+ keys → Key/Value table
Log.info("Config:", { host: "localhost", port: 8080, db: "mydb", pool: 10 })
// ┌──────┬───────────┐
// │ Key  │ Value     │
// ├──────┼───────────┤
// │ db   │ mydb      │
// │ host │ localhost │
// │ pool │ 10        │
// │ port │ 8080      │
// └──────┴───────────┘

// Map with ≤3 keys → inline
Log.info("Status:", { code: 200, ok: true })
// {code: 200, ok: true}

// Array of Maps → columnar table (console.table style)
Log.info("Users:", [{ name: "Alice", age: 30 }, { name: "Bob", age: 25 }])
// ┌─────┬───────┐
// │ age │ name  │
// ├─────┼───────┤
// │ 30  │ Alice │
// │ 25  │ Bob   │
// └─────┴───────┘
```

#### JSON Runtime Tables

```liva
let config, _err = JSON.parse(jsonString)
Log.info("Config:", config)  // Auto-detects Object/Array → table rendering
```

#### Compiler Changes

- **Desugaring**: `has_logging` flag for chrono dependency
- **Codegen**: `ArgKind` enum (Scalar, InlineMap, TableMap, TableArray, Json)
- **Runtime**: 5 helpers — `liva_log`, `liva_log_table_kv`, `liva_log_table_rows`, `liva_log_json`, `liva_log_set_level`
- **CLI**: `--verbose` passes `LIVA_VERBOSE=1` env var for debug output
- **Tests**: 14 snapshot tests (374 total)

### Added - Config Module `Config` 🔧

**Environment configuration loading from `.env` files with typed getters. No external dependencies.**

#### Basic Usage

```liva
let config, err = Config.load(".env")
if err {
    Log.error("Config error:", err)
}

let host, _ = Config.get(config, "HOST")           // "localhost"
let port, _ = Config.getInt(config, "PORT")          // 8080
let debug, _ = Config.getBool(config, "DEBUG")       // true
let all = Config.getAll(config)                      // Map<string, string>
```

#### .env Format

```env
# Supported format
HOST=localhost
PORT=8080
DEBUG=true
APP_NAME="My Application"    # Quotes stripped automatically
SECRET_KEY='super-secret'    # Single quotes too
# Comments and blank lines are skipped
```

#### Error Handling

```liva
let config, err = Config.load("missing.env")
if err {
    print("File error:", err)  // "Config load error: No such file..."
}

let val, err = Config.get(config, "MISSING_KEY")
if err {
    print("Key error:", err)   // "Config key not found: MISSING_KEY"
}
```

#### Compiler Changes

- **Desugaring**: `has_config` flag, detection in `TopLevel::Test` and `TopLevel::ExprStmt` blocks
- **Codegen**: Dispatch + 5 runtime helpers (`liva_config_load`, `liva_config_get`, `liva_config_get_int`, `liva_config_get_bool`, `liva_config_get_all`)
- **Bug fix**: Error binding pre-pass now excludes Config calls from `error_binding_vars` (same fix as File/Dir)
- **No external crates**: Uses only `std::fs`, `std::collections::HashMap`, `std::collections::BTreeMap`
- **Tests**: 7 codegen snapshot tests + 11 Liva end-to-end tests (392 total)

### Changed - CLI Subcommands 🔧

**Migrated from flat flags to proper subcommands for better discoverability and UX.**

| Before | After |
|--------|-------|
| `livac file.liva` | `livac build file.liva` |
| `livac file.liva --run` | `livac run file.liva` |
| `livac file.liva --check` | `livac check file.liva` |
| `livac file.liva --fmt` | `livac fmt file.liva` |
| `livac --test` | `livac test` |
| `livac --lsp` | `livac lsp` |
| `livac --update` | `livac update` |

- Refactored `Cli` struct to use clap `#[command(subcommand)]` with `Commands` enum
- Each subcommand has its own specific flags (e.g., `--release` only on `build`/`run`)
- Internal `CompileArgs` struct replaces flat `Cli` reference in `compile()`
- Updated VS Code extension LSP client args
- Updated all docs, examples, scripts, CI, and website

### Added - `livac init` Project Scaffolding 🏗️

**Create new Liva projects with a single command. Templates hardcoded in the binary, zero dependencies.**

```bash
livac init my-project                  # Default template
livac init my-cli --template cli       # CLI application template
livac init my-data --template data     # Data processing template
```

#### Default Template
- `main.liva` with hello world + string interpolation + for loop
- `tests/main.test.liva` with basic test suite
- `.gitignore` configured for Liva projects

#### CLI Template
- `main.liva` with `Sys.args()` parsing + match-based command dispatch
- `config.env` with sample configuration
- `tests/main.test.liva` with CLI function tests

#### Data Template
- `main.liva` with filter/map/sum pipeline + Config loading
- `config.env` with sample configuration
- `tests/main.test.liva` with data processing tests

#### Validation
- Project name: alphanumeric, hyphens, underscores only
- Duplicate directory detection
- Unknown template error with available list
- **Tests**: 6 integration tests (387 total)

### Fixed - AI Audit Bug Fixes (47 bugs) 🐛

**Complete audit of 10 AI-generated projects revealed and fixed 47 compiler bugs.**

#### Ownership & Move Semantics (8 fixes)
- B17: Auto-clone for struct/map passed to functions by value
- B36: Values moved in loop iterations — variable consumed on first iteration
- B35: Array index access as argument now clones instead of moving
- B21: `self.tokens[idx]` moves Token from Vec — now uses `.clone()`
- B44: `.clone()` added for non-Copy fields of `&self`
- B45: `for item in this.collection` iterates over copy — mutations preserved
- B47: Array concat `arr + [value]` no longer moves the value
- B34: Error binding vars now marked `mut` when reassigned

#### Error Binding & `or fail` (6 fixes)
- B01: `_` accepted in error binding (`let val, _ = fn()`)
- B19: Error binding for method calls — `fallible_methods` tracking
- B22: `or fail` works with method calls
- B20: `fail "msg"` scope tracking — Error::new when error var out of scope
- B23: Cross-file error binding — imports pre-populate fallible functions
- B38: Error variable scope leak between if/else branches

#### Field Access (4 fixes)
- B07: `get_field()` heuristic — locals/params use `.field` instead of JSON path
- B06: `enum_names` populated in `generate_module_code()`
- B05: `resp.body` with async generates dot notation instead of `get_field("body")`
- B10: `.count()` no longer collides with array built-in when called on class instances

#### Class Methods (5 fixes)
- B08: `&mut self` detection expanded beyond direct `this.field = x`
- B09: `&mut self` transitive propagation — methods calling `&mut` methods are marked
- B18: Arrow method return type `=> expr` infers correctly instead of `-> ()`
- B14: Enum field in class no longer breaks `Default` derive
- B46: serde derives triggered by `JSON.stringify` (not just `JSON.parse`)

#### Rust Interop (2 fixes)
- B42: `find_rust_blocks()` skips `rust` keyword inside `//` comments
- B43: `find_balanced_brace()` handles lifetimes/apostrophes vs char literals

#### Strings & Templates (5 fixes)
- B02: Template strings with nested quotes in interpolation (`$"{fn("arg")}"`)
- B25: `charAt()` returns char-compatible type for comparisons
- B26: Char escape sequences preserved (`'\n'` → `'\n'`, not `'\\'`)
- B28: String `+` generates `push_str` instead of `.extend()` (iterator)
- B29: Template `{}` (Display) for mutable vars instead of `{:?}` (Debug)

#### Arrays & Collections (3 fixes)
- B15: `.filter()` generates `.cloned()` instead of `.copied()` for non-Copy types
- B39: Array element assignment (`arr[i] = val`) generates valid LHS
- B16: `parseInt(x) or default` unwraps instead of generating tuple

#### Async (3 fixes)
- B24: `main()` marked async when `rust {}` contains `.await`
- B03: `async HTTP.get()` error binding uses String (not Option)
- B04: `spawn_async` adds `.await` for user-defined async functions

#### Types & Conversions (4 fixes)
- B40: `String >= &str` comparison — adds `.as_str()` for ordering operators
- B41: Cast priority `(pos + 1) as usize` — wraps expressions in parentheses
- B32: `f64 / i32` auto-cast — `float_vars` tracking with `as f64` for `.length`
- B31: `const X: string` generates `&str` instead of `String`

#### Misc (7 fixes)
- B11: `console.input` with template string — no nested `print!(format!(...))`
- B33: Single-var binding for fallible generates unwrap instead of tuple
- B37: `type` as field name — Rust reserved keyword escaped as `r#type`
- B27: Enum destructuring field name mapping in IR codegen
- B30: Hyphen in `use rust` crate names converted to `_`

**Test suite**: 388 → 439 tests (+51 regression tests)

### Improved - AI Skill (7 improvements) 📝

- S1: Reserved keywords section expanded with Rust keywords warning
- S2: `main()` auto-detect documented prominently
- S3: `console.input()` verified correct (no `console.prompt`)
- S4: `Sys.args()` behavior documented — args[0] = program name
- S5: Rust interop details — snake_case, Result types, var access, hyphenated crates
- S6: `number` = integer (i32) clarification — not for float math
- S7: Errors are plain strings, not objects with `.message`

---

## [Unreleased] - v1.4.0-dev

### Added - Stdlib P0: String, Array, Math 📚

**38 new methods/functions expanding the standard library for day-to-day programming.**

#### String — 15 new methods

```liva
let s = "hello world"

// Search & index
let pos = s.lastIndexOf("l")          // 9
let sub = s.slice(0, 5)               // "hello"

// Padding & repetition
let padded = s.padStart(20, "*")      // "*********hello world"
let padR = s.padEnd(20, ".")          // "hello world........."
let rep = "ha".repeat(3)              // "hahaha"

// Transform
let cap = s.capitalize()              // "Hello world"
let rev = s.reverse()                 // "dlrow olleh"
let trunc = s.truncate(5)            // "hello"

// Query
let blank = "  ".isBlank()            // true
let empty = "".isEmpty()              // true
let count = s.countMatches("l")       // 3

// Remove
let noPre = "prefix_val".removePrefix("prefix_")  // "val"
let noSuf = "file.txt".removeSuffix(".txt")        // "file"

// Split
let chars = s.chars()                 // ["h","e","l","l","o"," ","w","o","r","l","d"]
let replaced = s.replaceAll("l", "L") // "heLLo worLd"
```

#### Array — 20 new methods

```liva
let nums = [1, 2, 3, 4, 5]

// Slicing
let sliced = nums.slice(1, 3)         // [2, 3]
let top3 = nums.take(3)              // [1, 2, 3]
let rest = nums.drop(2)              // [3, 4, 5]

// Access
let f = nums.first()                 // 1
let l = nums.last()                  // 5
let empty = nums.isEmpty()           // false

// Transform
let sorted = nums.sort()             // [1, 2, 3, 4, 5]
let rev = nums.reversed()            // [5, 4, 3, 2, 1]
let uniq = [1, 2, 2, 3].distinct()   // [1, 2, 3]

// Combine & split
let nested = [[1, 2], [3, 4]]
let flat = nested.flat()             // [1, 2, 3, 4]
let chunked = nums.chunks(2)         // [[1, 2], [3, 4], [5]]
let zipped = [1, 2].zip(["a", "b"])  // [(1, "a"), (2, "b")]

// Aggregate
let total = nums.sum()               // 15
let lo = nums.min()                  // 1
let hi = nums.max()                  // 5

// Callback-based
let idx = nums.findIndex((n) => n > 3)    // 3
let fm = nums.flatMap((n) => [n, n * 10]) // [1, 10, 2, 20, ...]
let cnt = nums.count((n) => n > 2)        // 3
```

#### Math — 3 new functions

```liva
let clamped = Math.clamp(15, 0, 10)  // 10
let sign = Math.sign(-42)            // -1
let ln = Math.log(2.718)             // ~1.0
```

**Implementation details:**
- String `slice()` disambiguated from Array `slice()` via type tracking
- Array method `chunks()` (not `chunk`) to avoid conflict with parallel adapter keyword
- `object_is_class_instance` guard prevents array method handlers from intercepting class instance methods
- `sortBy(fn)` and `groupBy(fn)` deferred to future version (high complexity)

**Test coverage:** 341 tests (149 codegen snapshots), 0 failures, 19 new v1.4 snapshot tests + 1 integration test

### Fixed - Codegen bugfixes for stdlib methods 🐛

**6 codegen bugs fixed — all stdlib methods are now fully testable with `expect().toBe()` assertions.**

- **Option unwrapping:** `find()`, `first()`, `last()`, `min()`, `max()` returned `Option<T>` in generated Rust but Liva expects `T` — added automatic `.unwrap()` in normal context
- **`or fail` for Option methods:** `arr.first() or fail "empty"` was silently ignored — now generates `match Some/None` with `panic!` on `None`
- **`or value` for Option methods:** `arr.first() or 0` was silently ignored — now generates `.unwrap_or(default)`
- **`findIndex` closure type mismatch:** Lambda parameter `x` was `&T` from `.iter()` but comparison `x > 25` failed for non-Copy types — added `ref_lambda_params` tracking + extended deref to all comparison operators (`<`, `>`, `<=`, `>=`, not just `==`/`!=`)
- **`charAt` return type:** Generated `.unwrap_or(' ')` returning Rust `char` instead of `String` — fixed to `.map(|c| c.to_string()).unwrap_or_default()`
- **`.length()` as method call:** Only handled as property access, not method call — `zip`/`chunks` results couldn't use `.length()` — added handler generating `(.len() as i32)`

**Liva assertion tests:** 63 total (28 string + 26 array + 9 math) — all documented methods now have assertion test coverage.

---

## [Unreleased] - v1.3.0-dev

### Fixed - Dogfooding v2: Inventory Manager 🏗️

**8 codegen bugs found and fixed via comprehensive real-world program (350 lines, 21 test scenarios).**

- **Bug #75**: Map/Set class fields (`this.prices`, `this.tags`) not recognized for method routing → generated `.set()` instead of `.insert()`, `.add()` instead of `.insert()`, `.has()` instead of `.contains()`
- **Bug #76**: `is_map_get_call` didn't handle `this._field.get()` → `map.get or default` inside class methods generated `||` instead of `.unwrap_or()`
- **Bug #77**: String/class-instance variables not cloned when passed to instance method calls → `inv.getName(sku)` consumed `sku`, preventing reuse
- **Bug #78**: `or "string"` in fallible call generated `&str` literal → needed `.to_string()` for `Err(_) => "FALLBACK".to_string()`
- **Bug #79**: `some()`/`every()` lambda pattern used `|&&x|` instead of `|&x|` — `any`/`all` take `FnMut(Self::Item)`, not `FnMut(&Self::Item)`
- **Bug #80**: `for key, value in map` loop variables are references (`&K`, `&V`) — added `let key = key.clone()` at loop body start
- **Bug #81**: `map.get(key) or default` at expression level (return, let) used `||` operator → added `BinOp::Or` + `is_map_get_call` detection to generate `.unwrap_or()`
- **Bug #82**: Map `set`/Set `add`/`delete` methods not in `is_mutating_method` list → class methods calling them got `&self` instead of `&mut self`

**Additional codegen improvements:**
- Map/Set `set`/`add` operations clone string variable keys/values to avoid ownership moves
- `option_value_vars` no longer incorrectly tracks user fallible call bindings
- Sanitized names when registering `map_vars`/`set_vars` (camelCase → snake_case consistency)

**Test coverage:** 322 tests (130 codegen snapshots), 0 failures, 7 new regression tests (Bugs #75-#82)

### Added - Set<T> Collections 🎯

**Full Set support with HashSet-backed implementation.**

```liva
// Create sets
let empty: Set<string> = Set {}
let colors = Set { "red", "green", "blue" }

// CRUD operations
colors.add("yellow")
let found = colors.has("red")      // true
colors.delete("green")

// Iteration
for color in colors {
  print(color)
}
colors.forEach((c) => print(c))

// Set operations
let a = Set { 1, 2, 3 }
let b = Set { 3, 4, 5 }
let u = a.union(b)            // Set { 1, 2, 3, 4, 5 }
let i = a.intersection(b)     // Set { 3 }
let d = a.difference(b)       // Set { 1, 2 }

// Collection methods
let vals = colors.values()    // [string]
colors.clear()
```

**Implementation:**
- AST: `TypeRef::Set(Box<TypeRef>)`, `Expr::SetLiteral(Vec<Expr>)`
- Parser: `Set { value1, value2 }` literal, `Set<T>` type annotation
- Semantic: Set type inference from first element, `TypeRef::Set` arms in all type visitors
- IR: `SetLiteral(Vec<Expr>)` variant with `Type::from_ast` Set→Custom conversion
- Lowering: SetLiteral lowering + expression analysis
- Codegen: `HashSet::new()` / `HashSet::from([...])`, full method dispatch (add→`.insert()`, has→`.contains()`, delete→`.remove()`, values→`.iter().cloned().collect()`, forEach→`.iter().for_each()`, union/intersection/difference→set operations with `.cloned().collect()`, clear→`.clear()`)
- Formatter: `format_set_literal()`, `TypeRef::Set` formatting

**Tests:**
- 9 new snapshot tests (set_literal_empty, set_literal_values, set_add_has_delete, set_values, set_foreach, set_for_loop, set_clear, set_union_intersection_difference, set_type_annotation)
- 1 integration test (`examples/tests/test_set.liva`)
- All 315 tests passing

### Added - Map<K,V> Collections (Dictionaries) 🗺️

**Full Map/Dictionary support with HashMap-backed implementation.**

```liva
// Create maps
let empty: Map<string, int> = Map {}
let ages = Map { "Alice": 30, "Bob": 25 }

// CRUD operations
ages.set("Carlos", 35)
let age = ages.get("Alice") or 0    // 30
let found = ages.has("Bob")          // true
ages.delete("Bob")

// Iteration
for key, value in ages {
  print($"{key}: {value}")
}
ages.forEach((k, v) => print($"{k}={v}"))

// Collection methods
let keys = ages.keys()       // [string]
let vals = ages.values()     // [int]
let pairs = ages.entries()   // [(string, int)]
ages.clear()
```

**Implementation:**
- AST: `TypeRef::Map(Box<TypeRef>, Box<TypeRef>)`, `Expr::MapLiteral(Vec<(Expr, Expr)>)`, `ForStmt.var2: Option<String>` for destructured iteration
- Parser: `Map { key: value }` literal (intercepted before StructLiteral), `Map<K, V>` type annotation, `for key, value in map` syntax
- Semantic: Map type inference from first entry, `TypeRef::Map` arms in all type visitors
- IR: `MapLiteral(Vec<(Expr, Expr)>)` variant with `Type::from_ast` Map→Custom conversion
- Lowering: MapLiteral lowering + expression analysis
- Codegen: `HashMap::new()` / `HashMap::from([...])`, full method dispatch (get→`.get(&key).cloned()`, set→`.insert()`, has→`.contains_key()`, delete→`.remove()`, keys/values/entries→`.cloned().collect()`, forEach→`.iter().for_each(|(k,v)| ...)`, clear→`.clear()`), `map.get(key) or default` → `.unwrap_or()`, `for (k, v) in map.iter()` loop generation
- Formatter: `format_map_literal()`, `var2` in `format_for()`, `TypeRef::Map` formatting

**Tests:**
- 8 new snapshot tests (map_literal_empty, map_literal_entries, map_get_set_has_delete, map_keys_values_entries, map_foreach, map_for_loop, map_clear, map_type_annotation)
- 1 integration test (`examples/tests/test_map.liva`)
- All 306 tests passing

### Removed

- **`fire` keyword removed** — Fire-and-forget is now auto-inferred. When an `async` or `par` call appears as a statement (not assigned to a variable), it's automatically treated as fire-and-forget. No `fire` keyword needed.

### Added - Error Trace Chaining 🔍

**Automatic error trace with function names and source locations.**

When errors propagate through `fail`, `or fail`, or `if err => fail`, Liva now builds a chained error trace showing the full call path:

```liva
parsePort(s: string): number {
    fail "invalid port: " + s
}

loadConfig(path: string): string {
    let port = parsePort("abc") or fail "cannot load config"
    return port.toString()
}

startServer(): string {
    let config, err = loadConfig("/etc/app.conf")
    if err => fail "server failed to start"
    return config
}

main() {
    let server, err = startServer()
    if err {
        print(err)
    }
}
```

Output:
```
╭─ Error Trace ─────────────────────────────────────╮
│  ✗ server failed to start
│    → startServer()  main.liva:12
│  ⊘ cannot load config
│    → loadConfig()  main.liva:7
│  ⊘ invalid port: abc
│    → parsePort()  main.liva:3
╰───────────────────────────────────────────────────╯
```

**Implementation:**
- `liva_rt::Error`: New fields `cause: Option<Box<Error>>`, `function: &'static str`, `location: &'static str`
- `Error::new(msg, fn, loc)`: Creates error with location info
- `Error::chain(msg, fn, loc, cause)`: Creates error chaining from a previous error
- `Error::from(msg)`: Backward compatible constructor (no location)
- Parser: Captures source line for `fail` and `or fail` statements
- CodeGenerator: Tracks `current_function_name` and `source_filename`
- Display: Colored box trace (`✗` red for top error, `⊘` yellow for causes)
- `print(err)` shows full trace; `err.message` gives plain message string
- Zero syntax changes — all internal to compiler

### Added - `or <value>` — Default Value for Fallible Calls 🛡️

**New syntax: `let x = fallibleCall() or defaultValue`**

Provides a default value when a fallible function fails, similar to JavaScript's `||` operator:

```liva
let result = divide(10, 0) or 42          // 42 (division failed)
let result2 = divide(10, 2) or 42         // 5  (division succeeded)
let port = parsePort("abc") or 3000       // 3000 (parse failed)
```

Equivalent to:
```liva
let result, err = divide(10, 0)
let result = err ? 42 : result
```

**Implementation:**
- AST: Added `or_value: Option<Box<Expr>>` field to `VarDecl`
- Parser: Post-processes `Binary(Call, Or, value)` into `init=call, or_value=value` (since `or` is consumed by expression parser as logical OR)
- Codegen: Generates `let var = match expr { Ok(v) => v, Err(_) => default };`
- Semantic: `or_value` sets `is_fallible`, suppresses E0701

**Tests:**
- 1 new snapshot test: `feature_error_handling_or_value`
- All 297 tests passing

---

### Fixed - Parser bug: `if cond => fail/break/continue` 🐛

**Bug:** `if err => fail "msg"` was parsed as a lambda `|err| fail "msg"` instead of
an if-condition with arrow body, because `parse_expression()` detected `err =>` as a lambda.

**Fix:** Changed `parse_expression()` to `parse_expression_no_lambda()` for if-conditions
(same approach already used by while/for). Commit `a10b72c`.

---

### Changed - Remove `data` keyword (auto-detect data classes) 🚫🔑

**Breaking change:**
- The `data` keyword has been **removed**. Classes with fields but no explicit constructor are now **automatically** treated as data classes (auto-derive positional constructor, `PartialEq`, and `Display`).
- This is consistent with Liva's philosophy: avoid keywords when the compiler can infer intent from structure.
- **Before:** `data Point { x: number, y: number }`
- **After:** `Point { x: number, y: number }` (same behavior, no keyword needed)

**Rule:** If a class has a `constructor()`, it's a regular class. If not, it's a data class.

**Implementation:**
- Parser: removed `data` contextual keyword detection
- Codegen: auto-detects data classes (fields + no constructor → `is_data = true`)
- Formatter: no longer emits `data ` prefix

**Tests:**
- Updated 3 existing data class tests (removed `data` keyword from source)
- Added 3 new tests: `auto_data_class_fields_only`, `auto_data_class_with_methods`, `class_with_constructor_not_data`
- All 296 tests passing

**Docs:**
- Updated `QUICK_REFERENCE.md`, `classes-data.md`, `copilot-instructions.md`

---

### Added - Session 18: Dir Module, Sys Docs, string.contains() 🗂️

**New stdlib APIs:**
- `Dir.list(path)` — List directory entries with error binding, returns sorted `[string]`
- `Dir.isDir(path)` — Check if a path is a directory (returns `bool`)
- `string.contains(substring)` — Check if a string contains a substring (returns `bool`)

**Codegen improvements:**
- Track `split()` results as array variables (not string variables)
- Clone string variables when passed to functions (Rust move semantics)
- Add `&` prefix for variable arguments in pattern-based string methods (`contains`, `startsWith`, `endsWith`, `split`, `replace`)
- Remove unnecessary parentheses around `File.read`/`write`/`append` match expressions
- Extend `is_file_call()` to recognize `Dir.list` for error binding unwrapping

**Documentation:**
- Added Dir module to `docs/language-reference/file-io.md`
- Added `contains()` to `docs/language-reference/stdlib/strings.md`
- Created `docs/language-reference/stdlib/system.md` (Sys.args, Sys.env, Sys.exit)
- Updated `docs/QUICK_REFERENCE.md` with Dir, Sys, and contains()
- Updated `docs/language-reference/stdlib/README.md` index

**Demo app:**
- `buscador/` — Recursive grep-like text search tool written in Liva
  - Uses `Sys.args()`, `Dir.isDir()`, `Dir.list()`, `File.read()`, `string.contains()`, `string.split()`

**Tests:**
- 290 tests passing (2 new: `test_dir_list_and_isdir`, `test_string_contains`)

---

## [Unreleased] - v1.2.0-dev

### Added - Session 17: Enum Types & Release v1.2.0 🎯

**Enum Types (algebraic data types):**
- `enum` keyword for defining sum types with variants
- Unit variants: `enum Color { Red, Green, Blue }`
- Data variants with named fields: `enum Shape { Circle(radius: number), Rectangle(width: number, height: number), Point }`
- Enum variant construction: `Color.Red`, `Shape.Circle(5)`
- Pattern matching on enums in `switch` expressions
- Destructuring bindings: `Shape.Circle(r) => r * r`
- Auto-generated `#[derive(Debug, Clone, PartialEq)]` for all enums
- Auto-generated `Display` impl for unit-only enums
- Enum as function parameters and return types
- Full formatter support (`--fmt`)
- 5 snapshot tests + end-to-end test

**Release v1.2.0 published:**
- Fixed RPM path in release workflow (dynamic `find` instead of hardcoded path)
- All 7 release assets: `.deb`, `.rpm`, `.tar.gz` (Linux/macOS), `.zip` (Windows), checksums
- Rustfmt enforced in CI (removed `continue-on-error`)
- All 5 CI jobs green (Ubuntu, macOS, Windows, Clippy, Rustfmt)

**Files modified:**
- `src/lexer.rs` — Added `Token::Enum` keyword
- `src/ast.rs` — `TopLevel::Enum`, `EnumDecl`, `EnumVariant`, `EnumField`, `Pattern::EnumVariant`
- `src/parser.rs` — `parse_enum_decl` method, enum variant pattern parsing
- `src/semantic.rs` — Register enums in type system, pattern handling
- `src/lowering.rs` — Skip enums (bypass IR like classes)
- `src/codegen.rs` — `generate_enum`, enum variant construction, pattern matching
- `src/formatter.rs` — `format_enum_decl`, enum variant patterns
- `src/lsp/symbols.rs` — Handle `TopLevel::Enum` variant
- `.github/workflows/ci.yml` — Rustfmt enforced
- `.github/workflows/release.yml` — Fixed RPM path
- `tests/codegen_tests.rs` — 5 new snapshot tests

### Added - Session 16: CI/CD & Cross-Platform Releases 📦

**GitHub Actions CI hardened for cross-platform support:**
- CI runs on Ubuntu, macOS, and Windows with `fail-fast: false`
- Clippy linting (advisory mode, doesn't block CI)
- Rustfmt check (continue-on-error until codebase is formatted)
- All 278 tests pass on all 3 platforms

**Release workflow with multi-platform packaging:**
- Triggered on `v*` tags (e.g., `v1.2.0`)
- Builds 4 targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`
- `.deb` package via `cargo-deb` (Ubuntu/Debian)
- `.rpm` package via `cargo-generate-rpm` (Fedora/RHEL)
- `.tar.gz` archives for Linux and macOS
- `.zip` archive for Windows
- SHA-256 checksums for all artifacts
- Auto-generated release notes with installation table
- Version synced from git tag to `Cargo.toml`

**Cross-platform test fixes:**
- Normalized `\r\n` → `\n` line endings in all test helpers (lexer, parser, semantics, generics, desugar)
- Windows IO error message compatibility (`"No such file"` vs `"cannot find"`)
- LSP import tests marked `#[cfg(unix)]` for URI format differences

**Project metadata & packaging:**
- `Cargo.toml`: Added homepage, repository, license (MIT), keywords, categories
- `Cargo.toml`: `[package.metadata.deb]` and `[package.metadata.generate-rpm]` sections
- `LICENSE`: MIT License added
- `README.md`: CI/Release badges, complete installation section (5 platforms), uninstall instructions, Build from Source section

**Files modified:**
- `.github/workflows/ci.yml` — Clippy advisory, Rustfmt continue-on-error
- `.github/workflows/release.yml` — Complete rewrite with .deb/.rpm support
- `Cargo.toml` — Version 1.2.0, packaging metadata
- `LICENSE` — New MIT license file
- `README.md` — Badges, installation section, build from source
- `src/main.rs` — Cross-platform error message test
- `src/lsp/imports.rs` — `#[cfg(unix)]` on URI-dependent tests
- `tests/lexer_tests.rs` — `\r\n` normalization
- `tests/parser_tests.rs` — `\r\n` normalization
- `tests/semantics_tests.rs` — `\r\n` normalization
- `tests/generics_parser_tests.rs` — `\r\n` normalization
- `tests/desugar_tests.rs` — `\r\n` normalization

### Added - Session 15: Dogfooding & Bug Fixes 🐛

**Comprehensive dogfooding with "Student Grade Tracker" program (~300 lines):**
- Exercises ALL major Liva features: constants, data classes, interfaces, switch expressions with range patterns, classes, array/string methods, error handling, string templates, Math constants, break/continue, inclusive ranges, visibility
- Found and fixed **9 bugs** (#63-#74) across parser, semantic analysis, and code generation

**Bugs Fixed (9):**
- **Bug #63**: `return` without value before `}` caused parse error — check for `RBrace` as statement terminator
- **Bug #64**: Uppercase const + `{ continue }` misinterpreted as struct literal — added lookahead verification
- **Bug #65**: `.length` on member/method expressions rejected — added to `expr_supports_length()`
- **Bug #66**: Data class `Display` had unescaped braces in `write!()` — use `push_str` with escaped braces
- **Bug #67**: Data class `new()` had no parameters — generate field-based constructor
- **Bug #68**: Switch expression string arms returned `&str` — added `.to_string()`
- **Bug #69**: `this.field[i].prop` generated bracket notation — extended typed array detection
- **Bug #70**: Method `fail` didn't produce `Result` type — added `contains_fail` check to methods
- **Bug #71**: Method bodies didn't pre-analyze mutated variables — added `mutated_vars` analysis
- **Bug #74**: For loops consumed collections (ownership) — added `.clone()` for variable iterables

**6 new regression tests (tests/codegen_tests.rs):**
- `test_bug63_return_without_value` — return without value in void function
- `test_bug64_const_continue_struct_literal` — const + continue not misread as struct
- `test_bug66_data_class_display_and_constructor` — data class Display + constructor
- `test_bug68_switch_string_literals` — switch expression string arms
- `test_bug70_method_fail_result` — method with fail generates Result type
- `test_bug74_for_loop_ownership` — for loop doesn't consume collection

**Total tests: 278** (up from 272). Codegen snapshot tests: 86 (up from 80).

**Files modified (4 source files + tests):**
- `src/parser.rs` — Bug #63 (return without value), Bug #64 (struct literal lookahead)
- `src/semantic.rs` — Bug #65 (.length on member/method expressions)
- `src/codegen.rs` — Bugs #66-#71, #74 (data class, switch, methods, for loops)
- `tests/codegen_tests.rs` — 6 new regression tests
- 14 snapshot files updated/created

### Added - Session 14: 5 New Language Features 🚀

**5 pending language features implemented, tested, and documented:**

1. **`break` / `continue`** — Loop control flow statements
   - `break` exits the innermost loop immediately
   - `continue` skips to the next iteration
   - Works in `while` and `for` loops
   - Full pipeline: lexer → parser → AST → IR → lowering → codegen → formatter → LSP completions

2. **`..=` inclusive range in expressions** — `for i in 1..=10 { ... }`
   - Previously only supported in `switch` pattern matching
   - Now works as an expression range (e.g., `for` loops)
   - New `BinOp::RangeInclusive` in AST and `ir::Expr::RangeInclusive` in IR
   - Generates Rust `..=` operator

3. **`Math.PI` / `Math.E`** — Mathematical constants
   - `Math.PI` → `std::f64::consts::PI` (3.141592653589793)
   - `Math.E` → `std::f64::consts::E` (2.718281828459045)
   - Detected as member access on `Math` object

4. **`[string].join(separator)`** — Array join method
   - `words.join(", ")` → `words.join(", ")`
   - Works on string arrays, generates Rust's native `.join()` 

5. **`data` class sugar** — Auto-generated constructors and derives
   - `data Point { x: number, y: number }` — no constructor needed
   - Auto-generates: constructor, `PartialEq`, and `Display` impl
   - `data` is a **contextual keyword** — can still be used as a variable name
   - Supports methods: `data Color { r: number; sum() => r + g + b }`
   - `Display` auto-formats as `ClassName { field1: v1, field2: v2 }`

**8 new snapshot tests (tests/codegen_tests.rs):**
- `test_feature_math_constants` — Math.PI, Math.E usage
- `test_feature_array_join` — [string].join(sep)
- `test_feature_inclusive_range` — for i in 1..=5
- `test_feature_break` — while loop with break
- `test_feature_continue` — for loop with continue
- `test_feature_break_continue_combined` — both in same loop
- `test_feature_data_class` — basic data class
- `test_feature_data_class_with_methods` — data class with methods

**Total tests: 272** (up from 264). Codegen snapshot tests: 80 (up from 72).

**Files modified (10 source files):**
- `src/lexer.rs` — Added `Break`, `Continue` tokens; `data` as contextual keyword
- `src/ast.rs` — `BinOp::RangeInclusive`, `Stmt::Break/Continue`, `ClassDecl.is_data`
- `src/parser.rs` — Range expression parsing, break/continue, contextual `data` detection
- `src/ir.rs` — `ir::Expr::RangeInclusive`, `ir::Stmt::Break/Continue`
- `src/lowering.rs` — AST → IR for inclusive range, break, continue
- `src/codegen.rs` — All 5 features: Math.PI/E, join(), ..=, break/continue, data class
- `src/semantic.rs` — Break/continue validation
- `src/formatter.rs` — Break/continue formatting, data class prefix
- `src/lsp/server.rs` — break/continue keyword completions
- `tests/codegen_tests.rs` — 8 new snapshot tests

### Added - Comprehensive Feature Test Coverage 🧪

**44 new snapshot tests documenting ALL supported Liva syntax (tests/codegen_tests.rs):**

Total tests: 264 (up from 220). Codegen snapshot tests: 72 (up from 28).

| Category | Tests | Features Covered |
|----------|-------|-----------------|
| Variables & Constants | 2 | `let`, `const`, type annotations, mutability, top-level `const` |
| Types | 2 | Primitives (`number`, `float`, `bool`, `string`, `char`), Rust types (`i8`, `i16`, `i64`, `u64`, `f32`, `usize`) |
| Operators | 3 | Arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), logical (`and`/`or`/`not`, `&&`/`||`/`!`) |
| Functions | 2 | One-liner `=>`, block, default params, type annotations, lambdas/closures |
| Control Flow | 3 | `if`/`else`, ternary `? :`, one-liner ternary in `=>` |
| Pattern Matching | 3 | Switch statement (`case X:`), switch expression (`X => val`), or-patterns (`1 \| 2 \| 3`) |
| Loops | 4 | `while`, `for` range (`0..5`), `for` array, one-liner `=>` for, `for par` parallel |
| Classes & Interfaces | 2 | Class with constructor/fields/methods, interface declaration, one-liner methods |
| Error Handling | 3 | `fail`, error binding (`let x, err = ...`), `or fail`, `try`/`catch` |
| Concurrency | 3 | `async` calls, `par` concurrent, `task async`, auto fire-and-forget |
| Collections | 3 | `map`, `filter`, `reduce`, `find`, `some`, `every`, `forEach`, `includes`, `indexOf`, `length`, `push`, `pop`, chaining |
| Strings | 3 | Template strings (`$"..."`), all string methods, concatenation patterns |
| Console & IO | 1 | `print`, `console.log`, `console.error`, `console.warn`, `console.success` |
| Math & Conversions | 2 | `Math.sqrt/pow/abs/floor/ceil/round/min/max/random`, `parseInt`, `parseFloat`, `toString` |
| JSON & HTTP | 2 | `JSON.parse`, `JSON.stringify`, `HTTP.get`, `HTTP.post` |
| Visibility | 1 | `_` prefix private fields/methods |
| Test Framework | 1 | `expect().toBe/toEqual/toBeTruthy/toBeFalsy/toBeGreaterThan/toBeLessThan/toBeGreaterThanOrEqual/toBeLessThanOrEqual`, `.not` negation |
| Generics | 1 | Generic functions `<T>` |
| Tuples | 1 | Tuple literals, types, access (`.0`, `.1`), return tuples |
| Type Aliases | 1 | `type X = Y` |
| String Concat | 1 | String + String, String + Number auto-conversion |

**Syntax discoveries documented via tests:**
- Switch **statements** use `case X:` (colon), switch **expressions** use `X =>` (arrow, no `case` keyword)
- `try`/`catch` requires parentheses: `catch (err) { }`
- Ternary is an expression; `if` is a statement only
- `for` ranges only support exclusive `..` (not `..=`)
- JSON typed parse uses `int` not `number`: `let x: [int], err = JSON.parse(...)`
- `describe` is reserved for test framework — don't use as function name

### Fixed - Session 13: Edge Case Codegen Bugs 🐛

**8 codegen bugs found and fixed via dogfooding (Bugs #55-#62):**

- **Bug #55**: `substring(start, maxLen - 3)` generated `max_len - 3 as usize` (wrong operator precedence). Fixed: wrap args in `(expr) as usize`.
- **Bug #56**: `forEach` on `[string]` function parameters generated `|&s|` causing move error on String. Fixed: track `TypeRef::Array` params in `typed_array_vars`/`array_vars`.
- **Bug #57**: Array literals `["hello", "world"]` generated `vec!["hello", "world"]` (`Vec<&str>` instead of `Vec<String>`). Fixed: add `.to_string()` suffix to string elements.
- **Bug #58**: `char.toString() + char.toString()` used `+` operator instead of `format!()`. Fixed: `expr_is_stringy()` now detects `.toString()`, `.toUpperCase()`, `.toLowerCase()`, `.trim()` calls.
- **Bug #59**: `this.items.filter(...)` in class methods failed — `get_base_var_name()` didn't handle `Expr::Member`. Fixed: extract property name from `this.field` + register class field types before method codegen.
- **Bug #60**: `filter(|&item| item == query)` failed: `&String == String`. Fixed: added `ref_lambda_params` tracking — lambda params declared with `&` get `*` dereference in `==`/`!=` comparisons.
- **Bug #61**: `print(reversed)` where `reversed` is `Vec<i32>` from a function used `{}` instead of `{:?}`. Fixed: added `array_returning_functions` tracking — variables from array-returning calls are tracked in `array_vars`.
- **Bug #62**: `found[0]` on filter result `Vec<String>` failed (cannot move out of index). Fixed: propagate element type from source array through `filter()`/`map()` results, auto `.clone()` for string arrays.

**New CodeGenerator fields:**
- `ref_lambda_params: HashSet<String>` — lambda params that are `&T` references (need `*` dereference in comparisons)
- `array_returning_functions: HashSet<String>` — functions that return `[T]` (for tracking result variables as arrays)

**Improved tracking:**
- `get_base_var_name()` handles `Expr::Member` (`this.field` → field name)
- Class fields registered in `array_vars`, `typed_array_vars`, `string_vars` before method generation
- Filter/map results inherit element type from source array via `typed_array_vars`
- `expr_is_stringy()` detects string-returning method calls (`.toString()`, etc.)
- `generate_params()` tracks array parameters in `typed_array_vars`
- Print handler uses `{:?}` for variables in `array_vars`

### Added - Phase 12: Test Framework 🧪

**12.1 Test Runner** ✅
- New `livac test` CLI flag to discover and run test files
- Auto-discovery: finds `*.test.liva` files recursively
- `--filter <name>` to run only tests matching a substring
- Colorized output: PASS (green), FAIL (red), SKIP (yellow)
- Verbose mode: `--verbose` shows individual test results (✓/✗)
- Summary: test count, file count, total time
- Exit code: 0 = all pass, 1 = any failure
- Build directory cleanup after test runs
- Codegen fix: `throw` in `test` blocks generates `panic!()` instead of `return Err()`
- Examples:
  ```bash
  livac test                          # Run all *.test.liva
  livac test tests/math.test.liva     # Run specific file
  livac test --filter "sum"           # Filter by test name
  ```

**12.2 Test Library (`liva/test`)** ✅
- Virtual built-in module `liva/test` — no filesystem files needed
- Jest-like API: `describe`, `test`, `expect` with fluent matchers
- Import syntax: `import { describe, test, expect } from "liva/test"`
- **`describe(name, callback)`** — groups tests into `#[cfg(test)] mod`
- **`test(name, callback)`** — defines a `#[test] fn`
- **Matchers:**
  - `expect(x).toBe(y)` → `assert_eq!(x, y)`
  - `expect(x).toEqual(y)` → `assert_eq!(x, y)`
  - `expect(x).toBeTruthy()` → `assert!(x)`
  - `expect(x).toBeFalsy()` → `assert!(!(x))`
  - `expect(x).toBeGreaterThan(y)` → `assert!(x > y)`
  - `expect(x).toBeLessThan(y)` → `assert!(x < y)`
  - `expect(x).toBeGreaterThanOrEqual(y)` → `assert!(x >= y)`
  - `expect(x).toBeLessThanOrEqual(y)` → `assert!(x <= y)`
  - `expect(x).toContain(y)` → `assert!(x.contains(&y))`
  - `expect(x).toBeNull()` → `assert!(x.is_none())`
  - `expect(x).toThrow()` → `assert!(std::panic::catch_unwind(|| { x }).is_err())`
- **Negation:** `expect(x).not.toBe(y)` → `assert_ne!(x, y)` (all matchers support `.not`)
- **Lifecycle hooks:** `beforeEach`, `afterEach`, `beforeAll`, `afterAll`
- Virtual module system: `is_virtual_module()`, sentinel paths, semantic validation
- Parser changes: `Token::Test` as identifier in expression contexts, `Token::Not` as method name for `.not` chains
- New AST variant: `TopLevel::ExprStmt(Expr)` for top-level expression statements

**12.3 Lifecycle Hooks** ✅
- **Auto-invocation**: `beforeEach`/`afterEach` are automatically called at the start/end of every `test()` in the same `describe()` scope
- **Nested describe scoping**: hooks from parent `describe()` blocks are inherited by nested `describe()` blocks
  - Parent `beforeEach` runs first, then inner `beforeEach`
  - Inner `afterEach` runs first, then parent `afterEach`
- **Hook stack**: `test_hooks_stack` in CodeGenerator tracks active hooks per describe depth
- **Depth-based naming**: nested hooks generate unique function names (`before_each`, `before_each_1`, etc.)
- `beforeAll`/`afterAll` generate helper functions (module-level setup/teardown)
- 6 new codegen tests covering all hook scenarios
- E2E example: `examples/tests/lifecycle.test.liva`
- Example:
  ```liva
  import { describe, test, expect } from "liva/test"

  add(a: int, b: int): int => a + b

  describe("Math", () => {
      test("addition", () => {
          expect(add(2, 3)).toBe(5)
          expect(add(-1, 1)).toBe(0)
      })

      test("negation", () => {
          expect(add(1, 1)).not.toBe(3)
      })
  })
  ```

**12.4 Async Test Support** ✅
- **Auto-detection**: tests with `async` calls or `await` expressions automatically generate `#[tokio::test]` + `async fn`
- **Mixed sync/async**: sync tests use `#[test]`, async tests use `#[tokio::test]` — within the same `describe` block
- **Async lifecycle hooks**: `beforeEach`/`afterEach` containing async calls generate `async fn` + `.await` on invocation
- **Pending task resolution in expect chains**: fixed `expr_uses_var` to traverse `MethodCall` expressions (enables `expect(asyncResult).toBe(...)`)
- **Test runner**: counts both `#[test]` and `#[tokio::test]` for test discovery
- AST-level async detection: `ast_lambda_body_has_async`, `ast_stmt_has_async`, `ast_expr_has_async` helper functions
- 5 new codegen tests for async test scenarios
- E2E example: `examples/tests/async.test.liva`
- Example:
  ```liva
  import { describe, test, expect } from "liva/test"

  fetchUser(id: int): string => "User " + id.toString()

  describe("Async Tests", () => {
      test("sync test", () => {
          expect(2 + 3).toBe(5)
      })

      test("async fetch", () => {
          let user = async fetchUser(1)
          expect(user).toBe("User 1")
      })
  })
  ```

**Phase 12 Complete** ✅ — Full Jest-like test framework for Liva

---

## [Unreleased] - v1.1.0-dev

### Added - Phase 11: Syntax Sugar & Ergonomics 🍬

**11.1 `or fail` Operator** ✅
- New error propagation syntax: `let x = expr or fail "message"`
- Shorthand for fallible expressions that should fail on error
- Works with HTTP, File, JSON and any fallible function
- Examples:
  ```liva
  let response = HTTP.get(url) or fail "Connection error"
  let content = File.read("config.json") or fail "Cannot read config"
  let data = JSON.parse(text) or fail "Invalid JSON"
  ```

**11.2 One-liner `=>` for Control Flow** ✅
- `if condition => expr` - Single expression if
- `for item in items => expr` - Single expression for loop  
- `while condition => expr` - Single expression while loop
- `if cond => expr else => expr` - If-else one-liner
- Examples:
  ```liva
  if age >= 18 => print("Adult")
  for item in items => print(item)
  while running => tick()
  if valid => save() else => fail "Invalid"
  ```

**Additional Improvements**
- Top-level `const` declarations now supported
- `not` keyword works as unary operator (same as `!`)
- `&&` and `||` accepted alongside `and`/`or`
- Formatter simplifies `if err != ""` to `if err`

**11.3 Point-Free Function References** ✅
- Pass function names directly as callbacks: `items.forEach(print)`
- Works with `map`, `filter`, `forEach`, `find`, `some`, `every`
- Point-free `for =>` loops: `for item in items => print`
- Eliminates boilerplate lambdas for single-argument callbacks

**11.4 Method References with `::`** ✅
- New `object::method` syntax for instance method references
- Binds a method to a specific instance as a callback
- Works with `map`, `filter`, `forEach`, `find`, `some`, `every`
- Examples:
  ```liva
  let fmt = Formatter("Hello")
  let greetings = names.map(fmt::format)  // ["Hello: Alice", ...]
  names.forEach(fmt::format)
  ```
- Lexer: `::` DoubleColon token
- AST: `Expr::MethodRef { object, method }` node
- Semantic validation: checks class & method existence
- Codegen: generates closures with proper type conversions

---

## [1.0.2] - 2026-02-06 🎨

### Added - Phase 10: Code Formatter

**New `src/formatter.rs` module (~1500 lines)**
- AST-based pretty-printing with canonical style
- Comment preservation (standalone and inline)
- Configurable: indent_size, max_width, operator style
- Full language coverage: all Liva constructs
- 24 unit tests covering all patterns

**CLI Integration**
- `livac fmt file.liva` - Format file in place
- `livac fmt --check file.liva` - Check if formatting needed (exit 1 if not)

**LSP Integration**
- `textDocument/formatting` handler
- Uses editor's tab_size setting
- Works in VS Code / Cursor via extension

**CI Modernization**
- Updated to actions/checkout@v4
- Multi-platform release workflow
- Builds for: Linux x64, macOS x64, macOS ARM64, Windows x64
- Automatic GitHub releases with SHA256 checksums

---

## [1.0.0] - 2026-02-04 🎉

### 🚀 First Stable Release!

After extensive dogfooding with 10+ real CLI applications and fixing all 54 discovered bugs,
Liva v1.0.0 is ready for production use.

### Highlights
- **100% bug-free**: All 54 bugs found during dogfooding have been fixed
- **Complete language features**: Variables, functions, classes, generics, async/await, parallel execution
- **Full standard library**: File I/O, HTTP client, JSON parsing, Math utilities
- **Complete LSP support**: Syntax highlighting, autocompletion, go-to-definition, diagnostics
- **Automatic trait inference**: Clone and Display bounds detected automatically

### What's in 1.0.0
- TypeScript-like syntax with Python simplicity
- Compiles to safe, performant Rust
- Hybrid concurrency: `async` for I/O, `par` for CPU
- Explicit error handling with `fail` and error binding
- Generic programming with automatic bound inference
- Module system with imports/exports

### Tested Applications Built with Liva
- GitHub CLI - HTTP + JSON + Arrays
- Config Tool - File I/O + JSON
- Task Manager - CRUD operations
- Notes App - Classes + Methods + File I/O
- Weather CLI - Real API integration
- Crypto Tracker - CoinGecko API
- Todo API - RESTful operations
- Log Analyzer - Pattern matching
- Generic Tests - Box<T>, Stack<T>, Pair<A,B>

---

## [0.11.25] - 2025-02-04

### Fixed - Generic Bounds Inference & Trait Fixes 🐛

Major improvements to generic programming support with automatic trait bound inference.

**Bug #41: pop() returns Option<T>** ✅
- Array `pop()` method now adds `.expect("pop from empty array")` suffix
- Added `Stmt::VarDecl` case to mutation detection for proper `mut` inference
- Variables using `pop()` in initialization now correctly marked as mutable

**Bug #42: Generic array indexing with i32** ✅
- Now wraps entire index expression in parentheses before `as usize`
- `self.items[len - 1]` → `self.items[(len - 1) as usize]`
- Extended to handle `Expr::Member` (self.items) patterns

**Bug #44: Trait Eq uses Copy instead of Clone** ✅
- Changed trait bounds from `Copy` to `Clone` for Eq, Ord, Neg, Not traits
- Now generates `PartialEq + Clone` which works with String and non-Copy types
- Updated `src/traits.rs` with corrected trait definitions

**Bug #45-46: Automatic Clone bound inference** ✅
- Added `infer_type_param_bounds()` function in codegen
- Analyzes methods returning `T` from `this.field` or `this.items[i]`
- Automatically adds `Clone` bound to type parameters when needed
- Extended `expr_is_self_field()` to detect array indexing on self fields
- Adds `.clone()` suffix for array indexing returns

**Bug #54: Automatic Display bound inference** ✅
- Added `block_uses_type_in_template()` and `expr_uses_type_in_template()` functions
- Detects generic field usage in string templates `$"...{this.value}..."`
- Automatically adds `std::fmt::Display` bound to type parameters

### Technical Changes
- New helper functions in codegen.rs for bound inference:
  - `infer_type_param_bounds()` - Analyzes class for required bounds
  - `type_contains_param()` - Checks if TypeRef contains type parameter
  - `method_returns_self_field_of_type()` - Detects return patterns
  - `block_uses_type_in_template()` - Detects Display requirements
- Trait definitions updated in src/traits.rs to use Clone instead of Copy

### Test files added
- `bug41_pop_test.liva` - pop() with proper unwrap
- `bug42_generic_index_test.liva` - Generic Stack with array indexing
- `bug44_eq_clone_test.liva` - Eq constraint with String
- `bug54_display_test.liva` - Generic fields in templates

## [0.11.24] - 2025-02-04

### Fixed - Division & Template Bug Fixes 🐛

**Bug #52: Integer division with float return type** ✅
- Functions returning `float` but dividing integers now cast correctly
- Added `current_return_type` tracking in CodeGenerator
- Division `return x / y` in `-> f64` function generates `(x) as f64 / (y) as f64`
- Complex expressions like `(a + b) / 2` also handled correctly

**Bug #53: Field access in string templates** ✅
- Already fixed by Bug #51 from v0.11.23
- `$"{results[0].value}"` correctly generates `results[0].value`
- No longer uses `get_field()` for typed arrays

**Test files added:**
- `bug52_division_test.liva` - Integer division to float
- `bug53_template_test.liva` - Field access in templates

## [0.11.23] - 2025-02-03

### Fixed - Parallel & Filter Bug Fixes 🐛

Major bug fixes for generics, parallel operations, and field access patterns.

**Bug #43: mut inference for class instance methods** ✅
- Variables calling mutating methods (`push`/`pop`) now correctly detected as needing `mut`
- Fixed name sanitization in `collect_mutated_vars_in_expr` to match VarDecl lookup
- `let stack = Stack()` + `stack.push(x)` → `let mut stack`

**Bug #47-49: Parallel filter/reduce reference handling** ✅
- Parallel `filter()` now generates proper `|&&x|` pattern for dereferencing
- Parallel `reduce()` generates correct Rayon pattern: `.fold(|| identity, |acc, x| ...).reduce(|| identity, |a, b| a + b)`
- Added `.copied()` for Copy types before fold in parallel operations

**Bug #50: Regular filter() dereference** ✅
- Array literals now tracked with element types in `typed_array_vars`
- `[1,2,3]` tracked as "i32" type for proper Copy detection
- Generates `filter(|&&x| ...)` with `.copied().collect()` for Copy types

**Bug #51: Array indexing + field access** ✅
- `results[0].value` now generates direct field access instead of JSON bracket notation
- Detects typed arrays with class elements
- Added `.clone()` for String fields to avoid move errors

**Test files added:**
- `bug43_generic_test.liva` - Stack<T> with push/pop
- `bug47_parallel_test.liva` - Parallel filter/reduce
- `bug50_filter_test.liva` - Regular filter with primitives
- `bug51_field_access_test.liva` - Array indexing + field access

## [Unreleased]

### Dogfooding - Generics & Parallel Testing 🧪

Comprehensive testing of generics and parallel features revealed working functionality and documented areas for improvement.

**What Works Well:**
- ✅ Basic generics: `Box<T>`, `Pair<A,B>`, `Triple<X,Y,Z>`
- ✅ Nested generics: `Box(Pair(1, "one"))`
- ✅ Generic factory functions: `boxInt(n): Box<number>`
- ✅ Importing generic classes from other modules
- ✅ Parallel `map()` operations on arrays
- ✅ Regular `reduce()` with accumulators
- ✅ Generic classes with different type instantiations
- ✅ Regular and parallel `filter()` with proper dereference patterns
- ✅ Parallel `reduce()` with correct Rayon fold+reduce pattern
- ✅ Array indexing with direct field access for typed arrays

**Documented Issues for Future Work (Bugs #41-54):**
- Generic Stack<T> with array operations needs more work
- ~~Parallel filter/reduce need reference handling improvements~~ FIXED v0.11.23
- Generic trait bounds need refinement (Clone, Display)
- ~~Array indexing + field access combination needs fixes~~ FIXED v0.11.23

**Test Files Created:**
- `examples/generics_parallel_test/src/test1_stack.liva` - Box, Pair, Triple
- `examples/generics_parallel_test/src/test2_pair_methods.liva` - CacheEntry
- `examples/generics_parallel_test/src/test3_parallel.liva` - par.map
- `examples/generics_parallel_test/src/test4_parallel_generics.liva` - Stats + parallel
- `examples/generics_parallel_test/src/test5_import_generics.liva` - multi-file imports
- `examples/generics_parallel_test/src/lib/containers.liva` - generic library

## [0.11.22] - 2026-02-03

### Fixed - Wildcard Imports with Alias 🐛

**Bug #40**: Wildcard imports (`import * as alias`) now work correctly.

- **Problem**: `alias.function()` generated field access syntax instead of module path
- **Root Cause**: Module alias calls weren't recognized as module access
- **Fix**: Added `module_aliases` HashMap to track alias → module_name mappings

**Changes made:**
1. Added `module_aliases: HashMap<String, String>` to CodeGenerator
2. Register aliases when processing wildcard imports in `generate_entry_point()`
3. New `generate_module_function_call()` generates `module::function()` syntax
4. Proper type conversions for arguments (`.to_string()` for string literals)

**Example fixed:**
```liva
import * as mathlib from "./lib/math.liva"
import * as str from "./lib/strings.liva"

main() {
    print(mathlib.add(10, 20))        // Now works!
    print(str.concat("foo", "bar"))   // Now works!
}
```

### Tested - Import System Validation ✅

Comprehensive import testing confirmed working:
- ✅ Named function imports: `import { add, subtract } from "./math.liva"`
- ✅ Named class imports: `import { Circle, Rectangle } from "./shapes.liva"` 
- ✅ Wildcard with alias: `import * as math from "./lib/math.liva"`
- ✅ Mixed import styles in same file
- ✅ Private symbol rejection (E4007): `_prefixed` symbols cannot be imported
- ✅ Cross-module imports from subdirectories (`./lib/`, `./utils/`)

## [0.11.21] - 2026-02-03

### Fixed - JSON.stringify Without Error Binding 🐛

**Bug #39**: `JSON.stringify` without error binding now works correctly.

- **Problem**: `let json = JSON.stringify(obj)` generated tuple instead of string
- **Root Cause**: JSON.stringify returns `(Option<String>, String)` for error handling
- **Fix**: When used without error binding, extract value with `.0.unwrap_or_default()`

**Example fixed:**
```liva
let json = JSON.stringify({ title: "Test", userId: 1 })
HTTP.post(url, json)  // Now works - json is String, not tuple
```

Also removed unnecessary parentheses around `match` expressions for cleaner generated code.

## [0.11.20] - 2026-02-03

### Improved - Cleaner Generated Code 🧹

**Smarter mutability analysis and dead code suppression:**

- ✅ **Mutability Analysis**: Variables are now only declared as `mut` when actually mutated
  - Pre-analyzes function body to detect assignments and mutating method calls
  - Uses heuristics for class methods (getters vs mutating methods)
  - Eliminates 12+ "variable does not need to be mutable" warnings
  
- ✅ **Conditional liva_rt import**: Modules only import `liva_rt` when they use it
  - Eliminates "unused import: crate::liva_rt" warnings in simple modules
  
- ✅ **Dead code suppression for runtime**: `#[allow(dead_code)]` on `liva_rt` module
  - Runtime library functions that aren't used no longer produce warnings
  - User code still gets appropriate "unused" warnings

**Result**: 67% reduction in Rust warnings (48 → 16 for github-dashboard)

## [0.11.19] - 2026-02-03

### Fixed - JSON Value Conversion Methods 🐛

**JsonValue methods now return direct values:**

- ✅ **Bug #38**: `item.asString()`, `item.asBool()`, etc. now return direct values
  - **Problem**: `asString()` returned `Option<String>`, causing type errors in concatenation
  - **Root Cause**: No automatic unwrap for JSON value conversion methods
  - **Fix**: Add `.unwrap_or_default()` after `as_string()`, `as_bool()`, `as_i32()`, `as_f64()`

**Example fixed:**
```liva
let title = item["title"].asString()  // Now returns String, not Option<String>
print("Title: " + title)              // Works directly!
```

## [0.11.18] - 2026-02-03

### Fixed - Array join() Method 🐛

**join() method argument handling:**

- ✅ **Bug #37**: `array.join(", ")` no longer adds `.to_string()` to the separator
  - **Problem**: Generated `arr.join(", ".to_string())` but Rust's join expects `&str`
  - **Root Cause**: String literals were converted to String for all methods
  - **Fix**: Added `join` to the list of methods that keep `&str` arguments

**Example fixed:**
```liva
let names = ["a", "b", "c"]
print(names.join(", "))  // Now works!
```

## [0.11.17] - 2026-02-03

### Fixed - String Variables in Constructor Calls 🐛

**String ownership when passing to constructors:**

- ✅ **Bug #32**: String variables are now cloned when passed to constructors
  - **Problem**: `let p = Person(myName); print(myName)` failed because `myName` was moved
  - **Root Cause**: String variables passed to class constructors transferred ownership
  - **Fix**: Clone string variables when passed as constructor arguments
  - This allows reusing string variables after passing them to constructors

**Example fixed:**
```liva
let name = "Alice"
let person = Person(name)
print(name)  // Now works! 'name' was cloned
```

## [0.11.16] - 2026-02-03

### Fixed - Method Calls on Binary Expressions 🐛

**Operator precedence for chained method calls:**

- ✅ **Bug #36**: `(arr.length - 1).toString()` now generates correct code
  - **Problem**: Generated `((arr.len() as i32)) - 1.to_string()` where `.to_string()` binds to `1` not the whole expression
  - **Root Cause**: Method call on binary expression needs extra parentheses for correct precedence
  - **Fix**: Detect when method call object is a Binary expression and wrap in parentheses
  - Generates: `(((arr.len() as i32)) - 1).to_string()`

**Real-world testing (Dogfooding):**
- CSV Parser fully functional with split, forEach, while loops, dynamic indexing, and arithmetic expressions

## [0.11.15] - 2026-02-03

### Fixed - Array Indexing with Variables 🐛

**Integer variables as array indexes:**

- ✅ **Bug #34**: `lines[i]` now works when `i` is an `int` variable
  - **Problem**: `lines[i]` generated `lines[i]` but Rust requires `usize` for Vec indexing
  - **Fix**: Detect array indexing with non-literal indexes and add `as usize`
  - Generates: `lines[i as usize]` for variable indexes

**String array indexing returns clone:**

- Also added `.clone()` when indexing `[string]` arrays
  - **Problem**: `let line = lines[i]` fails because `lines[i]` returns `&String`, not `String`
  - **Fix**: Detect string array indexing and add `.clone()`
  - Generates: `lines[i as usize].clone()`

**Real-world testing (Dogfooding):**
- CSV Parser with split, while loops, and dynamic indexing now fully functional

## [0.11.14] - 2026-02-03

### Fixed - forEach/map on String Arrays 🐛

**Lambda pattern for string arrays:**

- ✅ **Bug #35**: `parts.forEach(p => ...)` on `[string]` arrays now compiles correctly
  - **Problem**: Generated `|&p|` which attempts to move String out of reference
  - **Root Cause**: Variables declared as `[string]` weren't tracked in `typed_array_vars`
  - **Fix**: 
    1. Track array types from explicit type declarations (`let parts: [string]`)
    2. Track `.split()` method results as returning `[string]`
    3. When element type is "string", use `will_use_cloned = true` to generate `|p|` not `|&p|`

**Real-world testing (Dogfooding):**
- String manipulation with split, forEach now fully functional

## [0.11.13] - 2026-02-03

### Fixed - Length Property Chain Method Calls 🐛

**String length with method chaining:**

- ✅ **Bug #31**: `text.length.toString()` now compiles correctly
  - **Problem**: `text.length.toString()` generated `text.len() as i32.to_string()` which fails
  - **Root Cause**: Rust's `as` operator has lower precedence than method calls
  - **Fix**: Wrap the cast in parentheses: `(text.len() as i32).to_string()`
  - Updated all three `.length` handlers in codegen.rs (lines ~3251, ~3280, ~8250)

**Real-world testing (Dogfooding):**
- String manipulation patterns with length conversion to string

## [0.11.12] - 2026-02-03

### Fixed - indexOf on Class Fields 🐛

**String indexOf on this.field access:**

- ✅ **Bug #30**: `this.url.indexOf(query)` now works correctly
  - **Problem**: indexOf on class fields generated `.iter().position()` instead of `.find()`
  - **Root Cause**: Detection logic only checked direct variable names, not member access
  - **Fix**: Expanded `is_string_indexof` to handle `Expr::Member` with `this/self` object
  - Also adds `&` prefix for String variable arguments (Pattern trait requirement)

**Real-world testing (Dogfooding):**
- Bookmark Manager CLI with class-based search functionality

## [0.11.11] - 2026-02-03

### Fixed - Switch with String Patterns 🐛

**Pattern matching with strings:**

- ✅ **Bug #29**: Switch/match with string literals now works correctly
  - **Problem**: `switch level { case "INFO": ... }` failed because `level` (String) couldn't match `"INFO"` (&str)
  - **Fix**: Detect string-based switches (any case is a string literal) and add `.as_str()` to discriminant
  - Generates: `match level.as_str() { "INFO" => ... }`

**Real-world testing (Dogfooding):**
- Log Analyzer CLI - Tests switch/match, File.exists, for loops with strings
- All switch patterns with string literals working!

## [0.11.10] - 2026-02-03

### Fixed - String Indexing & Multi-File Imports 🐛

**String character access:**

- ✅ **Bug #28**: String indexing `s[i]` now works correctly
  - **Problem**: `s[i]` generated `s.get(i)` which doesn't compile for Rust strings (UTF-8)
  - **Fix**: Generates `s.chars().nth(i as usize).map(|c| c.to_string()).unwrap_or_default()`
  - Check for `string_vars` BEFORE calling `generate_expr(object)` to avoid double output
  - Full UTF-8 support for character-by-character string manipulation

**Real-world testing (Dogfooding):**
- Todo API Client with POST/PUT/DELETE - all working!
- Modular App with math.liva + strings.liva imports - fully functional
- String reverse, repeat, padLeft all working with char indexing

## [0.11.9] - 2026-02-03

### Fixed - JSON Null Comparison & Array Methods 🐛

**JsonValue null comparison:**

- ✅ **Bug #25**: JsonValue comparison with `null` now works correctly
  - **Problem**: `coin != null` generated `coin != None` which doesn't compile
  - **Fix**: Translate `jsonVar != null` to `!jsonVar.is_null()`
  - Translate `jsonVar == null` to `jsonVar.is_null()`
  - Works for all JsonValue variables tracked in `json_value_vars`

**JsonValue convenience methods:**

- ✅ **Bug #26**: Added `as_float()` method to JsonValue
  - **Problem**: Users expected `.as_float()` but only `.as_f64()` existed
  - **Fix**: Added `as_float()` returning `f64` directly (unwrapped with 0.0 default)
  - More ergonomic for common use cases

**Vec<JsonValue> length:**

- ✅ **Bug #27**: `Vec<JsonValue>` from `.as_array()` now uses `.len()`
  - **Problem**: `coinList.length` generated `.length()` (JsonValue method) instead of `.len()` (Vec method)
  - **Fix**: Track variables initialized with `.as_array()` in `array_vars`
  - Generates `.len() as i32` for proper type compatibility

**Real-world testing (Dogfooding):**
- Crypto Tracker CLI fully functional with CoinGecko API
- Commands: `price <coin>`, `top`, `search <query>`
- Live data for Bitcoin ($78,359), top 10 cryptos, coin search

## [0.11.8] - 2026-02-02

### Fixed - HTTP Client & JSON Array Access 🐛

**HTTP module naming:**

- ✅ **Bug #23**: `Http.get()` not recognized, only `HTTP.get()` worked
  - **Problem**: Case-sensitive module name matching excluded `Http` variant
  - **Fix**: Added case-insensitive matching for HTTP module methods
  - `Http.get()`, `HTTP.get()`, `http.get()` all now work correctly

**JSON array access improvements:**

- ✅ **Bug #24**: `as_array()` returned `Option<Vec<JsonValue>>` causing type mismatches
  - **Problem**: Calling `.len()` on `as_array()` result failed (private field on Option)
  - **Problem**: Calling `.get(0)` failed (method not found on Option type)
  - **Fix**: `as_array()` now returns `Vec<JsonValue>` directly (empty vec for non-arrays)
  - **Fix**: Array indexing uses `.get(n as usize).cloned().unwrap_or_default()`
  - More ergonomic API - no unwrap needed for array iteration

**Real-world testing (Dogfooding):**
- Weather CLI fully functional with real Open-Meteo API calls
- Geocoding + Weather data with nested JSON parsing
- Tested with London, Tokyo, New York - all working!

## [0.11.7] - 2026-02-02

### Fixed - Class & String Handling Improvements 🐛

**String literal and concatenation fixes:**

- ✅ **Bug #17**: String literals now generate `.to_string()` when initializing variables
  - **Problem**: `let x = "["` inferred as `&str`, caused type mismatch on reassignment
  - **Fix**: All string literal initializations now produce `String` type
  - Prevents `&str vs String` type errors when variable is later reassigned

- ✅ **Bug #18**: Variables initialized with strings now detected in concatenations
  - `expr_is_stringy()` now checks `string_vars` for Identifier expressions
  - Variable names properly sanitized (camelCase → snake_case) for tracking
  - Variables initialized with string concatenations also tracked as strings
  - Enables `format!()` usage for all string concatenations

**Class constructor and method improvements:**

- ✅ **Bug #19**: Constructor body parsing for field assignments
  - **Problem**: `constructor(noteTitle)` with `this.title = noteTitle` generated wrong field names
  - **Fix**: Parse `this.field = value` statements in constructor body
  - Generate correct `Self { field: value, ... }` initializers
  - Supports parameters with different names than fields

- ✅ **Bug #20**: Detect mutating method calls on self fields
  - **Problem**: Methods calling `this.notes.push(note)` used `&self` instead of `&mut self`
  - **Fix**: Detect mutating methods (push/pop/remove/clear/insert/sort/reverse/extend/retain/truncate)
  - Check `this.field.method()` pattern for mutation detection

- ✅ **Bug #22**: forEach lambda pattern for non-Copy types
  - **Problem**: `|&note|` pattern in forEach caused "cannot move out of shared reference"
  - **Fix**: Don't add `&` prefix for class instances in forEach
  - Properly handle typed arrays of objects

**Real-world testing (Dogfooding):**
- Built Task Manager CLI (File I/O + JSON + String handling)
- Built Notes App CLI (Classes + Methods + Arrays + File I/O + JSON)
- Both apps fully functional with add/list/clear commands

## [0.11.6] - 2026-02-02

### Added - System Module & CLI Support 🖥️

**New `Sys` module for CLI applications:**

- ✅ **`Sys.args()`**: Returns command-line arguments as `Vec<String>`
  - `let args = Sys.args()` → first element is program name
  - Special `native_vec_string_vars` tracking for proper indexing

- ✅ **`Sys.env(key)`**: Get environment variable value
  - Returns empty string if not found
  - `let home = Sys.env("HOME")`

- ✅ **`Sys.exit(code)`**: Exit process with status code
  - `Sys.exit(1)` for error exit

### Fixed - JSON Nested Access 🐛

- ✅ **Bug #14**: Nested JSON field access didn't work
  - **Problem**: `issue["user"]["login"]` generated invalid Rust code
  - **Before**: `issue.get_field("user").unwrap_or_default()["login"]` ❌
  - **After**: `issue.get_field("user").unwrap_or_default().get_field("login").unwrap_or_default()` ✅
  - Added detection for nested `Expr::Index` to chain `get_field()` calls

- ✅ **Bug #15**: Variables from JSON indexing not tracked as JsonValue
  - **Problem**: `let items = result["items"]; items.forEach(...)` failed
  - Variables from `result["key"]` now properly tracked in `json_value_vars`
  - Enables correct `forEach`/`map`/`filter` lambda pattern generation

- ✅ **Bug #16**: JSON access with string variable used wrong method
  - **Problem**: `config[key]` where `key: string` generated `.get(key)` instead of `.get_field(&key)`
  - Now detects if index variable is in `string_vars` and uses `get_field()` for object access
  - Works for both `Option<JsonValue>` and direct `JsonValue`

**Real-world testing:**
- Built GitHub CLI helper tool in Liva
- Commands: `user <username>`, `repos <username>`, `issues <owner/repo>`, `search <query>`
- Successfully tested against live GitHub API
- Built Config Tool testing File I/O + JSON combination

## [0.11.5] - 2026-02-02

### Fixed - JSON/HTTP Dogfooding Bug Fixes 🐛

**JsonValue improvements for real API usage:**

- ✅ **Bug #10**: `.as_str()` not found on JsonValue 
  - Changed codegen to use `.as_string().unwrap_or_default()` instead of `.as_str().unwrap_or("")`
  
- ✅ **Bug #11**: JsonValue Display showed JSON quotes around strings
  - Improved Display impl to extract string content without JSON quotes
  - `user["name"]` now displays as `John` instead of `"John"`
  
- ✅ **Bug #12**: Nested JSON bracket access not supported
  - Added `impl Index<&str> for JsonValue` to support `json["a"]["b"]["c"]` chained access
  - Uses Box::leak for safe static references in read-only JSON traversal
  
- ✅ **Bug #13**: JsonValue cannot compare with `true`/`false`
  - Added `impl PartialEq<bool> for JsonValue`
  - Now supports: `if todo["completed"] == true { ... }`
  - Added `impl PartialEq<&str> for JsonValue` for string comparisons

**Test Suite:**
- Created comprehensive API tests with JSONPlaceholder
- Tested: HTTP GET/POST, nested JSON, arrays, boolean comparisons
- All tests passing with real HTTP endpoints

## [0.11.4] - 2026-02-02

### Fixed - Dogfooding Bug Fixes 🐛

**Complete bug fixes from GitHub Dashboard dogfooding session:**

**Array Methods with Non-Copy Types:**
- ✅ `.filter()` and `.find()` now use `.cloned()` for class instances instead of `.copied()`
- ✅ Track typed arrays from array literals containing class constructors
- ✅ Lambda patterns adjusted: use `|x|` with `.cloned()` instead of `|&&x|` with `.copied()`

**Option<T> Handling from .find():**
- ✅ Variables from `.find()` now tracked as `Option<T>` in `option_value_vars`
- ✅ `x != null` now transforms to `x.is_some()` 
- ✅ `x == null` now transforms to `x.is_none()`
- ✅ Field access on Option results auto-unwraps: `found.name` → `found.as_ref().unwrap().name`

**Previous Fixes (v0.11.3):**
- ✅ Private field underscore prefix preserved in snake_case conversion
- ✅ `.length` on strings/arrays generates `.len() as i32`
- ✅ Methods modifying `this.field` generate `&mut self`
- ✅ Assigning from `this.field` auto-clones
- ✅ String templates with ternary expressions (use Display format)
- ✅ JSON.parse error binding tracks `err` in `string_error_vars`

**Summary:** All 9 bugs from dogfooding session now fixed! 🎉

## [0.12.0] - In Development

### Added - Language Server Protocol (LSP) Planning 📝

**Documentation Created:**
- ✅ `LSP_IMPLEMENTATION_PLAN.md` - Complete 9-phase implementation roadmap
  - Phase breakdown with time estimates (8-10 hours total)
  - Success criteria and testing strategy
  - Dependencies and rollout plan
  - ~400 lines of comprehensive planning
  
- ✅ `docs/lsp/LSP_DESIGN.md` - Architecture and design documentation
  - System architecture diagrams
  - Module structure (src/lsp/ with handlers)
  - Data structures (LivaLanguageServer, DocumentState, SymbolTable)
  - LSP capabilities matrix
  - Performance optimization strategies
  - ~600 lines of technical specifications
  
- ✅ `docs/lsp/LSP_USER_GUIDE.md` - End-user documentation
  - Quick start guide
  - Feature walkthroughs (completion, navigation, diagnostics)
  - Configuration options
  - Troubleshooting guide
  - Tips & tricks
  - ~900 lines of user-facing docs
  
- ✅ `docs/lsp/LSP_API.md` - API reference for contributors
  - Complete handler APIs
  - Data structure documentation
  - Code examples
  - Extension points
  - ~900 lines of API documentation

**Implementation Plan:**
- **Phase 1:** LSP Infrastructure (2 hours) - tower-lsp setup, server lifecycle
- **Phase 2:** Document Synchronization (1 hour) - didOpen, didChange, didSave handlers
- **Phase 3:** Diagnostics (1.5 hours) - Real-time error reporting
- **Phase 4:** Autocompletion (2 hours) - Context-aware completions
- **Phase 5:** Go to Definition (1 hour) - Navigation
- **Phase 6:** Find References (1 hour) - Symbol search
- **Phase 7:** Hover Information (0.5 hours) - Type info display
- **Phase 8:** Rename Symbol (1 hour) - Refactoring
- **Phase 9:** VS Code Integration (1 hour) - Client setup

**Key Technologies:**
- `tower-lsp` 0.20 - LSP framework
- `tokio` 1.x - Async runtime
- `dashmap` 5.5 - Concurrent document storage
- JSON-RPC over stdio for communication

**Architecture:**
- Document-centric with AST caching
- Incremental parsing for performance
- Symbol table for fast lookups
- Performance targets: <100ms completion, <500ms diagnostics

**Status:** Planning complete, implementation ready to begin  
**Progress:** 4/4 documentation files complete, 0/9 implementation phases complete

## [0.11.3] - 2025-01-28

### Added - Pattern Matching for Union Types ✨

**Pattern Matching Integration:**
- ✅ Type patterns in switch expressions: `n: int => expr`
- ✅ Automatic type narrowing in each match arm
- ✅ Full exhaustiveness checking for union patterns
- ✅ Wildcard pattern support: `_ => default`

**Syntax:**
```liva
let x: int | string = 42
let result = switch x {
  n: int => n * 2,      // n has type int here
  s: string => s.len()  // s has type string here
}
```

**Implementation:**
- ✅ AST extension: `Pattern::Typed { name, type_ref }`
- ✅ Parser: Recognizes `variable: type` pattern syntax
- ✅ Codegen: Generates proper Rust enum variant matches
  - `Union_i32_String::Int(n) => ...`
  - `Union_i32_String::Str(s) => ...`
- ✅ Semantic validation: Ensures exhaustiveness and type safety

**Multi-Type Unions:**
```liva
let value: int | string | bool = "hello"
switch value {
  n: int => "Number",
  s: string => "String",
  b: bool => "Boolean"
}
```

**Documentation:**
- ✅ Comprehensive pattern matching section in `union-types.md`
- ✅ Examples: type narrowing, exhaustiveness, wildcards
- ✅ Code generation details and best practices

**Phase 7.2 Complete:** Union types are now fully usable with pattern matching support.

## [0.11.2] - 2025-01-28

### Added - Union Types ✨

**Basic Union Types:**
- ✅ Syntax: `int | string`, `T | U | V`
- ✅ Type-safe sum types with automatic variant wrapping
- ✅ Inline union annotations: `let x: int | string = 42`
- ✅ Multi-type unions: `int | string | bool`

**Code Generation:**
- ✅ Generates Rust enums with proper variants: `Union_i32_String { Int(i32), Str(String) }`
- ✅ Auto-wrapping values in correct variants: `42` → `Union_i32_String::Int(42)`
- ✅ Automatic `.to_string()` conversion for string literals
- ✅ Implements `Debug`, `Clone`, and `Display` traits for all unions

**Type Safety:**
- ✅ Union flattening: `(A | B) | C` becomes `A | B | C`
- ✅ Duplicate removal: `int | int | string` becomes `int | string`
- ✅ Full semantic validation
- ✅ Integration with existing type system

**Documentation:**
- ✅ Complete specification in `docs/language-reference/union-types.md`
- ✅ Examples: Result<T>, Option<T>, discriminated unions
- ✅ Comparison with TypeScript, Rust, and Haskell

**Known Limitations:**
- ⚠️ Type aliases with unions (e.g., `type Result<T> = T | Error`) not yet supported at top level
- ⚠️ Pattern matching integration pending (Phase 7.2.6)

## [0.11.1] - 2025-01-28

### Added - Type Aliases ✨

**Basic Type Aliases:**
- ✅ Simple syntax: `type UserId = int`
- ✅ Tuple aliases: `type Point = (int, int)`
- ✅ Complex types: `type Matrix = [[int]]`
- ✅ Inline expansion during compilation (zero runtime overhead)

**Generic Type Aliases:**
- ✅ Single parameter: `type Box<T> = (T,)`
- ✅ Multiple parameters: `type Pair<T, U> = (T, U)`
- ✅ Proper type parameter substitution
- ✅ Nested generic aliases: `type IntBox = Box<int>`

**Type Safety:**
- ✅ Circular reference detection with E0701 error
- ✅ Type parameter count validation with E0702 error
- ✅ Full semantic validation during type checking
- ✅ Integration with existing type system (tuples, arrays, optionals, generics)

**Code Generation:**
- ✅ Type aliases expanded inline in generated Rust code
- ✅ No Rust type aliases generated (simpler codegen, zero overhead)
- ✅ Works with all type annotations (let bindings, parameters, return types)

**Documentation:**
- ✅ Complete specification in `docs/language-reference/type-aliases.md`
- ✅ Examples, best practices, and restrictions
- ✅ Comparison with TypeScript, Rust, and Haskell

## [0.11.0] - 2025-01-28

### Added - Tuple Types & Literals ✨

**Tuple Literals:**
- ✅ New syntax: `(10, 20)` for multi-element tuples
- ✅ Single-element tuples with trailing comma: `(42,)` vs `(42)` (grouped expression)
- ✅ Empty tuples (unit type): `()`
- ✅ Nested tuples: `((1, 2), (3, 4))`
- ✅ Type inference for tuple literals

**Tuple Types:**
- ✅ Type annotations: `(int, int)`, `(string, bool, float)`
- ✅ Function return types: `fn(): (int, int)`
- ✅ Heterogeneous types (mixed types in single tuple)
- ✅ Rust interop: Direct mapping to Rust tuples with zero overhead

**Tuple Member Access:**
- ✅ Numeric property access: `tuple.0`, `tuple.1`, `tuple.2`
- ✅ Works in all expressions: assignments, conditions, string templates
- ⚠️ Chained access requires parentheses: `(matrix.0).0` instead of `matrix.0.0`
  - Root cause: Lexer tokenizes `.0.0` as Dot + FloatLiteral(0.0)
  - Documented workaround in all guides

**Pattern Matching Integration:**
- ✅ Tuple patterns in switch expressions: `(x, y) => ...`, `(0, _) => ...`
- ✅ Binding patterns work: `(x, y) if x > y => ...`
- ✅ Wildcard patterns: `(_, y) => ...`
- ✅ Nested tuple patterns: `((a, b), c) => ...`
- ✅ All pattern types supported (literals, bindings, wildcards, guards)

**Code Generation:**
- ✅ Generates clean Rust tuple syntax: `(i32, i32)`
- ✅ Single-element tuple handling: `(i32,)` in Rust
- ✅ Tuple literal codegen: `(10, 20)`
- ✅ Member access codegen: `.0`, `.1` direct field access
- ✅ Pattern matching codegen: Rust match with tuple destructuring

**Implementation Details:**
- **Phase 1 (AST):** Added `Expr::Tuple` and `TypeRef::Tuple` variants
- **Phase 2 (Parser):** 
  - Tuple literals with comma disambiguation: `(x)` vs `(x,)`
  - Tuple type parsing: `(int, int)`, `(string, bool)`
  - Numeric member access: `IntLiteral` case in `parse_method_name()`
- **Phase 3 (Semantic):**
  - Type inference for tuples: builds `TypeRef::Tuple` from element types
  - Validation: tuple member access with bounds checking
  - Type checking: validates numeric indices, returns element type
- **Phase 4 (Codegen):**
  - Tuple literal generation with single-element comma handling
  - Direct field access generation: `.0`, `.1` instead of get_field()
  - Fixed console.log to pass format strings directly
- **Testing:** 6 comprehensive test files, 5 of 6 passing (83% success rate)

**Test Files:**
1. `test_tuple_literals.liva` ✅ PASSING - Basic creation, empty, single, nested
2. `test_tuple_types.liva` ✅ PASSING - Type annotations
3. `test_tuple_access.liva` ✅ PASSING - Member access (with parentheses for chained)
4. `test_tuple_functions.liva` ❌ FAILING - Return type inference issue
5. `test_tuple_patterns.liva` ✅ PASSING - Switch expression pattern matching
6. `test_tuple_nested.liva` ✅ PASSING - Complex nested structures

**Known Limitations (v0.11.0):**
- ⚠️ Chained tuple access requires parentheses: `(matrix.0).0` instead of `matrix.0.0`
  - Root cause: Lexer tokenizes `.0.0` as Dot + FloatLiteral(0.0) (greedy float tokenization)
  - Workaround documented: Use parentheses for chained access
  
- ⚠️ Tuple destructuring in let bindings broken: `let (x, y) = tuple` fails
  - Parser expects identifier after `let`, doesn't recognize tuple pattern
  - Workaround: Use direct access: `let x = tuple.0, y = tuple.1`
  
- ⚠️ String type annotations cause &str vs String mismatch
  - `getUserInfo(): (string, int, bool)` generates `(String, i32, bool)` but returns `(&str, i32, bool)`
  - Workaround: Use type inference instead of explicit string types in tuples
  
- ⚠️ Return type inference doesn't work for functions without explicit return type
  - Functions without return type default to `f64` instead of inferring tuple type
  - Workaround: Always specify explicit return types for tuple-returning functions

**Documentation:**
- ✅ `TUPLE_IMPLEMENTATION_PLAN.md` - Complete 6-phase implementation plan (518 lines)
- ✅ `docs/language-reference/types.md` - Updated with Tuple Types section
- ✅ `docs/language-reference/pattern-matching.md` - Updated tuple pattern status
- ✅ `docs/language-reference/functions.md` - Added Tuple Returns section
- ✅ `docs/guides/tuples.md` - Comprehensive tutorial (600+ lines)
  - Basic usage, pattern matching, best practices
  - When to use tuples vs structs
  - Common patterns and real-world examples
  - Known limitations and workarounds

**Statistics:**
- **Time:** 4 hours (100% of estimate)
- **Code changes:** 7 files modified (ast.rs, parser.rs, semantic.rs, codegen.rs, ir.rs, lowering.rs, liva_rt.rs)
- **Tests:** 6 test files created, 5 passing (83% success rate)
- **Documentation:** 1,500+ lines (implementation plan, language reference updates, tutorial)
- **Commits:** 1 feature commit (0742d6a)

**Use Cases:**
```liva
// Multiple return values
getCoordinates(): (int, int) {
    return (10, 20)
}

// Pattern matching
let point = (10, 20)
let location = switch point {
    (0, 0) => "origin",
    (0, y) => $"on Y axis at {y}",
    (x, 0) => $"on X axis at {x}",
    (x, y) => $"at ({x}, {y})"
}

// Nested tuples
let matrix = ((1, 2), (3, 4))
let elem = (matrix.0).0  // Access with parentheses
```

**Future Work (v0.11.1+):**
- Fix tuple destructuring in let bindings
- Fix chained access without parentheses (lexer improvement)
- Fix string type annotation mismatch
- Fix return type inference for tuples

## [0.10.5] - 2025-01-27

### Added - Or-Patterns & Enhanced Pattern Matching ✨

**Or-Patterns:**
- ✅ New syntax: `1 | 2 | 3 => "value"` matches multiple patterns with one arm
- ✅ Works with integers, strings, and all literal types
- ✅ Significantly reduces code duplication in switch expressions
- ✅ Example: `"Saturday" | "Sunday" => true` for weekend checking
- ✅ Can combine multiple or-patterns in same switch: `1|2|3 => "small", 4|5|6 => "medium"`

**Enhanced Exhaustiveness Checking:**
- ✅ Extended to support or-patterns correctly
- ✅ Integer exhaustiveness (E0902) now processes or-patterns recursively
- ✅ String exhaustiveness (E0903) validates or-patterns properly
- ✅ Type inference improved to detect types inside or-patterns
- ✅ All existing exhaustiveness checks continue to work with or-patterns

**Lexer & Parser Extensions:**
- ✅ Added `|` (Pipe) token to lexer for or-pattern syntax
- ✅ Parser extended with `parse_or_pattern()` method
- ✅ Recursive pattern parsing: `parse_pattern() → parse_or_pattern() → parse_single_pattern()`
- ✅ Tuple and Array pattern AST nodes added (foundation for future work)

**Code Generation:**
- ✅ Or-patterns generate clean Rust match syntax: `1 | 2 | 3 => ...`
- ✅ Display trait updated for all new pattern types
- ✅ Seamless integration with existing codegen infrastructure

**Semantic Validation:**
- ✅ Added pattern binding extraction for future tuple/array validation
- ✅ Added validation framework for nested patterns
- ✅ E0906 error code reserved for incompatible or-pattern bindings (future use with tuples)

**Documentation:**
- ✅ Updated `pattern-matching.md` with or-pattern section
- ✅ Added examples for integer and string or-patterns
- ✅ Documented exhaustiveness behavior with or-patterns
- ✅ Updated version to v0.10.5 across documentation

**Tests:**
- ✅ `test_or_patterns_simple.liva` - Validates or-pattern code generation
- ✅ `test_or_patterns_non_exhaustive.liva` - Validates E0902 with or-patterns
- ✅ All existing exhaustiveness tests continue to pass

**Impact:**
- Makes switch expressions more concise and readable
- Reduces boilerplate when matching multiple values
- Maintains type safety and exhaustiveness guarantees
- Foundation laid for tuple/array destructuring in v0.10.6

## [0.10.4] - 2025-01-27

### Added - Optional Fields & Default Values for JSON Parsing ✨

**Optional Fields with `?` Syntax:**
- ✅ New syntax: `field?: Type` declares optional fields in classes
- ✅ Generates `Option<T>` wrapper in Rust code
- ✅ Auto-adds `#[serde(skip_serializing_if = "Option::is_none")]` attribute
- ✅ Handles missing fields, null values, and present values seamlessly
- ✅ Perfect for real-world APIs with optional/nullable fields

**Default Values with `=` Syntax:**
- ✅ New syntax: `field: Type = value` declares fields with default values
- ✅ Supports all literal types: int, float, string, bool
- ✅ Automatic string conversion: `"text"` → `"text".to_string()` for string fields
- ✅ Works with both default and parameterized constructors
- ✅ Non-parameter fields use their init value in constructors

**Optional Fields with Default Values:**
- ✅ Combined syntax: `field?: Type = value` for optional fields with defaults
- ✅ Generates serde default functions: `fn default_{class}_{field}() -> Option<T>`
- ✅ Adds `#[serde(default = "default_function")]` attribute
- ✅ When JSON missing the field, serde uses default value instead of None
- ✅ Makes defaults available in destructuring patterns automatically

### Fixed - Optional Fields Bug Fixes 🐛

**Constructor Generation:**
- Fixed optional field constructors to generate `None` instead of `String::new()`
- Both default and parameterized constructors now correctly initialize optional fields
- Fixed default values to wrap in `Some()` when field is optional
- String literals in default values automatically converted to `String` type

**Object Destructuring:**
- Fixed optional fields in lambda destructuring for `forEach`, `map`, `filter`, etc.
- Optional fields now correctly unwrap with `.as_ref().map(|v| v.clone()).unwrap_or_default()`
- Required fields correctly use `.clone()` without unnecessary unwrapping
- Added `current_lambda_element_type` to track class types through lambda generation
- Works correctly with parallel operations (`.parvec().forEach`)

**Nested Struct Access:**
- Fixed issue where nested structs were incorrectly treated as JsonValue
- Destructured nested class fields (e.g., `address` from `User`) now correctly identified as class instances
- Member access on nested structs now generates correct code (e.g., `address.zipcode` instead of `address.get_field("zipcode")`)
- Added type tracking for destructured fields that are themselves class types

**Serde Default Integration:**
- Optional fields with default values now generate serde default functions
- Default values correctly applied when field is missing from JSON (not just in constructors)
- Generated code: `#[serde(default = "default_{class}_{field}")]`
- Solves issue where defaults only worked in constructors, not during JSON deserialization

**Real-World Testing:**
- Tested with JSONPlaceholder API integration
- User class with optional `username?: string` field works correctly
- Nested struct access (`address.zipcode`) works correctly in string templates
- Object destructuring in forEach properly handles mixed optional/required fields
- Optional fields with defaults (`algo?: string = "hola"`) show default value when missing from JSON

**Example of Fixed Behavior:**
```liva
User {
    id: u32
    name: string
    username?: string  // ✨ Optional field
}

main() {
    let users: [User], err = async HTTP.get("https://api.example.com/users").json()
    
    // ✅ Now works correctly with optional username
    users.parvec().forEach(({id, name, username}) => {
        console.log($"User {id}: {name} (@{username})")
    })
}
```

**Why Optional Fields Matter:**
- **Type Safety:** Explicitly document which fields can be absent/null
- **No More Crashes:** Missing fields don't cause JSON parse failures
- **Better DX:** Code shows intent - optional vs required fields
- **API Ready:** Handle real-world JSON APIs with nullable fields

**Example Usage:**
```liva
User {
    id: u32          // Required field
    name: String     // Required field
    email?: String   // ✨ Optional - can be null or absent
    age?: u32        // ✨ Optional - can be null or absent
}

main() {
    // Works with all fields present
    let json1 = "{\"id\": 1, \"name\": \"Alice\", \"email\": \"alice@example.com\"}"
    let user1: User, err1 = JSON.parse(json1)
    
    // Works with email missing
    let json2 = "{\"id\": 2, \"name\": \"Bob\"}"
    let user2: User, err2 = JSON.parse(json2)  // ✅ No error!
    
    // Works with email null
    let json3 = "{\"id\": 3, \"name\": \"Carol\", \"email\": null}"
    let user3: User, err3 = JSON.parse(json3)  // ✅ No error!
}
```

**Generated Rust Code:**
```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,  // ✅ Wrapped in Option<T>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
}
```

**Real-World Use Case:**
```liva
// API response with optional fields
Post {
    id: u64
    title: String
    content: String
    publishedAt?: String  // May not be published yet
    authorEmail?: String  // Author may not have public email
    likes?: u32           // New field, older posts don't have it
}

main() {
    let response, err = async HTTP.get("https://api.example.com/posts")
    if err == "" {
        let posts: [Post], parseErr = JSON.parse(response.body)
        // All posts parse successfully, regardless of which fields are present! ✅
    }
}
```

**Implementation Details:**
- **Parser:** Already implemented in v0.10.3 (detects `?` token after field name)
- **AST:** `FieldDecl.is_optional: bool` field tracks optional status
- **Codegen:** `generate_field()` wraps type in `Option<T>` when `is_optional=true`
- **Serde:** Auto-adds skip attribute for efficient serialization
- **Time:** ~45 minutes (as estimated in Phase 7.0.5)

**Files Modified:**
- `src/codegen.rs` - Updated `generate_field()` function (20 lines)
- Tests: `test_optional_fields.liva` (comprehensive 4-case validation)

**Statistics:**
- Code changes: +20 lines in codegen.rs
- Test coverage: 4 test cases (all fields, missing, null, multiple missing)
- Generated code: Clean Option<T> with proper serde attributes

---

## [0.10.3] - 2025-01-26

### Added - Parameter Destructuring 🎯

**Destructuring in Function Parameters:**
- ✅ Array destructuring in parameters: `printPair([first, second]: [int]) { ... }`
- ✅ Object destructuring in parameters: `printUser({name, age}: User) { ... }`
- ✅ Rest patterns in parameters: `processList([head, ...tail]: [int]) { ... }`
- ✅ Full code generation with temporary parameter names
- ✅ Works in both functions and methods
- ✅ Semantic validation for destructured parameters

**Destructuring in Lambda Parameters:**
- ✅ Array destructuring in lambdas: `pairs.forEach(([x, y]) => ...)`
- ✅ Object destructuring in lambdas: `users.forEach(({id, name}) => ...)`
- ✅ Works with all array methods: `forEach`, `map`, `filter`, `reduce`
- ✅ Works with parallel variants: `parvec().forEach(([x, y]) => ...)`
- ✅ Parser recognizes `[x, y] =>` and `{x, y} =>` as lambda starts
- ✅ Codegen inserts destructuring in both regular and special paths

**Example Usage - Array Destructuring:**
```liva
// Function with array destructuring parameter
printPair([first, second]: [int]): int {
    print("First:", first)
    print("Second:", second)
    return first + second
}

main() {
    let nums = [100, 200]
    let sum = printPair(nums)  // First: 100, Second: 200
    print("Sum:", sum)         // Sum: 300
}
```

**Example Usage - Lambda Destructuring:**
```liva
// Array destructuring in forEach
let pairs = [[1, 2], [3, 4], [5, 6]]
pairs.forEach(([x, y]) => {
    print("x=${x}, y=${y}, sum=${x + y}")
})

// Object destructuring in forEach
let users = [
    {id: 1, name: "Alice"},
    {id: 2, name: "Bob"}
]
users.forEach(({id, name}) => {
    print("User #${id}: ${name}")
})

// Works with map
let sums = pairs.map(([a, b]) => a + b)

// Works with filter
let filtered = pairs.filter(([x, y]) => x > 2)
```

**Implementation Details:**
- Parser creates `BindingPattern` for both `Param` and `LambdaParam`
- Both use `pattern: BindingPattern` instead of `name: String`
- Lambda parser updated to recognize destructuring patterns as lambda starts
- Codegen generates temporary names (`_param_0`, `_param_1`, etc.)
- Destructuring code inserted at function/lambda start with `let` statements
- Special array method path (forEach/map/filter) now includes destructuring support
- Semantic analyzer validates patterns and declares variables
- Codegen generates temporary parameter names (`_param_0`, `_param_1`)
- Destructuring code inserted at function/method entry
- Supports nested destructuring (coming soon)

### Changed
- AST: `Param.name: String` → `Param.pattern: BindingPattern`
- All usages of `param.name` migrated to `param.name()` method
- `generate_params()` now handles destructured parameters with temp names

### Technical
- Added `generate_param_destructuring()` for code generation
- Added `parse_param_pattern()` for parsing patterns without type annotations
- Added `declare_param_pattern()` for semantic validation
- Comprehensive design document in `docs/PHASE_6.5.1_PARAM_DESTRUCTURING_DESIGN.md`

## [0.10.2] - 2025-01-26

### Added - Destructuring Patterns 🎯

**Object and Array Destructuring:**
- ✅ Object destructuring: `let {x, y} = point`
- ✅ Object destructuring with rename: `let {name: userName, age: userAge} = person`
- ✅ Array destructuring: `let [first, second] = array`
- ✅ Array destructuring with skip: `let [a, , c] = array`
- ✅ Rest patterns in arrays: `let [head, ...tail] = items`
- ✅ Type annotations with destructuring: `let {x, y}: Point = point`
- ✅ Full semantic validation (field existence, duplicate bindings, type checking)
- ✅ Comprehensive parser, semantic, and codegen support

**Example Usage - Object Destructuring:**
```liva
let point = { x: 10, y: 20 }
let {x, y} = point
print("x:", x, "y:", y)  // x: 10 y: 20

// Rename bindings
let person = { name: "Alice", age: 30 }
let {name: userName, age: userAge} = person
print("userName:", userName)  // userName: Alice
```

**Example Usage - Array Destructuring:**
```liva
let numbers = [1, 2, 3, 4, 5]

// Basic destructuring
let [first, second] = numbers
print("first:", first)  // first: 1

// Skip elements
let [a, , c] = numbers
print("a:", a, "c:", c)  // a: 1 c: 3

// Rest patterns
let [head, ...tail] = numbers
print("head:", head)  // head: 1
// tail is [2, 3, 4, 5]
```

**Implementation Details:**
- New `BindingPattern` enum in AST (Identifier, Object, Array)
- Parser support with proper error handling
- Semantic validation ensures fields exist on known types
- Duplicate binding detection
- Codegen generates temporary variables to avoid move issues
- Works with both JSON objects and Rust structs

See `examples/destructuring_demo.liva` for complete examples.

## [0.10.1] - 2025-01-26

### Added - response.json() Method 🌐

**Ergonomic JSON Parsing from HTTP Responses:**
- ✅ `response.json()` method on Response objects (like JavaScript fetch API)
- ✅ Returns `(JsonValue, String)` tuple for easy error handling
- ✅ Alternative to `JSON.parse(response.body)`
- ✅ Works with typed JSON parsing: `let user: User, err = response.json()`
- ✅ Automatic serde derives for classes used with response.json()
- ✅ Cleaner, more intuitive API for REST consumption

**Example Usage - Basic:**
```liva
let response, err = HTTP.get("https://api.github.com/users/octocat")
if err { return }

// Parse JSON directly from response (like fetch API)
let json, parseErr = response.json()
if parseErr { return }

console.log($"User data: {json}")
```

**Example Usage - Typed:**
```liva
User {
    name: string
    email: string
    company: string
}

let response, err = HTTP.get("https://api.example.com/users/1")
if err { return }

// Automatic deserialization to User class
let user: User, jsonErr = response.json()
if jsonErr { return }

console.log($"User: {user.name} at {user.company}")
```

**Implementation:**
- Runtime (liva_rt.rs): Added `json()` method to Response struct
- Codegen: Extended `is_json_parse_call()` to detect `.json()` methods
- Codegen: Updated `generate_typed_json_parse()` to use `.body` for response.json()
- Codegen: Fixed `is_builtin_conversion_call()` tuple detection logic
- Semantic: Extended JSON.parse validation to include `.json()` calls
- Semantic: Tracks `.json()` calls with type hints for serde derives

### Fixed
- is_builtin_conversion_call() now correctly detects .json() as tuple-returning method
- Moved .json() check to beginning of match statement (was unreachable in else block)

### Documentation
- Updated docs/language-reference/http.md with response.json() documentation (+171 lines)
- Added response.json() examples for basic and typed parsing
- Updated all HTTP examples to use ergonomic response.json() API

### VSCode Extension v0.8.0
- Added 16 new HTTP snippets: httpget, hget, httppost, hpost, httpput, hput, httpdelete, hdel, httpjson, httppostjson, resjson, resjsonc, httptyped, httpstatus, httpfull, restapi
- Updated README with comprehensive HTTP Client documentation
- Added HTTP keywords: http, rest-api, web
- Total snippets: 103 (87 existing + 16 new HTTP snippets)

## [0.10.0] - 2025-01-25

### Added - Typed JSON Parsing (Complete) 🎉

**Type-Safe JSON Parsing with Type Hints:**
- ✅ Parse JSON directly into typed values without `.as_i32().unwrap()` calls
- ✅ Type hints support: `let data: [i32], err = JSON.parse(json)`
- ✅ All Rust primitive types supported: i8-i128, u8-u128, isize, usize, f32, f64, bool, String
- ✅ Arrays of typed values: `[i32]`, `[f64]`, `[String]`, etc.
- ✅ **Custom classes with serde derives (Phase 2)**
- ✅ **Nested classes with recursive dependency tracking (Phase 4)**
- ✅ **Arrays of custom classes**
- ✅ Clean error handling with `(Type, String)` tuple (no Option!)
- ✅ Single binding mode: `let data: [i32] = JSON.parse(json)` (panics on error)

**Example Usage - Primitives and Arrays:**
```liva
// OLD syntax (v0.9.x) - verbose with .unwrap()
let data = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n.as_i32().unwrap() * 2)

// NEW syntax (v0.10.0) - clean and type-safe! ✨
let data: [i32], err = JSON.parse("[1, 2, 3]")
let doubled = data.map(n => n * 2)  // No .unwrap() needed!
```

**Example Usage - Custom Classes (Phase 2):**
```liva
User {
    id: u32
    name: String
    age: i32
}

let userJson = "{\"id\": 1, \"name\": \"Alice\", \"age\": 30}"
let user: User, err = JSON.parse(userJson)

if err == "" {
    print($"User: {user.name}, age {user.age}")  // Direct field access!
}
```

**Example Usage - Nested Classes (Phase 4):**
```liva
Address {
    street: String
    city: String
}

User {
    name: String
    address: Address    // Nested class
}

Comment {
    text: String
    author: String
}

Post {
    title: String
    comments: [Comment]  // Array of classes
}

let json = "{\"title\": \"Hello\", \"comments\": [{\"text\": \"Great!\", \"author\": \"Bob\"}]}"
let post: Post, err = JSON.parse(json)
// Both Post and Comment automatically get serde derives!
```

**Phase 1 - Primitives and Arrays (4.5h):**
- Parser: Type hints already supported in let statements
- Semantic: `validate_json_parse_type_hint()` validates serializable types
- Codegen: Generates `serde_json::from_str::<T>` with proper error handling
- Support for all Rust integer types, floats, bool, String
- Arrays: `[T]` maps to `Vec<T>`

**Phase 2 - Custom Classes (1h):**
- AST: Added `needs_serde: bool` to `ClassDecl`
- Semantic: Tracks classes used with JSON.parse in `json_classes` HashSet
- Semantic: `mark_json_classes()` updates AST before codegen
- Codegen: Conditional serde derives: `#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]`
- Codegen: Tracks class instances in `class_instance_vars` for proper member access
- Cargo.toml: Automatically adds `serde = { version = "1.0", features = ["derive"] }`
- Note: Field names must match JSON keys exactly (no automatic camelCase/snake_case conversion)

**Phase 4 - Nested Classes (30min):**
- Semantic: `collect_class_dependencies()` - Recursively finds all class dependencies
- Semantic: `collect_type_dependencies()` - Handles TypeRef (Simple, Array, Optional)
- Semantic: `is_class_type()` - Distinguishes classes from primitives
- All dependent classes automatically get serde derives
- Supports arbitrary nesting depth
- Supports arrays of nested classes: `[Comment]` inside `Post`

**Semantic Validation:**
- Validates that types used with JSON.parse are serializable
- Recursive validation for arrays, optionals, and generics
- Checks class existence for custom types
- Validates nested class dependencies exist

**Code Generation:**
- Generates `serde_json::from_str::<T>(&json)` instead of JsonValue wrapper
- Error handling: `match` expression with Ok/Err branches
- Default values on error: Vec::new(), 0, 0.0, false, String::new(), Default::default()
- Single binding: generates `.expect("JSON parse failed")` for simplicity
- Direct field access for class instances (no `.get_field()`)

**Files Modified:**
- `src/ast.rs`: Added `needs_serde` field to ClassDecl
- `src/semantic.rs`: Added validation and dependency tracking (lines 2687-2840)
- `src/codegen.rs`: Added typed JSON parsing and serde support (lines 119-162, 1540-1720)
- `Cargo.toml`: Template updated to include serde dependency

**Test Files:**
- `test_json_typed_parse.liva`: Primitives and arrays
- `test_json_class_basic.liva`: Simple custom classes
- `test_json_snake_case.liva`: Field name matching demo
- `test_json_nested.liva`: Nested classes (User with Address)
- `test_json_nested_arrays.liva`: Arrays of nested classes (Post with [Comment])

**Documentation:**
- `/docs/language-reference/json.md`: Updated to v0.10.0 with comprehensive type-safe parsing section
- `/docs/guides/json-typed-parsing.md`: New 400+ line guide with examples, best practices, and troubleshooting

**Breaking Changes:**
- None! Old JsonValue syntax still works for untyped parsing

**Known Limitations:**
- Lambda parameters in forEach/map don't track class types (requires full type inference)
- Optional fields (`field?: Type`) not yet supported - use manual Option<T> workaround if needed

**Phase 3 Skipped:**
- Optional fields deferred as general language feature (not JSON-specific)
- `tests/integration/proj_json/test_map.liva`: Updated
- `tests/integration/proj_json/test_parvec_json.liva`: Updated

**Coming in Phase 2:**
- Custom classes with serde derive
- Snake_case field conversion
- Optional fields with `field?: Type`
- Default values with `field: Type = value`
- Nested classes

## [0.9.11] - 2025-01-25

### Fixed - JsonValue Parallel Execution

**JsonValue.parvec() Support:**
- ✅ Fixed parallel execution for JsonValue from JSON.parse()
- JsonValue now converts to Vec with `.to_vec().into_par_iter()` instead of `.par_iter()`
- Lambda patterns correctly use `|x|` (owned) instead of `|&x|` (reference) for JsonValue parallel iteration
- Complete HTTP → JSON → parvec workflow now fully functional

**Code Generation Improvements:**
- Detect `is_direct_json` flag for JsonValue from JSON.parse()
- Par/ParVec adapters: generate `.to_vec().into_par_iter()` for JsonValue
- Lambda pattern generation: extended to handle Par/ParVec with JsonValue (no & prefix)

**Example Usage:**
```liva
// Complete integration: HTTP + JSON + Parallel Processing
async fn fetch_and_process() {
    let res, err = async HTTP.get("https://jsonplaceholder.typicode.com/posts")
    let posts = JSON.parse(res.body)
    
    // Parallel processing of JSON array - NOW WORKS! ✅
    posts.parvec().forEach(post => {
        console.log($"Post {post.id}: {post.title}")
    })
}
```

**Technical Details:**
- JsonValue is a wrapper over serde_json::Value, not a Vec
- `.par_iter()` requires IntoParallelRefIterator trait (not satisfied)
- `.to_vec().into_par_iter()` returns owned values (IntoParIter<JsonValue>)
- Lambda receives owned JsonValue, not reference

## [0.9.10] - 2025-01-25

### Fixed - Parser and Concurrency Support (Phase 6.4.3 - 2h)

**Parser Fix for Reserved Keywords:**
- ✅ Fixed parsing error with `.parvec()`, `.par()`, `.vec()` method calls
- Reserved keywords (Par, Vec, ParVec) can now be used as method names
- Added `parse_method_name()` helper that accepts both identifiers and keyword tokens

**Concurrency Policy Support:**
- ✅ **parvec() works on all arrays**: Parallel execution with Rayon
- ✅ Automatic rayon dependency detection via `ArrayAdapter::Par|ParVec`
- ✅ Correct code generation: `.par_iter()` for parallel, `.collect()` for map
- ✅ Import `use rayon::prelude::*` when parallel execution is detected

**Code Generation Fixes:**
- Map with parallel adapter: generates `.collect::<Vec<_>>()` (no `.cloned()`)
- Filter with parallel adapter: generates `.cloned().collect::<Vec<_>>()`
- Added rayon imports at top level (after liva_rt module)

**Comprehensive Tests:**
- ✅ 10 integration tests in `tests/integration/proj_json/`
  * test_parse_no_error.liva - JSON.parse without binding
  * test_for_in_loop.liva - for...in on JSON
  * test_dot_notation.liva - property access
  * test_foreach_arrow.liva - forEach with arrows
  * test_map.liva - map transformation
  * test_filter.liva - filter selection
  * test_chaining.liva - map then filter
  * test_objects_array.liva - array of objects
  * test_parvec_json.liva - parallel execution
  * test_comprehensive.liva - all features combined

**Example Files:**
- ✅ 4 comprehensive examples in `examples/`
  * json_natural_syntax.liva - v0.9.8 features demo
  * json_arrow_functions.liva - v0.9.9 features demo
  * json_parallel.liva - parvec() demo
  * json_api_processing.liva - Real-world API processing

**Example Usage:**
```liva
main() {
    let data = "[1, 2, 3, 4, 5, 6, 7, 8]"
    let numbers = JSON.parse(data)
    
    // Sequential
    let doubled = numbers.map(n => n.as_i32().unwrap() * 2)
    
    // Parallel 🔥 NEW!
    let par_doubled = numbers.parvec().map(n => n.as_i32().unwrap() * 2)
    
    par_doubled.forEach(n => print($"  {n}"))
}
```

**Technical Details:**
- Parser now distinguishes between identifiers and keyword tokens in method position
- Desugaring phase detects ArrayAdapter usage and sets `ctx.has_parallel = true`
- Cargo.toml generation includes rayon when parallel execution is detected
- Code generator emits appropriate iterator methods based on adapter type

## [0.9.9] - 2025-01-25

### Added - Arrow Functions for JSON Arrays (Phase 6.4.2 - 3h)

**Full Array Method Support for JSON:**
- ✅ **forEach with arrow functions**: `posts.forEach(post => print(post.title))`
- ✅ **map**: `numbers.map(n => n * 2)` - Transform JSON arrays
- ✅ **filter**: `numbers.filter(n => n > 25)` - Filter JSON arrays
- ✅ **find/some/every**: Complete array method support
- ✅ **Chaining**: `posts.filter(p => p.id > 5).forEach(p => print(p.title))`

**Implementation Details:**

**1. JsonValue Iterator Methods:**
- Added `.iter()` → returns `std::vec::IntoIter<JsonValue>` (owned clones)
- Added `.to_vec()` → converts to `Vec<JsonValue>`
- JsonValue already implements `Clone`, so iteration clones values

**2. Lambda Pattern Detection:**
- Tracks which variables are JsonValue via `json_value_vars` HashSet
- Detects when `map`/`filter`/`forEach` is called on JsonValue
- For normal arrays: generates `|&item|` (borrow from iterator)
- For JsonValue: generates `|item|` (owned values from `.iter()`)

**3. Vec<JsonValue> Handling:**
- Results of `.map()`/`.filter()` are `Vec<JsonValue>`
- Tracked separately to handle iteration properly
- Uses `.iter().cloned()` for Vec<JsonValue> to clone elements
- Avoids `.copied()` (which only works for Copy types)

**4. Type Conversion Methods:**
- Added all conversion methods to generated JsonValue:
  * `as_i32()`, `as_f64()`, `as_string()`, `as_bool()`
  * `is_null()`, `is_array()`, `is_object()`
  * `to_json_string()`
- Prevents string literal conversion for `get`/`get_field` methods

**Complete Example:**
```liva
main() {
    // HTTP request (v0.9.6)
    let res, err = async HTTP.get("https://jsonplaceholder.typicode.com/posts")
    
    if res.status == 200 {
        // Natural JSON parsing (v0.9.8)
        let posts = JSON.parse(res.body)
        
        // Arrow functions on JSON arrays (v0.9.9) ✅ NEW!
        posts.forEach(post => {
            print($"Post {post.id}: {post.title}")
        })
        
        // Map and filter work too ✅ NEW!
        let ids = posts.map(p => p.id)
        let filtered = posts.filter(p => p.id > 5)
        
        filtered.forEach(p => print($"Filtered: {p.title}"))
    }
}
```

**Technical Highlights:**
- Smart detection: distinguishes `JsonValue` (direct) from `Vec<JsonValue>` (derived)
- Memory efficient: uses cloning only when necessary
- Iterator consistency: `.iter()` on JsonValue matches `.into_iter()` semantics
- No breaking changes: normal arrays continue working as before

**Performance Notes:**
- JsonValue.iter() clones elements (JsonValue contains serde_json::Value)
- Acceptable for typical JSON use cases (small-medium datasets)
- For large datasets, consider streaming or direct serde_json manipulation

## [0.9.8] - 2025-01-25

### Added - Natural JSON Syntax (Phase 6.4.1 - 2h)

**Ergonomic JSON Improvements:**
- ✅ **JSON.parse without error binding**: `let posts = JSON.parse(body)` - Auto-unwraps or panics on error
- ✅ **for...in loops**: `for post in posts { ... }` - Natural iteration over JSON arrays
- ✅ **Dot notation**: `post.id`, `post.title` - Direct property access instead of brackets

**Implementation Details:**

**1. JSON.parse Auto-unwrap:**
- Detects single-binding pattern in VarDecl: `let posts = JSON.parse(...)`
- Generates: `.0.expect("JSON parse failed")` automatically
- No need for error binding when you want to panic on error

**2. IntoIterator for JsonValue:**
- Implemented `IntoIterator` trait on `JsonValue`
- Returns `std::vec::IntoIter<JsonValue>` for arrays
- Empty iterator for non-arrays
- Embedded in both liva_rt.rs and generated runtime

**3. Dot Notation for Properties:**
- Heuristic detection: if variable is not array/class → treat as JsonValue
- Generates `.get_field("property").unwrap()` automatically
- Works in: assignments, conditions, string templates, function args

**4. Smart Length Detection:**
- `JsonValue.length()` for JSON arrays/objects
- `.len()` for Rust arrays and strings
- Automatic detection based on variable tracking

**Complete Natural Example:**
```liva
main() {
  let res, err = async HTTP.get("https://api.example.com/posts?_limit=5")

  if err {
    console.log($"Error: {err}")
  } else {
    if res.status == 200 {
      let posts = JSON.parse(res.body)  // ✅ No error binding
      for post in posts {                // ✅ for...in loop
        // ✅ Dot notation for properties
        console.log($"Post ID: {post.id}, Title: {post.title}")
      }
    }
  }
}
```

**Comparison:**

Before (v0.9.7):
```liva
let posts, jsonErr = JSON.parse(res.body)
if jsonErr == "" {
    let i = 0
    while i < posts.length {
        let post = posts[i]
        let id = post["id"]
        let title = post["title"]
        print($"Post {id}: {title}")
        i = i + 1
    }
}
```

After (v0.9.8):
```liva
let posts = JSON.parse(res.body)
for post in posts {
    print($"Post {post.id}: {post.title}")
}
```

**Code Changes:**
- Modified VarDecl generation to detect and unwrap JSON.parse
- Added IntoIterator impl to JsonValue (20 lines)
- Enhanced Member expression generation for JsonValue dot notation
- Smart .length() vs .len() detection based on context

## [0.9.7] - 2025-01-25

### Added - JSON Array/Object Support (Phase 6.4 - 3h)

**JsonValue Wrapper:**
- Created `JsonValue` struct wrapping `serde_json::Value` with Liva-friendly interface
- Implements `Display` trait for easy printing and string interpolation
- Provides type-safe methods for common JSON operations

**JSON Methods:**
- `length() -> usize` - Get array/object/string length
- `get(index: usize) -> Option<JsonValue>` - Array element access
- `get_field(key: &str) -> Option<JsonValue>` - Object field access
- `as_i32()`, `as_f64()`, `as_string()`, `as_bool()` - Type conversions
- `is_array()`, `is_object()`, `is_null()` - Type checking

**JSON Operations:**
- ✅ Array indexing: `arr[0]`, `arr[i]` - Access array elements
- ✅ Object key access: `obj["name"]` - Access object fields
- ✅ Length property: `arr.length` - Get array/object size
- ✅ String templates: `print($"Value: {jsonVar}")` - Direct interpolation
- ✅ Iteration support: Use `.length` with `while` loops

**Parser Support (Modified JSON.parse):**
- Changed return type from `(Option<Value>, Option<Error>)` to `(Option<JsonValue>, String)`
- Error messages as strings for consistency with HTTP client
- JsonValue automatically embedded in generated runtime

**Code Generation:**
- Added option_value_vars tracking for variables from tuple-returning functions
- Special handling for JsonValue.length() on Option<JsonValue>
- Heuristic detection of direct JsonValue variables (non-Option)
- String template unwrapping for Option<JsonValue> in interpolations
- Index access generates .get()/.get_field() calls automatically

**Semantic Analysis:**
- Made `.length` validation permissive for identifiers (validated at codegen)
- Allows `.length` on JSON variables without full type inference

**Working Example:**
```liva
main() {
    let res, err = async HTTP.get("https://api.example.com/posts?_limit=5")
    
    if err == "" && res.status == 200 {
        let posts, jsonErr = JSON.parse(res.body)
        
        if jsonErr == "" {
            let i = 0
            while i < posts.length {
                let post = posts[i]
                let id = post["id"]
                let title = post["title"]
                print($"Post {id}: {title}")
                i = i + 1
            }
        }
    }
}
```

**Limitations:**
- Direct `obj["key"]` in string templates (e.g., `$"{obj["key"]}"`) not supported due to parser limitations with nested quotes
- Workaround: use intermediate variables
- No `for...in` loop support yet (use `while` with `.length`)

**Bug Fixes:**
- ✅ Fixed hints.rs panic on empty error codes (added defensive guard)
- ✅ Fixed Option<Struct> consuming with multiple field access (use `.as_ref().unwrap()`)
- ✅ Fixed string template interpolation for option_value_vars

## [0.9.6] - 2025-01-25

### Added - HTTP Client (Phase 6.3 - 5h)

**HTTP Methods:**
- `HTTP.get(url: string) -> (Option<Response>, string)` - GET request
- `HTTP.post(url: string, body: string) -> (Option<Response>, string)` - POST request
- `HTTP.put(url: string, body: string) -> (Option<Response>, string)` - PUT request
- `HTTP.delete(url: string) -> (Option<Response>, string)` - DELETE request

**Response Object:**
- `status: i32` - HTTP status code (200, 404, etc.)
- `statusText: string` - Status text ("OK", "Not Found", etc.)
- `body: string` - Response body as string
- `headers: [string]` - Response headers

**Features:**
- ✅ Async by default using Liva's lazy evaluation (`async HTTP.get()`)
- ✅ Error binding pattern: `let response, err = async HTTP.get(url)`
- ✅ Tuple return type: `(Option<Response>, String)` for success/error
- ✅ 30-second timeout with reqwest
- ✅ TLS support via rustls (no OpenSSL dependency)
- ✅ Comprehensive error handling (network, DNS, timeout, HTTP errors)

**Implementation:**
- Runtime: 150+ lines in liva_rt with LivaHttpResponse struct
- Semantic Analysis: 120+ lines detecting HTTP.*, validation, async/fallible marking
- Parser: Enhanced parse_exec_call() to support `async HTTP.method()` syntax
- Code Generation: 300+ lines across 4 locations for HTTP support
- Dependencies: reqwest 0.11 with rustls-tls features

**Bug Fixes:**
- ✅ Fixed error binding code generation for tuple-returning functions
- ✅ Added returns_tuple field to TaskInfo for correct await generation
- ✅ Enhanced is_builtin_conversion_call() to detect Call wrapping MethodCall
- ✅ Fixed Option<Struct> field access to generate `value.unwrap().field`
- ✅ Prevented String error vars from being tracked as Option<Error>

**Examples:**
```liva
// Simple GET request
let response, err = async HTTP.get("https://api.example.com/data")
if err {
    console.error($"Error: {err}")
} else {
    print($"Status: {response.status}")
    print($"Body: {response.body}")
}

// POST with data
let postResp, postErr = async HTTP.post("https://api.example.com/users", userData)
if postErr == "" {
    print($"Created! Status: {postResp.status}")
}
```

**Time Breakdown:**
- Design & Documentation: 30 min
- Setup & Dependencies: 30 min
- Runtime Implementation: 1.5 hours (all 4 methods)
- Semantic Analysis: 30 min (detection, validation)
- Parser Enhancement: 15 min (async MethodCall)
- Code Generation: 1.5 hours (HTTP calls, embedding, deps)
- Bug Fixes: 1 hour (error binding, tuple handling)
- Testing: 30 min (all methods verified)

**Tests:**
- ✅ test_http_simple.liva - Basic GET with error handling
- ✅ test_http_quick.liva - GET and DELETE
- ✅ examples/manual-tests/test_http_all.liva - Comprehensive (all 4 methods)

## [0.9.5] - 2025-01-24

### Added - Enhanced Pattern Matching (Phase 6.4 - 3.5h)

**Switch Expressions:**
- Switch can now be used as an expression (returns a value)
- Can be used anywhere an expression is valid
- All arms must return the same type

**Pattern Types:**
- **Literal patterns**: `1 => "one"`, `"hello" => greet()`, `true => yes()`
- **Wildcard pattern**: `_ => default_case` (catch-all)
- **Binding patterns**: `n => n * 2` (capture value in variable)
- **Range patterns**: `1..10`, `1..=10`, `..10`, `10..` (inclusive/exclusive)

**Pattern Guards:**
- Add conditional logic with `if` clauses: `x if x < 20 => "teenager"`
- Guards can use any boolean expression
- Guards have access to bound variables

**Exhaustiveness Checking (✅ NEW):**
- ✅ **Bool exhaustiveness**: Compiler ensures both `true` and `false` cases are covered
- Error `E0901`: Non-exhaustive pattern matching on bool
- Accepts wildcard `_` or binding patterns as catch-all
- Helpful error messages with suggestions
- Example:
  ```liva
  // ❌ Error: E0901 - missing 'false' case
  let result = switch flag {
      true => "yes"
  };
  
  // ✅ OK - both cases covered
  let result = switch flag {
      true => "yes",
      false => "no"
  };
  ```

**Implementation:**
- Added `Pattern` enum to AST (Literal, Wildcard, Binding, Range)
- Added `SwitchExpr`, `SwitchArm`, `SwitchBody` to AST
- Added `Token::Underscore` and `Token::DotDotEq` to lexer
- Implemented `parse_switch_expr()` and `parse_pattern()` in parser
- Switch expressions pass through IR as `Unsupported` (handled in codegen)
- Generate Rust `match` expressions with proper pattern translation
- Semantic validation for switch expressions and guards
- ✅ **Exhaustiveness checking** in `check_switch_exhaustiveness()`
- Full await detection for async switch expressions

**Testing:**
- Created `test_switch_expr.liva` with 5 comprehensive test cases
- Created `test_exhaustiveness.liva` with exhaustive patterns
- Created `test_exhaustiveness_error.liva` to verify E0901 error
- Created `test_exhaustiveness_complete.liva` with all scenarios
- All patterns working: literals, ranges, guards, bindings, wildcards
- Tested with integers, strings, booleans
- All 6 tests passing ✅

**Documentation:**
- Complete language reference: `docs/language-reference/pattern-matching.md` (650+ lines)
- Comprehensive design document: `docs/PHASE_6.4_PATTERN_MATCHING_DESIGN.md` (800+ lines)
- Pattern types, guards, exhaustiveness, examples, best practices
- Error codes: E0901 (non-exhaustive bool)

**Examples:**
```liva
// Basic literal matching
let result = switch x {
    1 => "one",
    2 => "two",
    _ => "other"
};

// Range patterns
let grade = switch score {
    90..=100 => "A",
    80..=89 => "B",
    70..=79 => "C",
    _ => "F"
};

// Pattern guards
let category = switch age {
    x if x < 13 => "child",
    x if x < 20 => "teenager",
    x if x < 65 => "adult",
    _ => "senior"
};

// Exhaustiveness checking
let result = switch flag {
    true => "yes",
    false => "no"  // Both cases required!
};
```

**Future Enhancements (v0.9.6+):**
- Full exhaustiveness checking for all types (int, string, enum)
- Tuple/array destructuring patterns
- Enum variant patterns
- Or patterns (`|` operator)

## [0.9.4] - 2025-01-21

### Added - File I/O Operations (Phase 6.2 - 2.5h)

**File API:**
- `File.read(path: string): (string?, Error?)` - Read entire file contents as string
- `File.write(path, content: string): (bool?, Error?)` - Write/overwrite file
- `File.append(path, content: string): (bool?, Error?)` - Append to end of file
- `File.exists(path: string): bool` - Check if file/directory exists
- `File.delete(path: string): (bool?, Error?)` - Delete file from filesystem

**Implementation:**
- Added `generate_file_function_call()` to code generator (82 lines)
- Rust backend using `std::fs`: `read_to_string`, `write`, `OpenOptions`, `Path::exists`, `remove_file`
- Extended `is_builtin_conversion_call()` to recognize File methods (except `exists`)
- Added `option_value_vars` tracking for proper string concatenation with Option types

**Features:**
- Error binding integration for all operations (except `exists`)
- UTF-8 file encoding
- Synchronous I/O (blocking operations)
- Graceful error handling for missing files, permissions, disk full scenarios

**Testing:**
- 5 basic tests in `test_file_simple.liva`
- 27 comprehensive tests in `test_file_complex.liva` covering all operations, edge cases, workflows
- All tests passing ✅

**Documentation:**
- Complete API reference: `docs/language-reference/file-io.md` (450 lines)
- Design document: `docs/PHASE_6.2_FILE_IO_API_DESIGN.md` (430 lines)
- Implementation summary: `docs/PHASE_6.2_FILE_IO_SUMMARY.md` (280 lines)
- Total: 1,160+ lines of documentation

### Fixed
- Option value variables now properly unwrap in string concatenation
- Error binding variables (first in tuple) tracked separately for type-safe string operations

## [0.9.3] - 2025-01-21

### Added - JSON Parsing & Serialization (Phase 6.1 - 4h)

**JSON API:**
- `JSON.parse(json: string): (any?, Error?)` - Parse JSON strings to Liva types
- `JSON.stringify(value: any): (string?, Error?)` - Serialize Liva values to JSON

**Implementation:**
- Added `generate_json_function_call()` to code generator
- Integrated `serde_json` crate for runtime JSON operations
- Extended `is_builtin_conversion_call()` to recognize JSON methods
- Error binding pattern support for both functions

**Type Mapping:**
- JSON → Liva: null→none, bool→bool, number→int/float, string→string, array→array, object→object
- Liva → JSON: Full bidirectional mapping with error handling

**Error Handling:**
- Parse errors: Invalid syntax, unexpected EOF, malformed numbers
- Stringify errors: Unsupported types (functions, tasks), circular references
- All errors use error binding pattern: `let result, err = JSON.parse(str)`

**Examples:**
```liva
// Parse JSON
let data, err = JSON.parse("{\"name\": \"Alice\", \"age\": 30}")
if err { fail err }

// Stringify
let json, err2 = JSON.stringify({name: "Bob", age: 25})
if err2 { fail err2 }
```

**Test Coverage:**
- `test_json_simple.liva` - Basic parse/stringify tests
- Tests valid JSON parsing
- Tests invalid JSON error handling
- Round-trip conversion tests

**Documentation:**
- `docs/language-reference/json.md` - Complete API reference (400 lines)
- Type mapping tables
- Error handling guide
- 4 complete examples

## [0.9.2] - 2025-10-23

### Added - Trait Aliases (Phase 5.10 - 2h)

**Intuitive Trait Aliases:**
- `Numeric` = Add + Sub + Mul + Div + Rem + Neg (all arithmetic)
- `Comparable` = Ord + Eq (equality and ordering)
- `Number` = Numeric + Comparable (complete number operations)
- `Printable` = Display + Debug (formatting)

**Implementation:**
- Added `aliases` HashMap to TraitRegistry
- `register_trait_aliases()` defines 4 built-in aliases
- `is_alias()` checks if constraint is an alias
- `expand_alias()` returns underlying traits
- `expand_constraints()` expands all aliases in a list
- Semantic analyzer expands aliases before registering constraints
- Code generation automatically expands aliases to Rust traits

**Examples:**
```liva
// Simple and intuitive
sum<T: Numeric>(a: T, b: T): T => a + b
max<T: Comparable>(a: T, b: T): T { ... }
clamp<T: Number>(value: T, min: T, max: T): T { ... }

// Mix with granular traits
formatAndCompare<T: Comparable + Display>(a: T, b: T) { ... }
debugCalc<T: Numeric + Printable>(a: T, b: T) { ... }

// Still can use granular for precise control
addOnly<T: Add>(a: T, b: T): T => a + b
```

**Test Coverage:**
- test_trait_aliases.liva (comprehensive test of all aliases)
- Tests mixing aliases with granular traits
- Verifies expansion to correct Rust bounds

**Documentation:**
- Updated generics.md with trait aliases section
- Added comparison table (aliases vs granular)
- Best practices guide
- Examples of mixing approaches

### Philosophy
Liva now offers **the best of both worlds**:
- **Beginners:** Use intuitive aliases (`Numeric`, `Comparable`, `Number`)
- **Advanced:** Use granular traits for precise control (`Add`, `Ord`, etc.)
- **Flexible:** Mix both approaches as needed

## [0.9.1] - 2025-10-23

### Added - Multiple Constraints & Type Arguments (Phase 5.9 - 3h)

**Type Arguments in Function Calls:**
- Added `type_args` field to CallExpr AST
- Parser recognizes `function<Type>(args)` syntax
- Handles both type keywords (float, bool, string) and identifiers
- Lookahead logic to distinguish `<` as type arg vs comparison
- Code generation with Rust turbofish operator `::< Type >`
- Examples: `identity<int>(42)`, `sum<float>(3.5, 2.5)`

**Multiple Constraints with + Operator:**
- Updated `TypeParameter` to use `Vec<String>` for constraints
- Parser supports `<T: Trait1 + Trait2 + Trait3>` syntax
- Semantic analyzer validates ALL constraints in vector
- Code generation emits `<T: Trait1 + Trait2>` format
- Composable constraint system (like Rust/Swift/C#)
- Examples:
  * `clamp<T: Ord + Add + Sub>(value, min, max)`
  * `printIfEqual<T: Eq + Display>(a, b)`
  * `average<T: Add + Div>(a, b, divisor)`

**Available Traits:**
- Arithmetic: Add, Sub, Mul, Div, Rem, Neg
- Comparison: Eq, Ord
- Utilities: Clone, Copy, Display, Debug
- Logical: Not

**Test Coverage:**
- test_multi_constraints.liva (comprehensive multi-trait tests)
- All arithmetic + comparison combinations validated
- Generates correct Rust trait bounds

**Documentation:**
- Updated generics.md with multiple constraints section
- Updated ROADMAP.md with Phase 5.9 completion
- All examples working end-to-end

### Changed
- TypeParameter AST now uses `constraints: Vec<String>` instead of `constraint: Option<String>`
- Display trait for TypeParameter now joins constraints with " + "

## [0.9.0] - 2025-10-23

### Added - Generics System (Phase 5 - CORE COMPLETE, 16.5h - Released! 🎉)

**Phase 5.1: Generic Syntax Design (2h) ✅**
- Complete specification in docs/language-reference/generics.md (785 lines)
- Syntax: `<T>`, `<T: Constraint>`, `<T, U>` for multiple parameters
- Monomorphization strategy (compile-time specialization like Rust)
- Standard library integration plan (Array<T>, Result<T,E>, Option<T>, Map<K,V>, Set<T>)
- Comprehensive examples and edge cases

**Phase 5.2: Parser & AST Extensions (3h) ✅**
- New `TypeParameter` struct with name and optional constraint
- Updated AST nodes: ClassDecl, TypeDecl, FunctionDecl, MethodDecl
- Implemented `parse_type_parameters()` function
- Parser handles `<T>`, `<T: Constraint>`, `<T, U>` syntax
- **Discovery:** Liva has no `class` keyword - classes are `Name<T> { }`
- Fixed codegen to emit proper generic Rust code:
  * `pub struct Name<T: Constraint>`
  * `impl<T: Constraint> Name<T> { }`
- Added `[T]` array type syntax support
- Parser handles type parameters in type annotations (T, U, etc.)
- Added `?` and `!` suffix parsing for Optional and Fallible types
- 11 parser test files with full insta snapshot coverage
- All tests passing (11/11)

**Phase 5.3: Code Generation (2.5h) ✅**
- Generic functions compile and execute correctly! 🎉
  * Example: `identity<T>(value: T): T => value`
  * Test output: Works with int, string, bool types
  * Generated Rust: `fn identity<T>(value: T) -> T { value }`
- Generic classes with single type parameter working! 🎉
  * Example: `Box<T> { value: T }`
  * Generates: `pub struct Box<T> { pub value: T }`
  * Impl blocks: `impl<T> Box<T> { pub fn new(value: T) -> Self { ... } }`
- Generic classes with multiple type parameters working! 🎉
  * Example: `Pair<T, U> { first: T, second: U }`
  * Generates: `pub struct Pair<T, U> { ... }`
  * All combinations tested: int/string, bool/float, string/int
- Array type annotations working! 🎉
  * Example: `firstInt(arr: [int]): int`
  * Generates: `fn first_int(arr: Vec<i32>) -> i32`
  * Tested with firstInt, lastInt, sum functions
- **No codegen changes needed** - infrastructure already supported it!
- Monomorphization delegated to Rust compiler (optimal)

**Known Issue:**
- Field access on method return values generates `["field"]` instead of `.field`
- Workaround: Assign to intermediate variable first

**Working Examples:**
- `examples/test_array_generic.liva` - identity<T> function
- `examples/test_generic_class.liva` - Box<T> class
- `examples/test_generic_methods.liva` - Pair<T,U> class
- `examples/test_array_syntax.liva` - Array type annotations

**Commits:** 8ee5bc1 (specification), ae39b05 (parser tests), d4dc6d2 (array syntax), 72c3878 (working generics!), 677c552 (generic classes), 5669a17 (multiple type params), 2d8c6d3 (docs update), 4b7d0fd (array types)

**Phase 5.4: Standard Library Validation (2h) ✅**
- Test `Option<T>` with generics working! 🎉
  * Created Option<T> class with isSome(), isNone() methods
  * Works with: int, string, bool types
  * File: `examples/test_option_generic.liva`
  * Compiles and executes correctly
- Test `Result<T, E>` with generics working! 🎉
  * Created Result<T,E> class with isSuccess(), isError() methods
  * Works with: Result<int,string>, Result<bool,int>
  * File: `examples/test_result_generic.liva`
  * Compiles and executes correctly

**Important Findings:**

✅ **What Works:**
- Generic classes instantiate correctly with different types
- Multiple type parameters work (`Result<T, E>`)
- Type safety enforced by Rust's type system
- Methods with `&self` work for predicates (bool returns)

⚠️ **Limitations Discovered:**

1. **Ownership Issue:**
   - Methods with `&self` cannot return `T` by value
   - Rust error: "cannot move out of `self.value` which is behind a shared reference"
   - Workaround: Access fields directly instead of getter methods
   - Future solution: Add Clone constraint or make methods consume self

2. **Semantic Analyzer Interference:**
   - Function names like `parseInt` trigger fallible builtin detection
   - Compiler tries to parse string literals instead of calling the function
   - Workaround: Use different names (`parseNumber` instead of `parseInt`)
   - Future solution: Improve semantic analysis to distinguish user functions

3. **VSCode Language Server Bug:**
   - LSP shows parse error on generic class declarations (`Option<T> {`)
   - Actual compiler works fine - error is only in IDE
   - Error message: "Expected LParen" (false positive)
   - Impact: Cosmetic only - doesn't affect compilation

**Commits:** 1594d4d (Option<T>), 17bbef2 (Result<T,E>)

**Phase 5.5: Type System Implementation (1h) ⏸️ PARTIAL**
- Type parameter validation implemented! ✅
  * Added `type_parameters` tracking to SemanticAnalyzer
  * Implemented scope management for type parameters
  * Enhanced `validate_type_ref()` to check T exists in scope
  * Validates type parameters in functions, classes, and methods
  * Methods inherit class type parameters correctly
  * File: `examples/test_type_param_validation.liva`
  * **Status:** Type validation working correctly
- Constraint checking deferred to v0.9.1
  * Advanced features like `T: Clone`, `T: Display` validation
  * Type inference for generic calls (implicit type arguments)
  * Type substitution for complex scenarios
- **Rationale:** Core generics are working. Advanced features can be added incrementally without blocking release.

**Commit:** 2c75280 (type parameter validation)

**Phase 5.7: Documentation & Examples (0.5h) ✅**
- Created comprehensive generics quick-start tutorial
  * File: `docs/guides/generics-quick-start.md` (338 lines)
  * Introduction to generics with motivation
  * Basic generic functions (identity<T>)
  * Generic classes (Box<T>, Pair<T,U>)
  * Array type annotations
  * Option<T> pattern with examples
  * Result<T,E> pattern with examples
  * Best Practices section (Do's and Don'ts)
  * Common Patterns (Stack<T>, Wrapper<T>)
  * Known Limitations clearly documented
  * "What's Next" roadmap for v0.9.1
  * Complete list of working examples
- Updated ROADMAP.md with Phase 5 completion status
- Updated CHANGELOG.md with full release notes

**Commit:** a45acec (tutorial), b6f1f5b (roadmap/changelog updates)

**Phase 5.8: Constraint Checking System (~5h) ✅**
- Implemented complete trait registry system
  * File: `src/traits.rs` (279 lines, 13 built-in traits)
  * Traits: Add, Sub, Mul, Div, Rem (arithmetic operators)
  * Traits: Eq, Ord (comparison operators)
  * Traits: Neg, Not (unary operators)
  * Traits: Clone, Display, Debug, Copy, Default (utility traits)
  * Automatic trait hierarchy (Ord requires Eq, Copy requires Clone)
  * Rust mapping: Add→std::ops::Add<Output=T> + Copy
- Enhanced semantic analyzer with constraint validation
  * `validate_binary_op_constraints()` - validates +, -, *, /, %, ==, !=, <, >, <=, >=
  * `validate_unary_op_constraints()` - validates unary -, !
  * E5001 error: Unknown trait constraint (with suggestions)
  * E5002 error: Missing constraint for operator usage
  * Integrated TraitRegistry into SemanticAnalyzer
- Updated codegen for complete Rust trait bounds
  * Generate bounds: `T: std::ops::Add<Output=T> + Copy`
  * Auto-include Copy for value return types
  * Handle implicit requirements (Ord includes Eq)
  * Updated generate_function() and generate_class()
- Created comprehensive test suite (4 files)
  * test_constraint_arithmetic.liva - All arithmetic operators (+, -, *, /, %, unary-)
  * test_constraint_comparison.liva - Ord tests (max, min, clamp), Eq tests
  * test_constraint_error.liva - E5002 error detection
  * test_generic_stack.liva - Real-world utility functions
- **All tests passing ✅** - Java-level completeness achieved

**Working Examples:**
```liva
// Arithmetic with constraints
sum<T: Add>(a: T, b: T): T => a + b                    // ✅ Works!
modulo<T: Rem>(a: T, b: T): T => a % b                  // ✅ Works!
negate<T: Neg>(value: T): T => -value                   // ✅ Works!

// Comparison with constraints
max<T: Ord>(a: T, b: T): T {                            // ✅ Works!
    if a > b { return a }
    return b
}
clamp<T: Ord>(value: T, min: T, max: T): T { ... }     // ✅ Works!

// Error detection
sum_no_constraint<T>(a: T, b: T): T => a + b           // ❌ E5002: Missing Add constraint
```

**Commit:** 240b814 (constraint checking system complete)

**Summary - v0.9.0 Production Ready:**

✅ **Completed Features:**
- Generic functions: `identity<T>(value: T): T`
- Generic classes: `Box<T>`, `Pair<T, U>`
- **Constraint checking: `sum<T: Add>`, `max<T: Ord>`, `negate<T: Neg>`** 🎉
- Array type annotations: `[int]` → `Vec<i32>`
- Option<T> and Result<T,E> validated and working
- Type parameter validation in semantic analyzer
- **13 built-in traits with automatic validation** 🎉
- 15+ tests passing (parser + integration)
- **4 constraint test files - all passing** 🎉
- 10 working example files

📊 **Statistics:**
- **Time:** 16.5 hours (110% of 15h estimate - exceeded expectations!)
- **Commits:** 18 (all on feature branch)
- **Files created:** 10 examples + 11 parser tests + 2 documentation files + 1 traits module
- **Lines added:** ~2,560 (parser, semantic, codegen, traits, examples, docs, tutorial)
- **Documentation:** 1,123 lines (785 generics.md + 338 quick-start.md)

🎯 **What You Can Do in v0.9.0:**
```liva
// Generic functions
identity<T>(value: T): T => value

// Generic functions with constraints 🎉
sum<T: Add>(a: T, b: T): T => a + b
max<T: Ord>(a: T, b: T): T { if a > b { return a } return b }
negate<T: Neg>(value: T): T => -value

// Generic classes
Box<T> { value: T }
Pair<T, U> { first: T, second: U }
Stack<T: Clone> { items: [T] }

// Array type annotations
sum(numbers: [int]): int { ... }

// Optional types
Option<T> { value: T, hasValue: bool }
Result<T, E> { value: T, error: E }

// All operators with constraints:
// Arithmetic: +, -, *, /, % (Add, Sub, Mul, Div, Rem)
// Comparison: >, <, >=, <=, ==, != (Ord, Eq)
// Unary: -, ! (Neg, Not)
```

⚠️ **Known Limitations (to be addressed in v0.9.1):**
1. Methods with `&self` cannot return `T` by value (use field access)
2. Type inference not implemented (must specify `<T>` explicitly)
3. Multiple constraints syntax `<T: Add + Mul>` not yet supported (use single constraint per function)
4. VSCode LSP shows false positive parse errors (compiler works fine)

**Deferred to v0.9.1:**
- Multiple constraints syntax (`<T: Add + Mul>`)
- Type inference for generic calls
- Advanced type system features

## [0.8.1] - 2025-10-23

**🎉 Phase 5: Enhanced Error Messages - Developer-friendly diagnostics**

Comprehensive error system with "Did you mean?" suggestions, enhanced context, error categorization, intelligent hints, code examples, and documentation links. Quality comparable to Rust and Elm.

### Added - Enhanced Error Messages (Phase 5 - 8h, 100% complete)

**Phase 5.1: "Did You Mean?" Suggestions (~2h) ✅**
- Levenshtein distance algorithm for typo detection
- Smart suggestions for:
  * Undefined variables (max 2 character edits)
  * Undefined functions
  * Undefined types/classes
  * Module import symbols
- `suggestions.rs` module (265 lines)
- Comprehensive test suite (test_suggestions.liva)

**Phase 5.2: Enhanced Error Context (~2h) ✅**
- Show 2 lines before and 2 lines after error location
- Precise token underlining using actual token length (not fixed 3 chars)
- Line numbers for all context lines
- Extended ErrorLocation structure:
  * `length: Option<usize>` - Token length for precise highlighting
  * `context_before: Option<Vec<String>>` - Lines before error
  * `context_after: Option<Vec<String>>` - Lines after error
- get_context_lines() function in semantic analyzer
- Visual improvements with exact caret positioning

**Phase 5.3: Error Categories & Codes (~1h) ✅**
- Organized error codes by category (E0xxx-E7xxx):
  * E0xxx: Lexical errors (invalid tokens, unclosed strings)
  * E1xxx: Syntax errors (grammar violations, unexpected tokens)
  * E2xxx: Semantic errors (undefined symbols, type errors)
  * E3xxx: Control flow errors (invalid return, break, continue)
  * E4xxx: Module errors (import failures, circular dependencies)
  * E5xxx: Concurrency errors (async/parallel misuse)
  * E6xxx: Standard library errors
  * E7xxx: I/O errors
- `error_codes.rs` module (190 lines) with ErrorCategory enum
- Category displayed in error messages: `[Semantic] E2001: ...`
- Complete error reference (ERROR_CODES.md, 316 lines)

**Phase 5.4: Intelligent Hints & Help (~2h) ✅**
- `hints.rs` module (176 lines) with automatic contextual help
- Functions for each error code:
  * `get_hint()` - Actionable advice
  * `get_example()` - Code examples showing correct vs incorrect
  * `get_doc_link()` - Links to documentation
  * `get_common_fixes()` - Common solutions by category
  * `get_tip()` - Additional improvement tips
- Automatic hint injection when manual help not provided
- Coverage for 15+ error codes with plans for more

**Phase 5.5: Documentation (~1h) ✅**
- ERROR_CODES.md (316 lines) - Complete error reference
- ERROR_HANDLING_GUIDE.md (522 lines) - Comprehensive guide
- TROUBLESHOOTING.md (493 lines) - Quick reference
- compiler-internals/enhanced-error-context.md (125 lines)
- Updated README.md with error system showcase
- Best practices and contributing guidelines

**Phase 5.6: VS Code Extension Integration (v0.4.0) ✅**
- Extended JSON error format with Phase 5 fields:
  * `suggestion`, `hint`, `example`, `doc_link`, `category`
- Auto-population of fields in `to_json()` methods
- Builder pattern for error creation:
  * `.with_suggestion()`, `.with_hint()`, `.with_example()`
  * `.with_doc_link()`, `.with_category()`
- Refactored semantic.rs to use builder pattern
- Cleaner, more maintainable error creation

### Changed
- Error message format now includes category badges
- ErrorLocation structure extended with context and length fields
- Error display shows more context (5 lines total vs 1 line)
- Float literals now use `_f64` suffix for type clarity
- Improved error messages with automatic suggestions

### Fixed
- Integration test float literal format (accept both 0.0 and 0_f64)
- async/parallel test with proper function calls

### Statistics
- **21 files changed**: +2,509 insertions, -60 deletions
- **3 new modules**: suggestions.rs, error_codes.rs, hints.rs
- **4 new documentation files**: 1,500+ lines total
- **8 test files**: Comprehensive coverage
- **10 commits**: Feature development complete

### Developer Experience Improvements
**Before:**
- Generic error messages
- No suggestions for typos
- Single line context
- Fixed 3-character underlines

**After:**
- Categorized errors with codes
- "Did you mean?" suggestions
- 5 lines of context (2 before, error, 2 after)
- Precise token-length underlining
- Automatic hints and examples
- Documentation links
- One-click fixes in VS Code

**Example Error:**
```
● E2001: Undefined variable [Semantic]
────────────────────────────────────────────────────────────
  → test.liva:5:12

   3 │     let userName = "Alice"
   4 │     
   5 │     console.log(usrName)
     │                 ^^^^^^^

  ⓘ Cannot find variable 'usrName' in current scope

  💡 Did you mean 'userName'?

  💡 Hint: Check spelling or declare the variable before use

  📝 Example:
     let userName = "value"
     console.log(userName)  // Correct

  📚 https://liva-lang.org/docs/errors/semantic#e2001
────────────────────────────────────────────────────────────
```

### Console API Enhancement
- `console.input()` function for user input
  * `console.input()` - Read without prompt
  * `console.input(message)` - Read with prompt
- ANSI color support:
  * `console.error()` - Red color
  * `console.warn()` - Yellow/amber color  
  * `console.success()` - Green color (NEW)
- Updated documentation and test suite

## [0.8.0] - 2025-10-21

**🚀 Phase 3: Module System - Multi-file projects**

Complete implementation of multi-file project support with JavaScript-style imports, automatic public/private visibility based on naming convention, circular dependency detection, and comprehensive error messages.

#### Added - Module System (Phase 3 - 17h actual, 3.1x faster than estimated)

**Phase 3.1: Design (2h) ✅ Complete**
- Module system specification document (400+ lines)
- Syntax comparison document (4 options evaluated)
- Implementation roadmap (TODO_MODULES.md, 700+ lines)
- Design decisions:
  * Public by default (no prefix)
  * Private with `_` prefix (consistent with Liva)
  * JavaScript-style import syntax
  * Relative paths (`./, ../`)

**Phase 3.2: Parser & AST (2h) ✅ Complete**
- Added `ImportDecl` struct to AST with Display trait
- Added `from` keyword to lexer
- Implemented `parse_import_decl()` method (~60 lines)
- Support for named imports: `import { a, b } from "path"`
- Support for wildcard imports: `import * as name from "path"`
- Handles comma-separated imports with trailing commas
- Comprehensive error handling for malformed imports

**Phase 3.3: Module Resolver (4h) ✅ Complete**
- Created `module.rs` with 400+ lines of infrastructure:
  * **Module struct**: Loads .liva files, extracts public/private symbols
  * **DependencyGraph**: DFS-based cycle detection, topological sort
  * **ModuleResolver**: Recursive loading with caching
- Path resolution for relative imports (`./, ../`)
- Symbol extraction based on `_` prefix
- Circular dependency detection with clear error messages (E4003)
- File not found errors with helpful context (E4004)
- Integration with compiler pipeline:
  * `compile_with_modules()` function
  * Auto-detection of import statements
  * `resolve_all()` returns modules in compilation order
- Unit tests for cycle detection (3 tests)
- Example files: math.liva, operations.liva, utils.liva

**Phase 3.4: Semantic Analysis (3h) ✅ Complete**
- Symbol validation during import resolution
- Check if imported symbols exist in target module
- Private symbol import detection (E4007 error)
- Name collision detection:
  * Import vs local definition (E4008)
  * Import vs import (E4009)
- Module context tracking for semantic analysis
- Integration with existing semantic analyzer

**Phase 3.6: Integration & Polish (in progress) 🔄**
- **Calculator Example** (65 lines, 3 modules):
  * `examples/calculator/calculator.liva` - Main entry point
  * `examples/calculator/basic.liva` - Basic operations (+, -, *, /)
  * `examples/calculator/advanced.liva` - Advanced operations
  * Demonstrates: named imports, public/private visibility
  * Tested: compiles and runs successfully
- **Documentation Updates**:
  * Updated `docs/getting-started/quick-start.md` with module section
  * Created `docs/guides/module-best-practices.md` (500+ lines)
  * Project structure patterns, naming conventions
  * Import patterns, visibility guidelines
  * Common patterns and anti-patterns
  * Performance tips and comprehensive examples
- **Error Message Polish**:
  * Enhanced E4003-E4009 with helpful hints
  * Specific suggestions (e.g., use aliases for collisions)
  * Better context for circular dependencies
  * Actionable guidance for resolving issues
- **Testing**:
  * Multi-module compilation verified
  * Calculator example runs correctly
  * Import syntax examples working
  * Error messages tested

**Phase 3.4: Semantic Analysis (3h) ✅ Complete (original)**
- Extended SemanticAnalyzer with import context:
  * New fields: imported_modules, imported_symbols
  * New function: analyze_with_modules() - accepts module context
  * validate_imports() - iterates all imports in program
  * validate_import() - validates single import declaration
- Import validation checks (180+ lines of code):
  * **E4004**: Module not found - with path resolution
  * **E4006**: Imported symbol not found in module
  * **E4007**: Cannot import private symbol (starts with _)
  * **E4008**: Import conflicts with local definition
  * **E4009**: Import conflicts with another import
- Path resolution:
  * Resolves relative paths (./,  ../)
  * Canonicalizes paths for matching
  * Fallback by filename matching
- Symbol registration:
  * Adds imported symbols to function registry
  * Permissive arity checking (accepts any arg count)
  * Prevents "function not found" errors
- Integration with compile_with_modules():
  * Builds module context map from resolved modules
  * Passes public_symbols and private_symbols
  * Uses analyze_with_modules() instead of analyze_with_source()

**Phase 3.5: Multi-File Code Generation (2h) ✅ Complete**
- Multi-file Rust project generation (180+ lines):
  * **generate_multifile_project()**: Main orchestrator
  * **generate_module_code()**: Per-module code generation
  * **generate_entry_point()**: main.rs with mod declarations
  * **generate_use_statement()**: Import → use conversion
  * **write_multifile_output()**: File writing system
- Import conversion:
  * `import { add } from "./math.liva"` → `use crate::math::add;`
  * `import { a, b } from "./m.liva"` → `use crate::m::{a, b};`
  * Wildcard imports with same-name alias simplified
- Visibility modifiers:
  * Functions without `_` prefix → `pub fn name()`
  * Private functions → `fn name()` (prefix removed)
  * Classes follow same rules
- Module declarations:
  * Automatic `mod` statements in main.rs
  * One .rs file per .liva module
- File structure:
  * src/main.rs - Entry point with mod declarations
  * src/math.rs, src/operations.rs, etc. - Module files
  * Cargo.toml - Project configuration
- Made CodeGenerator.output pub(crate) for access
- Made DesugarContext Clone-able for reuse
- Integration with compile_with_modules() pipeline
- Tested with examples/modules/test_import_syntax.liva:
  * ✅ Generates 4 files (main.rs + 3 modules)
  * ✅ Compiles successfully: `cargo build`
  * ✅ Executes correctly: "10 + 20 = 30"
- Documentation: docs/compiler-internals/multifile-codegen.md (650+ lines)

**Current Status:**
- ✅ Import syntax parsing works
- ✅ Module resolution with cycle detection works
- ✅ Loads all dependencies recursively
- ✅ Returns modules in topological order
- ✅ Import validation complete (all error codes)
- ✅ Symbol existence and visibility checks working
- ✅ Name collision detection working
- ✅ Multi-file Rust project generation working
- ✅ Pub/private visibility correctly applied
- ✅ Import → use conversion functional
- 📋 More examples and polish needed (Phase 3.6)

**Example:**
```liva
// math.liva
add(a: number, b: number): number => a + b
subtract(a: number, b: number): number => a - b
_internal_calc(x: number): number => x * 2  // Private

// main.liva
import { add } from "./math.liva"

main() {
    let result = add(10, 20)
    print($"Result: {result}")
}
```

**Generated Output:**
```
project/
├── Cargo.toml
└── src/
    ├── main.rs      (mod math; use crate::math::add; ...)
    └── math.rs      (pub fn add, pub fn subtract, fn internal_calc)
```

**Progress:**
- ✅ Phase 3.1: Design (2h)
- ✅ Phase 3.2: Parser (2h)
- ✅ Phase 3.3: Module Resolver (4h)
- ✅ Phase 3.4: Semantic Analysis (3h)
- ✅ Phase 3.5: Code Generation (2h)
- 📋 Phase 3.6: Integration & Examples (pending)
- **Total: 13h actual / 53h estimated (83% complete, 4x faster)**

**Next Steps:**
- Phase 3.6: Integration & Examples (9h) - Calculator example, polish, release

---

## [0.7.0] - 2025-10-20

**🎉 Phase 2 Complete: Standard Library - 37 functions implemented in one day!**

### Added - Standard Library (Phase 2)

#### Array Methods (9 methods)
- **`map(fn)`** - Transform each element
  - Sequential: `[1,2,3].map(x => x * 2)` → `[2,4,6]`
  - Uses `.iter().map(|&x| ...).collect()`
- **`filter(fn)`** - Keep elements matching predicate
  - Sequential: `[1,2,3,4,5].filter(x => x > 2)` → `[3,4,5]`
  - Uses `.iter().filter(|&&x| ...).copied().collect()`
- **`reduce(fn, initial)`** - Reduce to single value
  - Example: `[1,2,3,4,5].reduce((acc, x) => acc + x, 0)` → `15`
  - Uses `.iter().fold(initial, |acc, &x| expr)`
- **`forEach(fn)`** - Iterate with side effects
  - Example: `[1,2,3].forEach(x => print(x))`
  - Uses `.iter().for_each(|&x| { ... })`
- **`find(fn)`** - Find first element matching predicate
  - Example: `[1,5,10,15].find(x => x > 10)` → `Some(15)`
  - Returns `Option<T>`, uses `.iter().find(|&&x| pred).copied()`
- **`some(fn)`** - Check if any element matches
  - Example: `[2,4,6].some(x => x % 2 == 0)` → `true`
  - Returns `bool`, uses `.iter().any(|&x| pred)`
- **`every(fn)`** - Check if all elements match
  - Example: `[2,4,6].every(x => x % 2 == 0)` → `true`
  - Returns `bool`, uses `.iter().all(|&x| pred)`
- **`indexOf(value)`** - Find index of value
  - Example: `[10,20,30].indexOf(30)` → `2`
  - Returns `i32` (-1 if not found), uses `.iter().position(|&x| x == value)`
- **`includes(value)`** - Check if array contains value
  - Example: `[10,20,30].includes(20)` → `true`
  - Returns `bool`, uses `.iter().any(|&x| x == value)`

#### String Methods (11 methods)
- **`split(delimiter)`** - Split string into array
  - Example: `"apple,banana,orange".split(",")` → `["apple","banana","orange"]`
  - Returns `Vec<String>`, uses `.split(delim).map(|s| s.to_string()).collect()`
- **`replace(old, new)`** - Replace substring
  - Example: `"hello world".replace("world", "Liva")` → `"hello Liva"`
  - Uses `.replace(old, new)`
- **`toUpperCase()`** - Convert to uppercase
  - Example: `"hello".toUpperCase()` → `"HELLO"`
  - Uses `.to_uppercase()`
- **`toLowerCase()`** - Convert to lowercase
  - Example: `"HELLO WORLD".toLowerCase()` → `"hello world"`
  - Uses `.to_lowercase()`
- **`trim()`** - Remove leading/trailing whitespace
  - Example: `"  hello  ".trim()` → `"hello"`
  - Uses `.trim()`
- **`trimStart()`** - Remove leading whitespace
  - Example: `"  hello".trimStart()` → `"hello"`
  - Uses `.trim_start()`
- **`trimEnd()`** - Remove trailing whitespace
  - Example: `"hello  ".trimEnd()` → `"hello"`
  - Uses `.trim_end()`
- **`startsWith(prefix)`** - Check if starts with prefix
  - Example: `"hello.liva".startsWith("hello")` → `true`
  - Returns `bool`, uses `.starts_with(prefix)`
- **`endsWith(suffix)`** - Check if ends with suffix
  - Example: `"file.pdf".endsWith(".pdf")` → `true`
  - Returns `bool`, uses `.ends_with(suffix)`
- **`substring(start, end)`** - Extract substring
  - Example: `"Hello World".substring(0, 5)` → `"Hello"`
  - Uses slice syntax `[start as usize..end as usize].to_string()`
- **`charAt(index)`** - Get character at index
  - Example: `"Hello".charAt(0)` → `'H'`
  - Uses `.chars().nth(index as usize).unwrap_or(' ')` for UTF-8 safety
- **`indexOf(substring)`** - Find position of substring
  - Example: `"The quick brown fox".indexOf("quick")` → `4`
  - Returns `i32` (-1 if not found), uses `.find(substring).map(|i| i as i32).unwrap_or(-1)`
  - Disambiguated from array `indexOf` by argument type detection

#### Math Functions (9 functions)
- **`Math.sqrt(x)`** - Square root
  - Example: `Math.sqrt(16.0)` → `4.0`
  - Uses `x.sqrt()` method on f64
- **`Math.pow(base, exp)`** - Power/exponentiation
  - Example: `Math.pow(5.0, 2.0)` → `25.0`
  - Uses `base.powf(exp)` method on f64
- **`Math.abs(x)`** - Absolute value
  - Example: `Math.abs(-10.5)` → `10.5`
  - Uses `x.abs()` method with parentheses for unary expressions
- **`Math.floor(x)`** - Round down to integer
  - Example: `Math.floor(3.7)` → `3`
  - Returns `i32`, uses `x.floor() as i32`
- **`Math.ceil(x)`** - Round up to integer
  - Example: `Math.ceil(3.2)` → `4`
  - Returns `i32`, uses `x.ceil() as i32`
- **`Math.round(x)`** - Round to nearest integer
  - Example: `Math.round(3.5)` → `4`, `Math.round(3.4)` → `3`
  - Returns `i32`, uses `x.round() as i32`
- **`Math.min(a, b)`** - Minimum of two values
  - Example: `Math.min(10.5, 20.3)` → `10.5`
  - Uses `a.min(b)` method on f64
- **`Math.max(a, b)`** - Maximum of two values
  - Example: `Math.max(10.5, 20.3)` → `20.3`
  - Uses `a.max(b)` method on f64
- **`Math.random()`** - Random float between 0.0 and 1.0
  - Example: `Math.random()` → `0.8025414370953201` (varies)
  - Uses `rand::random::<f64>()`, automatically adds `rand` crate dependency

#### Type Conversion Functions (3 functions)
- **`parseInt(str)`** - Parse string to integer with error binding
  - Example: `let num, err = parseInt("42")` → `(42, None)`
  - Example: `let num, err = parseInt("abc")` → `(0, Some("Invalid integer format"))`
  - Returns tuple `(i32, Option<Error>)` using Liva's error binding pattern
  - Uses Rust's `.parse::<i32>()`  internally
- **`parseFloat(str)`** - Parse string to float with error binding
  - Example: `let value, err = parseFloat("3.14")` → `(3.14, None)`
  - Example: `let value, err = parseFloat("xyz")` → `(0.0, Some("Invalid float format"))`
  - Returns tuple `(f64, Option<Error>)` using Liva's error binding pattern
  - Uses Rust's `.parse::<f64>()` internally
- **`toString(value)`** - Convert any value to string
  - Example: `toString(42)` → `"42"`
  - Example: `toString(3.14)` → `"3.14"`
  - Example: `toString(true)` → `"true"`
  - Uses `format!("{}", value)` with Rust's Display trait
  - Works with all primitive types (Int, Float, Bool)

#### Console/IO Functions (6 functions - Hybrid Approach)
- **`print(...args)`** - Simple output for end users
  - Format: Display `{}` (clean, no quotes on strings)
  - Example: `print("Hello")` → `Hello`
  - Example: `print($"Name: {name}")` → `Name: Alice`
  - Uses `println!("{}", ...)` for user-facing output
  - Best for: Final output, status messages, simple scripts
- **`console.log(...args)`** - Debug output for developers
  - Format: Debug `{:?}` (shows structure, quotes strings)
  - Example: `console.log("Hello")` → `"Hello"` (with quotes)
  - Example: `console.log([1,2,3])` → `[1, 2, 3]`
  - Uses `println!("{:?}", ...)` for stdout
  - Best for: Debugging, data inspection, development
- **`console.error(...args)`** - Print to stderr
  - Format: Display `{}` (clean, readable error messages)
  - Example: `console.error("File not found!")` → `File not found!`
  - Uses `eprintln!("{}", ...)` for error output
  - Useful for separating errors from normal output
- **`console.warn(...args)`** - Print warning to stderr
  - Format: Display `{}` (clean, readable warning messages)
  - Example: `console.warn("Deprecated feature")` → `Warning: Deprecated feature`
  - Uses `eprintln!("Warning: {}", ...)` with prefix
  - Writes to stderr with "Warning: " prefix
- **`console.readLine()`** - Read line from stdin
  - Example: `let input = console.readLine()`
  - Generates inline block with `std::io::stdin().read_line()`
  - Returns trimmed string
  - Blocks until user provides input
- **`console.prompt(message)`** - Display message and read input
  - Example: `let name = console.prompt("Enter name: ")`
  - Generates inline block with `print!()` + `flush()` + `read_line()`
  - Returns trimmed string
  - Combines prompt display + input reading in one call

**Design Decision: Hybrid I/O Approach**
- **`print()`** - Simple function for beginners and user-facing output
  - Uses Display format `{}` for clean, readable output
  - Strings without quotes: `"Hello"` → `Hello`
  - Best for final results and status messages
- **`console.*`** - Professional namespace for debugging and development
  - Uses Debug format `{:?}` for detailed inspection
  - Strings with quotes: `"Hello"` → `"Hello"`
  - Arrays formatted: `[1, 2, 3]`
  - Organized under single namespace for discoverability
  - Familiar to JavaScript/Node.js developers

### Changed
- **`print()` now uses Display format `{}`** - Clean output for end users (no quotes)
- **`console.log()` uses Debug format `{:?}`** - Shows data structure for debugging
- **`console.error()` and `console.warn()` use Display format `{}`** - Readable error messages
- Extended `generate_method_call_expr()` in codegen.rs to handle string and console methods
- Added `generate_string_method_call()` function for string-specific code generation
- Added `generate_math_function_call()` function for Math namespace functions
- Added `generate_console_function_call()` function for console.* methods
- Added `parseInt()`, `parseFloat()`, `toString()`, `readLine()`, and `prompt()` as built-in functions
- Added `is_builtin_conversion_call()` helper to detect conversion functions
- Fixed VarDecl code generation to properly destructure tuples from built-in conversions
- Fixed method name sanitization - custom methods now convert camelCase to snake_case
- Method call detection now differentiates between array, string, Math, and console methods
- `indexOf` method now supports both arrays (numeric search) and strings (substring search)
- Float literals now generate with `_f64` suffix for explicit typing
- Added `has_random` flag to `DesugarContext` for dependency detection
- Auto-detection of `Math.random()` usage in desugaring phase
- Cargo.toml generation now includes `rand` crate when `Math.random()` is used

### Technical Details
- Array methods use iterator patterns for efficient processing
- String methods map directly to Rust standard library methods
- Math functions use namespace style (`Math.*`) and map to Rust f64 methods
- Console functions use namespace style (`console.*`) and map to println!/eprintln! macros
- Type conversion functions use error binding pattern: `(value, Option<Error>)` tuples
- parseInt/parseFloat return default values (0 or 0.0) on error with error message
- toString uses Rust's Display trait for universal type conversion
- readLine/prompt generate inline blocks with stdin operations
- All methods tested with comprehensive test suites
- Reused existing `MethodCall` and `CallExpr` AST nodes (no parser changes required)
- Fixed precedence issue with `abs()` by wrapping unary expressions in parentheses
- **Critical Fix**: Error binding variables now destructure correctly from built-in functions

### Tests
- Created 6 test files for array methods
- Created 4 test files for string methods
- Created 2 test files for Math functions (basic and comprehensive)
- Created 1 test file for Type Conversion functions (3 functions)
- Created 1 test file for Console/IO functions (3 console functions tested)
- Created 1 comprehensive comparison file (print vs console.log)
- All 37 functions (9 array + 11 string + 9 Math + 3 conversion + 5 I/O) implemented
- 35 functions verified working (readLine/prompt require interactive testing)

### Documentation
- Complete documentation for all stdlib functions in `docs/language-reference/stdlib/`
- Hybrid I/O approach extensively documented (print vs console.*)
- Updated README.md with Standard Library examples
- Updated ROADMAP.md with design decisions
- Created comparison examples showing format differences

---

## [0.6.1] - 2025-10-20

### Fixed
- Removed 26 compiler warnings across the codebase
  - Fixed unreachable code in codegen.rs after early returns
  - Fixed unreachable pattern in lowering.rs
  - Prefixed unused variables with `_`
  - Marked intentionally unused code with `#[allow(dead_code)]`
- Fixed `ir_codegen_string_templates` test
  - Implemented variable type tracking for correct format specifiers
  - Use `{}` for Display types (identifiers, literals, member access)
  - Use `{:?}` for Debug types (arrays, objects)
- Fixed error variable formatting in string templates
  - Added `.unwrap_or("")` when error variables used in templates
  - Prevents `Option<&str>` Display trait errors
- Fixed double semicolons in fire calls
  - Removed trailing semicolon from fire call closures
- Removed illegal class inheritance from test examples
  - Fixed `proj_comprehensive` test: replaced `Empleado : Persona` with composition
  - Clarified distinction between interface implementation (legal) and class inheritance (illegal)

### Changed
- All tests now pass (178 tests total)
  - 82 codegen tests
  - 50 desugar tests
  - 11 integration tests
  - 9 lexer tests
  - 21 parser tests
  - 6 property tests
  - 17 semantics tests
  - 3 doc tests
- Zero compiler warnings
- Improved code quality and consistency

### Documentation
- Updated TODO.md with detailed Phase 1 consolidation progress
- Skipped semantic unit tests restoration (incompatible with current AST)
- Verified all documentation correctly describes interface-only inheritance model

## [0.6.0] - 2025-10-19

### BREAKING CHANGES

#### Removed `protected` Visibility
- **Rationale:** Liva doesn't support class inheritance, only interface implementation
- **Migration:**
  - Old `_protectedField` → Now private (same syntax, different meaning)
  - Old `__privateField` → Now use `_privateField`
  - Protected methods no longer have special semantics

**Before (v0.5.x):**
```liva
Person {
  name: string        // public
  _age: number        // protected (accessible in subclasses)
  __ssn: string       // private (class-only)
}
```

**After (v0.6.0):**
```liva
Person {
  name: string        // public
  _age: number        // private (class-only)
}
```

### Added
- Interface implementation support
  - Classes can implement interfaces using `:` syntax
  - Interfaces are pure contracts (only method signatures, no fields)
  - Multiple interface implementation supported

### Changed
- Visibility model simplified to public/private only
- `_` prefix now means private (was protected)
- `__` prefix removed (no longer needed)

### Migration Guide

#### Class Inheritance → Composition
If you were using class inheritance patterns:

**Before:**
```liva
// This was never officially supported but might have worked
Empleado : Persona {
  empresa: string
}
```

**After:**
```liva
// Use composition instead
Empleado {
  persona: Persona
  empresa: string
  
  init(nombre: string, edad: number, empresa: string) {
    this.persona = Persona(nombre, edad)
    this.empresa = empresa
  }
}
```

#### Interfaces (Still Supported)
Interfaces remain unchanged:

```liva
// Interface (only signatures)
Animal {
  makeSound(): string
  getName(): string
}

// Implementation (has fields + implementations)
Dog : Animal {
  name: string
  
  constructor(name: string) {
    this.name = name
  }
  
  makeSound() => "Woof!"
  getName() => this.name
}
```

---

[Unreleased]: https://github.com/liva-lang/livac/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/liva-lang/livac/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/liva-lang/livac/releases/tag/v0.6.0
