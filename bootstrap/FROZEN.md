# Frozen bootstrap (Phase F.3)

> **Do not modify** unless updating the seed compiler used to bootstrap
> a fresh clone of the workspace. The canonical `livac` binary is now
> built from the Liva sources in `compiler/src/*.liva` (gen-2). This
> Rust crate exists solely so a new contributor with no `livac` binary
> on disk can run `cargo build --release -p livac-bootstrap` and obtain
> gen-0, which then drives `compiler/tests/rebuild_selfhost.sh` to
> produce gen-1 → gen-2.

## What this crate provides

- `livac-bootstrap` package, exposing both:
  - `lib` named `livac` — consumed by `liva-tools` for AST/lexer/parser/semantic/error types.
  - `bin` named `livac` — the gen-0 Rust compiler binary.

## What lives here

- `src/{lexer,parser,ast,semantic,desugaring,codegen,...}.rs` — the
  full Rust implementation of the compiler as of v2.0.0-rc1.
- `src/liva_rt_template.rs.in` — the runtime template embedded in
  emitted Rust code via `include_str!`.
- `tests/` — the 538-test bootstrap regression suite.

## What does NOT live here

- The self-host compiler (`compiler/src/*.liva`) — that is the
  canonical compiler going forward.
- `formatter`, `linter`, `lsp` — moved to `liva-tools/` in Phase F.2.

## When to update

Only when:

1. A vulnerability or critical miscompile is found in the bootstrap path.
2. Adding a language feature that gen-N still can't self-compile (rare
   after v2.1).

In both cases, the change must also land in the self-host
(`compiler/src/`) and pass `compiler/tests/run_all.sh` end-to-end.
