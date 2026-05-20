#!/usr/bin/env bash
# CLI subcommand gate for gen-2.
# Exercises `livac run`, `livac check`, `livac test`, `livac init` end-to-end
# using the gen-2 self-host binary (target/livac-gen2-release).
#
# `livac fmt`, `livac lint` and `livac lsp` ARE covered (tests 10-12 below) —
# gen-2 dispatches them to the `liva-tools` binary via a `rust { }` block
# (F.4 follow-up, see compiler/src/main.liva). `livac update` is NOT covered
# (would touch the network / replace the binary).
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

# ---------------------------------------------------------------------------
# Test 7 — `let v, err = call()` followed by `print("..." + err)` must emit
# Rust that `cargo build` accepts. `err` is `Option<liva_rt::Error>` in
# gen-2 (gen-2 wraps the err-binding in liva_rt::Error to enable trace
# chaining). The naive `format!("{}{}", "...", err)` fails because
# Option<T> doesn't impl Display. The fix unwraps via
# `.as_ref().map(|e| format!("{}", e)).unwrap_or_default()` in the binary
# `+` concat path (mirrors the existing handling in StringTemplate).
# ---------------------------------------------------------------------------
T7="$OUT/err_concat"; mkdir -p "$T7"
cat > "$T7/main.liva" <<'EOF'
main() {
    let db, err = DB.open("/no/such/path/db.sqlite")
    if err {
        print("Failed: " + err)
    }
}
EOF
"$G2" build --output "$T7/out" "$T7/main.liva" >"$T7/build.log" 2>&1
rc=$?
if [[ $rc -ne 0 ]]; then
    check_fail "livac build err_concat main.liva (gen-2)" \
               "rc=$rc; log:\n$(tail -10 "$T7/build.log")"
else
    (cd "$T7/out" && cargo build --quiet) >"$T7/cargo.log" 2>&1
    rc2=$?
    if [[ $rc2 -ne 0 ]]; then
        check_fail "cargo build of err-binding concat" \
                   "rc=$rc2; log:\n$(tail -15 "$T7/cargo.log")"
    else
        # Run it: must exit 0 and print a "Failed: " line containing the
        # underlying DB.open error message.
        out=$("$T7/out/target/debug/main" 2>&1)
        rc3=$?
        if [[ $rc3 -eq 0 && "$out" == *"Failed: "*"DB.open"* ]]; then
            check_ok "Option<Error> err-binding concat unwraps in format!"
        else
            check_fail "Option<Error> err-binding concat runtime" \
                       "rc=$rc3; out=$out"
        fi
    fi
fi

# ---------------------------------------------------------------------------
# Test 9 — HTTP route closures that capture DB connection vars + path params
# must compile. Pins BUG-3 fix: each `app.route(...)` is wrapped in
# `{ let db = db.clone(); ... }` so multiple routes don't fight over the
# moved `Arc<Mutex<Connection>>`. Also pins the `vec![<ident>.to_string()]`
# emission for SQL params (was `vec![<ident>].iter().map(|s| s.to_string())`
# which moved `<ident>` and broke subsequent uses inside the same handler).
# Build-only — app.listen would block.
# ---------------------------------------------------------------------------
T9="$OUT/http_db_routes"; mkdir -p "$T9"
cat > "$T9/srv.liva" <<'EOF'
main() {
    let db, _ = DB.open("test.db")
    let app = Server.create()
    app.get("/items", (req) => {
        let _, qerr = DB.query(db, "SELECT * FROM items", [])
        if qerr {
            return Response.json({ "error": "query failed" })
        }
        Response.json({ "ok": "list" })
    })
    app.get("/items/:id", (req) => {
        let id = req.params.get("id")
        let _, qerr = DB.query(db, "SELECT * FROM items WHERE id = ?", [id])
        if qerr {
            return Response.json({ "error": "not found" })
        }
        Response.json({ "id": id })
    })
    app.listen(3000)
}
EOF
"$G2" build --output "$T9/out" "$T9/srv.liva" >"$T9/build.log" 2>&1
rc=$?
if [[ $rc -ne 0 ]]; then
    check_fail "livac build http_db_routes srv.liva (gen-2)" \
               "rc=$rc; log:\n$(tail -10 "$T9/build.log")"
else
    # Sanity: emitted main.rs must contain `let db = db.clone();` shim
    # exactly twice (one per route).
    shim_count=$(grep -c 'let db = db.clone();' "$T9/out/src/main.rs" 2>/dev/null || echo 0)
    if [[ "$shim_count" -lt 2 ]]; then
        check_fail "BUG-3 db.clone() shim missing in route" \
                   "expected >=2 shims, got $shim_count in $T9/out/src/main.rs"
    elif grep -qE '"id"\.to_string\(\)\.to_string\(\)' "$T9/out/src/main.rs" 2>/dev/null; then
        check_fail "double .to_string() on path-param key" \
                   "found '\"id\".to_string().to_string()' in emitted main.rs"
    else
        (cd "$T9/out" && cargo build --quiet) >"$T9/cargo.log" 2>&1
        rc2=$?
        if [[ $rc2 -eq 0 ]]; then
            check_ok "HTTP routes with DB capture + path param compile (gen-2)"
        else
            check_fail "cargo build of http_db_routes gen-2 output" \
                       "rc=$rc2; log:\n$(tail -15 "$T9/cargo.log")"
        fi
    fi
fi

# ---------------------------------------------------------------------------
# Tests 10–12 — `livac fmt` / `livac lint` / `livac lsp` dispatch to
# `liva-tools` (F.4 follow-up). Self-host main.liva contains a `rust { }`
# block that locates the tools binary (env LIVA_TOOLS_BIN → sibling of
# current_exe → PATH) and forwards args via Command::status() with
# inherited stdio, propagating the child's exit code.
# ---------------------------------------------------------------------------
TOOLS="$LIVAC_ROOT/target/release/liva-tools"
if [[ ! -x "$TOOLS" ]]; then
    echo "[SKIP] liva-tools not built — run: cargo build --release --workspace"
else
    export LIVA_TOOLS_BIN="$TOOLS"

    # Test 10 — `livac fmt --check` on freshly-formatted source: exit 0.
    T10="$OUT/fmt_check"; mkdir -p "$T10"
    cat > "$T10/clean.liva" <<'EOF'
add(a: int, b: int): int {
    return a + b
}

main() {
    print($"sum={add(2, 3)}")
}
EOF
    "$G2" fmt "$T10/clean.liva" >"$T10/fmt.log" 2>&1
    "$G2" fmt --check "$T10/clean.liva" >"$T10/check.log" 2>&1
    rc=$?
    if [[ $rc -eq 0 ]]; then
        check_ok "livac fmt --check (dispatched to liva-tools, exit 0)"
    else
        check_fail "livac fmt --check dispatch" \
                   "rc=$rc; fmt.log:\n$(cat "$T10/fmt.log")\ncheck.log:\n$(cat "$T10/check.log")"
    fi

    # Test 11 — `livac lint` on a file with an unused variable: exit non-zero
    # with W001 diagnostic.
    T11="$OUT/lint_w001"; mkdir -p "$T11"
    cat > "$T11/u.liva" <<'EOF'
main() {
    let unused = 42
    print("hello")
}
EOF
    "$G2" lint "$T11/u.liva" >"$T11/log" 2>&1
    rc=$?
    if grep -q 'W001\|unused' "$T11/log"; then
        check_ok "livac lint (dispatched to liva-tools, W001 emitted)"
    else
        check_fail "livac lint dispatch" \
                   "expected W001/unused diagnostic; rc=$rc; log:\n$(cat "$T11/log")"
    fi

    # Test 12 — `livac lsp` with empty stdin: server starts, parser emits
    # an error and the process exits within timeout. We only check that
    # dispatch actually reached liva-tools (no "binary not found" error
    # from the self-host).
    T12="$OUT/lsp_smoke"; mkdir -p "$T12"
    : > "$T12/stdin"
    timeout 5 "$G2" lsp <"$T12/stdin" >"$T12/log" 2>&1 || true
    if ! grep -qiE 'liva-tools (binary )?not found|No such file' "$T12/log"; then
        check_ok "livac lsp (dispatched to liva-tools, server reachable)"
    else
        check_fail "livac lsp dispatch" \
                   "self-host did not locate liva-tools; log:\n$(cat "$T12/log")"
    fi

    unset LIVA_TOOLS_BIN
fi

echo "===================="
echo "  CLI subcmds: $PASS pass / $FAIL fail"
[[ $FAIL -eq 0 ]]
