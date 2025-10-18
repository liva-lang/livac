# Liva Compiler Overhaul — Working Plan

> Branch: `feature/compiler-overhaul`

This document tracks the incremental work needed to turn the livac prototype into
an end-to-end compiler that generates compilable Rust projects and exposes
actionable diagnostics.

## 1. Front-End Refinement

- [x] **Lexer**: keep as-is for v0.6, but add token span helpers for better diagnostics.
- [ ] **Parser**:
  - [ ] Modularise production parsing (declarations vs. statements vs. expressions).
  - [ ] Support `const` declarations, assignment statements, and chained/member assignments.
  - [ ] Parse string templates into `Expr::StringTemplate` using interpolated parts.
  - [ ] Enforce block delimiters and optional-semicolon policy consistently.
- [ ] **AST**:
  - [ ] Ensure nodes carry span metadata where diagnostics need it.
  - [ ] Separate expression-bodied vs. block-bodied routines without duplication.

## 2. Semantic Analysis

- [ ] Introduce symbol tables with lexical scoping and visibility checks.
- [ ] Perform single-pass type inference for basic primitives (`number`, `float`, `bool`, `string`).
- [ ] Validate function/method signatures against call sites.
- [ ] Rework async inference as an iterative, fixed-point analysis.
- [ ] Produce structured diagnostics for missing symbols, type mismatches, and visibility errors.

## 3. Intermediate Representation (IR)

- [x] Design a minimal, typed IR for functions/tests with concurrency metadata (structs/impls pending follow-up).
- [x] Encode ownership of async/parallel constructs explicitly to guide desugaring/runtime helpers.
- [x] Provide lowering passes from AST → IR (statements, expressions, concurrency ops); extend to types/classes next.

## 4. Code Generation

- [ ] Replace string concatenation with emission through `quote!`/`syn` builders.
- [x] Generate `Cargo.toml` based on IR capabilities (async crates, user `use rust` statements).
- [x] Embed an inline `liva_rt` helper module for async/parallel dispatch (extract to crate + expand helpers next).
- [ ] Guarantee `cargo check` succeeds for the integration suites.

## 5. Tooling & Diagnostics

- [ ] Extend the CLI to emit rich diagnostics (Ariadne) and machine-readable JSON.
- [ ] Update the VS Code extension to consume the JSON diagnostics directly.
- [ ] Add incremental compilation hooks (`--watch`, `--check` optimisations).

## 6. Testing & Automation

- [ ] Augment unit tests with negative suites (parser, semantics).
- [ ] Run the generated projects through `cargo fmt` + `cargo check` inside integration tests.
- [x] Add IR-focused codegen snapshots (`tests/codegen_ir_tests.rs`).
- [ ] Configure CI (GitHub Actions) to enforce formatting, linting, tests, and integration checks.

## 7. Deliverables & Milestones

1. **Milestone A – Front-End Alignment**
   - Parser + semantic passes updated, tests stabilised.
2. **Milestone B – Rust-Ready Codegen**
   - IR in place, codegen produces compilable Rust, integration tests compile.
   - Runtime helpers emitted alongside generated projects (string concat, logging, task dispatch).
3. **Milestone C – Developer Tooling**
   - CLI diagnostics, VS Code extension integration, CI pipeline.

Each milestone will come with CHANGELOG entries, documentation updates, and
release candidates (`v0.7.0-alphaX`).

### Immediate Action Items (Q1 Sprint)

- [x] Finalise initial IR design (functions/tests covered; structs/impls tracked separately).
- [x] Implement lowering: AST → IR (statements, expressions, concurrency ops).
- [ ] Replace `codegen::generate_with_ast` with IR-driven emitter using `quote!`.
- [ ] Add `runtime/` crate with helpers for printing, string concat, async utilities (currently using inline module).
- [ ] Re-enable strict semantic checks once runtime + IR cover external functions.
- [ ] Extend integration tests to run `cargo check` once Rust output is stable.

---

_Document maintained by Codex & collaborators. Update checklist items as work progresses._
