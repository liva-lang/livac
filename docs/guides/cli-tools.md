# Liva CLI Tools (v2.3+)

This guide covers the tooling subcommands added during the v2.1 → v2.3
self-hosting cycle. They share one binary (`livac`) and are dispatched
the same way as `build` / `run` / `check`.

| Subcommand        | Purpose                                                    | Since |
|-------------------|------------------------------------------------------------|-------|
| `livac repl`      | Interactive read-eval-print loop                           | v2.3  |
| `livac doc`       | Generate Markdown reference from `///` doc-comments        | v2.3  |
| `livac test`      | Run `test_*` functions and Jest-style suites               | v2.0+ |
| `livac test --coverage` | Wrap test run with `cargo-llvm-cov`                  | v2.3  |
| `livac bench`     | Run `bench_*` functions, print one `BENCH … ms` line each  | v2.3  |
| `livac fmt`       | Format Liva source files                                   | v2.1  |
| `livac lint`      | Detect unused vars/imports, dead code, always-true/false   | v1.8  |
| `livac lsp`       | Start the Language Server (stdio)                          | v2.0  |

---

## `livac repl` — Interactive REPL

Starts a session where each line is parsed and evaluated against a
persistent environment. Arrow keys browse history (powered by
`rustyline`), and Ctrl-D / `:quit` exits.

```bash
$ livac repl
liva> let x = 21
liva> x * 2
42
liva> :quit
```

Built-in commands (prefixed with `:`):

- `:help` — list available commands
- `:type <expr>` — print the inferred type of `<expr>`
- `:reset` — clear the environment
- `:quit` — exit

The REPL shares the parser, semantic analyzer and codegen with `build`,
so anything that compiles in a `.liva` file works at the prompt
(modulo top-level statements being implicitly wrapped in `main`).

---

## `livac doc` — Doc Generator

Walks every `.liva` file in the project, extracts `///` doc-comments
attached to top-level declarations, and emits Markdown grouped by
module. Perfect for publishing a reference for a library crate.

```liva
/// Returns the greatest common divisor of `a` and `b`.
///
/// # Examples
/// ```
/// assert_eq(gcd(12, 18), 6)
/// ```
fn gcd(a: int, b: int): int {
    if b == 0 => a
    return gcd(b, a % b)
}
```

```bash
$ livac doc src/ -o docs/api.md
Generated docs/api.md (12 modules, 87 declarations)
```

Options:

- `-o <FILE>` — output path (default: `target/doc/index.md`)
- `--private` — include `priv` declarations too
- `--quiet` — only print errors

---

## `livac test --coverage` — Coverage Reports

Runs the project's tests under [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov),
producing line/branch coverage of the **generated Rust crate** that
backs your Liva program.

```bash
$ cargo install cargo-llvm-cov   # one-off
$ livac test --coverage
…
running 18 tests
test test_gcd ... ok
…
test result: ok. 18 passed; 0 failed

Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover
src/main.rs                       412                21    94.90%          47                 2    95.74%         521                42    91.94%
```

Notes:

- Coverage is collected at the Rust level. Lines map back to Liva
  source via `# … line directives` emitted by codegen.
- Add `--html` to produce a browsable report at `target/llvm-cov/html/`.
- Combine with `livac doc` to publish coverage alongside API docs in CI.

---

## `livac bench` — Microbenchmark Runner

Executes every top-level function whose name begins with `bench_`,
times the body in release mode, and prints one machine-parseable line
per benchmark:

```liva
fn bench_sum_loop() {
    let mut total = 0
    for i in 0..1_000_000 {
        total = total + i
    }
}
```

```bash
$ livac bench src/perf.liva
BENCH bench_sum_loop 2 ms
BENCH bench_array_pipeline 1 ms
```

Wire it into CI by asserting that every fixture emits a `BENCH` line —
that's exactly what `compiler/tests/bench/run.sh` does for the
compiler's own gates. There is intentionally **no absolute time
threshold** baked in: CI runners are too noisy. Track regressions by
diffing across runs in your CI tool of choice.

---

## Jest-style Tests (`liva/test`)

Beyond the simple `test_*()` runner, v2.3 ships a stdlib module with
a Jest-flavoured API:

```liva
import { describe, test, beforeEach, afterEach, expect } from "liva/test"

describe("Math.clamp", () => {
    let calls = 0

    beforeEach(() => { calls = calls + 1 })
    afterEach(() => { /* cleanup */ })

    test("clamps below low", () => {
        expect(Math.clamp(-10.0, 0.0, 100.0)).toBe(0.0)
    })
    test("clamps above high", () => {
        expect(Math.clamp(250.0, 0.0, 100.0)).toBe(100.0)
    })
})
```

Available matchers: `.toBe`, `.toEqual`, `.toContain`, `.toBeTruthy`,
`.toBeFalsy`, `.toBeNull`, `.toBeGreaterThan`, `.toBeLessThan`, plus
`.not.<matcher>` for any of them.

Lifecycle hooks: `beforeAll`, `afterAll`, `beforeEach`, `afterEach` —
each scoped to the enclosing `describe(...)` block.

`describe`/`test` files are auto-discovered by `livac test`; you do
not need a `main fn`. Mix and match with the older `test_*()` style
in the same project — both run in one pass.

---

## Environment Variables

| Variable        | Effect                                                                        |
|-----------------|-------------------------------------------------------------------------------|
| `LIVA_STRICT=1` | Codegen emits a tighter `#![allow(...)]` prelude, surfacing more warnings. Useful when you want `-D warnings` to bite without scaffolding noise. |
| `LIVAC_ROOT`    | Override the path used by tests/scripts to find the `livac` binary.           |
| `LIVA_DEBUG=1`  | Print parser/semantic/codegen phase timings to stderr.                        |

---

## Related

- [`docs/lsp/LSP_USER_GUIDE.md`](../lsp/LSP_USER_GUIDE.md) — IDE setup
- [`docs/QUICK_REFERENCE.md`](../QUICK_REFERENCE.md) — language at a glance
- [`compiler/tests/bench/`](../../compiler/tests/bench/) — bench gate fixtures
