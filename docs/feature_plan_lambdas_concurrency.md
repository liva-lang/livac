# Liva v0.6 → v0.7 Feature Plan (Lambdas, Concurrency, Policies, `length`)

This document captures the task breakdown needed to implement the new language rules described in the user specification. Work can be tackled incrementally; each bullet should become a PR-sized unit where possible.

---

## 1. Language Surface

- **Lexer**
  - [x] Added keyword tokens for `par`, `move`, `seq`, `vec`, `boost`, `with`, and all for-option flags (`ordered`, `chunk`, `threads`, `simdWidth`, `prefetch`, `reduction`, `schedule`, `detect`, `auto`, `safe`, `fast`, `static`, `dynamic`) so downstream stages can consume the new syntax.
  - [x] Remove `parallel` token usages (keep for backwards compatibility warnings if desired).
  - [x] Ensure `async`, `par`, `task`, `fire` are recognised as separate modifiers (not identifiers).

- **Parser**
  - [x] Member expressions: allow `.length` property access; emit diagnostics for `len(x)` at parse or semantic stage.
  - [x] Lambda literals support `[move] (param list) => expr|block`, single-identifier heads, and optional return annotations.
  - [x] Call expressions now carry execution policy metadata (`normal`, `async`, `par`, `task async`, `task par`, `fire async`, `fire par`).
  - [x] Disallow modifiers on declarations (diagnostic if `async foo() {}` uses keyword wrongly).
  - [x] `for` statement accepts policy keywords (`seq`/`par`/`vec`/`boost`) and parses `with` clause options.
  - [x] Await expressions remain unary and interoperate with policy-decorated calls.

- **AST Updates**
  - [x] Introduce `CallExpr` structure with `exec_policy` field; remove `AsyncCall`, `ParallelCall`, `TaskCall`, `FireCall` in favour of unified calls.
  - [x] Add `LambdaExpr` node with `is_move`, `params`, `return_type`, `body`, `captures`.
  - [x] Extend `ForStmt` with `policy: DataParallelPolicy` and `options: ForPolicyOptions`.
  - [x] Add enums/structs for policy options (`ThreadOption`, `SimdWidthOption`, etc.).
  - [x] Ensure serialization/deserialization (serde derives) reflect new layout (needed for tests).

---

## 2. Semantic Analysis

- Enforce `.length` correctness:
  - [x] Arrays/strings expose `length`; sequences must error with E0701 (suggest `.count()` or `.collect().length`).
  - [x] Replace legacy `len(x)` with warning W0700 + quick-fix suggestions.
- Validate lambda syntax:
  - Infer/verify parameter scopes, handle move-capture diagnostics (E0510 for non-Send captures later).
  - Ensure return type annotation validity.
- Execution policies:
  - [x] Ensure modifiers only appear at call sites; produce diagnostics for invalid combinations (e.g. double modifiers).
  - Track async inference through new `CallExpr.exec_policy`.
  - [x] Detect tasks that are never awaited (W0601), double await (E0604), invalid awaits (E0603), etc.
  - Enforce concurrency safety rules (E0401/E0402/E0510/E0511 placeholders).
- For-loop policies:
  - [x] Validate option compatibility with chosen policy (semantic checks complete; codegen still pending so loops emit sequential Rust).
  - [x] Check numeric ranges (positive chunk sizes, etc.).
  - [x] Flag illegal constructs (await inside `par/boost` body; non-Send captures pending, and runtime execution remains sequential until codegen lands).
- Extend symbol/type tracking to support lambda parameters and inference in new constructs.

---

## 3. IR & Lowering

- Update IR to mirror AST changes:
  - `ir::Expr::Call(CallExpr)` with execution policy enum.
  - `ir::Expr::Lambda(LambdaExpr)` and associated param/body structs.
  - [x] `ir::Stmt::For` now carries policy + options (codegen still sequential until policy handlers land).
  - [x] Introduce `DataParallelPolicy`, `ForPolicyOptions`, `ThreadOption`, etc. in IR (currently used for metadata only).
- Lowering pass adjustments:
  - Map AST structures into updated IR forms.
  - Ensure async inference still works with new call representation.
  - Capture lambda information (move semantics, return types, captures placeholder).

---

## 4. Code Generation

- Call generation:
  - [x] Render `.length` property with correct Rust translation (arrays → `.len()`, strings → `.chars().count()` as needed).
  - [x] Replace special casing of `len(x)` with new diagnostics; remove old conversion path.
  - Map execution policies:
    - `async` → spawn + implicit await on first use.
    - `par` → thread pool execution (Rayon or `std::thread` wrappers).
    - `task` / `fire` produce handles or fire-and-forget semantics.
  - Update runtime glue (`liva_rt`) to expose required helpers (task handles, join/await wrappers, policy-specific APIs).
- Lambda codegen:
  - Emit Rust closures (`|args|` or `move |args|`) with inference of async/parallel usage inside.
  - Support block bodies and typed parameters.
- For policies:
  - [x] Generate Rayon-backed loops for `par/vec/boost` (runtime currently shares a Rayon-based fallback; SIMD/boost specialisations and advanced scheduling still pending).
  - Honour `ordered`, `chunk`, `threads`, `simdWidth`, etc. with safe fallbacks.
  - Produce runtime warnings/errors where features are not yet available.
- Update generated diagnostics and error messages to reference new codes.

---

## 5. Runtime (`liva_rt`)

- Extend helper module to support:
  - Task handles with await/join semantics, including consumption tracking.
  - Fire-and-forget primitives (spawn without awaiting).
  - [x] Data-parallel execution adapters (initial Rayon-backed helpers in place; SIMD-specialised helpers still TODO).
  - Count operations for sequences (`seq.count()`, `await aseq.count()`).
- Ensure runtime enforces Send/'static checks for `par`/`boost` contexts; provide graceful error handling.

---

## 6. Diagnostics & Tooling

- Implement new warning/error codes:
  - W0700 (len deprecated), E0701 (length misuse).
  - W0601, E0602, E0603, E0604, E0401, E0402, W0403, E0510, E0511.
- Add quick-fix hints where applicable (e.g. suggest `.count()`).
- Update CLI messages and IDE tooling hooks if any rely on old codes.

---

## 7. Tests

- Lexer snapshots for new tokens.
- Parser golden files covering:
  - Lambda syntax variations, move capture, return types.
  - Calls with each execution modifier.
  - For loops with policies/options (positive and negative cases).
- Semantic tests:
  - Valid usages of `.length`, `.count()`, concurrency modifiers.
  - Error snapshots for each diagnostic listed above.
- Lowering/IR/codegen snapshot updates (insta tests).
- Runtime integration tests ensuring behaviour of async/par/task/fire.
- Update integration examples (`tests/integration/**`, `proj_examples`, etc.) to cover new features.

---

## 8. Documentation & Examples

- Specs: update `docs/Liva_v0.6_spec.md`, `Liva_v0.6_Desugaring.md`, `Liva_v0.6_EBNF_AST.md` to reflect new grammar and semantics.
- Draft new sections highlighting:
  - Lambda syntax usage.
  - Concurrency consumption rules.
  - Data-parallel policies and options table.
  - `.length` vs `.count()` guidance and migration notes.
- Update `main.liva` (example entry point) with sample code demonstrating each feature.
- Provide migration guide from `len(x)` and old `parallel` keyword.

---

## 9. Build & Release

- Update `livac/script.sh`, `run_tests.sh`, and CI workflows to include new test suites.
- Bump crate version (Cargo.toml) once implementation stabilises.
- Prepare release notes summarising user-visible changes and deprecations.

---

## 10. Iteration Roadmap

Recommended order of execution:

1. Lexer + AST groundwork (length property, new tokens, call/lambda/for structures).
2. Parser implementation (new constructs, ensure tests compile).
3. Semantic layer adjustments (diagnostics, inference updates).
4. IR/lowering alignment.
5. Runtime & codegen support for new execution policies and loops.
6. Documentation/test overhaul alongside incremental implementation.
7. Final integration pass, CLI verification, regression testing.

Use this checklist to track progress across sessions; update with links to commits or PRs as work completes.
