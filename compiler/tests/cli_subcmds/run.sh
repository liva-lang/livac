#!/usr/bin/env bash
# CLI subcommand gate for gen-2.
# Exercises `livac run`, `livac check`, `livac test`, `livac init` end-to-end
# using the gen-2 self-host binary (target/livac-gen2-release).
#
# `livac fmt`, `livac lint`, `livac lsp` and `livac update` are NOT covered:
# fmt/lint/lsp are not yet implemented in gen-2 (see compiler/docs/PLAN.md
# Bloque B), and `update` would touch the network / replace the binary.
#
# Each sub-test prints `[OK ]` or `[FAIL]` and the script exits non-zero if
# any sub-test fails.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVAC_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
G2="$LIVAC_ROOT/target/livac-gen2-release"
OUT="${TMPDIR:-/tmp}/liva_cli_subcmds_out"
rm -rf "$OUT"; mkdir -p "$OUT"

[[ -x "$G2" ]] || { echo "gen-2 missing: bash compiler/tests/rebuild_selfhost.sh"; exit 2; }

PASS=0; FAIL=0

check_ok()   { echo "[OK ] $1"; PASS=$((PASS+1)); }
check_fail() { echo "[FAIL] $1"; echo "  $2"; FAIL=$((FAIL+1)); }

# ---------------------------------------------------------------------------
# Test 1 — `livac run` compiles and executes, stdout matches expectation.
# ---------------------------------------------------------------------------
T1="$OUT/run_basic"; mkdir -p "$T1"
cat > "$T1/hello.liva" <<'EOF'
main() {
    let x = 21
    print($"answer={x * 2}")
}
EOF
cd "$T1"
out=$("$G2" run hello.liva 2>&1) || true
cd - >/dev/null
# `run` prints compile diagnostics + program stdout interleaved; just ensure
# the program stdout line is present.
if echo "$out" | grep -q '^answer=42$'; then
    check_ok "livac run hello.liva (stdout contains 'answer=42')"
else
    check_fail "livac run hello.liva" "expected 'answer=42' in stdout, got: $out"
fi

# ---------------------------------------------------------------------------
# Test 2 — `livac check` on a clean file: no semantic errors, exit 0.
# ---------------------------------------------------------------------------
T2="$OUT/check_clean"; mkdir -p "$T2"
cat > "$T2/clean.liva" <<'EOF'
main() {
    let n: int = 7
    print($"n={n}")
}
EOF
"$G2" check "$T2/clean.liva" >"$T2/log" 2>&1
rc=$?
if [[ $rc -eq 0 ]]; then
    check_ok "livac check clean.liva (exit 0)"
else
    check_fail "livac check clean.liva" "expected exit 0, got $rc; log:\n$(cat "$T2/log")"
fi

# ---------------------------------------------------------------------------
# Test 3 — `livac check` on a file with a parser-level error:
# must exit non-zero and emit a diagnostic. (Semantic-only errors are not
# yet caught by gen-2 `check` — see compiler/docs/PLAN.md Bloque B.1; this
# test pins the parser-error path which IS implemented.)
# ---------------------------------------------------------------------------
T3="$OUT/check_dirty"; mkdir -p "$T3"
cat > "$T3/dirty.liva" <<'EOF'
main() {
    let x =
}
EOF
"$G2" check "$T3/dirty.liva" >"$T3/log" 2>&1
rc=$?
if [[ $rc -ne 0 ]] && grep -qi 'expected\|error' "$T3/log"; then
    check_ok "livac check dirty.liva (parser error, non-zero exit + diagnostic)"
else
    check_fail "livac check dirty.liva" "expected non-zero exit and diagnostic; rc=$rc; log:\n$(cat "$T3/log")"
fi

# ---------------------------------------------------------------------------
# Test 4 — `livac test`: emits #[test] for `test "name" { ... }` blocks
# and delegates to cargo test. Two tests, both pass.
# ---------------------------------------------------------------------------
T4="$OUT/test_runner"; mkdir -p "$T4"
cat > "$T4/sample.test.liva" <<'EOF'
test "addition works" {
    let x = 2 + 3
    if x != 5 {
        fail $"expected 5, got {x}"
    }
}

test "string length" {
    let s = "hello"
    if s.length != 5 {
        fail $"expected length 5, got {s.length}"
    }
}
EOF
"$G2" test "$T4/sample.test.liva" >"$T4/log" 2>&1
rc=$?
# Accept either the "PASS" formatted line OR a successful cargo test result.
if [[ $rc -eq 0 ]] && (grep -qE 'PASS|test result: ok|2 passed' "$T4/log"); then
    check_ok "livac test sample.test.liva (exit 0, 2 tests pass)"
else
    check_fail "livac test sample.test.liva" "rc=$rc; log tail:\n$(tail -20 "$T4/log")"
fi

# ---------------------------------------------------------------------------
# Test 5 — `livac init`: scaffolds a project directory that compiles and
# runs end-to-end with gen-2.
# ---------------------------------------------------------------------------
T5="$OUT/init_scaffold"; mkdir -p "$T5"
cd "$T5"
"$G2" init demo_proj >"$T5/init.log" 2>&1
rc=$?
cd - >/dev/null
if [[ $rc -eq 0 ]] && [[ -f "$T5/demo_proj/main.liva" ]] && [[ -f "$T5/demo_proj/.gitignore" ]]; then
    check_ok "livac init demo_proj (main.liva + .gitignore present)"
else
    check_fail "livac init demo_proj" "rc=$rc; files:\n$(ls -la "$T5/demo_proj" 2>&1)"
fi

# Compile the scaffolded project to make sure init produces buildable code.
if [[ -f "$T5/demo_proj/main.liva" ]]; then
    cd "$T5/demo_proj"
    "$G2" build main.liva >"$T5/build.log" 2>&1
    rc=$?
    cd - >/dev/null
    if [[ $rc -eq 0 ]]; then
        check_ok "scaffolded project compiles with gen-2"
    else
        check_fail "scaffolded project build" "rc=$rc; log tail:\n$(tail -15 "$T5/build.log")"
    fi
fi

# ---------------------------------------------------------------------------
# Test 6 — `livac build` on an HTTP file that uses `Response.json` with
# both bare-ident and quoted-string keys must produce Rust that
# `cargo build` accepts. This pins the fix for the
# `serde_json::json!({"key".to_string(): ...})` bug. We don't run the
# binary (it would block on app.listen), only verify cargo accepts it.
# ---------------------------------------------------------------------------
T6="$OUT/http_json_keys"; mkdir -p "$T6"
cat > "$T6/srv.liva" <<'EOF'
main() {
    let app = Server.create()
    app.get("/health", (req) => {
        Response.json({ status: "ok", service: "demo" })
    })
    app.get("/err", (req) => {
        Response.json({ "error": "Not found" })
    })
    app.listen(3000)
}
EOF
"$G2" build --output "$T6/out" "$T6/srv.liva" >"$T6/build.log" 2>&1
rc=$?
if [[ $rc -ne 0 ]]; then
    check_fail "livac build srv.liva (gen-2)" "rc=$rc; log:\n$(tail -10 "$T6/build.log")"
else
    # Inspect emitted main.rs: keys must be bare quoted, not "k".to_string().
    if grep -qE '"[A-Za-z_][A-Za-z0-9_]*"\.to_string\(\)\s*:' "$T6/out/src/main.rs" 2>/dev/null; then
        check_fail "Response.json keys are bare in serde_json::json!" \
                   "found '\"k\".to_string():' in emitted main.rs"
    else
        # Cargo build to make sure rustc accepts the json! macro.
        (cd "$T6/out" && cargo build --quiet) >"$T6/cargo.log" 2>&1
        rc2=$?
        if [[ $rc2 -eq 0 ]]; then
            check_ok "Response.json with bare+quoted keys compiles via gen-2 + cargo"
        else
            check_fail "cargo build of HTTP gen-2 output" "rc=$rc2; log:\n$(tail -15 "$T6/cargo.log")"
        fi
    fi
fi

echo "===================="
echo "  CLI subcmds: $PASS pass / $FAIL fail"
[[ $FAIL -eq 0 ]]
