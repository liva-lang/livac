# Phase F — Cutting the Rust Bootstrap (v2.1)

> **Status:** PROPOSAL — pending user approval
> **Last updated:** 2026-05-19
> **Owner:** Fran Nadal
> **Branch target:** `feat/self-hosting-v2` → `main` (v2.1 tag)

---

## Goal

Eliminate the Rust bootstrap as the **canonical** compiler and ship a Liva-self-hosted
`livac` binary as the v2.1 release. After Phase F:

- `livac` is built by compiling `compiler/src/*.liva` with the previous-generation
  `livac` (a Liva-self-hosted binary), not by `cargo build` of `livac/src/*.rs`.
- The Rust source tree under `livac/src/` is either **deleted** or **frozen as a
  one-shot bootstrap crate** (decision below).
- `liva_rt` becomes a standalone crate (no compiler imports).

---

## What we have today (post Cycle 65)

| Component                | Rust (`livac/src/`) | Liva self-host (`compiler/src/`) |
|--------------------------|---------------------|----------------------------------|
| Lexer / Parser / AST     | 5,178 LOC           | ✅ ported                        |
| Semantic / Desugaring    | 4,818 LOC           | ✅ ported                        |
| Codegen                  | 18,203 LOC          | ✅ ported (33 files, ~17k LOC)   |
| Module resolution        | 629 LOC             | ✅ ported (235 LOC)              |
| CLI (`main.rs`)          | 1,764 LOC           | ✅ ported (`main.liva`)          |
| Formatter (`livac fmt`)  | 2,367 LOC           | ❌ **not ported**                |
| Linter   (`livac lint`)  | 1,228 LOC           | ❌ **not ported**                |
| LSP server (`livac lsp`) | 1,847 LOC           | ❌ **not ported**                |
| Error / hints / suggestions / traits | 1,358 LOC | partially absorbed in semantic |
| `liva_rt` (runtime)      | 527 LOC             | n/a (linked into generated Rust) |

**Bottom line:** cutting the rope today would lose `fmt`, `lint`, and `lsp`.
Phase F must decide what to do with them.

---

## Decision points (require user input)

### D1. Bootstrap distribution
How does a fresh clone obtain the first Liva compiler?

| Option | Pros | Cons |
|--------|------|------|
| **A. Frozen Rust crate** under `bootstrap/` (snapshot of current `src/`, never touched). `cargo build -p livac-bootstrap` produces gen-0. | Reproducible from source only. No external download. | Keeps ~36k LOC of Rust in the repo forever. |
| **B. Pre-built binaries** in GitHub Releases; `make bootstrap` downloads the right arch. | Repo stays clean. | Trust chain (signing), offline builds break, must publish for each arch. |
| **C. Hybrid** — frozen crate **and** prebuilt download; user picks. | Best of both. | Two paths to maintain. |

**Recommendation:** **A (frozen crate)** for the v2.1 cut; revisit B for v2.2.
The Rust source becomes an immutable artifact, renamed `bootstrap/livac-bootstrap/`,
excluded from CI's main build but kept compilable.

### D2. Fate of `livac fmt` / `lint` / `lsp`

| Option | Effort | Result |
|--------|--------|--------|
| **F-2a.** Port all three to Liva before Phase F. | High (~5,500 LOC of compiler code to rewrite). Blocks v2.1. | Pure self-host; can delete all of `src/`. |
| **F-2b.** Keep them as a separate Rust binary `livac-tools` that depends on `liva_rt` + a small `livac-cli-shim`. | Medium. | `livac` self-hosted; `livac fmt/lint/lsp` invoke `livac-tools` under the hood. Some Rust persists. |
| **F-2c.** Drop `fmt` / `lint` / `lsp` from v2.1; re-add later in Liva. | Low (regression). | Cleanest cut but loses developer-experience features. **Not recommended** — vscode-extension hard-depends on `livac lsp`. |

**Recommendation:** **F-2b** — split the project into two crates:
- `liva-bootstrap/` (frozen Rust, used once per release cycle to seed gen-0).
- `liva-tools/` (Rust, contains formatter + linter + LSP, kept maintained).
- `liva-rt/` (Rust, runtime helpers linked into emitted code; published to crates.io).
- `livac` binary (compiled from Liva, dispatches `fmt`/`lint`/`lsp` to `liva-tools`).

This keeps the rope cut **for the compiler proper** (lexer→parser→semantic→codegen
no longer exist in Rust) while preserving developer tooling.

### D3. Test harness
Today `cargo test --release` runs 531 tests against the bootstrap. After Phase F:

| Option | |
|--------|---|
| **T-3a.** Keep `cargo test` on the frozen bootstrap as a CI gate. Add a parallel `livac test` harness for the self-host. | Easiest; tests stay where they are. |
| **T-3b.** Port the 531 tests to a Liva-native test framework, run via gen-N. | Long; depends on a mature `livac test` (currently basic). |

**Recommendation:** **T-3a** for v2.1, plan T-3b for v2.2.

### D4. `liva_rt` location
Currently `livac/src/liva_rt.rs` (527 LOC) is bundled into every emitted Rust crate
via a `mod liva_rt;` inclusion.

| Option | |
|--------|---|
| **R-4a.** Publish `liva-rt` to crates.io; emitted Rust depends on it. | Clean; standard Rust idiom. |
| **R-4b.** Inline `liva_rt.rs` source into emitted projects (status quo, just relocated). | Zero ecosystem risk; no crates.io commitment. |

**Recommendation:** **R-4b** initially. Publishing to crates.io can wait until the
API stabilises (post-v2.1).

---

## Proposed repo layout after Phase F

```
livac-project/
├── livac/
│   ├── bootstrap/                  # 🔒 FROZEN — gen-0 seed
│   │   ├── Cargo.toml              #   one-shot Rust bootstrap
│   │   ├── src/                    #   snapshot of current livac/src
│   │   └── README.md               #   "Do not modify. Used to bootstrap gen-1."
│   ├── compiler/                   # 📝 EDITABLE — the actual compiler
│   │   └── src/*.liva              #   40 files, ~20k LOC of Liva
│   ├── liva-rt/                    # 📦 runtime, linked into emitted code
│   │   └── src/liva_rt.rs
│   ├── liva-tools/                 # 📦 Rust crate — fmt + lint + lsp
│   │   └── src/{formatter,linter,lsp}.rs
│   ├── tests/                      # unchanged
│   ├── compiler/tests/             # gen-N gates
│   ├── Makefile                    # build orchestration
│   └── target/livac                # 🎯 final binary, built from compiler/src/*.liva
└── ...
```

**Build flow** (`make livac`):
```
1. cargo build --release -p livac-bootstrap          # gen-0 (one shot)
2. ./bootstrap/target/release/livac-bootstrap build  # gen-1 from compiler/src/*.liva
   ./gen-1 build compiler/src                        # gen-2 from gen-1
3. cargo build --release -p liva-tools               # fmt/lint/lsp binary
4. cp gen-2 ./target/livac                           # final binary
```

After Phase F, only steps 2–4 run in CI on PRs; step 1 only when `bootstrap/` changes
(rare — ideally never until v3.0).

---

## Execution roadmap (assuming D1=A, D2=F-2b, D3=T-3a, D4=R-4b)

> **2026-05-19 update:** the order below was the initial proposal. After
> discovering the runtime-API divergence between bootstrap and self-host
> (see F.1b-followup notes), a new `F.runtime-conv` milestone has been
> inserted **before** F.2. The new order is: F.1 → **F.runtime-conv**
> → F.2 → F.3 → F.4 → F.5 → F.6.

### F.runtime-conv — Converge runtime APIs (NEW, blocking)

Picks the bootstrap's runtime API as canonical (it matches the
documented stdlib semantics user code depends on) and rewrites
`compiler/src/codegen_generate.liva` to emit the same API.

Sub-slices:
- **conv-1.** `JsonValue` wrapper struct + method names (`.as_i32`,
  `.as_f64`, `.get`, `.get_field`, `.length`).
- **conv-2.** `liva_http_get/post/put/delete` free functions on top of
  reqwest, matching the bootstrap signatures.
- **conv-3.** `spawn_async`, `spawn_parallel`, `fire_async`,
  `fire_parallel`.
- **conv-4.** `string_mul` and `StringOrInt` trait.

Each sub-slice: codegen change + selfhost compiles a probe program
that uses the feature + 7/7 gates green + gen-2 ≡ gen-3.

Once F.runtime-conv is done, F.1b-followup becomes trivially solvable
by either L1 (`embedFile()` builtin) or by mechanical line-by-line
translation of the now-aligned template.

### F.1 — Carve out `liva-rt` (1 PR)

**Discovery (2026-05-19):** The bootstrap's `livac/src/liva_rt.rs` (527 LOC)
was **dead code**. The "real" runtime is **inline-emitted** as string-writes
in `codegen.rs:1912+` (`self.writeln("mod liva_rt {")` followed by hundreds
of `writeln` calls). The dead file was removed and the bootstrap rebuilt
cleanly (commit `<TBD>`, 531 cargo tests + 7/7 self-host gates green).

This reshapes F.1 into two sub-steps:

**F.1a — Dead-code cleanup** ✅ DONE (2026-05-19)
- Removed `livac/src/liva_rt.rs` and the `pub mod liva_rt;` declaration in
  `lib.rs`. Build green, tests green, gen-2 ≡ gen-3.

**F.1b — Extract the inline runtime to a real source file** ✅ DONE (2026-05-19)
- Captured emitted `mod liva_rt { ... }` block (382 lines) from a hello-world
  compile and stored as `livac/src/liva_rt_template.rs.in`. Replaced 652
  `self.writeln(...)` lines in `codegen.rs` with a single
  `self.output.push_str(include_str!("liva_rt_template.rs.in"))`.
- `codegen.rs` shrank 18203 → 17552 LOC (-651). Emitted output is
  **byte-identical** to before the refactor (verified by diff). 538 cargo
  tests + 7/7 self-host gates green; gen-2 ≡ gen-3 idempotent.

**F.1b-followup — Mirror in the Liva self-host** 🚧 BLOCKED (2026-05-19)

Mirroring the same `include_str!` refactor in `compiler/src/codegen_generate.liva`
requires two preconditions that don't yet exist:

1. **Runtime parity gap (deeper than initially scoped)**:

   First-pass analysis (size): the self-host emits ~45 LOC of runtime
   vs. ~382 LOC for the bootstrap. Easy fix, right? Translate template
   lines into `_writeln(...)` calls and done.

   Second-pass analysis (2026-05-19): **the two emitters produce
   structurally different runtime APIs**, not just different sizes.
   - **Bootstrap** emits `pub struct JsonValue(pub serde_json::Value)`
     — a wrapper newtype with methods `.as_i32()`, `.as_f64()`,
     `.as_string()`, `.get(i)`, `.get_field(k)`, `.length()`, etc. User
     code obtained from `JSON.parse(...)` gets a `JsonValue` value.
   - **Self-host** emits `pub trait JsonValueExt for serde_json::Value
     { fn as_int(&self) -> i64; fn as_float(&self) -> f64;
     fn as_string(&self) -> String; fn as_array_owned() -> Vec<...>;
     fn length() -> usize; }` — extension methods on
     `serde_json::Value` directly. User code gets a `serde_json::Value`
     and calls extension methods.

   These two APIs are **not source-compatible**: `.as_i32()` (bootstrap)
   vs `.as_int()` (self-host); `i64` vs `i32` return type for ints; etc.
   A program written against bootstrap's API will fail under gen-N and
   vice-versa.

   Once the bootstrap is cut, this gap must be closed in one of two ways:
   - **Align bootstrap to self-host's JsonValueExt API.** Removes the
     wrapper, simpler runtime, but breaks any existing user code that
     depends on the `JsonValue` wrapper type.
   - **Align self-host to bootstrap's JsonValue wrapper API.** Preserves
     the documented stdlib semantics; requires self-host codegen to emit
     the wrapper instead of the trait. Larger self-host change.

   Same parity analysis is needed for HTTP (bootstrap emits `liva_http_*`
   free functions; self-host emits only the `LivaHttpResponse` struct
   with no client functions at all), and `spawn_*` / `fire_*` (bootstrap
   emits 4 functions; self-host emits zero).

   **Recommended:** schedule a dedicated "runtime convergence" milestone
   (call it F.runtime-conv) as a pre-requisite for the F-rope-cut.
   Pick the bootstrap's API as canonical (it's the documented one users
   rely on) and reshape the self-host emission to match. This is its own
   multi-PR effort, not a slice of F.1b-followup.

2. **Liva lacks include_str!**:
   No multi-line string literals, no compile-time file embedding. Even
   if API parity is closed, the self-host needs a mechanism to emit
   ~400 LOC of runtime without manual line-by-line `_writeln`. Three
   resolution paths sketched (L1: add embedFile() builtin, L2: add """...
   """ literals, L3: runtime File.read). L1 recommended as smallest
   slice. **Deferred until F.runtime-conv has fixed the API divergence;
   no point embedding a template that won't match the canonical API.**

**F.1c — Publish `liva-rt` as a crate (deferred to v2.2+)**
- Bumps D4 from R-4b to R-4a. Out of scope for v2.1 cut.

### F.2 — Carve out `liva-tools` (1 PR)
- Move `formatter.rs`, `linter.rs`, `src/lsp/*` into `livac/liva-tools/src/`.
- New `Cargo.toml` produces binary `liva-tools`.
- Modify CLI dispatch in self-host `main.liva` (and Rust `main.rs` while bootstrap
  still runs): `livac fmt FOO` → `Process.exec("liva-tools", ["fmt", "FOO"])`.
- VS Code extension already configurable; update to invoke `liva-tools lsp` if
  `livac lsp` is unavailable.

### F.3 — Freeze the bootstrap (1 PR)
- `git mv livac/src/ livac/bootstrap/src/`
- `git mv livac/Cargo.toml livac/bootstrap/Cargo.toml`
- Strip the workspace's top-level `Cargo.toml` of the `livac` package; keep a
  workspace root that lists `bootstrap`, `liva-rt`, `liva-tools` as members.
- Add `livac/bootstrap/FROZEN.md` (we already have `livac/src/FROZEN.md` for the
  codegen-side BS-FRAG fences — extend its scope).

### F.4 — Make Liva-built binary the canonical `livac` (1 PR)
- Update `Makefile` to define `make livac` per the build flow above.
- Replace `target/release/livac` references in `compiler/tests/*.sh` and CI with
  the gen-2 path.
- Add `compiler/tests/phaseF_smoke.sh` — runs `livac build`, `livac test`,
  `livac fmt`, `livac lint`, `livac lsp --probe` against the new layout.

### F.5 — CI rewire (1 PR)
- `.github/workflows/ci.yml`:
  - Drop the standalone `test` job (which built only the Rust bootstrap).
  - Replace with `bootstrap-test` (runs `cargo test -p livac-bootstrap`) **only when
    `bootstrap/**` files change** (path filter).
  - `selfhost-quick` becomes the default PR gate.
  - Add `tools-test` (runs `cargo test -p liva-tools`).
- `release.yml`: ship 4 binaries: `livac` (gen-2), `liva-tools`, `liva-rt` source,
  `livac-bootstrap` (rare).

### F.6 — Documentation + tag
- Update top-level READMEs, `ROADMAP.md`, `CHANGELOG.md`, `BACKLOG.md` Fase F.
- Update workspace banners (`livac/.github/copilot-instructions.md`,
  `livac-project/.github/copilot-instructions.md`).
- Update agent skill `skills/liva-lang/SKILL.md` if needed.
- `git tag v2.1.0 -s -m "Liva is fully self-hosted"`

---

## Risks & mitigations

| Risk | Mitigation |
|------|------------|
| gen-2 binary regression vs. bootstrap on edge cases. | `compiler/tests/e2e_selfhost.sh` already enforces stdout-identity on 5 programs. Extend to 20+ before tagging v2.1. |
| LSP latency increases because `liva-tools` is a separate process. | LSP is already a separate process per client; user-visible cost is negligible. |
| `liva-rt` API drift between gen-N codegen output and the runtime crate. | Pin `liva-rt` version in emitted `Cargo.toml`; codegen and runtime version-bump in lockstep. |
| New contributors can't bootstrap easily. | `make livac` does steps 1-4 unattended. Document in `README.md`. |
| BS-FRAG bugs in current Rust bootstrap surface during frozen lifetime. | Bootstrap is a one-shot path; gen-1 onwards uses self-host. Frozen ≠ unmaintained — security fixes still allowed. |

---

## Open questions

1. **Do we want `livac doc` (v2.x backlog) before Phase F?** It would be easier to
   port to Liva alongside `fmt`/`lint` (option F-2a) than to add to `liva-tools`.
2. **Should `liva-rt` be published to crates.io for v2.1?** Currently leaning no (R-4b),
   but a public crate would dramatically simplify cross-project use.
3. **Windows build story for `liva-tools`?** LSP currently works cross-platform; need to
   verify the carve-out doesn't change that.

---

## Next steps (requires user decision)

1. **Approve / amend decisions D1–D4.**
2. If approved as recommended (A / F-2b / T-3a / R-4b), start with **F.1 (carve out
   `liva-rt`)** — smallest, lowest-risk slice. ETA: one focused session.
3. Open question Q1 above is the only one that could re-sequence the roadmap (would
   bring Q1 → F-2a → drop the need for `liva-tools` as a crate).

